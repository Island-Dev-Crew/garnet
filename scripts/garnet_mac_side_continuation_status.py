#!/usr/bin/env python3
"""Report Mac-side Garnet continuation lanes that can move without blocked credentials.

This reporter is intentionally narrower than `garnet_mit_readiness_status.py`.
It answers: what can an agent keep doing from this macOS checkout while Apple
Developer ID identity verification and Windows/Linux runtime work are blocked
or delegated elsewhere?
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import garnet_mit_readiness_status  # noqa: E402


@dataclass(frozen=True)
class MacContinuationLane:
    id: str
    label: str
    status: str
    category: str
    mac_actionable: bool
    evidence: str
    next_slice: str
    blocked_by: list[str]


@dataclass(frozen=True)
class MacContinuationStatus:
    source: str
    objective_completion_percent: float
    current_truth: list[str]
    lanes: list[MacContinuationLane]


def read_status() -> MacContinuationStatus:
    mit = garnet_mit_readiness_status.read_status()
    lanes = [
        MacContinuationLane(
            id="reusable_dogfood_skill",
            label="Reusable dogfood-readiness skill",
            status="published",
            category="repo/tooling",
            mac_actionable=True,
            evidence=(
                "Navigata1/dogfood-readiness is the standalone portable skill/toolkit "
                "for evidence-first PR and product readiness gates."
            ),
            next_slice="Use it as the gate for future Garnet Mac-side PRs and non-Garnet projects.",
            blocked_by=[],
        ),
        MacContinuationLane(
            id="macos_studio_unsigned_quality",
            label="macOS Studio unsigned/local quality",
            status="active-partial",
            category="macOS productization",
            mac_actionable=True,
            evidence=(
                "Local SwiftPM tests, DMG packaging, mounted-copy smoke, packaged PWA smoke, "
                "and agentic dogfood bundles can keep improving without Developer ID credentials."
            ),
            next_slice="Strengthen first-run UX, bundled evidence export, and unsigned DMG smoke before signing.",
            blocked_by=[],
        ),
        MacContinuationLane(
            id="apple_developer_id",
            label="Apple Developer ID notarization",
            status="blocked-external",
            category="account-holder gate",
            mac_actionable=False,
            evidence=(
                "Repo preflight can record missing signing identity and notary profile, "
                "but Apple account identity verification must be resolved by the account holder."
            ),
            next_slice="Contact Apple Developer Support for alternate identity verification; resume notarization after certificates exist.",
            blocked_by=[
                "Apple Developer Program identity verification",
                "Developer ID Application certificate",
                "notarytool profile",
            ],
        ),
        MacContinuationLane(
            id="website_status_presentation",
            label="Website, status, and MIT presentation surfaces",
            status="active-partial",
            category="public narrative",
            mac_actionable=True,
            evidence=(
                "Landing/status split, promo composition source, public-site embed evidence, "
                "and objective reporter are all local Mac-side repo surfaces."
            ),
            next_slice="Tighten hook copy, presentation deck, and demo route while preserving status-page truth boundaries.",
            blocked_by=["human/aesthetic acceptance review"],
        ),
        MacContinuationLane(
            id="converter_advisory",
            label="Converter advisory and provider-neutral LLM planning",
            status="active-partial",
            category="converter/product adoption",
            mac_actionable=True,
            evidence=(
                "Active conversion remains Rust/Ruby/Python/Go; broader languages have "
                "assist plans, advisory bundles, review gates, and handoff packets."
            ),
            next_slice="Improve local advisory review quality and fixtures without calling providers or claiming active LLM conversion.",
            blocked_by=["provider-backed runtime boundary", "deterministic frontend slices"],
        ),
        MacContinuationLane(
            id="proof_benchmark_empirics",
            label="Proof, benchmark, and empirical evidence",
            status="active-partial",
            category="research evidence",
            mac_actionable=True,
            evidence=(
                "`scripts/garnet_proof_benchmark_status.py` now inventories "
                "Criterion benchmark harnesses and research protocols while keeping "
                "measurements, mechanized proof, and external studies unclaimed."
            ),
            next_slice="Run one benchmark no-run or measurement evidence bundle at a time, tied to machine metadata and dogfood evidence.",
            blocked_by=["mechanized proof depth", "fresh empirical validation budget"],
        ),
        MacContinuationLane(
            id="windows_linux_studio",
            label="Windows/Linux Studio",
            status="handoff-only",
            category="delegated platform",
            mac_actionable=False,
            evidence=(
                "A Windows/Linux Studio handoff packet exists; runtime proof belongs on those target systems."
            ),
            next_slice="Keep Mac-side docs aligned, but do not claim Windows/Linux runtime completion from macOS.",
            blocked_by=["Windows runtime execution", "Linux runtime execution"],
        ),
    ]

    return MacContinuationStatus(
        source=str(ROOT),
        objective_completion_percent=mit.completion_percent,
        current_truth=[
            "Mac-side work can continue without Apple Developer ID credentials",
            "Developer ID notarization is externally blocked and must not be claimed",
            "Windows/Linux Studio runtime proof remains delegated to target systems",
            "provider-backed LLM conversion and native backend lowering remain unimplemented",
        ],
        lanes=lanes,
    )


def render_markdown(status: MacContinuationStatus) -> str:
    lines = [
        "# Garnet Mac-Side Continuation Status",
        "",
        f"Source: `{status.source}`",
        f"Overall MIT/productization objective: **{status.objective_completion_percent:.1f}%**",
        "",
        "## Current Truth",
        "",
        *[f"- {truth}" for truth in status.current_truth],
        "",
        "| Lane | Status | Category | Mac-actionable | Next slice | Blocked by |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for lane in status.lanes:
        blocked = "<br>".join(lane.blocked_by) if lane.blocked_by else "None"
        actionable = "yes" if lane.mac_actionable else "no"
        lines.append(
            f"| {lane.label} | `{lane.status}` | {lane.category} | {actionable} | "
            f"{lane.next_slice} | {blocked} |"
        )
    return "\n".join(lines) + "\n"


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=("markdown", "json"), default="markdown")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    status = read_status()
    if args.format == "json":
        print(json.dumps(asdict(status), indent=2))
    else:
        print(render_markdown(status), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
