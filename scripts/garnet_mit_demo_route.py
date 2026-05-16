#!/usr/bin/env python3
"""Build a bounded MIT demo route from current Garnet readiness evidence."""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import garnet_mac_side_continuation_status  # noqa: E402
import garnet_mit_readiness_status  # noqa: E402
import garnet_readiness_status  # noqa: E402


@dataclass(frozen=True)
class DemoBeat:
    id: str
    title: str
    duration_seconds: int
    surface: str
    story: str
    command: str
    evidence: str


@dataclass(frozen=True)
class BlockedGate:
    id: str
    label: str
    reason: str
    next_unlock: str


@dataclass(frozen=True)
class DemoRouteStatus:
    source: str
    overall_status: str
    objective_completion_percent: float
    tracked_slices: str
    total_duration_seconds: int
    current_truth: list[str]
    beats: list[DemoBeat]
    blocked_gates: list[BlockedGate]
    forbidden_claims: list[str]
    next_best_slices: list[str]


def _tracked_slices() -> str:
    plan = garnet_readiness_status.read_status(
        ROOT / "F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md"
    )
    return f"{plan.completed_slices}/{plan.total_slices}"


def _blocked_gates() -> list[BlockedGate]:
    return [
        BlockedGate(
            id="developer-id-notarization",
            label="Developer ID notarization",
            reason="Apple account-holder identity verification, Developer ID certificate, and notary profile are still external prerequisites.",
            next_unlock="Resolve Apple Developer Program identity verification, then rerun notarization preflight and signing evidence.",
        ),
        BlockedGate(
            id="windows-linux-studio",
            label="Windows/Linux Studio runtime proof",
            reason="Runtime proof belongs on target systems, not this macOS checkout.",
            next_unlock="Use the Windows/Linux handoff packet on those machines and preserve platform-native evidence.",
        ),
        BlockedGate(
            id="provider-backed-llm-conversion",
            label="Provider-backed LLM conversion",
            reason="Only provider-neutral advisory planning, bundles, reviews, and handoff packets are active.",
            next_unlock="Add a secure provider/runtime boundary, lineage, sandbox, garnet check, dogfood, and human audit gates.",
        ),
        BlockedGate(
            id="native-backend-lowering",
            label="Native backend lowering",
            reason="Wasm/LLVM-style lowering remains planned architecture, not implemented compiler evidence.",
            next_unlock="Create a separate backend proof slice with deterministic fixtures and performance evidence.",
        ),
        BlockedGate(
            id="mobile-distribution",
            label="Mobile distribution",
            reason="No iOS, Android, Expo, TestFlight, or app-store lane is implemented yet.",
            next_unlock="Define the mobile product surface, then build target-platform CI and store/test evidence.",
        ),
        BlockedGate(
            id="final-acceptance",
            label="Final MIT/productization acceptance",
            reason="Human/aesthetic promo review, proof/empirics, distribution credentials, and target-platform gates remain separate.",
            next_unlock="Close each deferred gate with falsifiable evidence before claiming final acceptance.",
        ),
    ]


def _beats() -> list[DemoBeat]:
    return [
        DemoBeat(
            id="objective-pulse",
            title="Open with current truth",
            duration_seconds=45,
            surface="repo reporter and public landing pulse",
            story="Show that Garnet separates 87/87 tracked slices from the broader 58.6% MIT/productization objective.",
            command="python3 scripts/garnet_mit_readiness_status.py --format markdown",
            evidence="scripts/garnet_mit_readiness_status.py plus docs/index.html Objective Pulse.",
        ),
        DemoBeat(
            id="studio-continuation",
            title="Show Garnet Studio as the local workbench",
            duration_seconds=75,
            surface="macOS Studio Release panel",
            story="Run the app loop, point to Objective Pulse and Continuation Pulse, and keep unsigned/local evidence distinct from notarized distribution.",
            command="./script/build_and_run.sh --verify",
            evidence="dist/Garnet Studio.app, packaged DMG smoke bundles, and scripts/garnet_mac_side_continuation_status.py.",
        ),
        DemoBeat(
            id="converter-advisory",
            title="Demonstrate honest migration help",
            duration_seconds=75,
            surface="converter advisory workflow",
            story="Use provider-neutral Assist Plan, Advisory Bundle, Advisory Review, and Advisory Handoff to guide migration without activating provider-backed conversion.",
            command="python3 scripts/garnet_converter_advisory_handoff.py --help",
            evidence="converter feasibility, context pack, advisory bundle, review, and handoff reporters.",
        ),
        DemoBeat(
            id="agentic-dogfood",
            title="Run the agent-facing proof surface",
            duration_seconds=90,
            surface="source-checkout dogfood matrix",
            story="Show agent orchestration, converter planning, web/PWA, memory integrity, release integrity, and app/productization probes as one falsifiable evidence surface.",
            command="PYTHONDONTWRITEBYTECODE=1 python3 scripts/run_agentic_dogfood_matrix.py --copy-to-desktop --strict --skip-app-workbench",
            evidence="Latest source dogfood matrix Desktop bundle and matrix report.",
        ),
        DemoBeat(
            id="web-pwa-live",
            title="Show the public surface",
            duration_seconds=60,
            surface="garnet-lang.org and docs PWA",
            story="Open garnet-lang.org, verify the Studio workbench, Continuation Pulse hook, PWA shell, and status split without hiding deferred gates.",
            command="scripts/smoke_garnet_pages_pwa.sh --copy-to-desktop --strict",
            evidence="Live Pages smoke, local web/PWA smoke, browser offline PWA smoke, and docs/status.html.",
        ),
        DemoBeat(
            id="boundaries-and-ask",
            title="Close with blocked gates and the ask",
            duration_seconds=75,
            surface="continuation reporter and status page",
            story="Name the exact next unlocks: Developer ID credentials, Windows/Linux runtime proof, provider boundary, native backend proof, mobile lane, and final acceptance.",
            command="python3 scripts/garnet_mac_side_continuation_status.py --format markdown",
            evidence="scripts/garnet_mac_side_continuation_status.py, CURRENT_STATE.md, and the dogfood readiness phase log.",
        ),
    ]


def read_route() -> DemoRouteStatus:
    mit = garnet_mit_readiness_status.read_status()
    continuation = garnet_mac_side_continuation_status.read_status()
    beats = _beats()
    return DemoRouteStatus(
        source=str(ROOT),
        overall_status=mit.overall_status,
        objective_completion_percent=mit.completion_percent,
        tracked_slices=_tracked_slices(),
        total_duration_seconds=sum(beat.duration_seconds for beat in beats),
        current_truth=[
            "tracked implementation plan is complete",
            "MIT/productization objective remains active-partial",
            "demo route is a presentation artifact, not full MIT/productization completion",
            "live claims must stay tied to local, remote, and Desktop dogfood evidence",
        ],
        beats=beats,
        blocked_gates=_blocked_gates(),
        forbidden_claims=[
            "Apple Developer ID notarization",
            "Windows/Linux Studio runtime proof",
            "provider-backed LLM conversion",
            "native backend lowering",
            "mobile distribution",
            "production-ready language",
            "final MIT/productization acceptance",
        ],
        next_best_slices=[
            lane.next_slice
            for lane in continuation.lanes
            if lane.mac_actionable and lane.id in {"website_status_presentation", "proof_benchmark_empirics"}
        ],
    )


def render_markdown(route: DemoRouteStatus) -> str:
    minutes = route.total_duration_seconds / 60.0
    lines = [
        "# Garnet MIT Demo Route",
        "",
        f"Source: `{route.source}`",
        f"Overall status: **{route.overall_status}**",
        f"Objective completion: **{route.objective_completion_percent:.1f}%**",
        f"Tracked slices: **{route.tracked_slices}**",
        f"Target runtime: **{minutes:.1f} minutes**",
        "",
        "Current truth: this route packages verified surfaces for review; it is not full MIT/productization completion.",
        "",
        "## Demo Beats",
        "",
        "| Beat | Time | Surface | Story | Command | Evidence |",
        "| --- | ---: | --- | --- | --- | --- |",
    ]
    for beat in route.beats:
        lines.append(
            f"| {beat.title} | {beat.duration_seconds}s | {beat.surface} | "
            f"{beat.story} | `{beat.command}` | {beat.evidence} |"
        )
    lines.extend(
        [
            "",
            "## Blocked Gates",
            "",
            "| Gate | Reason | Next unlock |",
            "| --- | --- | --- |",
        ]
    )
    for gate in route.blocked_gates:
        lines.append(f"| {gate.label} | {gate.reason} | {gate.next_unlock} |")
    lines.extend(
        [
            "",
            "## Forbidden Claims",
            "",
            *[f"- {claim}" for claim in route.forbidden_claims],
            "",
            "## Next Best Slices",
            "",
            *[f"- {item}" for item in route.next_best_slices],
        ]
    )
    return "\n".join(lines) + "\n"


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_bundle(route: DemoRouteStatus, output_dir: Path) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    data_path = output_dir / "garnet-mit-demo-route.json"
    report_path = output_dir / "garnet-mit-demo-route.md"
    manifest_path = output_dir / "MANIFEST.sha256"
    data_path.write_text(json.dumps(asdict(route), indent=2) + "\n", encoding="utf-8")
    report_path.write_text(render_markdown(route), encoding="utf-8")
    files = [data_path, report_path]
    manifest_path.write_text(
        "".join(f"{_sha256(path)}  ./{path.name}\n" for path in files),
        encoding="utf-8",
    )


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=("markdown", "json"), default="markdown")
    parser.add_argument("--output-dir", type=Path, help="write JSON, Markdown, and manifest evidence")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    route = read_route()
    if args.output_dir:
        write_bundle(route, args.output_dir)
    if args.format == "json":
        print(json.dumps(asdict(route), indent=2))
    else:
        print(render_markdown(route), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
