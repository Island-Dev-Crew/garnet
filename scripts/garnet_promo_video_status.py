#!/usr/bin/env python3
"""Report the Garnet promo video readiness contract.

This reporter is deliberately not a video renderer. It gives agents, PRs, and
the public site a deterministic contract for the requested 30-second Garnet
promo lane while keeping the current truth clear: no verified rendered video or
website-ready export exists yet.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class PromoVideoStatus:
    source: str
    status: str
    completion_percent: float
    target_duration_seconds: int
    rendered_video_present: bool
    visual_qa_present: bool
    website_export_present: bool
    public_site_embed_present: bool
    composition_source_present: bool
    visual_identity_locked: bool
    source_surfaces_locked: bool
    current_truth: list[str]
    required_gates: list[str]
    completed_gates: list[str]
    open_gates: list[str]
    locked_assets: list[dict[str, str | bool]]
    source_surfaces: list[dict[str, str | bool]]
    composition_source: dict[str, str | int | bool]
    storyboard_beats: list[dict[str, str | int]]
    production_rules: list[str]
    forbidden_claims: list[str]
    next_steps: list[str]


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _desktop_dogfood_dir() -> Path:
    return Path(os.environ.get("GARNET_PROMO_VIDEO_DESKTOP_DIR", str(Path.home() / "Desktop" / "dogfood")))


def _candidate_artifact_paths() -> tuple[list[Path], list[Path], list[Path]]:
    desktop = _desktop_dogfood_dir()
    rendered_candidates = [
        desktop / "garnet-promo-video" / "garnet-promo.mp4",
        desktop / "garnet-promo-video" / "garnet-promo.webm",
    ]
    visual_qa_candidates = [
        desktop / "garnet-promo-video-visual-qa" / "promo-visual-qa-data.json",
    ]
    website_candidates = [
        desktop / "garnet-promo-video-website-export" / "promo-website-export-data.json",
        desktop / "garnet-promo-video" / "website-export",
    ]
    return rendered_candidates, visual_qa_candidates, website_candidates


def _visual_qa_passed(path: Path) -> bool:
    if not path.is_file():
        return False
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False
    checks = data.get("checks", [])
    return (
        data.get("status") == "visual-qa-ready"
        and data.get("verdict") == "pass"
        and isinstance(checks, list)
        and bool(checks)
        and all(isinstance(check, dict) and check.get("passed") is True for check in checks)
    )


def _website_export_passed(path: Path) -> bool:
    if path.is_dir():
        return True
    if not path.is_file():
        return False
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False
    checks = data.get("checks", [])
    return (
        data.get("status") == "website-export-ready"
        and data.get("verdict") == "pass"
        and isinstance(checks, list)
        and bool(checks)
        and all(isinstance(check, dict) and check.get("passed") is True for check in checks)
    )


def _site_sync_passed(desktop: Path) -> bool:
    sync_data = desktop / "garnet-promo-video-site-sync" / "promo-site-sync-data.json"
    if not sync_data.is_file():
        return False
    try:
        data = json.loads(sync_data.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False
    checks = data.get("checks", [])
    return (
        data.get("status") == "public-site-embedded"
        and data.get("verdict") == "pass"
        and isinstance(checks, list)
        and bool(checks)
        and all(isinstance(check, dict) and check.get("passed") is True for check in checks)
    )


def _public_site_embed_passed(desktop: Path) -> bool:
    if not _site_sync_passed(desktop):
        return False
    site = ROOT / "docs" / "index.html"
    worker = ROOT / "docs" / "service-worker.js"
    assets = [
        ROOT / "docs" / "assets" / "garnet-promo.mp4",
        ROOT / "docs" / "assets" / "garnet-promo.webm",
        ROOT / "docs" / "assets" / "garnet-promo-poster.png",
    ]
    if not site.is_file() or not worker.is_file():
        return False
    if not all(path.is_file() and path.stat().st_size > 0 for path in assets):
        return False
    site_text = site.read_text(encoding="utf-8")
    worker_text = worker.read_text(encoding="utf-8")
    return all(
        token in site_text
        for token in (
            'id="promo"',
            'class="promo-video"',
            'poster="assets/garnet-promo-poster.png"',
            'src="assets/garnet-promo.webm"',
            'src="assets/garnet-promo.mp4"',
            "Public-site embedded",
            "human/aesthetic acceptance",
            "not full MIT/productization completion",
        )
    ) and all(
        token in worker_text
        for token in (
            "assets/garnet-promo.mp4",
            "assets/garnet-promo.webm",
            "assets/garnet-promo-poster.png",
        )
    )


def _asset_entry(id: str, path: Path, role: str, kind: str) -> dict[str, str | bool]:
    exists = path.is_file()
    return {
        "id": id,
        "role": role,
        "kind": kind,
        "path": str(path.relative_to(ROOT)),
        "exists": exists,
        "sha256": _sha256(path) if exists else "",
    }


def _surface_entry(id: str, path: Path, role: str, required_phrase: str) -> dict[str, str | bool]:
    exists = path.is_file()
    phrase_present = False
    if exists:
        phrase_present = required_phrase in path.read_text(encoding="utf-8")
    return {
        "id": id,
        "role": role,
        "path": str(path.relative_to(ROOT)),
        "exists": exists,
        "required_phrase": required_phrase,
        "phrase_present": phrase_present,
    }


def _locked_assets() -> list[dict[str, str | bool]]:
    return [
        _asset_entry(
            "root-logo",
            ROOT / "assets" / "garnet-logo.png",
            "canonical square Garnet brand mark for opening and closing shots",
            "image",
        ),
        _asset_entry(
            "studio-logo",
            ROOT
            / "apps"
            / "garnet-studio-macos"
            / "Sources"
            / "GarnetStudio"
            / "Resources"
            / "garnet-logo.png",
            "macOS workbench identity asset for product-surface shots",
            "image",
        ),
        _asset_entry(
            "pwa-icon-512",
            ROOT / "docs" / "icons" / "garnet-512.png",
            "website/PWA install identity asset for public-surface shots",
            "image",
        ),
        _asset_entry(
            "pwa-icon-192",
            ROOT / "docs" / "icons" / "garnet-192.png",
            "small website/PWA icon for mobile-sized product surfaces",
            "image",
        ),
    ]


def _source_surfaces() -> list[dict[str, str | bool]]:
    return [
        _surface_entry(
            "public-site",
            ROOT / "docs" / "index.html",
            "public website copy and install/PWA surface",
            "Objective accounting",
        ),
        _surface_entry(
            "studio-app",
            ROOT / "apps" / "garnet-studio-macos" / "Sources" / "GarnetStudio" / "GarnetStudioApp.swift",
            "macOS Studio workbench surface",
            "Garnet Studio",
        ),
        _surface_entry(
            "agentic-matrix",
            ROOT / "scripts" / "run_agentic_dogfood_matrix.py",
            "dogfood evidence surface for the evidence beat",
            "Run agent-facing Garnet dogfood",
        ),
        _surface_entry(
            "mit-status",
            ROOT / "scripts" / "garnet_mit_readiness_status.py",
            "machine-readable current truth for render copy",
            "MIT/productization objective",
        ),
    ]


def _composition_source() -> dict[str, str | int | bool]:
    composition_path = ROOT / "docs" / "promo" / "composition.html"
    design_path = ROOT / "docs" / "promo" / "DESIGN.md"
    exists = composition_path.is_file()
    design_exists = design_path.is_file()
    text = composition_path.read_text(encoding="utf-8") if exists else ""
    composition_id = "garnet-promo-main"
    timeline_registration = f'window.__timelines["{composition_id}"]'
    duration_token = 'data-duration="30"'
    uses_locked_assets = "../icons/garnet-512.png" in text
    return {
        "path": str(composition_path.relative_to(ROOT)),
        "design_contract_path": str(design_path.relative_to(ROOT)),
        "tool": "hyperframes-html",
        "exists": exists,
        "design_contract_exists": design_exists,
        "composition_id": composition_id,
        "duration_seconds": 30,
        "duration_declared": duration_token in text,
        "timeline_registered": timeline_registration in text,
        "uses_locked_assets": uses_locked_assets,
        "sha256": _sha256(composition_path) if exists else "",
        "design_sha256": _sha256(design_path) if design_exists else "",
    }


def read_status() -> PromoVideoStatus:
    rendered_candidates, visual_qa_candidates, website_candidates = _candidate_artifact_paths()
    desktop = _desktop_dogfood_dir()
    rendered_video_present = any(path.is_file() for path in rendered_candidates)
    visual_qa_present = rendered_video_present and any(_visual_qa_passed(path) for path in visual_qa_candidates)
    website_export_present = rendered_video_present and visual_qa_present and any(
        _website_export_passed(path) for path in website_candidates
    )
    public_site_embed_present = website_export_present and _public_site_embed_passed(desktop)
    locked_assets = _locked_assets()
    source_surfaces = _source_surfaces()
    composition_source = _composition_source()
    visual_identity_locked = all(bool(asset["exists"]) and bool(asset["sha256"]) for asset in locked_assets)
    source_surfaces_locked = all(
        bool(surface["exists"]) and bool(surface["phrase_present"])
        for surface in source_surfaces
    )
    composition_source_present = all(
        bool(composition_source[key])
        for key in (
            "exists",
            "design_contract_exists",
            "duration_declared",
            "timeline_registered",
            "uses_locked_assets",
            "sha256",
            "design_sha256",
        )
    )
    completed_gates = []
    if visual_identity_locked:
        completed_gates.append("visual identity lock")
    if source_surfaces_locked:
        completed_gates.append("source surface lock")
    completed_gates.append("30-second storyboard and shot list")
    if composition_source_present:
        completed_gates.append("HyperFrames or Remotion composition")
    if rendered_video_present:
        completed_gates.append("rendered MP4 or WebM artifact")
    if visual_qa_present:
        completed_gates.append("visual QA verdict")
    if website_export_present:
        completed_gates.append("website-ready export")
        completed_gates.append("Desktop dogfood evidence bundle")
    if public_site_embed_present:
        completed_gates.append("repo/site copy check for overclaims")

    required_gates = [
        "visual identity lock",
        "30-second storyboard and shot list",
        "HyperFrames or Remotion composition",
        "rendered MP4 or WebM artifact",
        "visual QA verdict",
        "website-ready export",
        "Desktop dogfood evidence bundle",
        "repo/site copy check for overclaims",
        "human/aesthetic acceptance",
    ]
    open_gates = [gate for gate in required_gates if gate not in completed_gates]
    status = "planned-contract"
    completion_percent = 25.0
    if status == "planned-contract" and rendered_video_present:
        status = "rendered-artifact-ready"
        completion_percent = 65.0
        open_gates = [gate for gate in required_gates if gate not in completed_gates]
    if status == "rendered-artifact-ready" and visual_qa_present:
        status = "visual-qa-ready"
        completion_percent = 80.0
        open_gates = [gate for gate in required_gates if gate not in completed_gates]
    if status == "visual-qa-ready" and website_export_present:
        status = "website-export-ready"
        completion_percent = 90.0
        open_gates = [gate for gate in required_gates if gate not in completed_gates]
    if status == "website-export-ready" and public_site_embed_present:
        status = "public-site-embedded"
        completion_percent = 95.0
        open_gates = [gate for gate in required_gates if gate not in completed_gates]
    if status == "planned-contract" and composition_source_present and visual_identity_locked and source_surfaces_locked:
        status = "composition-ready"
        completion_percent = 50.0
    if status == "planned-contract" and visual_identity_locked and source_surfaces_locked:
        status = "source-locked"
        completion_percent = 35.0

    return PromoVideoStatus(
        source=str(ROOT),
        status=status,
        completion_percent=completion_percent,
        target_duration_seconds=30,
        rendered_video_present=rendered_video_present,
        visual_qa_present=visual_qa_present,
        website_export_present=website_export_present,
        public_site_embed_present=public_site_embed_present,
        composition_source_present=composition_source_present,
        visual_identity_locked=visual_identity_locked,
        source_surfaces_locked=source_surfaces_locked,
        current_truth=[
            (
                "The verified promo export is embedded on the public site, but human/aesthetic acceptance remains open."
                if public_site_embed_present
                else "A website export package exists for the rendered and visual-QA-checked promo artifact, but public-site embedding remains open."
                if website_export_present
                else "A rendered MP4/WebM promo artifact has automated visual QA evidence, but website export remains open."
                if visual_qa_present
                else "A rendered MP4/WebM promo artifact is present, but visual QA and website export remain open."
                if rendered_video_present
                else "No verified rendered promo video is present."
            ),
            (
                "Public-site embedded evidence is present and manifest-ready; it is not final human acceptance."
                if public_site_embed_present
                else "No verified public-site promo embed is present."
                if website_export_present
                else "No verified website-ready promo export is present."
            ),
            (
                "Visual identity and source surfaces remain locked to real repo assets for visual QA and export."
                if rendered_video_present
                else "Visual identity and source surfaces are locked to real repo assets for the next render slice."
            ),
            (
                "A HyperFrames-compatible composition source exists for local rendering and future exports."
                if rendered_video_present
                else "A HyperFrames-compatible composition source exists for a later render slice."
            ),
            f"The video lane is {status}, not a completed marketing asset.",
        ],
        required_gates=required_gates,
        completed_gates=completed_gates,
        open_gates=open_gates,
        locked_assets=locked_assets,
        source_surfaces=source_surfaces,
        composition_source=composition_source,
        storyboard_beats=[
            {
                "id": "hook",
                "duration_seconds": 5,
                "purpose": "Open with Garnet as a dual-mode language for agents and humans.",
            },
            {
                "id": "evidence",
                "duration_seconds": 7,
                "purpose": "Show dogfood matrix, safe-mode checks, release integrity, and readiness evidence.",
            },
            {
                "id": "workbench",
                "duration_seconds": 7,
                "purpose": "Show Garnet Studio and the web/PWA path as local workbench surfaces.",
            },
            {
                "id": "assist",
                "duration_seconds": 6,
                "purpose": "Show converter assist as gated advisory planning, not active broad conversion.",
            },
            {
                "id": "close",
                "duration_seconds": 5,
                "purpose": "Close with MIT-readiness momentum and honest remaining gates.",
            },
        ],
        production_rules=[
            "Use current repo and Desktop dogfood evidence only.",
            "Prefer real product surfaces, terminal output, and generated readiness artifacts over abstract decoration.",
            "Keep claims aligned with `scripts/garnet_mit_readiness_status.py`.",
            "Do not use a rendered asset on the public site until visual QA and manifest verification pass.",
        ],
        forbidden_claims=[
            (
                "Do not claim the public-site embedded promo has final human/aesthetic acceptance."
                if public_site_embed_present
                else "Do not claim the promo artifact is embedded on the public site."
                if website_export_present
                else "Do not claim the rendered promo artifact is website-ready."
                if visual_qa_present
                else "Do not claim the rendered promo artifact is visual-QA-approved or website-ready."
                if rendered_video_present
                else "Do not claim a rendered promo video exists."
            ),
            "Do not claim notarized macOS distribution.",
            "Do not claim mobile distribution is active.",
            "Do not claim provider-backed LLM conversion is active.",
            "Do not claim full MIT/productization completion.",
        ],
        next_steps=[
            (
                "Complete human/aesthetic acceptance review before using the promo as final marketing creative."
                if public_site_embed_present
                else "Review and wire the website export package before public-site embedding."
                if website_export_present
                else "Review representative visual-QA frames before public-site embedding."
                if visual_qa_present
                else "Run visual QA against the rendered MP4/WebM outputs."
                if rendered_video_present
                else "Render MP4/WebM outputs and preserve them in Desktop dogfood."
            ),
            "Export website-ready promo assets only after rendered media and visual QA pass.",
        ],
    )


def render_markdown(status: PromoVideoStatus) -> str:
    lines = [
        "# Garnet Promo Video Readiness Contract",
        "",
        f"Source: `{status.source}`",
        f"Status: **{status.status}**",
        f"Completion: **{status.completion_percent:.1f}%**",
        f"Target duration: **{status.target_duration_seconds} seconds**",
        "",
        "## Current Truth",
        "",
    ]
    lines.extend(f"- {item}" for item in status.current_truth)

    lines.extend(["", "## Required Gates", ""])
    lines.extend(f"- {gate}" for gate in status.required_gates)

    lines.extend(["", "## Completed Gates", ""])
    lines.extend(f"- {gate}" for gate in status.completed_gates)

    lines.extend(["", "## Open Gates", ""])
    lines.extend(f"- {gate}" for gate in status.open_gates)

    lines.extend(["", "## Visual Identity Lock", ""])
    for asset in status.locked_assets:
        marker = "locked" if asset["exists"] and asset["sha256"] else "missing"
        lines.append(
            f"- **{asset['id']}** ({marker}): `{asset['path']}`"
        )

    lines.extend(["", "## Source Surface Lock", ""])
    for surface in status.source_surfaces:
        marker = "locked" if surface["exists"] and surface["phrase_present"] else "missing"
        lines.append(
            f"- **{surface['id']}** ({marker}): `{surface['path']}`"
        )

    lines.extend(["", "## Composition Source", ""])
    composition = status.composition_source
    marker = "ready" if status.composition_source_present else "missing"
    lines.append(
        f"- **{composition['composition_id']}** ({marker}): `{composition['path']}`"
    )
    lines.append(f"- Design contract: `{composition['design_contract_path']}`")
    lines.append(f"- Tool: `{composition['tool']}`")
    lines.append(f"- Duration: {composition['duration_seconds']} seconds")

    lines.extend(["", "## Storyboard Contract", ""])
    for beat in status.storyboard_beats:
        lines.append(
            f"- **{beat['id']}** ({beat['duration_seconds']}s): {beat['purpose']}"
        )

    lines.extend(["", "## Production Rules", ""])
    lines.extend(f"- {rule}" for rule in status.production_rules)

    lines.extend(["", "## Forbidden Claims", ""])
    lines.extend(f"- {claim}" for claim in status.forbidden_claims)

    lines.extend(["", "## Next Steps", ""])
    lines.extend(f"- {step}" for step in status.next_steps)
    return "\n".join(lines) + "\n"


def write_output_dir(status: PromoVideoStatus, output_dir: Path) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    json_path = output_dir / "garnet-promo-video-status.json"
    md_path = output_dir / "garnet-promo-video-status.md"
    manifest_path = output_dir / "MANIFEST.sha256"

    json_path.write_text(json.dumps(asdict(status), indent=2) + "\n", encoding="utf-8")
    md_path.write_text(render_markdown(status), encoding="utf-8")
    entries = []
    for path in (json_path, md_path):
        entries.append(f"{_sha256(path)}  {path.name}\n")
    manifest_path.write_text("".join(entries), encoding="utf-8")


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--format",
        choices=("markdown", "json"),
        default="markdown",
        help="output format",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        help="write JSON, Markdown, and MANIFEST.sha256 into this directory",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    status = read_status()
    if args.output_dir:
        write_output_dir(status, args.output_dir)

    if args.format == "json":
        print(json.dumps(asdict(status), indent=2))
    else:
        print(render_markdown(status), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
