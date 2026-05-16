#!/usr/bin/env python3
"""Report the broader Garnet MIT/productization objective status.

This reporter intentionally differs from `garnet_readiness_status.py`: the
tracked implementation plan can be complete while the larger public-readiness
goal still has distribution, mobile, video, proof, and converter-assist gates.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import garnet_converter_status  # noqa: E402
import garnet_readiness_status  # noqa: E402


@dataclass(frozen=True)
class ObjectiveLane:
    id: str
    label: str
    status: str
    completion_percent: float
    evidence: str
    blocked_by: list[str]
    deferred: list[str]


@dataclass(frozen=True)
class MitReadinessStatus:
    source: str
    overall_status: str
    completion_percent: float
    current_truth: list[str]
    lanes: list[ObjectiveLane]


def _lane_score(lane: ObjectiveLane) -> float:
    if lane.status == "verified":
        return 1.0
    if lane.status == "active-partial":
        return 0.5
    if lane.status == "planned-contract":
        return 0.25
    return 0.0


def read_status() -> MitReadinessStatus:
    plan = garnet_readiness_status.read_status(
        ROOT / "F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md"
    )
    converter = garnet_converter_status.read_status()
    contract = converter.intelligent_assist_contract

    lanes = [
        ObjectiveLane(
            id="tracked_implementation_plan",
            label="Tracked implementation plan",
            status="verified" if plan.completion_percent == 100.0 else "active-partial",
            completion_percent=plan.completion_percent,
            evidence=(
                f"`scripts/garnet_readiness_status.py` reports "
                f"{plan.completed_slices}/{plan.total_slices} slices."
            ),
            blocked_by=[],
            deferred=[] if plan.completion_percent == 100.0 else [item.title for item in plan.open_slices],
        ),
        ObjectiveLane(
            id="agentic_dogfood_matrix",
            label="Agentic dogfood matrix",
            status="verified",
            completion_percent=100.0,
            evidence=(
                "`scripts/run_agentic_dogfood_matrix.py --copy-to-desktop --strict` "
                "covers the current advanced source-checkout domains with Desktop evidence."
            ),
            blocked_by=[],
            deferred=["Use future slices to add domains when product surfaces expand."],
        ),
        ObjectiveLane(
            id="converter_truth",
            label="Converter adoption truth",
            status="verified",
            completion_percent=100.0,
            evidence=(
                "`scripts/garnet_converter_status.py` separates Rust/Ruby/Python/Go active "
                "lanes from planned language and LLM-assist lanes."
            ),
            blocked_by=[],
            deferred=[
                "New deterministic frontends require separate tested slices.",
                "LLM assist requires a secure advisory implementation before activation.",
            ],
        ),
        ObjectiveLane(
            id="macos_studio_dmg",
            label="macOS Studio DMG",
            status="active-partial",
            completion_percent=75.0,
            evidence=(
                "Garnet Studio DMG build, mounted-copy smoke, packaged PWA smoke, and "
                "agentic matrix evidence are active in Desktop dogfood bundles."
            ),
            blocked_by=[
                "No valid local Developer ID Application identity",
                "No notarization profile",
            ],
            deferred=["Clean-machine Gatekeeper install evidence"],
        ),
        ObjectiveLane(
            id="developer_id_notarization",
            label="Developer ID notarization",
            status="blocked",
            completion_percent=0.0,
            evidence=(
                "`scripts/preflight_garnet_studio_notarization.sh --copy-to-desktop` "
                "records the current blockers without claiming notarization."
            ),
            blocked_by=[
                "APPLE_DEV_ID_APP",
                "APPLE_NOTARY_PROFILE",
                "valid Developer ID Application certificate",
                "stapled DMG ticket",
            ],
            deferred=["Signed + notarized macOS distribution"],
        ),
        ObjectiveLane(
            id="web_pwa",
            label="Web/PWA productization",
            status="verified",
            completion_percent=100.0,
            evidence=(
                "Local service-worker, local PWA, browser offline, live Pages, and CI "
                "web/PWA smoke gates are active."
            ),
            blocked_by=[],
            deferred=["Full browser IDE/workbench remains future product scope."],
        ),
        ObjectiveLane(
            id="mobile_distribution",
            label="Mobile distribution",
            status="planned",
            completion_percent=0.0,
            evidence="No iOS, Android, Expo, TestFlight, or app-store lane is implemented yet.",
            blocked_by=["product surface decision", "mobile build/test pipeline"],
            deferred=["iOS", "Android", "Expo", "TestFlight", "Play Store"],
        ),
        ObjectiveLane(
            id="promo_video",
            label="Promo video",
            status="planned",
            completion_percent=0.0,
            evidence="No verified HyperFrames/Remotion promo artifact is present in the repo or Desktop dogfood bundle.",
            blocked_by=["storyboard", "rendered artifact", "visual QA"],
            deferred=["30-second Garnet promo video", "website-ready export"],
        ),
        ObjectiveLane(
            id="llm_assist",
            label="LLM assist",
            status=contract.status,
            completion_percent=25.0,
            evidence="Garnet-aware assist contract is machine-readable, but no provider-backed assist lane is active.",
            blocked_by=["secure advisory implementation", "provider/runtime boundary", "dogfood gate"],
            deferred=contract.analysis_targets + contract.required_gates,
        ),
        ObjectiveLane(
            id="broad_converter_frontends",
            label="Broad converter frontends",
            status="planned",
            completion_percent=0.0,
            evidence="Only Rust/Ruby/Python/Go deterministic frontends are active today.",
            blocked_by=["frontend implementation slices", "corpus fixtures", "lineage/sandbox/check gates"],
            deferred=[language.label for language in converter.planned_languages],
        ),
        ObjectiveLane(
            id="proof_empirics",
            label="Proof and empirical validation",
            status="active-partial",
            completion_percent=40.0,
            evidence="Proof and benchmark lanes are documented as scaffold/partial rather than complete.",
            blocked_by=["mechanized proof", "fresh empirical validation budget"],
            deferred=["native backend proof", "benchmarks", "external user study"],
        ),
    ]

    percent = round(sum(_lane_score(lane) for lane in lanes) / len(lanes) * 100.0, 1)
    return MitReadinessStatus(
        source=str(ROOT),
        overall_status="verified" if percent == 100.0 else "active-partial",
        completion_percent=percent,
        current_truth=[
            "tracked implementation plan is complete",
            "goal remains active",
            "100% tracked slices is not full MIT/productization completion",
        ],
        lanes=lanes,
    )


def render_markdown(status: MitReadinessStatus) -> str:
    lines = [
        "# Garnet MIT Readiness Objective Status",
        "",
        f"Source: `{status.source}`",
        "",
        f"Overall status: **{status.overall_status}**",
        f"Objective completion: **{status.completion_percent:.1f}%**",
        "",
        (
            "Current truth: the tracked implementation plan is complete, but that is "
            "not full MIT/productization completion."
        ),
        "",
        "| Lane | Status | Percent | Evidence | Blocked / deferred |",
        "|---|---|---:|---|---|",
    ]
    for lane in status.lanes:
        blockers = lane.blocked_by + lane.deferred
        blocked_text = "<br>".join(blockers) if blockers else "None"
        lines.append(
            f"| {lane.label} | `{lane.status}` | {lane.completion_percent:.1f}% | "
            f"{lane.evidence} | {blocked_text} |"
        )
    return "\n".join(lines) + "\n"


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--format",
        choices=("markdown", "json"),
        default="markdown",
        help="output format",
    )
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
