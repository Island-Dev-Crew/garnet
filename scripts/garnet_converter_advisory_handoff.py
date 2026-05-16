#!/usr/bin/env python3
"""Create a provider-neutral Garnet converter advisory handoff packet.

This script is the last local packaging step before a human chooses to hand
converter advisory context to an agent or model. It consumes a manifested
advisory bundle plus the review-gate output, emits a no-source handoff packet,
and refuses to promote blocked reviews. It does not call a provider, execute
source code, or enable conversion.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class ConverterAdvisoryHandoff:
    source: str
    bundle_dir: str
    review_dir: str
    status: str
    bundle_status: str
    review_status: str
    language: str
    language_id: str
    source_sha256: str
    source_included: bool
    source_privacy_passed: bool
    manifest_verified: bool
    provider_boundary_passed: bool
    conversion_boundary_passed: bool
    required_gates_present: bool
    provider_backed_conversion_allowed: bool
    conversion_active: bool
    model_called: bool
    network_required: bool
    human_review_required: bool
    allowed_handoff_use: str
    blockers: list[str]
    required_before_model_or_agent: list[str]
    forbidden_claims: list[str]
    included_artifacts: list[str]
    handoff_prompt: str


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_manifest(output_dir: Path) -> None:
    manifest_path = output_dir / "MANIFEST.sha256"
    verify_path = output_dir / "MANIFEST.verify.log"
    files = sorted(
        path
        for path in output_dir.rglob("*")
        if path.is_file() and path.name not in {manifest_path.name, verify_path.name}
    )
    manifest_path.write_text(
        "\n".join(f"{file_sha256(path)}  {path.relative_to(output_dir)}" for path in files) + "\n",
        encoding="utf-8",
    )
    verify_path.write_text(
        "".join(f"{path.relative_to(output_dir)}: OK\n" for path in files),
        encoding="utf-8",
    )


def _prompt(bundle: dict[str, Any], review: dict[str, Any], blockers: list[str]) -> str:
    language = str(bundle.get("language", review.get("language", "unknown")))
    source_sha = str(
        bundle.get("source_summary", {}).get("sha256", review.get("source_sha256", "unknown"))
    )
    status_line = (
        "The reviewed bundle is ready for provider-neutral advisory notes only."
        if not blockers
        else "The reviewed bundle is blocked; do not hand it to a model or agent yet."
    )
    required_sections = bundle.get("required_output_sections", [])
    section_lines = "\n".join(f"- {section}" for section in required_sections)
    blocker_lines = "\n".join(f"- {blocker}" for blocker in (blockers or ["none"]))
    return "\n".join(
        [
            "# Garnet Converter Advisory Handoff Prompt",
            "",
            status_line,
            "",
            f"Language: {language}",
            f"Source SHA-256: {source_sha}",
            "Source text: omitted from this packet by design.",
            "",
            "## Required Output Sections",
            "",
            section_lines,
            "",
            "## Required Before Any Candidate Output Is Trusted",
            "",
            "- lineage per emitted node",
            "- @sandbox by default",
            "- migrate_todo evidence for unsupported constructs",
            "- garnet check",
            "- dogfood evidence bundle",
            "- human audit before unquarantine",
            "",
            "## Blockers",
            "",
            blocker_lines,
            "",
            "## Hard Boundary",
            "",
            "Do not claim autonomous conversion, provider-backed conversion, broad planned-language frontend support, or safety before lineage, sandboxing, garnet check, dogfood evidence, and human audit are complete.",
            "",
        ]
    )


def read_handoff(bundle_dir: Path, review_dir: Path) -> ConverterAdvisoryHandoff:
    bundle_dir = bundle_dir.expanduser().resolve()
    review_dir = review_dir.expanduser().resolve()
    bundle = load_json(bundle_dir / "garnet-converter-advisory-bundle.json")
    review = load_json(review_dir / "garnet-converter-advisory-review.json")

    review_blockers = [str(item) for item in review.get("blockers", [])]
    blockers = list(review_blockers)
    if review.get("status") != "ready-for-human-advisory-review":
        blockers.append(f"review status is {review.get('status')}")
    if bundle.get("source_included") or review.get("source_included"):
        if "source text included" not in blockers:
            blockers.append("source text included")
    if bundle.get("conversion_active") or not review.get("conversion_boundary_passed", False):
        blockers.append("conversion boundary failed")
    if review.get("provider_backed_conversion_allowed"):
        blockers.append("provider-backed conversion unexpectedly allowed")
    if not review.get("manifest_verified", False):
        blockers.append("review manifest not verified")

    status = (
        "blocked-advisory-handoff"
        if blockers
        else "ready-for-provider-neutral-advisory-handoff"
    )
    source_summary = bundle.get("source_summary", {})
    source_sha = str(source_summary.get("sha256", review.get("source_sha256", "unknown")))
    prompt = _prompt(bundle, review, blockers)
    return ConverterAdvisoryHandoff(
        source=str(Path(__file__).resolve().parents[1]),
        bundle_dir=str(bundle_dir),
        review_dir=str(review_dir),
        status=status,
        bundle_status=str(bundle.get("status", "unknown")),
        review_status=str(review.get("status", "unknown")),
        language=str(bundle.get("language", review.get("language", "unknown"))),
        language_id=str(bundle.get("language_id", review.get("language_id", "unknown"))),
        source_sha256=source_sha,
        source_included=bool(bundle.get("source_included") or review.get("source_included")),
        source_privacy_passed=bool(review.get("source_privacy_passed")),
        manifest_verified=bool(review.get("manifest_verified")),
        provider_boundary_passed=bool(review.get("provider_boundary_passed")),
        conversion_boundary_passed=bool(review.get("conversion_boundary_passed")),
        required_gates_present=bool(review.get("required_gates_present")),
        provider_backed_conversion_allowed=False,
        conversion_active=False,
        model_called=False,
        network_required=False,
        human_review_required=True,
        allowed_handoff_use="provider-neutral advisory notes only; deterministic converter output remains authoritative",
        blockers=blockers,
        required_before_model_or_agent=[
            "lineage per emitted node",
            "@sandbox by default",
            "migrate_todo evidence",
            "garnet check",
            "dogfood evidence bundle",
            "human audit before unquarantine",
        ],
        forbidden_claims=[
            "autonomous conversion is enabled",
            "provider-backed conversion is active",
            "broad planned-language frontend support is complete",
            "candidate output is safe before lineage, sandbox, garnet check, dogfood, and human audit",
        ],
        included_artifacts=[
            "garnet-converter-advisory-bundle.json",
            "garnet-converter-advisory-review.json",
            "garnet-converter-advisory-handoff.md",
        ],
        handoff_prompt=prompt,
    )


def render_markdown(handoff: ConverterAdvisoryHandoff) -> str:
    lines = [
        "# Garnet Converter Advisory Handoff Packet",
        "",
        f"Source: `{handoff.source}`",
        f"Bundle: `{handoff.bundle_dir}`",
        f"Review: `{handoff.review_dir}`",
        f"Status: **{handoff.status}**",
        f"Language: **{handoff.language}**",
        f"Source SHA-256: `{handoff.source_sha256}`",
        "",
        "## Boundary",
        "",
        f"- Bundle status: `{handoff.bundle_status}`",
        f"- Review status: `{handoff.review_status}`",
        f"- Source included: {str(handoff.source_included).lower()}",
        f"- Provider-backed conversion allowed: {str(handoff.provider_backed_conversion_allowed).lower()}",
        f"- Conversion active: {str(handoff.conversion_active).lower()}",
        f"- Model called: {str(handoff.model_called).lower()}",
        f"- Network required: {str(handoff.network_required).lower()}",
        f"- Allowed handoff use: {handoff.allowed_handoff_use}",
        "",
        "## Blockers",
        "",
    ]
    lines.extend(f"- {item}" for item in (handoff.blockers or ["none"]))
    lines.extend(["", "## Required Before Model Or Agent", ""])
    lines.extend(f"- {item}" for item in handoff.required_before_model_or_agent)
    lines.extend(["", "## Forbidden Claims", ""])
    lines.extend(f"- {item}" for item in handoff.forbidden_claims)
    lines.extend(["", "## Handoff Prompt", "", handoff.handoff_prompt])
    return "\n".join(lines)


def write_outputs(handoff: ConverterAdvisoryHandoff, output_dir: Path) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    (output_dir / "garnet-converter-advisory-handoff.json").write_text(
        json.dumps(asdict(handoff), indent=2) + "\n",
        encoding="utf-8",
    )
    (output_dir / "garnet-converter-advisory-handoff.md").write_text(
        render_markdown(handoff),
        encoding="utf-8",
    )
    write_manifest(output_dir)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle-dir", type=Path, required=True, help="advisory bundle directory")
    parser.add_argument("--review-dir", type=Path, required=True, help="advisory review directory")
    parser.add_argument("--format", choices=("markdown", "json"), default="markdown")
    parser.add_argument("--output-dir", type=Path, help="write handoff JSON/Markdown plus MANIFEST.sha256")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        handoff = read_handoff(args.bundle_dir, args.review_dir)
    except (json.JSONDecodeError, OSError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    if args.output_dir:
        write_outputs(handoff, args.output_dir.expanduser().resolve())

    if args.format == "json":
        print(json.dumps(asdict(handoff), indent=2))
    else:
        print(render_markdown(handoff), end="")
    return 0 if not handoff.blockers else 1


if __name__ == "__main__":
    raise SystemExit(main())
