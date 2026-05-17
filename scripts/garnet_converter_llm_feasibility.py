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
class ProviderOption:
    id: str
    label: str
    status: str
    best_role: str
    why_consider_it: str
    caution: str
    first_safe_use: str
    provider_backed_conversion_allowed: bool
    enabled_by_default: bool
    source_inclusion_default: str
    requires_privacy_review: bool
    requires_human_approval: bool


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
    native_boundary_languages: list[garnet_converter_status.LanguageLane]
    planned_language_assist_coverage: list[PlannedLanguageCoverage]
    provider_options: list[ProviderOption]
    recommended_pipeline: list[str]
    required_context: list[str]
    analysis_targets: list[str]
    required_gates: list[str]
    blocking_gates: list[str]
    recommendation: list[str]


def provider_options() -> list[ProviderOption]:
    common_first_use = "advisory risk inventory or migration-plan review only"
    return [
        ProviderOption(
            id="openai-gpt-5-5-class",
            label="OpenAI GPT-5.5 class models",
            status="candidate-to-evaluate",
            best_role="deep migration reasoning, policy synthesis, high-quality review",
            why_consider_it="strong instruction following and code reasoning for multi-file migrations",
            caution="cost and privacy controls must be explicit",
            first_safe_use=common_first_use,
            provider_backed_conversion_allowed=False,
            enabled_by_default=False,
            source_inclusion_default="omit-source-by-default",
            requires_privacy_review=True,
            requires_human_approval=True,
        ),
        ProviderOption(
            id="anthropic-claude-opus-sonnet-class",
            label="Anthropic Claude Opus/Sonnet class models",
            status="candidate-to-evaluate",
            best_role="long-context planning, careful rewrite review, human-readable handoffs",
            why_consider_it="strong codebase-scale reasoning and conservative critique",
            caution="source inclusion must stay opt-in and reviewed",
            first_safe_use=common_first_use,
            provider_backed_conversion_allowed=False,
            enabled_by_default=False,
            source_inclusion_default="omit-source-by-default",
            requires_privacy_review=True,
            requires_human_approval=True,
        ),
        ProviderOption(
            id="xai-grok-code",
            label="xAI Grok code models",
            status="candidate-to-evaluate",
            best_role="fast exploratory code review and alternate migration hypotheses",
            why_consider_it="useful as a second-opinion reviewer when available",
            caution="treat output as advisory until local gates pass",
            first_safe_use=common_first_use,
            provider_backed_conversion_allowed=False,
            enabled_by_default=False,
            source_inclusion_default="omit-source-by-default",
            requires_privacy_review=True,
            requires_human_approval=True,
        ),
        ProviderOption(
            id="kimi-moonshot-k-series",
            label="Kimi/Moonshot Kimi K-series",
            status="candidate-to-evaluate",
            best_role="large-context source understanding and lower-cost batch analysis",
            why_consider_it="attractive for repo-scale risk inventory and summaries",
            caution="verify API availability, privacy terms, and model behavior before provider integration",
            first_safe_use=common_first_use,
            provider_backed_conversion_allowed=False,
            enabled_by_default=False,
            source_inclusion_default="omit-source-by-default",
            requires_privacy_review=True,
            requires_human_approval=True,
        ),
        ProviderOption(
            id="google-gemini-gemma",
            label="Google Gemini/Gemma",
            status="candidate-to-evaluate",
            best_role="multimodal docs plus code-context review; local variants for private runs",
            why_consider_it="broad ecosystem coverage and possible local/private model variants",
            caution="do not mix marketing-site claims with unproven conversion",
            first_safe_use=common_first_use,
            provider_backed_conversion_allowed=False,
            enabled_by_default=False,
            source_inclusion_default="omit-source-by-default",
            requires_privacy_review=True,
            requires_human_approval=True,
        ),
        ProviderOption(
            id="deepseek-coder",
            label="DeepSeek coder models",
            status="candidate-to-evaluate",
            best_role="cost-sensitive code translation drafts and risk extraction",
            why_consider_it="useful for cheap batch advisory passes",
            caution="needs strict hallucination and security gates",
            first_safe_use=common_first_use,
            provider_backed_conversion_allowed=False,
            enabled_by_default=False,
            source_inclusion_default="omit-source-by-default",
            requires_privacy_review=True,
            requires_human_approval=True,
        ),
        ProviderOption(
            id="qwen-coder",
            label="Qwen coder models",
            status="candidate-to-evaluate",
            best_role="multilingual codebase understanding and local/open-weight paths",
            why_consider_it="good coverage across languages and deployment options",
            caution="must be benchmarked on Garnet-specific truth",
            first_safe_use=common_first_use,
            provider_backed_conversion_allowed=False,
            enabled_by_default=False,
            source_inclusion_default="omit-source-by-default",
            requires_privacy_review=True,
            requires_human_approval=True,
        ),
        ProviderOption(
            id="local-1-58-bit",
            label="local 1.58-bit models",
            status="candidate-to-evaluate",
            best_role="private-code advisory summaries on developer machines",
            why_consider_it="useful bridge for proprietary codebases where source cannot leave local hardware",
            caution="quality may be uneven; use for triage, not authority",
            first_safe_use=common_first_use,
            provider_backed_conversion_allowed=False,
            enabled_by_default=False,
            source_inclusion_default="omit-source-by-default",
            requires_privacy_review=True,
            requires_human_approval=True,
        ),
        ProviderOption(
            id="domain-fine-tuned-garnet-adapter",
            label="Domain-fine-tuned Garnet adapter",
            status="long-term-candidate",
            best_role="Garnet-specific syntax and policy reconstruction",
            why_consider_it="best long-term quality if enough verified Garnet examples exist",
            caution="only after the language and conformance suite stabilize",
            first_safe_use="advisory syntax critique after a larger verified corpus exists",
            provider_backed_conversion_allowed=False,
            enabled_by_default=False,
            source_inclusion_default="omit-source-by-default",
            requires_privacy_review=True,
            requires_human_approval=True,
        ),
        ProviderOption(
            id="multi-model-reviewer-quorum",
            label="Multi-model reviewer quorum",
            status="long-term-candidate",
            best_role="cross-check migration plans before candidate output",
            why_consider_it="helps separate consensus risks from one-model style bias",
            caution="expensive and slower; still needs deterministic gates",
            first_safe_use="advisory review quorum for already-manifested no-source handoff packets",
            provider_backed_conversion_allowed=False,
            enabled_by_default=False,
            source_inclusion_default="omit-source-by-default",
            requires_privacy_review=True,
            requires_human_approval=True,
        ),
    ]


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
            "broad planned languages require separate deterministic frontend slices or native-boundary handoffs",
        ],
        active_languages=converter.active_languages,
        planned_languages=converter.planned_languages,
        native_boundary_languages=converter.native_boundary_languages,
        planned_language_assist_coverage=coverage,
        provider_options=provider_options(),
        recommended_pipeline=contract.pipeline,
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
    native = ", ".join(language.label for language in status.native_boundary_languages)
    coverage = "\n".join(
        f"| {language.label} | `{language.status}` | "
        f"{'yes' if language.deterministic_converter_available else 'no'} | "
        f"{'yes' if language.llm_conversion_active else 'no'} | {language.required_next_gate} |"
        for language in status.planned_language_assist_coverage
    )
    options = "\n".join(
        f"| {option.label} | `{option.status}` | {option.best_role} | "
        f"{option.first_safe_use} | provider-backed conversion allowed: "
        f"{str(option.provider_backed_conversion_allowed).lower()} | "
        f"{option.source_inclusion_default} |"
        for option in status.provider_options
    )
    return f"""# Garnet Converter LLM Feasibility

Source: `{status.source}`

Status: **{status.status}**

Advisory assist is feasible as a provider-neutral planning lane. Autonomous LLM conversion is not feasible yet.

## Current Truth

- Active deterministic converter lanes: {active}.
- Planned language lanes: {planned}.
- Native boundary recommended lanes: {native}.
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

## Provider Option Registry

Provider choices are evaluation candidates for future advisory review lanes only. They do not enable provider-backed conversion.

| Option | Status | Best role | First safe use | Conversion boundary | Source default |
| --- | --- | --- | --- | --- | --- |
{options}

## Native Boundary Coverage

{chr(10).join(f"- {language.label}: {language.notes}" for language in status.native_boundary_languages)}

## Recommended Pipeline

{chr(10).join(f"- {step}" for step in status.recommended_pipeline)}

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
