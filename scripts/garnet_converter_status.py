#!/usr/bin/env python3
"""Report the current and planned Garnet converter adoption surface."""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class LanguageLane:
    id: str
    label: str
    status: str
    evidence: list[str]
    notes: str
    fit_rationale: str


@dataclass(frozen=True)
class TrustBoundaries:
    sandbox_on_by_default: bool
    lineage_required: bool
    migrate_todo_required: bool
    source_execution_allowed: bool
    human_audit_required_to_unquarantine: bool


@dataclass(frozen=True)
class LlmAssist:
    status: str
    advisory_only: bool
    enabled_by_default: bool
    requires_lineage: bool
    requires_sandbox: bool
    requires_garnet_check: bool
    requires_dogfood_readiness: bool
    notes: list[str]


@dataclass(frozen=True)
class IntelligentAssistContract:
    status: str
    provider_required: bool
    model_required: bool
    pipeline: list[str]
    required_context: list[str]
    analysis_targets: list[str]
    required_gates: list[str]
    notes: list[str]


@dataclass(frozen=True)
class BackendLoweringStrategy:
    status: str
    planned_targets: list[str]
    use_cases: list[str]
    guardrails: list[str]


@dataclass(frozen=True)
class ConverterStatus:
    source: str
    converter_scope: str
    active_languages: list[LanguageLane]
    planned_languages: list[LanguageLane]
    native_boundary_languages: list[LanguageLane]
    trust_boundaries: TrustBoundaries
    llm_assist: LlmAssist
    intelligent_assist_contract: IntelligentAssistContract
    backend_lowering_strategy: BackendLoweringStrategy


def active_languages() -> list[LanguageLane]:
    return [
        LanguageLane(
            id="rust",
            label="Rust",
            status="active",
            evidence=[
                "garnet-convert/src/frontends/rust.rs",
                "garnet-convert/tests/corpus.rs",
            ],
            notes="Stylized Rust function/type shapes lower through CIR into sandboxed Garnet.",
            fit_rationale=(
                "Best fit: explicit ownership and module boundaries in Rust map directly into "
                "deterministic lowering with lineage and capability annotations."
            ),
        ),
        LanguageLane(
            id="ruby",
            label="Ruby",
            status="active",
            evidence=[
                "garnet-convert/src/frontends/ruby.rs",
                "garnet-convert/tests/corpus.rs",
            ],
            notes="Stylized Ruby methods and dynamic-risk patterns emit explicit migration evidence.",
            fit_rationale=(
                "Best fit: Ruby style orchestration and scripting surfaces can be migrated as "
                "policy/flow code with manageable risk captured in migrate_todo output."
            ),
        ),
        LanguageLane(
            id="python",
            label="Python",
            status="active",
            evidence=[
                "garnet-convert/src/frontends/python.rs",
                "garnet-convert/tests/corpus.rs",
            ],
            notes="Stylized Python defs/classes and decorators become Garnet plus migrate_todo items.",
            fit_rationale=(
                "Best fit: high-level Python data-flow and service glue often convert cleanly, "
                "while risky runtime patterns are surfaced by migration evidence."
            ),
        ),
        LanguageLane(
            id="go",
            label="Go",
            status="active",
            evidence=[
                "garnet-convert/src/frontends/go.rs",
                "garnet-convert/tests/corpus.rs",
            ],
            notes="Stylized Go funcs/structs lower into Garnet with ownership and lineage evidence.",
            fit_rationale=(
                "Best fit: Go service and utility constructs already have a stable deterministic "
                "frontend shape suitable for migration assistant output."
            ),
        ),
    ]


def planned_languages() -> list[LanguageLane]:
    planned = [
        (
            "javascript",
            "JavaScript",
            (
                "Parser and runtime shape are dynamic and side-effect-heavy, so deterministic "
                "frontend output must be proven before direct conversion."
            ),
        ),
        (
            "typescript",
            "TypeScript",
            (
                "Type-stripping conversion is currently incomplete for complex generics and emits "
                "insufficient deterministic ownership evidence."
            ),
        ),
        (
            "swift",
            "Swift",
            (
                "ARC lifecycle and Objective-C interoperability must remain source-of-truth through "
                "native modules until compiler and memory guarantees are fully mapped."
            ),
        ),
        (
            "java",
            "Java",
            (
                "JVM bytecode/runtime contracts and exception-flow semantics require a proven deterministic "
                "frontend and migration test suite first."
            ),
        ),
        (
            "c",
            "C",
            (
                "ABI, pointer semantics, and platform-specific layout assumptions argue for native modules/FFI "
                "unless a backend lowering strategy is proven."
            ),
        ),
        (
            "cpp",
            "C++",
            (
                "Templates, overload resolution, and low-level object/memory behavior require backend-backed "
                "lowering before source conversion is reliable."
            ),
        ),
        (
            "csharp",
            "C#",
            (
                "CLR identity, reflection, and runtime type behaviors need deterministic frontend and "
                "evidence gates before deterministic migration."
            ),
        ),
        (
            "perl",
            "Perl",
            (
                "Dynamic symbol tables and context-dependent execution patterns prevent safe deterministic translation "
                "without a dedicated frontend."
            ),
        ),
        (
            "kotlin",
            "Kotlin",
            (
                "Coroutines, platform-specific nullability, and JVM/runtime interactions require proven checks and "
                "frontend maturity."
            ),
        ),
        (
            "shell",
            "Shell",
            (
                "Process graph execution and environment state coupling do not preserve well to source conversion today."
            ),
        ),
        (
            "sql",
            "SQL",
            (
                "Schema migration, dialect variance, and execution side effects require dedicated planning rather "
                "than direct converter output."
            ),
        ),
        (
            "other",
            "Other",
            (
                "No deterministic frontend is available yet; classification, risk inventory, and pilot plans "
                "must precede converter claims."
            ),
        ),
    ]
    return [
        LanguageLane(
            id=identifier,
            label=label,
            status="planned",
            evidence=[],
            notes=(
                "Not implemented in the deterministic converter yet; must enter through "
                "a tested frontend, CIR lineage, sandbox output, and dogfood readiness gate."
            ),
            fit_rationale=(
                rationale
                + " Requires a deterministic frontend, checker support, and evidence pipeline before conversion can be active."
            ),
        )
        for identifier, label, rationale in planned
    ]


def native_boundary_languages() -> list[LanguageLane]:
    native = [
        ("c", "C"),
        ("cpp", "C++"),
        ("objective_c", "Objective-C"),
        ("assembly", "Assembly"),
        ("cuda", "CUDA"),
        ("platform_specific", "platform-specific code"),
    ]
    return [
        LanguageLane(
            id=identifier,
            label=label,
            status="native-boundary-recommended",
            evidence=[],
            notes=(
                "Prefer native modules or FFI with explicit Garnet CapCaps, memory declarations, "
                "lineage, and sandbox boundaries instead of direct source-to-source conversion."
            ),
            fit_rationale=(
                "Low-level control, ABI constraints, or platform contracts would be lost by "
                "direct source translation without a bound compiler backend."
            ),
        )
        for identifier, label in native
    ]


def read_status() -> ConverterStatus:
    return ConverterStatus(
        source=str(ROOT / "garnet-convert"),
        converter_scope="bidirectional-advisory-migration-surface",
        active_languages=active_languages(),
        planned_languages=planned_languages(),
        native_boundary_languages=native_boundary_languages(),
        trust_boundaries=TrustBoundaries(
            sandbox_on_by_default=True,
            lineage_required=True,
            migrate_todo_required=True,
            source_execution_allowed=False,
            human_audit_required_to_unquarantine=True,
        ),
        llm_assist=LlmAssist(
            status="proposed-gated",
            advisory_only=True,
            enabled_by_default=False,
            requires_lineage=True,
            requires_sandbox=True,
            requires_garnet_check=True,
            requires_dogfood_readiness=True,
            notes=[
                "LLM output must never replace deterministic converter evidence.",
                "Model suggestions should be treated as migrate_todo guidance until checked.",
                "No provider, model, or network dependency is required for the current converter.",
            ],
        ),
        intelligent_assist_contract=IntelligentAssistContract(
            status="planned-contract",
            provider_required=False,
            model_required=False,
            pipeline=[
                "source language classifier",
                "risk inventory",
                "Garnet-aware context pack",
                "advisory plan",
                "review handoff",
                "human-approved candidate",
                "garnet check/test/dogfood",
            ],
            required_context=[
                "CURRENT_STATE.md",
                "README.md",
                "C_Language_Specification/GARNET_v1_0_Mini_Spec.md",
                "C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md",
                "F_Project_Management/DOGFOOD/GARNET_v0_5_DOGFOOD_READINESS_PHASE_LOG.md",
            ],
            analysis_targets=[
                "safe-mode ownership candidates",
                "memory declarations",
                "CapCaps/capability boundaries",
                "actor/orchestration mappings",
                "migration risk inventory",
            ],
            required_gates=[
                "lineage per emitted node",
                "@sandbox default",
                "migrate_todo evidence",
                "garnet check",
                "dogfood readiness bundle",
                "human audit before unquarantine",
            ],
            notes=[
                "The contract describes what a future LLM or agentic assist lane must preserve.",
                "It does not make any source language active without a deterministic frontend.",
                "Provider choice remains outside the converter contract until a secure execution design exists.",
            ],
        ),
        backend_lowering_strategy=BackendLoweringStrategy(
            status="planned-two-way-architecture",
            planned_targets=[
                "Wasm",
                "LLVM-style native targets",
                "native package toolchains",
            ],
            use_cases=[
                "performance-sensitive kernels written in Garnet",
                "system integration where Garnet should lower out rather than replace native code",
                "sandboxed plugin surfaces that need portable execution",
            ],
            guardrails=[
                "Do not claim source-to-source conversion preserves low-level fidelity.",
                "Do not claim Wasm, LLVM, or native backends are implemented until compiler evidence lands.",
                "Keep C/C++/Objective-C/Assembly/CUDA/platform-specific code behind FFI/native-module contracts until then.",
            ],
        ),
    )


def render_markdown(status: ConverterStatus) -> str:
    active_labels = "Rust, Ruby, Python, and Go"
    native_labels = ", ".join(language.label for language in status.native_boundary_languages)
    backend_targets = ", ".join(status.backend_lowering_strategy.planned_targets)
    lines = [
        "# Garnet Converter Adoption Status",
        "",
        f"Source: `{status.source}`",
        "",
        (
            f"Active today: {active_labels}. This is a "
            "stylized migration assistant, not a full transpiler."
        ),
        "",
        "Best-fit imports: agent orchestration, app glue, capability-aware workflows, memory-aware services, and migration planning.",
        "",
        "Bad direct-conversion fits: kernels, drivers, pointer-heavy native systems, GPU kernels, hard real-time paths, SIMD loops, and platform ABI glue.",
        "",
        "## Active Deterministic Lanes",
        "",
    ]
    for language in status.active_languages:
        evidence = ", ".join(f"`{item}`" for item in language.evidence)
        lines.append(
            f"- {language.label}: {language.notes} Why: {language.fit_rationale} Evidence: {evidence}."
        )

    lines.extend(
        [
            "",
            "## Planned Adoption Lanes",
            "",
            "JavaScript / TypeScript are currently the first planned frontend expansion path, but not implemented yet.",
            "All planned languages are advisory-only until deterministic fronts are proven.",
            "",
        ]
    )
    for language in status.planned_languages:
        lines.append(
            f"- {language.label}: Why this is not a direct fit yet: {language.fit_rationale}"
        )
    lines.extend(
        [
            "",
            "## Native Boundary Recommended",
            "",
        ]
    )
    for language in status.native_boundary_languages:
        lines.append(f"- {language.label}: {language.notes} Why: {language.fit_rationale}")
    lines.extend(
        [
            "",
            f"- Native summary: {native_labels}.",
            "",
            "## Backend Lowering Strategy",
            "",
            f"- Status: {status.backend_lowering_strategy.status}.",
            f"- Planned targets: {backend_targets}.",
            "- Garnet should lower out to native backends for performance and system integration rather than pretending every low-level source maps cleanly into Garnet.",
            "",
            "## Trust Boundaries",
            "",
            "- Converted output stays sandboxed by default.",
            "- Source code is not executed during conversion.",
            "- Lineage and migrate_todo evidence remain required.",
            "- Human audit is required before unquarantine.",
            "",
            "## Gated LLM Assist",
            "",
            (
                "LLM assist is proposed only as a gated advisory lane: it may suggest "
                "Garnet-aware rewrites, safe-mode opportunities, or migration notes, but "
                "deterministic converter output, lineage, sandboxing, `garnet check`, and "
                "dogfood readiness remain the authority."
            ),
            "",
            "## Planned Garnet-Aware Assist Contract",
            "",
            (
                "A future intelligent assist lane must read the current Garnet context pack, "
                "remain provider-optional, and produce only auditable migration guidance until "
                "the deterministic converter/checker accepts the result."
            ),
            "",
            "Required context:",
        ]
    )
    for item in status.intelligent_assist_contract.required_context:
        lines.append(f"- `{item}`")

    lines.extend(["", "Required analysis targets:"])
    for item in status.intelligent_assist_contract.analysis_targets:
        lines.append(f"- {item}")

    lines.extend(["", "Required gates:"])
    for item in status.intelligent_assist_contract.required_gates:
        lines.append(f"- {item}")

    lines.extend(["", "Recommended pipeline:"])
    lines.append(" -> ".join(status.intelligent_assist_contract.pipeline))

    lines.extend(
        [
            "",
            (
                "This keeps broad-language adoption possible without claiming that LLM "
                "conversion, provider execution, or non-deterministic rewrites are active today."
            ),
        ]
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
