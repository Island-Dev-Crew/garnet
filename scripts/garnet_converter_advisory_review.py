#!/usr/bin/env python3
"""Review a Garnet converter advisory bundle before agent/model handoff.

This is a provider-neutral safety gate. It validates the manifested advisory
bundle produced by `garnet_converter_advisory_bundle.py`, confirms that the
default no-source boundary is intact, and emits a human-review checklist. It
does not call a provider, execute source code, or enable conversion.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

REQUIRED_BUNDLE_FILES = (
    "garnet-converter-advisory-bundle.json",
    "garnet-converter-advisory-bundle.md",
    "garnet-converter-advisory-request.md",
    "MANIFEST.sha256",
    "MANIFEST.verify.log",
)

REQUIRED_GATE_MARKERS = (
    "lineage",
    "@sandbox",
    "garnet check",
    "dogfood",
    "human audit",
)

REQUIRED_BEFORE_UNQUARANTINE = (
    "review lineage per emitted node",
    "keep candidate output under @sandbox by default",
    "run garnet check on any candidate output",
    "attach dogfood evidence",
    "complete human audit before unquarantine",
)


@dataclass(frozen=True)
class ConverterAdvisoryReview:
    source: str
    bundle_dir: str
    status: str
    manifest_verified: bool
    source_privacy_passed: bool
    provider_boundary_passed: bool
    conversion_boundary_passed: bool
    required_gates_present: bool
    human_review_required: bool
    provider_backed_conversion_allowed: bool
    source_included: bool
    language: str
    language_id: str
    source_sha256: str
    blockers: list[str]
    required_before_unquarantine: list[str]
    allowed_next_actions: list[str]
    forbidden_claims: list[str]
    manifest_check_log: str


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def verify_manifest(bundle_dir: Path) -> tuple[bool, str]:
    manifest = bundle_dir / "MANIFEST.sha256"
    if not manifest.exists():
        return False, "MANIFEST.sha256 missing"
    result = subprocess.run(
        ["shasum", "-a", "256", "-c", "MANIFEST.sha256"],
        cwd=bundle_dir,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    return result.returncode == 0, result.stdout


def read_review(bundle_dir: Path, *, allow_source_included: bool = False) -> ConverterAdvisoryReview:
    bundle_dir = bundle_dir.expanduser().resolve()
    blockers: list[str] = []
    for filename in REQUIRED_BUNDLE_FILES:
        if not (bundle_dir / filename).exists():
            blockers.append(f"missing {filename}")

    data: dict[str, Any] = {}
    bundle_json = bundle_dir / "garnet-converter-advisory-bundle.json"
    if bundle_json.exists():
        data = load_json(bundle_json)

    manifest_verified, manifest_log = verify_manifest(bundle_dir)
    if not manifest_verified:
        blockers.append("manifest verification failed")

    source_included = bool(data.get("source_included"))
    source_privacy_passed = (
        (not source_included and data.get("source_text") is None) or allow_source_included
    )
    if not source_privacy_passed:
        blockers.append("source text included")

    provider_boundary_passed = not any(
        bool(data.get(field))
        for field in ("provider_required", "model_required", "network_required")
    )
    if not provider_boundary_passed:
        blockers.append("provider/model/network boundary failed")

    conversion_boundary_passed = not bool(data.get("conversion_active"))
    if not conversion_boundary_passed:
        blockers.append("conversion marked active")

    human_review_required = bool(data.get("human_review_required"))
    if not human_review_required:
        blockers.append("human review requirement missing")

    gates_blob = "\n".join(
        str(item)
        for item in (
            data.get("assist_plan", {}).get("required_gates", [])
            + data.get("current_truth", [])
            + data.get("required_output_sections", [])
        )
    ).lower()
    required_gates_present = all(marker.lower() in gates_blob for marker in REQUIRED_GATE_MARKERS)
    if not required_gates_present:
        blockers.append("required lineage/sandbox/check/dogfood/human-audit gates missing")

    if blockers == ["source text included"]:
        status = "blocked-source-included"
    elif blockers:
        status = "blocked-advisory-review"
    else:
        status = "ready-for-human-advisory-review"

    source_summary = data.get("source_summary", {})
    return ConverterAdvisoryReview(
        source=str(Path(__file__).resolve().parents[1]),
        bundle_dir=str(bundle_dir),
        status=status,
        manifest_verified=manifest_verified,
        source_privacy_passed=source_privacy_passed,
        provider_boundary_passed=provider_boundary_passed,
        conversion_boundary_passed=conversion_boundary_passed,
        required_gates_present=required_gates_present,
        human_review_required=human_review_required,
        provider_backed_conversion_allowed=False,
        source_included=source_included,
        language=str(data.get("language", "unknown")),
        language_id=str(data.get("language_id", "unknown")),
        source_sha256=str(source_summary.get("sha256", "unknown")),
        blockers=blockers,
        required_before_unquarantine=list(REQUIRED_BEFORE_UNQUARANTINE),
        allowed_next_actions=[
            "perform local human review",
            "ask an agent/model for advisory notes only after privacy approval",
            "keep deterministic converter output authoritative",
            "turn accepted suggestions into separate tested implementation slices",
        ],
        forbidden_claims=[
            "provider-backed conversion is active",
            "autonomous conversion is enabled",
            "broad planned-language frontend support is complete",
            "candidate output is safe before lineage, sandbox, garnet check, dogfood, and human audit",
        ],
        manifest_check_log=manifest_log,
    )


def render_markdown(review: ConverterAdvisoryReview) -> str:
    lines = [
        "# Garnet Converter Advisory Review Gate",
        "",
        f"Source: `{review.source}`",
        f"Bundle: `{review.bundle_dir}`",
        f"Status: **{review.status}**",
        f"Language: **{review.language}**",
        f"Source SHA-256: `{review.source_sha256}`",
        "",
        "## Safety Boundary",
        "",
        f"- Manifest verified: {str(review.manifest_verified).lower()}",
        f"- Source privacy passed: {str(review.source_privacy_passed).lower()}",
        f"- Source included: {str(review.source_included).lower()}",
        f"- Provider boundary passed: {str(review.provider_boundary_passed).lower()}",
        f"- Conversion boundary passed: {str(review.conversion_boundary_passed).lower()}",
        f"- Required gates present: {str(review.required_gates_present).lower()}",
        f"- Human review required: {str(review.human_review_required).lower()}",
        f"- Provider-backed conversion allowed: {str(review.provider_backed_conversion_allowed).lower()}",
        "",
        "## Blockers",
        "",
    ]
    lines.extend(f"- {item}" for item in (review.blockers or ["none"]))
    lines.extend(["", "## Required Before Unquarantine", ""])
    lines.extend(f"- {item}" for item in review.required_before_unquarantine)
    lines.extend(["", "## Allowed Next Actions", ""])
    lines.extend(f"- {item}" for item in review.allowed_next_actions)
    lines.extend(["", "## Forbidden Claims", ""])
    lines.extend(f"- {item}" for item in review.forbidden_claims)
    lines.extend(
        [
            "",
            (
                "This gate permits advisory migration notes only. It preserves "
                "the requirement for lineage, `@sandbox`, `migrate_todo`, "
                "`garnet check`, dogfood evidence, and human audit before "
                "unquarantine."
            ),
            "",
        ]
    )
    return "\n".join(lines)


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
    manifest_lines = [
        f"{file_sha256(path)}  {path.relative_to(output_dir)}"
        for path in files
    ]
    manifest_path.write_text("\n".join(manifest_lines) + "\n", encoding="utf-8")
    verify_path.write_text(
        "".join(f"{path.relative_to(output_dir)}: OK\n" for path in files),
        encoding="utf-8",
    )


def write_outputs(review: ConverterAdvisoryReview, output_dir: Path) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    (output_dir / "garnet-converter-advisory-review.json").write_text(
        json.dumps(asdict(review), indent=2) + "\n",
        encoding="utf-8",
    )
    (output_dir / "garnet-converter-advisory-review.md").write_text(
        render_markdown(review),
        encoding="utf-8",
    )
    write_manifest(output_dir)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle-dir", type=Path, required=True, help="advisory bundle directory")
    parser.add_argument(
        "--allow-source-included",
        action="store_true",
        help="permit review of a bundle that explicitly embedded source text",
    )
    parser.add_argument("--format", choices=("markdown", "json"), default="markdown")
    parser.add_argument("--output-dir", type=Path, help="write review JSON/Markdown plus MANIFEST.sha256")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        review = read_review(args.bundle_dir, allow_source_included=args.allow_source_included)
    except (json.JSONDecodeError, OSError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    if args.output_dir:
        write_outputs(review, args.output_dir.expanduser().resolve())

    if args.format == "json":
        print(json.dumps(asdict(review), indent=2))
    else:
        print(render_markdown(review), end="")
    return 0 if not review.blockers else 1


if __name__ == "__main__":
    raise SystemExit(main())
