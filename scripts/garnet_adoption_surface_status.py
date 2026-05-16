#!/usr/bin/env python3
"""Report the public Garnet adoption surface without overclaiming readiness.

This reporter exists so the repository, website, PR bodies, and future agents
can share one adoption story: what is worth trying today, what is planned, and
which proof gates still block broader claims.
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
import garnet_converter_llm_feasibility  # noqa: E402
import garnet_mit_readiness_status  # noqa: E402


@dataclass(frozen=True)
class UseCase:
    id: str
    label: str
    hook: str
    evidence: list[str]
    status: str


@dataclass(frozen=True)
class AdoptionSurface:
    source: str
    headline: str
    accurate_pitch: str
    active_converter_languages: list[str]
    planned_converter_languages: list[str]
    llm_assist_status: str
    llm_assist_truth: list[str]
    verified_use_cases: list[UseCase]
    repo_site_contract: list[str]
    open_gates: list[str]


def verified_use_cases() -> list[UseCase]:
    return [
        UseCase(
            id="dual-mode-programming",
            label="Dual-mode programming",
            hook="Use managed mode for orchestration and safe mode for ownership-sensitive boundaries.",
            evidence=[
                "C_Language_Specification/GARNET_v1_0_Mini_Spec.md",
                "garnet-check-v0.3/tests/borrow.rs",
                "garnet-check-v0.3/tests/caps.rs",
            ],
            status="verified",
        ),
        UseCase(
            id="agent-toolbelt",
            label="Agent toolbelt examples",
            hook="Run small agent decision programs for triage, capability budgeting, memory recall, release gates, and repair planning.",
            evidence=[
                "examples/agent_toolbelt_01_triage_router.garnet",
                "examples/agent_toolbelt_02_capability_budget.garnet",
                "examples/agent_toolbelt_03_memory_recall.garnet",
                "examples/agent_toolbelt_04_release_gate.garnet",
                "examples/agent_toolbelt_05_repair_planner.garnet",
                "garnet-cli/tests/dogfood_readiness_examples.rs",
            ],
            status="verified",
        ),
        UseCase(
            id="migration-assistant",
            label="Migration assistant",
            hook="Convert stylized Rust, Ruby, Python, and Go into sandboxed Garnet with lineage and migrate_todo evidence.",
            evidence=[
                "garnet-convert/src/frontends/rust.rs",
                "garnet-convert/src/frontends/ruby.rs",
                "garnet-convert/src/frontends/python.rs",
                "garnet-convert/src/frontends/go.rs",
                "garnet-convert/tests/corpus.rs",
            ],
            status="active-partial",
        ),
        UseCase(
            id="agentic-dogfood-matrix",
            label="Agentic dogfood matrix",
            hook="Exercise Garnet through orchestration, diagnostics, conversion, safe mode, release integrity, app, web/PWA, and memory-analysis probes.",
            evidence=[
                "scripts/run_agentic_dogfood_matrix.py",
                "F_Project_Management/DOGFOOD/GARNET_AGENTIC_DOGFOOD_STRESS_PLAN.md",
            ],
            status="verified",
        ),
        UseCase(
            id="macos-workbench",
            label="macOS workbench",
            hook=(
                "Open Garnet Studio locally through Codex Run or `dist/Garnet Studio.app` "
                "to run health checks, examples, deterministic conversion, Assist Plan, "
                "Advisory Bundle packaging, Advisory Review gating, the MIT objective "
                "pulse, release status, and agentic stress tests."
            ),
            evidence=[
                "apps/garnet-studio-macos/Sources/GarnetStudio/GarnetStudioApp.swift",
                ".codex/environments/environment.toml",
                "script/build_and_run.sh",
                "scripts/test_garnet_studio_run_button.py",
                "scripts/garnet_mit_readiness_status.py",
                "scripts/package_garnet_studio_macos.sh",
                "scripts/smoke_garnet_studio_dmg.sh",
            ],
            status="active-partial",
        ),
    ]


def read_surface() -> AdoptionSurface:
    converter = garnet_converter_status.read_status()
    feasibility = garnet_converter_llm_feasibility.read_status()
    mit = garnet_mit_readiness_status.read_status()
    lanes = {lane.id: lane for lane in mit.lanes}

    active_languages = [language.label for language in converter.active_languages]
    planned_languages = [language.label for language in converter.planned_languages]

    return AdoptionSurface(
        source=str(ROOT),
        headline="Rust rigor, Ruby velocity, agent-native dogfood evidence.",
        accurate_pitch=(
            "Garnet is currently strongest as a dual-mode language prototype with "
            "verified safe-mode checks, managed-mode ergonomics, an agentic dogfood "
            "matrix, a local macOS workbench, and a stylized migration assistant. "
            "It is not yet a notarized product, broad transpiler, native backend, "
            "or provider-backed LLM converter."
        ),
        active_converter_languages=active_languages,
        planned_converter_languages=planned_languages,
        llm_assist_status=lanes["llm_assist"].status,
        llm_assist_truth=[
            f"converter LLM feasibility is {feasibility.status}",
            "advisory planning is feasible, autonomous LLM conversion is not feasible yet",
            "deterministic context pack is active",
            "deterministic planned-language assist plan is active",
            "provider-neutral advisory bundle is active",
            "advisory review gate is active before model handoff",
            "provider-neutral advisory handoff packet is active",
            "provider-backed conversion is not active",
            "suggestions must preserve lineage, sandboxing, garnet check, dogfood evidence, and human audit",
            "deterministic converter output remains authoritative",
        ],
        verified_use_cases=verified_use_cases(),
        repo_site_contract=[
            "Site copy must say active converter lanes are Rust, Ruby, Python, and Go only.",
            "Site copy may discuss JavaScript, TypeScript, Swift, Java, C, C++, C#, and Perl only as planned lanes.",
            "LLM assist must be described as gated advisory context and assist planning, not active conversion.",
            "Install and app copy must not claim Developer ID notarization or clean-machine Gatekeeper success.",
            "Use-case hooks must point to current tests, scripts, examples, or dogfood bundles.",
        ],
        open_gates=[
            "Developer ID notarization",
            "mobile distribution",
            "promo video",
            "provider-backed LLM assist",
            "broad deterministic converter frontends",
            "native backend/proof/empirics",
        ],
    )


def render_markdown(surface: AdoptionSurface) -> str:
    lines = [
        "# Garnet Adoption Surface Status",
        "",
        f"Source: `{surface.source}`",
        "",
        f"Headline: **{surface.headline}**",
        "",
        surface.accurate_pitch,
        "",
        "## Converter Truth",
        "",
        f"Active deterministic lanes: {', '.join(surface.active_converter_languages)}.",
        f"Planned lanes only: {', '.join(surface.planned_converter_languages)}.",
        f"LLM assist status: **{surface.llm_assist_status}**.",
        "",
        "LLM assist truth:",
    ]
    lines.extend(f"- {item}" for item in surface.llm_assist_truth)

    lines.extend(["", "## Verified Use Cases", ""])
    for use_case in surface.verified_use_cases:
        evidence = ", ".join(f"`{item}`" for item in use_case.evidence)
        lines.append(f"- **{use_case.label}** ({use_case.status}): {use_case.hook} Evidence: {evidence}.")

    lines.extend(["", "## Repo/Site Contract", ""])
    lines.extend(f"- {item}" for item in surface.repo_site_contract)

    lines.extend(["", "## Open Gates", ""])
    lines.extend(f"- {item}" for item in surface.open_gates)

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
    surface = read_surface()
    if args.format == "json":
        print(json.dumps(asdict(surface), indent=2))
    else:
        print(render_markdown(surface), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
