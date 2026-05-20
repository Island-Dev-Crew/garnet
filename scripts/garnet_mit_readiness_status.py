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
import garnet_proof_benchmark_status  # noqa: E402
import garnet_promo_video_status  # noqa: E402
import garnet_readiness_status  # noqa: E402
import garnet_windows_linux_studio_status  # noqa: E402


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
    if lane.status == "rendered-artifact-ready":
        return 0.65
    if lane.status == "visual-qa-ready":
        return 0.8
    if lane.status == "website-export-ready":
        return 0.9
    if lane.status == "public-site-embedded":
        return 0.95
    if lane.status == "composition-ready":
        return 0.5
    if lane.status == "source-locked":
        return 0.35
    if lane.status == "planned-contract":
        return 0.25
    if lane.status == "source-present":
        return 0.6
    return 0.0


def _dedupe(items: list[str]) -> list[str]:
    seen: set[str] = set()
    deduped: list[str] = []
    for item in items:
        if item not in seen:
            deduped.append(item)
            seen.add(item)
    return deduped


def _lsp_source_present() -> bool:
    required = [
        ROOT / "garnet-lsp" / "Cargo.toml",
        ROOT / "garnet-lsp" / "src" / "lib.rs",
        ROOT / "garnet-lsp" / "src" / "main.rs",
        ROOT / "editors" / "vscode" / "package.json",
        ROOT / "editors" / "vscode" / "src" / "extension.ts",
    ]
    return all(path.exists() for path in required)


def _vm_scaffold_present(proof: garnet_proof_benchmark_status.ProofBenchmarkStatus) -> bool:
    return any(
        bench.id == "vm_parse_compile_execute"
        and bench.bench_file_exists
        and bench.cargo_entry_present
        for bench in proof.benchmarks
    )


def read_status() -> MitReadinessStatus:
    plan = garnet_readiness_status.read_status(
        ROOT / "F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md"
    )
    converter = garnet_converter_status.read_status()
    contract = converter.intelligent_assist_contract
    promo = garnet_promo_video_status.read_status()
    proof = garnet_proof_benchmark_status.read_status()
    vm_scaffold_present = _vm_scaffold_present(proof)
    wls = garnet_windows_linux_studio_status.read_status()
    if promo.public_site_embed_present:
        promo_evidence_tail = "records local rendered MP4/WebM evidence, automated visual QA evidence, a website export package, and public-site embedding while keeping human/aesthetic acceptance open."
    elif promo.website_export_present:
        promo_evidence_tail = "records local rendered MP4/WebM evidence, automated visual QA evidence, and a website export package while keeping public-site embedding open."
    elif promo.visual_qa_present:
        promo_evidence_tail = "records local rendered MP4/WebM evidence plus automated visual QA evidence, and keeps website export open."
    elif promo.rendered_video_present:
        promo_evidence_tail = "records local rendered MP4/WebM evidence, and keeps visual QA plus website export open."
    else:
        promo_evidence_tail = "preserves that no verified rendered artifact exists."
    promo_blockers = ["website-ready export"]
    if promo.public_site_embed_present:
        promo_blockers = ["human/aesthetic acceptance review"]
    elif promo.website_export_present:
        promo_blockers = ["public-site embedding and review"]
    if not promo.visual_qa_present:
        promo_blockers.insert(0, "visual QA verdict")
    if not promo.rendered_video_present:
        promo_blockers.insert(0, "rendered artifact")

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
                "records the current blockers without claiming notarization; "
                "`scripts/garnet_studio_notarization_status.py` summarizes that "
                "bundle for agents and PR/site evidence."
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
            id="windows_linux_distribution",
            label="Windows/Linux distribution",
            status="active-partial",
            completion_percent=50.0,
            evidence=(
                "`scripts/garnet_windows_linux_studio_status.py` now reports the "
                "Tauri v2 shell scaffold in `apps/garnet-studio`, minimal webview "
                "permissions, Windows local release build/smoke evidence, and "
                "open Linux plus clean-machine package gates."
            ),
            blocked_by=list(wls.user_assistance_needed),
            deferred=list(wls.next_slices),
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
            id="editor_lsp_adoption",
            label="Editor/LSP adoption",
            status="source-present" if _lsp_source_present() else "planned",
            completion_percent=60.0 if _lsp_source_present() else 0.0,
            evidence=(
                "`garnet-lsp/` and `editors/vscode/` provide the S1 source "
                "surface for diagnostics, hover, and basic go-to-definition. "
                "`scripts/smoke_garnet_lsp_protocol.py` proves those paths over "
                "stdio, and Desktop evidence records Cursor diagnostic plus "
                "clean-open screenshots. Published VSIX and the full manual "
                "screenshot trio remain review hardening work."
            )
            if _lsp_source_present()
            else "No committed LSP or VSCode extension source is present yet.",
            blocked_by=[] if _lsp_source_present() else ["S1 LSP implementation"],
            deferred=[
                "manual VSCode hover/go-to-def screenshot confirmation",
                "published VSIX",
                "safe-mode hover",
                "workspace symbols",
                "rename",
                "CST-grade incremental precision",
            ],
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
            status=promo.status,
            completion_percent=promo.completion_percent,
            evidence=(
                "`scripts/garnet_promo_video_status.py` defines the 30-second "
                "promo contract, locks visual identity/source surfaces to repo evidence, "
                f"adds a HyperFrames-compatible composition source, and {promo_evidence_tail}"
            ),
            blocked_by=promo_blockers,
            deferred=["human/aesthetic acceptance review"],
        ),
        ObjectiveLane(
            id="llm_assist",
            label="LLM assist",
            status="active-partial",
            completion_percent=40.0,
            evidence=(
                "Garnet-aware assist contract, deterministic local context pack, "
                "planned-language assist-plan reporter, converter LLM feasibility "
                "gate, advisory-only provider option registry, Studio provider-options "
                "evidence action, provider-neutral advisory bundle, advisory review "
                "gate, and provider-neutral handoff packet are active, but no "
                "provider-backed assist lane or LLM conversion is active."
            ),
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
            completion_percent=45.0 if vm_scaffold_present else 40.0,
            evidence=(
                "`scripts/garnet_proof_benchmark_status.py` inventories the current "
                f"{len(proof.benchmarks)} Criterion benchmark harnesses plus proof/study "
                "protocols"
                + (
                    ", including the S2 VM parse/compile/execute harness,"
                    if vm_scaffold_present
                    else ""
                )
                + " while reporting measurements, mechanized proof, and empirical "
                "study execution as unclaimed."
            ),
            blocked_by=proof.blocked_by,
            deferred=proof.deferred,
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
            "S1 LSP source is tracked separately from full editor dogfood completion",
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
        blockers = _dedupe(lane.blocked_by + lane.deferred)
        blocked_text = "<br>".join(blockers) if blockers else "None"
        lines.append(
            f"| {lane.label} | `{lane.status}` | {lane.completion_percent:.1f}% | "
            f"{lane.evidence} | {blocked_text} |"
        )
    return "\n".join(lines) + "\n"


DEFAULT_BASELINE = (
    ROOT / "F_Project_Management" / "GARNET_v0_5_READINESS_BASELINE.json"
)


def _baseline_lanes(baseline: dict) -> dict[str, float]:
    """Extract `id -> completion_percent` from a baseline JSON snapshot.

    The baseline is the output of this same script's --format json invocation,
    captured at a known-good point and committed to the repo. Lanes not present
    in the baseline cannot regress (the gate is one-directional).
    """
    return {lane["id"]: float(lane["completion_percent"]) for lane in baseline.get("lanes", [])}


def check_no_regression(
    status: MitReadinessStatus, baseline_path: Path
) -> tuple[list[str], list[str]]:
    """Return (regressions, missing_lanes).

    A regression is a lane whose live percent dropped below the baseline.
    A missing_lane is a lane present in baseline but absent from live output
    (a slice was deleted or renamed without updating the baseline).
    """
    if not baseline_path.exists():
        return (
            [],
            [f"baseline missing at {baseline_path}; run with --format json > {baseline_path} to seed."],
        )
    baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
    baseline_pct = _baseline_lanes(baseline)
    live_pct = {lane.id: lane.completion_percent for lane in status.lanes}
    regressions: list[str] = []
    missing: list[str] = []
    for lane_id, baseline_value in baseline_pct.items():
        if lane_id not in live_pct:
            missing.append(lane_id)
            continue
        if live_pct[lane_id] + 1e-9 < baseline_value:
            regressions.append(
                f"{lane_id}: live {live_pct[lane_id]:.1f}% < baseline {baseline_value:.1f}%"
            )
    return regressions, missing


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--format",
        choices=("markdown", "json"),
        default="markdown",
        help="output format",
    )
    parser.add_argument(
        "--check-no-regression",
        action="store_true",
        help=(
            "Exit 1 if any lane in the committed baseline has regressed in the "
            "current run. Used by CI to prevent silent readiness drift across "
            "PRs. Baseline path defaults to "
            f"{DEFAULT_BASELINE.relative_to(ROOT)} and can be overridden with "
            "--baseline."
        ),
    )
    parser.add_argument(
        "--baseline",
        type=Path,
        default=DEFAULT_BASELINE,
        help="Path to the readiness baseline JSON (for --check-no-regression).",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    status = read_status()
    if args.format == "json":
        print(json.dumps(asdict(status), indent=2))
    else:
        print(render_markdown(status), end="")
    if args.check_no_regression:
        regressions, missing = check_no_regression(status, args.baseline)
        if regressions or missing:
            print("", file=sys.stderr)
            print("Readiness regression detected vs baseline:", file=sys.stderr)
            for r in regressions:
                print(f"  - regressed: {r}", file=sys.stderr)
            for m in missing:
                print(f"  - missing lane (baseline-only): {m}", file=sys.stderr)
            return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
