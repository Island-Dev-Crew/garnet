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
    website_export_present: bool
    visual_identity_locked: bool
    source_surfaces_locked: bool
    current_truth: list[str]
    required_gates: list[str]
    completed_gates: list[str]
    open_gates: list[str]
    locked_assets: list[dict[str, str | bool]]
    source_surfaces: list[dict[str, str | bool]]
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


def _candidate_artifact_paths() -> tuple[list[Path], list[Path]]:
    desktop = Path.home() / "Desktop" / "dogfood"
    rendered_candidates = [
        ROOT / "docs" / "assets" / "garnet-promo.mp4",
        ROOT / "docs" / "assets" / "garnet-promo.webm",
        desktop / "garnet-promo-video" / "garnet-promo.mp4",
        desktop / "garnet-promo-video" / "garnet-promo.webm",
    ]
    website_candidates = [
        ROOT / "docs" / "assets" / "garnet-promo-poster.png",
        ROOT / "docs" / "promo" / "index.html",
        desktop / "garnet-promo-video" / "website-export",
    ]
    return rendered_candidates, website_candidates


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


def read_status() -> PromoVideoStatus:
    rendered_candidates, website_candidates = _candidate_artifact_paths()
    rendered_video_present = any(path.is_file() for path in rendered_candidates)
    website_export_present = any(path.exists() for path in website_candidates)
    locked_assets = _locked_assets()
    source_surfaces = _source_surfaces()
    visual_identity_locked = all(bool(asset["exists"]) and bool(asset["sha256"]) for asset in locked_assets)
    source_surfaces_locked = all(
        bool(surface["exists"]) and bool(surface["phrase_present"])
        for surface in source_surfaces
    )
    completed_gates = []
    if visual_identity_locked:
        completed_gates.append("visual identity lock")
    if source_surfaces_locked:
        completed_gates.append("source surface lock")
    completed_gates.append("30-second storyboard and shot list")

    required_gates = [
        "visual identity lock",
        "30-second storyboard and shot list",
        "HyperFrames or Remotion composition",
        "rendered MP4 or WebM artifact",
        "visual QA verdict",
        "website-ready export",
        "Desktop dogfood evidence bundle",
        "repo/site copy check for overclaims",
    ]
    open_gates = [gate for gate in required_gates if gate not in completed_gates]
    status = "verified" if rendered_video_present and website_export_present else "planned-contract"
    completion_percent = 100.0 if rendered_video_present and website_export_present else 25.0
    if status == "planned-contract" and visual_identity_locked and source_surfaces_locked:
        status = "source-locked"
        completion_percent = 35.0

    return PromoVideoStatus(
        source=str(ROOT),
        status=status,
        completion_percent=completion_percent,
        target_duration_seconds=30,
        rendered_video_present=rendered_video_present,
        website_export_present=website_export_present,
        visual_identity_locked=visual_identity_locked,
        source_surfaces_locked=source_surfaces_locked,
        current_truth=[
            "No verified rendered promo video is present.",
            "No verified website-ready promo export is present.",
            "Visual identity and source surfaces are locked to real repo assets for the next render slice.",
            "HyperFrames or Remotion can be used in a later rendering slice.",
            "The video lane is source-locked, not a completed marketing asset.",
        ],
        required_gates=required_gates,
        completed_gates=completed_gates,
        open_gates=open_gates,
        locked_assets=locked_assets,
        source_surfaces=source_surfaces,
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
            "Do not claim a rendered promo video exists.",
            "Do not claim notarized macOS distribution.",
            "Do not claim mobile distribution is active.",
            "Do not claim provider-backed LLM conversion is active.",
            "Do not claim full MIT/productization completion.",
        ],
        next_steps=[
            "Create the HyperFrames or Remotion composition in a separate rendering slice.",
            "Render MP4/WebM outputs and preserve them in Desktop dogfood.",
            "Run visual QA before embedding or linking the video from the website.",
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
