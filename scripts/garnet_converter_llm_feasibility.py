#!/usr/bin/env python3
"""Report feasibility for a future Garnet-aware converter LLM assist lane.

This is a planning/evidence surface, not an LLM integration. It answers the
current product question directly: an LLM can be viable as a provider-neutral
advisory planner, but broad/autonomous conversion is still blocked until the
deterministic converter, checker, sandbox, dogfood, and human-audit gates exist.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import garnet_assist_context_pack  # noqa: E402
import garnet_converter_status  # noqa: E402


@dataclass(frozen=True)
class PlannedLanguageCoverage:
    id: str
    label: str
    status: str
    deterministic_converter_available: bool
    llm_conversion_active: bool
    required_next_gate: str


@dataclass(frozen=True)
class ConverterLlmFeasibility:
    source: str
    status: str
    recommended_first_lane: str
    conversion_active: bool
    autonomous_conversion_feasible: bool
    provider_required: bool
    model_required: bool
    network_required: bool
    enabled_by_default: bool
    current_truth: list[str]
    active_languages: list[garnet_converter_status.LanguageLane]
    planned_languages: list[garnet_converter_status.LanguageLane]
    planned_language_assist_coverage: list[PlannedLanguageCoverage]
    required_context: list[str]
    analysis_targets: list[str]
    required_gates: list[str]
    blocking_gates: list[str]
    recommendation: list[str]


def read_status() -> ConverterLlmFeasibility:
    converter = garnet_converter_status.read_status()
    pack = garnet_assist_context_pack.read_pack()
    contract = converter.intelligent_assist_contract
    active_ids = {language.id for language in converter.active_languages}

    coverage = [
        PlannedLanguageCoverage(
            id=language.id,
            label=language.label,
            status="planned-assist-required",
            deterministic_converter_available=language.id in active_ids,
            llm_conversion_active=False,
            required_next_gate=(
                "deterministic frontend + corpus fixtures + lineage/sandbox/check dogfood gate"
            ),
        )
        for language in converter.planned_languages
    ]

    return ConverterLlmFeasibility(
        source=str(ROOT),
        status="advisory-feasible",
        recommended_first_lane="provider-neutral advisory planning",
        conversion_active=False,
        autonomous_conversion_feasible=False,
        provider_required=contract.provider_required,
        model_required=contract.model_required,
        network_required=False,
        enabled_by_default=False,
        current_truth=[
            "advisory planning can be useful now",
            "not active LLM conversion",
            "source code must not be executed during analysis",
            "deterministic converter output remains authoritative",
            "broad planned languages require separate deterministic frontend slices",
        ],
        active_languages=converter.active_languages,
        planned_languages=converter.planned_languages,
        planned_language_assist_coverage=coverage,
        required_context=[document.path for document in pack.context_documents if document.exists],
        analysis_targets=contract.analysis_targets,
        required_gates=contract.required_gates,
        blocking_gates=[
            "secure advisory implementation",
            "provider/runtime boundary",
            "dogfood gate",
            "deterministic planned-language frontend gates",
            "human audit before unquarantine",
        ],
        recommendation=[
            "Start with an offline/provider-neutral feasibility and prompt-pack lane.",
            "Use any future model only to draft migration notes or quarantined Garnet candidates.",
            "Require lineage, @sandbox defaults, migrate_todo evidence, garnet check, dogfood evidence, and human audit before unquarantine.",
            "Do not place LLM output in the authoritative converter path until deterministic frontends and CI gates exist.",
        ],
    )


def render_markdown(status: ConverterLlmFeasibility) -> str:
    active = ", ".join(language.label for language in status.active_languages)
    planned = ", ".join(language.label for language in status.planned_languages)
    coverage = "\n".join(
        f"| {language.label} | `{language.status}` | "
        f"{'yes' if language.deterministic_converter_available else 'no'} | "
        f"{'yes' if language.llm_conversion_active else 'no'} | {language.required_next_gate} |"
        for language in status.planned_language_assist_coverage
    )
    return f"""# Garnet Converter LLM Feasibility

Source: `{status.source}`

Status: **{status.status}**

Advisory assist is feasible as a provider-neutral planning lane. Autonomous LLM conversion is not feasible yet.

## Current Truth

- Active deterministic converter lanes: {active}.
- Planned language lanes: {planned}.
- Recommended first lane: `{status.recommended_first_lane}`.
- Conversion active: `{str(status.conversion_active).lower()}`.
- Provider required now: `{str(status.provider_required).lower()}`.
- Model required now: `{str(status.model_required).lower()}`.
- Network required now: `{str(status.network_required).lower()}`.
- Autonomous conversion feasible: `{str(status.autonomous_conversion_feasible).lower()}`.
{chr(10).join(f"- {item}." for item in status.current_truth)}

## Planned Language Coverage

| Language | Status | Deterministic converter | LLM conversion active | Required next gate |
| --- | --- | --- | --- | --- |
{coverage}

## Required Gates

{chr(10).join(f"- {gate}" for gate in status.required_gates)}

## Blocking Gates

{chr(10).join(f"- {gate}" for gate in status.blocking_gates)}

## Recommendation

{chr(10).join(f"- {item}" for item in status.recommendation)}
"""


def write_outputs(status: ConverterLlmFeasibility, output_dir: Path) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    (output_dir / "garnet-converter-llm-feasibility.json").write_text(
        json.dumps(asdict(status), indent=2),
        encoding="utf-8",
    )
    (output_dir / "garnet-converter-llm-feasibility.md").write_text(
        render_markdown(status),
        encoding="utf-8",
    )
    subprocess.run(
        "find . -type f ! -name MANIFEST.sha256 ! -name MANIFEST.verify.log -print0 | "
        "sort -z | xargs -0 shasum -a 256 > MANIFEST.sha256 && "
        "shasum -a 256 -c MANIFEST.sha256 > MANIFEST.verify.log",
        cwd=output_dir,
        shell=True,
        check=True,
    )


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=("markdown", "json"), default="markdown")
    parser.add_argument("--output-dir", help="write JSON/Markdown plus MANIFEST.sha256")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    status = read_status()
    if args.output_dir:
        write_outputs(status, Path(args.output_dir).expanduser().resolve())
    if args.format == "json":
        print(json.dumps(asdict(status), indent=2))
    else:
        print(render_markdown(status))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
