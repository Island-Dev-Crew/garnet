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
class ConverterStatus:
    source: str
    converter_scope: str
    active_languages: list[LanguageLane]
    planned_languages: list[LanguageLane]
    trust_boundaries: TrustBoundaries
    llm_assist: LlmAssist


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
        ),
    ]


def planned_languages() -> list[LanguageLane]:
    planned = [
        ("javascript", "JavaScript"),
        ("typescript", "TypeScript"),
        ("swift", "Swift"),
        ("java", "Java"),
        ("c", "C"),
        ("cpp", "C++"),
        ("csharp", "C#"),
        ("perl", "Perl"),
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
        )
        for identifier, label in planned
    ]


def read_status() -> ConverterStatus:
    return ConverterStatus(
        source=str(ROOT / "garnet-convert"),
        converter_scope="stylized-migration-assistant",
        active_languages=active_languages(),
        planned_languages=planned_languages(),
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
    )


def render_markdown(status: ConverterStatus) -> str:
    active_labels = "Rust, Ruby, Python, and Go"
    planned_labels = " / ".join(["JavaScript", "TypeScript"])
    other_planned = ", ".join(
        language.label
        for language in status.planned_languages
        if language.label not in {"JavaScript", "TypeScript"}
    )
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
        "## Active Deterministic Lanes",
        "",
    ]
    for language in status.active_languages:
        evidence = ", ".join(f"`{item}`" for item in language.evidence)
        lines.append(f"- {language.label}: {language.notes} Evidence: {evidence}.")

    lines.extend(
        [
            "",
            "## Planned Adoption Lanes",
            "",
            (
                f"- {planned_labels}: planned as the first web/frontend expansion lane, "
                "but not implemented yet."
            ),
            f"- {other_planned}: planned only after a tested frontend boundary exists.",
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
