#!/usr/bin/env python3
"""Build a bounded MIT deck outline from current Garnet readiness evidence."""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import garnet_adoption_surface_status  # noqa: E402
import garnet_mit_demo_route  # noqa: E402
import garnet_mit_readiness_status  # noqa: E402
import garnet_readiness_status  # noqa: E402


@dataclass(frozen=True)
class DeckSlide:
    id: str
    title: str
    headline: str
    body: list[str]
    evidence: list[str]
    speaker_note: str


@dataclass(frozen=True)
class DeckOutlineStatus:
    source: str
    overall_status: str
    objective_completion_percent: float
    tracked_slices: str
    target_slide_count: int
    current_truth: list[str]
    slides: list[DeckSlide]
    blocked_gates: list[garnet_mit_demo_route.BlockedGate]
    forbidden_claims: list[str]


def _tracked_slices() -> str:
    plan = garnet_readiness_status.read_status(
        ROOT / "F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md"
    )
    return f"{plan.completed_slices}/{plan.total_slices}"


def _slides(
    route: garnet_mit_demo_route.DemoRouteStatus,
    adoption: garnet_adoption_surface_status.AdoptionSurface,
) -> list[DeckSlide]:
    return [
        DeckSlide(
            id="title-current-truth",
            title="Current Truth",
            headline="Garnet is presentation-ready evidence, not final acceptance.",
            body=[
                f"MIT/productization objective: {route.objective_completion_percent:.1f}%.",
                f"Tracked implementation slices: {route.tracked_slices}.",
                "The deck distinguishes verified surfaces from blocked product gates.",
            ],
            evidence=[
                "scripts/garnet_mit_readiness_status.py",
                "docs/index.html",
                "docs/status.html",
            ],
            speaker_note="Open by saying this is not final MIT/productization acceptance and not full MIT/productization completion.",
        ),
        DeckSlide(
            id="language-hook",
            title="Why Garnet",
            headline=adoption.headline,
            body=[
                "Dual-mode programming keeps managed ergonomics and safe-mode ownership boundaries in one language story.",
                "Agent-facing examples, converter planning, and dogfood evidence make the language useful before broad product claims.",
                "Use cases are tied to examples, tests, reporters, and Desktop evidence instead of slogan-only copy.",
            ],
            evidence=[
                "examples/agent_toolbelt_01_triage_router.garnet",
                "garnet-cli/tests/dogfood_readiness_examples.rs",
                "scripts/garnet_adoption_surface_status.py",
            ],
            speaker_note="Use the hook as a reviewer orientation, then immediately ground it in evidence.",
        ),
        DeckSlide(
            id="studio-workbench",
            title="Local Workbench",
            headline="Garnet Studio gives reviewers a Mac-first path into real workflows.",
            body=[
                "The unsigned local app can run health checks, examples, conversion, advisory reports, release status, and dogfood probes.",
                "Objective Pulse, Continuation Pulse, and Demo Route expose current evidence without replacing terminal verification.",
                "DMG smoke remains unsigned/local until Developer ID credentials exist.",
            ],
            evidence=[
                "apps/garnet-studio-macos/Sources/GarnetStudio/GarnetStudioApp.swift",
                "scripts/package_garnet_studio_macos.sh",
                "scripts/smoke_garnet_studio_dmg.sh",
            ],
            speaker_note="Show the Release panel and name Demo Route as a rehearsal artifact, not a release certificate.",
        ),
        DeckSlide(
            id="converter-advisory",
            title="Converter Strategy",
            headline="Convert where fidelity is honest; advise where native boundaries matter.",
            body=[
                "provider-neutral handoff packets exist; provider-backed LLM conversion is not active.",
                "Active deterministic conversion remains Rust, Ruby, Python, and Go.",
                "Advisory planning covers JavaScript, TypeScript, Swift, Java, C, C++, C#, Perl, Kotlin, Shell, SQL, and Other.",
                "Native-boundary languages stay native or behind FFI with CapCaps, memory declarations, lineage, and sandbox policy.",
            ],
            evidence=[
                "scripts/garnet_converter_status.py",
                "scripts/garnet_converter_advisory_handoff.py",
                "F_Project_Management/GARNET_CONVERTER_AND_PLATFORM_STRATEGY.md",
            ],
            speaker_note="Do not imply autonomous conversion. The point is reviewed advisory planning plus deterministic lanes.",
        ),
        DeckSlide(
            id="dogfood-evidence",
            title="Dogfood Evidence",
            headline="The project has a falsifiable readiness surface, not a vibes-only checklist.",
            body=[
                "The agentic matrix covers orchestration, diagnostics, conversion, safe mode, release integrity, app, web/PWA, and memory-analysis probes.",
                "Remote CI repeats core checks across formatting, linting, tests, docs, audit, deny, SBOM, packages, and web/PWA smoke.",
                "Desktop bundles preserve local evidence outside transient build directories.",
            ],
            evidence=[
                "scripts/run_agentic_dogfood_matrix.py",
                ".github/workflows/agentic-dogfood-matrix.yml",
                "/Users/idc2.0/Desktop/dogfood",
            ],
            speaker_note="Invite reviewers to inspect the matrix report and manifests before trusting the narrative.",
        ),
        DeckSlide(
            id="web-pwa-surface",
            title="Public Surface",
            headline="The landing page can hook attention while the status page keeps the truth ledger.",
            body=[
                "Landing copy names the verified product hooks without burying the first screen in caveats.",
                "Status copy keeps notarization, platform, LLM, native backend, mobile, and final acceptance gates explicit.",
                "PWA smoke evidence protects install/offline behavior from drifting during productization.",
            ],
            evidence=[
                "docs/index.html",
                "docs/status.html",
                "scripts/smoke_garnet_pages_pwa.sh",
            ],
            speaker_note="Use this to show polish and honesty together: compelling front door, precise status room.",
        ),
        DeckSlide(
            id="blocked-gates",
            title="Blocked And Deferred Gates",
            headline="The remaining gates are named, scoped, and evidence-shaped.",
            body=[
                "Developer ID notarization is blocked by account-holder identity verification and credentials.",
                "Windows/Linux Studio runtime proof belongs on those target systems.",
                "Provider-backed LLM conversion, native backend lowering, mobile distribution, and final acceptance remain separate future gates.",
            ],
            evidence=[
                "scripts/garnet_mac_side_continuation_status.py",
                "F_Project_Management/GARNET_WINDOWS_LINUX_STUDIO_HANDOFF_2026_05_16.md",
                "scripts/garnet_studio_notarization_status.py",
            ],
            speaker_note="This slide earns trust by refusing to flatten blocked gates into success language.",
        ),
        DeckSlide(
            id="ask-and-next-slices",
            title="Ask And Next Slices",
            headline="The next work is narrower proof, better UX, and target-platform closure.",
            body=[
                "Run one falsifiable proof, benchmark, or Studio UX slice per PR.",
                "Resume Apple signing only after Developer ID credentials and notary profile exist.",
                "Use Windows/Linux handoff packets for target-runtime evidence and keep this Mac lane focused.",
            ],
            evidence=[
                "scripts/garnet_mit_demo_route.py",
                "scripts/garnet_mac_side_continuation_status.py",
                "F_Project_Management/DOGFOOD/GARNET_v0_5_DOGFOOD_READINESS_PHASE_LOG.md",
            ],
            speaker_note="Close with what reviewers can run now and what evidence unlocks the next readiness percentage.",
        ),
    ]


def read_outline() -> DeckOutlineStatus:
    route = garnet_mit_demo_route.read_route()
    mit = garnet_mit_readiness_status.read_status()
    adoption = garnet_adoption_surface_status.read_surface()
    slides = _slides(route, adoption)
    return DeckOutlineStatus(
        source=str(ROOT),
        overall_status=mit.overall_status,
        objective_completion_percent=mit.completion_percent,
        tracked_slices=_tracked_slices(),
        target_slide_count=len(slides),
        current_truth=[
            "tracked implementation plan is complete",
            "deck outline is a presentation planning artifact",
            "not full MIT/productization completion",
            "blocked gates require separate evidence before success claims",
        ],
        slides=slides,
        blocked_gates=route.blocked_gates,
        forbidden_claims=[
            "Apple Developer ID notarization",
            "App Store distribution",
            "Windows/Linux Studio runtime proof",
            "provider-backed LLM conversion",
            "native backend lowering",
            "mobile distribution",
            "production-ready language",
            "final MIT/productization acceptance",
        ],
    )


def render_markdown(outline: DeckOutlineStatus) -> str:
    lines = [
        "# Garnet MIT Deck Outline",
        "",
        f"Source: `{outline.source}`",
        f"Overall status: **{outline.overall_status}**",
        f"Objective completion: **{outline.objective_completion_percent:.1f}%**",
        f"Tracked slices: **{outline.tracked_slices}**",
        f"Target slide count: **{outline.target_slide_count}**",
        "",
        "Current truth: this deck outline packages verified surfaces for review; it is not full MIT/productization completion.",
        "",
        "## Slides",
        "",
    ]
    for index, slide in enumerate(outline.slides, start=1):
        lines.extend(
            [
                f"## Slide {index}: {slide.title}",
                "",
                f"**Headline:** {slide.headline}",
                "",
                "**Body:**",
                *[f"- {item}" for item in slide.body],
                "",
                "**Evidence:**",
                *[f"- `{item}`" for item in slide.evidence],
                "",
                f"**Speaker note:** {slide.speaker_note}",
                "",
            ]
        )
    lines.extend(
        [
            "## Blocked Gates",
            "",
            "| Gate | Reason | Next unlock |",
            "| --- | --- | --- |",
        ]
    )
    for gate in outline.blocked_gates:
        lines.append(f"| {gate.label} | {gate.reason} | {gate.next_unlock} |")
    lines.extend(
        [
            "",
            "## Forbidden Claims",
            "",
            *[f"- {claim}" for claim in outline.forbidden_claims],
        ]
    )
    return "\n".join(lines) + "\n"


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_bundle(outline: DeckOutlineStatus, output_dir: Path) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    data_path = output_dir / "garnet-mit-deck-outline.json"
    report_path = output_dir / "garnet-mit-deck-outline.md"
    manifest_path = output_dir / "MANIFEST.sha256"
    data_path.write_text(json.dumps(asdict(outline), indent=2) + "\n", encoding="utf-8")
    report_path.write_text(render_markdown(outline), encoding="utf-8")
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
    outline = read_outline()
    if args.output_dir:
        write_bundle(outline, args.output_dir)
    if args.format == "json":
        print(json.dumps(asdict(outline), indent=2))
    else:
        print(render_markdown(outline), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
