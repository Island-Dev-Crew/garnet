#!/usr/bin/env python3
"""Build a provider-neutral Garnet converter advisory bundle.

This script is the safe handoff layer between deterministic converter evidence
and any future LLM or agentic migration assistant. It combines the current
Garnet context pack, the per-file assist plan, and the converter LLM feasibility
decision into one manifested bundle. It does not call a provider, execute source
code, or enable autonomous conversion.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import garnet_assist_context_pack  # noqa: E402
import garnet_converter_assist_plan  # noqa: E402
import garnet_converter_llm_feasibility  # noqa: E402


@dataclass(frozen=True)
class ConverterAdvisoryBundle:
    source: str
    status: str
    language: str
    language_id: str
    recommended_first_lane: str
    provider_required: bool
    model_required: bool
    network_required: bool
    enabled_by_default: bool
    conversion_active: bool
    source_included: bool
    source_privacy_mode: str
    source_text: str | None
    human_review_required: bool
    current_truth: list[str]
    required_inputs: list[str]
    required_output_sections: list[str]
    source_summary: dict[str, Any]
    context_pack: dict[str, Any]
    assist_plan: dict[str, Any]
    llm_feasibility: dict[str, Any]


def read_bundle(language: str, source: Path, include_source: bool = False) -> ConverterAdvisoryBundle:
    plan = garnet_converter_assist_plan.read_plan(language, source)
    pack = garnet_assist_context_pack.read_pack()
    feasibility = garnet_converter_llm_feasibility.read_status()
    source_text = source.read_text(encoding="utf-8") if include_source else None

    return ConverterAdvisoryBundle(
        source=str(ROOT),
        status="active-advisory-bundle",
        language=plan.language,
        language_id=plan.language_id,
        recommended_first_lane=feasibility.recommended_first_lane,
        provider_required=False,
        model_required=False,
        network_required=False,
        enabled_by_default=False,
        conversion_active=False,
        source_included=include_source,
        source_privacy_mode=(
            "source included for local or explicitly approved provider handoff"
            if include_source
            else "source omitted by default; attach source only for local or explicitly approved provider handoff"
        ),
        source_text=source_text,
        human_review_required=True,
        current_truth=[
            "advisory bundle is not active conversion",
            "provider-backed conversion remains inactive",
            "source text is omitted by default",
            "source code is not executed during analysis",
            "deterministic converter output remains authoritative",
            "human audit is required before unquarantine",
        ],
        required_inputs=[
            "assist context pack JSON",
            "assist plan JSON",
            "converter LLM feasibility JSON",
            "source language id",
            "source file sha256",
            "human review objective",
        ],
        required_output_sections=pack.prompt_pack.required_output_sections,
        source_summary=asdict(plan.source_summary),
        context_pack=asdict(pack),
        assist_plan=asdict(plan),
        llm_feasibility=asdict(feasibility),
    )


def render_markdown(bundle: ConverterAdvisoryBundle) -> str:
    lines = [
        "# Garnet Converter Advisory Bundle",
        "",
        f"Source: `{bundle.source}`",
        f"Language: **{bundle.language}**",
        f"Status: **{bundle.status}**",
        "",
        (
            "Current truth: this is not active conversion. It is a "
            "provider-neutral advisory bundle for future Garnet-aware migration "
            "review."
        ),
        "",
        "## Boundaries",
        "",
        f"- Recommended first lane: `{bundle.recommended_first_lane}`",
        f"- Provider required: {str(bundle.provider_required).lower()}",
        f"- Model required: {str(bundle.model_required).lower()}",
        f"- Network required: {str(bundle.network_required).lower()}",
        f"- Conversion active: {str(bundle.conversion_active).lower()}",
        f"- Source text included: {str(bundle.source_included).lower()}",
        f"- Human review required: {str(bundle.human_review_required).lower()}",
        f"- Source privacy mode: {bundle.source_privacy_mode}",
        "",
        "## Current Truth",
        "",
    ]
    lines.extend(f"- {item}" for item in bundle.current_truth)
    lines.extend(["", "## Required Inputs", ""])
    lines.extend(f"- {item}" for item in bundle.required_inputs)
    lines.extend(["", "## Required Output Sections", ""])
    lines.extend(f"- {item}" for item in bundle.required_output_sections)
    lines.extend(["", "## Source Summary", ""])
    for key, value in bundle.source_summary.items():
        lines.append(f"- {key}: `{value}`")
    lines.extend(["", "## Required Gates", ""])
    lines.extend(f"- {gate}" for gate in bundle.assist_plan["required_gates"])
    lines.extend(
        [
            "",
            (
                "Do not treat this bundle as an LLM-backed converter, "
                "broad-language frontend, or replacement for lineage, sandboxing, "
                "`garnet check`, dogfood readiness, and human audit before "
                "unquarantine."
            ),
        ]
    )
    return "\n".join(lines) + "\n"


def render_request(bundle: ConverterAdvisoryBundle) -> str:
    source_section = (
        f"```{bundle.language_id}\n{bundle.source_text}\n```"
        if bundle.source_included and bundle.source_text is not None
        else "Source text omitted. Attach source only after local/privacy review approval."
    )
    system_prompt = bundle.context_pack["prompt_pack"]["system_prompt"]
    user_prompt_template = bundle.context_pack["prompt_pack"]["user_prompt_template"]
    return "\n".join(
        [
            "# Garnet Converter Advisory Request",
            "",
            "This request is provider-neutral advisory planning only. It is not active conversion.",
            "",
            "## System Prompt",
            "",
            "```text",
            system_prompt,
            "```",
            "",
            "## User Prompt Template",
            "",
            "```text",
            user_prompt_template,
            "```",
            "",
            "## Bundle Inputs",
            "",
            "- `garnet-converter-advisory-bundle.json`",
            "- `garnet-converter-advisory-bundle.md`",
            "- Source file SHA-256: `" + str(bundle.source_summary["sha256"]) + "`",
            "- Source text included: `" + str(bundle.source_included).lower() + "`",
            "",
            "## Source",
            "",
            source_section,
            "",
            "## Hard Boundary",
            "",
            "Return advisory migration notes or sandboxed candidate Garnet only. Do not claim autonomous conversion, broad planned-language support, provider-backed execution, or safety before lineage, `@sandbox`, `migrate_todo`, `garnet check`, dogfood evidence, and human audit are complete.",
            "",
        ]
    )


def file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_manifest(output_dir: Path) -> None:
    manifest_path = output_dir / "MANIFEST.sha256"
    verify_path = output_dir / "MANIFEST.verify.log"
    files = sorted(
        path
        for path in output_dir.rglob("*")
        if path.is_file() and path.name not in {manifest_path.name, verify_path.name}
    )
    manifest_path.write_text(
        "\n".join(f"{file_sha256(path)}  {path.relative_to(output_dir)}" for path in files) + "\n",
        encoding="utf-8",
    )
    verify_path.write_text(
        "".join(f"{path.relative_to(output_dir)}: OK\n" for path in files),
        encoding="utf-8",
    )


def write_outputs(bundle: ConverterAdvisoryBundle, output_dir: Path) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    (output_dir / "garnet-converter-advisory-bundle.json").write_text(
        json.dumps(asdict(bundle), indent=2) + "\n",
        encoding="utf-8",
    )
    (output_dir / "garnet-converter-advisory-bundle.md").write_text(
        render_markdown(bundle),
        encoding="utf-8",
    )
    (output_dir / "garnet-converter-advisory-request.md").write_text(
        render_request(bundle),
        encoding="utf-8",
    )
    write_manifest(output_dir)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--language", required=True, help="source language id or common alias")
    parser.add_argument("--source", type=Path, required=True, help="source file to inspect")
    parser.add_argument("--include-source", action="store_true", help="include source text in the advisory request")
    parser.add_argument("--format", choices=("markdown", "json"), default="markdown")
    parser.add_argument("--output-dir", help="write bundle artifacts plus MANIFEST.sha256")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        bundle = read_bundle(args.language, args.source, include_source=args.include_source)
    except (FileNotFoundError, ValueError, UnicodeDecodeError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    if args.output_dir:
        write_outputs(bundle, Path(args.output_dir).expanduser().resolve())

    if args.format == "json":
        print(json.dumps(asdict(bundle), indent=2))
    else:
        print(render_markdown(bundle), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
