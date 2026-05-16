#!/usr/bin/env python3
"""Build the local Garnet-aware assist context pack.

This is a deterministic documentation and status bundle for future LLM or
agentic converter assistance. It does not call a provider, execute source code,
or make any planned language frontend active.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import garnet_converter_status  # noqa: E402
import garnet_mit_readiness_status  # noqa: E402


@dataclass(frozen=True)
class ContextDocument:
    path: str
    role: str
    exists: bool
    bytes: int
    sha256: str


@dataclass(frozen=True)
class AssistContextPack:
    source: str
    status: str
    assist_contract_status: str
    provider_required: bool
    model_required: bool
    network_required: bool
    enabled_by_default: bool
    llm_conversion_active: bool
    current_truth: list[str]
    active_languages: list[str]
    planned_languages: list[str]
    context_documents: list[ContextDocument]
    analysis_targets: list[str]
    required_gates: list[str]
    system_boundaries: list[str]
    prompt_pack: PromptPack
    objective_status: str
    objective_completion_percent: float


@dataclass(frozen=True)
class PromptPack:
    status: str
    provider_required: bool
    network_required: bool
    conversion_active: bool
    required_inputs: list[str]
    required_output_sections: list[str]
    forbidden_claims: list[str]
    system_prompt: str
    user_prompt_template: str


DOCUMENT_ROLES = {
    "CURRENT_STATE.md": "current-truth",
    "README.md": "public-entry",
    "C_Language_Specification/GARNET_v1_0_Mini_Spec.md": "spec",
    "C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md": "conformance",
    "F_Project_Management/DOGFOOD/GARNET_v0_5_DOGFOOD_READINESS_PHASE_LOG.md": "dogfood",
}


def _prompt_pack(contract: garnet_converter_status.IntelligentAssistContract) -> PromptPack:
    required_output_sections = [
        "current truth and scope",
        "source summary and lineage notes",
        "risk inventory",
        "candidate Garnet output or migrate_todo evidence",
        "required gates evidence",
        "human audit notes",
    ]
    forbidden_claims = [
        "Never claim conversion is active.",
        "Never claim broad planned-language support.",
        "Never claim provider-backed LLM conversion is enabled.",
        "Never mark unchecked Garnet output safe.",
        "Never remove sandbox, lineage, dogfood, or human-audit gates.",
    ]
    gates = "; ".join(contract.required_gates)
    targets = "; ".join(contract.analysis_targets)
    system_prompt = "\n".join(
        [
            "You are a Garnet migration assistant.",
            "Use only the provided Garnet context pack, source summary, and assist plan.",
            "Do not execute source code.",
            "Never claim conversion is active.",
            "Treat deterministic converter output as authoritative when an active frontend exists.",
            "For planned languages, emit advisory migration evidence only.",
            f"Analyze these targets: {targets}.",
            f"Preserve these gates: {gates}.",
            "Keep output sandboxed by default and require human audit before unquarantine.",
        ]
    )
    user_prompt_template = "\n".join(
        [
            "Given:",
            "- Garnet assist context pack JSON",
            "- Garnet converter assist plan JSON",
            "- Source language and source file hash",
            "",
            "Return Markdown with these sections:",
            *[f"- {section}" for section in required_output_sections],
            "",
            "Do not execute source. Do not claim active conversion.",
            "Every candidate output must preserve lineage per emitted node, @sandbox default,",
            "migrate_todo evidence for unsupported constructs, garnet check before validity",
            "claims, a dogfood readiness bundle path, and human audit before unquarantine.",
        ]
    )
    return PromptPack(
        status="provider-neutral-assist-prompt",
        provider_required=False,
        network_required=False,
        conversion_active=False,
        required_inputs=[
            "assist context pack JSON",
            "assist plan JSON",
            "source language id",
            "source file sha256",
            "human review objective",
        ],
        required_output_sections=required_output_sections,
        forbidden_claims=forbidden_claims,
        system_prompt=system_prompt,
        user_prompt_template=user_prompt_template,
    )


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _document(path_text: str) -> ContextDocument:
    path = ROOT / path_text
    if not path.exists():
        return ContextDocument(
            path=path_text,
            role=DOCUMENT_ROLES[path_text],
            exists=False,
            bytes=0,
            sha256="",
        )
    return ContextDocument(
        path=path_text,
        role=DOCUMENT_ROLES[path_text],
        exists=True,
        bytes=path.stat().st_size,
        sha256=_sha256(path),
    )


def read_pack() -> AssistContextPack:
    converter = garnet_converter_status.read_status()
    contract = converter.intelligent_assist_contract
    objective = garnet_mit_readiness_status.read_status()
    required_context = list(dict.fromkeys(contract.required_context))

    return AssistContextPack(
        source=str(ROOT),
        status="active-context-pack",
        assist_contract_status=contract.status,
        provider_required=contract.provider_required,
        model_required=contract.model_required,
        network_required=False,
        enabled_by_default=False,
        llm_conversion_active=False,
        current_truth=[
            "not active conversion today",
            "deterministic converter output remains authoritative",
            "provider-backed assist remains a future gated lane",
            "planned languages are not active until tested frontends land",
        ],
        active_languages=[language.label for language in converter.active_languages],
        planned_languages=[language.label for language in converter.planned_languages],
        context_documents=[_document(path) for path in required_context],
        analysis_targets=contract.analysis_targets,
        required_gates=contract.required_gates,
        system_boundaries=[
            "deterministic converter output remains authoritative",
            "source code is not executed during analysis",
            "LLM suggestions are migrate_todo guidance until checked",
            "converted output stays @sandbox by default",
            "human audit is required before unquarantine",
        ],
        prompt_pack=_prompt_pack(contract),
        objective_status=objective.overall_status,
        objective_completion_percent=objective.completion_percent,
    )


def render_markdown(pack: AssistContextPack) -> str:
    lines = [
        "# Garnet Assist Context Pack",
        "",
        f"Source: `{pack.source}`",
        "",
        f"Status: **{pack.status}**",
        f"Assist contract: **{pack.assist_contract_status}**",
        "",
        (
            "Current truth: this is not active conversion today. It is a "
            "provider-optional, local context pack for future Garnet-aware "
            "LLM or agentic assist."
        ),
        "",
        f"Active deterministic converter lanes: {', '.join(pack.active_languages)}.",
        f"Planned language lanes: {', '.join(pack.planned_languages)}.",
        "",
        "## Boundaries",
        "",
    ]
    for boundary in pack.system_boundaries:
        lines.append(f"- {boundary}")

    lines.extend(["", "## Context Documents", ""])
    for document in pack.context_documents:
        status = "present" if document.exists else "missing"
        lines.append(
            f"- `{document.path}` ({document.role}): {status}, "
            f"{document.bytes} bytes, sha256 `{document.sha256}`"
        )

    lines.extend(["", "## Analysis Targets", ""])
    for target in pack.analysis_targets:
        lines.append(f"- {target}")

    lines.extend(["", "## Required Gates", ""])
    for gate in pack.required_gates:
        lines.append(f"- {gate}")

    lines.extend(["", "## Provider-Neutral Prompt Pack", ""])
    lines.extend(
        [
            f"Status: **{pack.prompt_pack.status}**",
            f"Provider required: {str(pack.prompt_pack.provider_required).lower()}",
            f"Network required: {str(pack.prompt_pack.network_required).lower()}",
            f"Conversion active: {str(pack.prompt_pack.conversion_active).lower()}",
            "",
            "Required output sections:",
        ]
    )
    for section in pack.prompt_pack.required_output_sections:
        lines.append(f"- {section}")
    lines.extend(["", "Forbidden claims:", ""])
    for claim in pack.prompt_pack.forbidden_claims:
        lines.append(f"- {claim}")

    lines.extend(
        [
            "",
            "## Objective Accounting",
            "",
            (
                f"The broader MIT/productization objective remains "
                f"`{pack.objective_status}` at "
                f"{pack.objective_completion_percent:.1f}%."
            ),
            "",
            (
                "Do not treat this context pack as a provider-backed converter, "
                "a broad-language frontend, or a replacement for lineage, "
                "sandboxing, `garnet check`, dogfood readiness, and human audit."
            ),
        ]
    )
    return "\n".join(lines) + "\n"


def render_prompt_pack(pack: AssistContextPack) -> str:
    prompt = pack.prompt_pack
    lines = [
        "# Garnet Provider-Neutral Assist Prompt Pack",
        "",
        f"Status: **{prompt.status}**",
        "",
        "Current truth: this prompt pack is not an active converter, not a model",
        "provider integration, and not permission to remove Garnet's migration gates.",
        "",
        "## Required Inputs",
        "",
    ]
    lines.extend(f"- {item}" for item in prompt.required_inputs)
    lines.extend(["", "## System Prompt", "", "```text", prompt.system_prompt, "```", ""])
    lines.extend(["## User Prompt Template", "", "```text", prompt.user_prompt_template, "```", ""])
    lines.extend(["## Forbidden Claims", ""])
    lines.extend(f"- {claim}" for claim in prompt.forbidden_claims)
    return "\n".join(lines) + "\n"


def write_output_dir(pack: AssistContextPack, output_dir: Path) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    json_path = output_dir / "garnet-assist-context-pack.json"
    md_path = output_dir / "garnet-assist-context-pack.md"
    prompt_path = output_dir / "garnet-assist-prompt-pack.md"
    manifest_path = output_dir / "MANIFEST.sha256"

    json_path.write_text(json.dumps(asdict(pack), indent=2) + "\n", encoding="utf-8")
    md_path.write_text(render_markdown(pack), encoding="utf-8")
    prompt_path.write_text(render_prompt_pack(pack), encoding="utf-8")
    entries = []
    for path in (json_path, md_path, prompt_path):
        entries.append(f"{_sha256(path)}  {path.name}\n")
    manifest_path.write_text("".join(entries), encoding="utf-8")


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--format",
        choices=("markdown", "json"),
        default="markdown",
        help="output format",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        help="write JSON, Markdown, and MANIFEST.sha256 into this directory",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    pack = read_pack()
    if args.output_dir:
        write_output_dir(pack, args.output_dir)

    if args.format == "json":
        print(json.dumps(asdict(pack), indent=2))
    else:
        print(render_markdown(pack), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
