#!/usr/bin/env python3
"""Summarize Garnet Studio notarization preflight evidence.

The shell preflight writes detailed artifacts. This reporter turns that bundle
into a stable JSON/Markdown status surface for agents, PR bodies, and the public
readiness story without submitting to Apple or claiming notarization.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class Check:
    status: str
    label: str
    evidence: str
    recommendation: str


@dataclass(frozen=True)
class NotarizationStatus:
    source: str
    overall_status: str
    blocker_count: int
    warning_count: int
    current_truth: list[str]
    blockers: list[Check]
    warnings: list[Check]
    passes: list[Check]
    next_actions: list[str]
    manifest_present: bool
    manifest_verification_log_present: bool
    credential_values_redacted: bool
    app_path: str | None
    dmg_path: str | None


def parse_env(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    if not path.is_file():
        return values
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        if not raw_line or raw_line.startswith("#") or "=" not in raw_line:
            continue
        key, value = raw_line.split("=", 1)
        values[key] = value
    return values


def parse_checks(path: Path) -> list[Check]:
    checks: list[Check] = []
    for line_no, raw_line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if not raw_line.strip():
            continue
        parts = raw_line.split("\t")
        if len(parts) != 4:
            raise ValueError(f"malformed checks.tsv line {line_no}: expected 4 tab-separated fields")
        checks.append(Check(*parts))
    return checks


def find_latest_bundle() -> Path:
    candidates: list[Path] = []
    roots = [
        ROOT / "target" / "macos",
        Path.home() / "Desktop" / "dogfood",
    ]
    for root in roots:
        if root.is_dir():
            candidates.extend(root.glob("garnet-studio-notarization-preflight-*"))
    if not candidates:
        raise FileNotFoundError(
            "preflight bundle not found: run scripts/preflight_garnet_studio_notarization.sh --copy-to-desktop"
        )
    return max(candidates, key=lambda path: path.stat().st_mtime)


def read_status(bundle: Path | None = None) -> NotarizationStatus:
    bundle = find_latest_bundle() if bundle is None else bundle
    if not bundle.is_dir():
        raise FileNotFoundError(f"preflight bundle not found: {bundle}")

    checks_path = bundle / "checks.tsv"
    if not checks_path.is_file():
        raise FileNotFoundError(f"preflight checks not found: {checks_path}")

    data = parse_env(bundle / "notarization-preflight-data.env")
    checks = parse_checks(checks_path)
    blockers = [check for check in checks if check.status == "blocker"]
    warnings = [check for check in checks if check.status == "warning"]
    passes = [check for check in checks if check.status == "pass"]

    if blockers:
        overall_status = "blocked"
    elif warnings:
        overall_status = "warnings"
    else:
        overall_status = "ready-for-notary-submit"

    next_actions = []
    for check in blockers + warnings:
        if check.recommendation != "None." and check.recommendation not in next_actions:
            next_actions.append(check.recommendation)

    return NotarizationStatus(
        source=str(bundle),
        overall_status=overall_status,
        blocker_count=len(blockers),
        warning_count=len(warnings),
        current_truth=[
            "preflight only",
            "does not submit to Apple",
            "does not claim notarization",
            "Developer ID and notary credentials remain external prerequisites",
        ],
        blockers=blockers,
        warnings=warnings,
        passes=passes,
        next_actions=next_actions,
        manifest_present=(bundle / "MANIFEST.sha256").is_file(),
        manifest_verification_log_present=(bundle / "MANIFEST.verify.log").is_file(),
        credential_values_redacted=True,
        app_path=data.get("app_path"),
        dmg_path=data.get("dmg_path"),
    )


def render_markdown(status: NotarizationStatus) -> str:
    lines = [
        "# Garnet Studio Notarization Status",
        "",
        f"Source: `{status.source}`",
        "",
        f"Overall status: **{status.overall_status}**",
        f"Blockers: **{status.blocker_count}**",
        f"Warnings: **{status.warning_count}**",
        "",
        "Current truth: this is preflight evidence only. This is not a notarization claim.",
        "",
    ]

    if status.app_path:
        lines.append(f"App: `{status.app_path}`")
    if status.dmg_path:
        lines.append(f"DMG: `{status.dmg_path}`")
    if status.app_path or status.dmg_path:
        lines.append("")

    def add_section(title: str, checks: list[Check]) -> None:
        lines.extend([f"## {title}", ""])
        if not checks:
            lines.extend(["None.", ""])
            return
        lines.extend(["| Check | Evidence | Recommendation |", "| --- | --- | --- |"])
        for check in checks:
            lines.append(f"| {check.label} | `{check.evidence}` | {check.recommendation} |")
        lines.append("")

    add_section("Blockers", status.blockers)
    add_section("Warnings", status.warnings)

    lines.extend(["## Next Actions", ""])
    if status.next_actions:
        lines.extend(f"- {action}" for action in status.next_actions)
    else:
        lines.append("- Submit with notarytool, staple, and preserve a new evidence bundle.")
    lines.extend(
        [
            "",
            "## Evidence Integrity",
            "",
            f"- Manifest present: `{str(status.manifest_present).lower()}`",
            f"- Manifest verification log present: `{str(status.manifest_verification_log_present).lower()}`",
            "- Credential values redacted: `true`",
        ]
    )
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle", type=Path, help="Preflight bundle to summarize")
    parser.add_argument("--format", choices=("markdown", "json"), default="markdown")
    args = parser.parse_args(argv)

    try:
        status = read_status(args.bundle)
    except (FileNotFoundError, ValueError) as exc:
        print(str(exc), file=sys.stderr)
        return 1

    if args.format == "json":
        print(json.dumps(asdict(status), indent=2))
    else:
        print(render_markdown(status), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
