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
    objective_status: str
    objective_completion_percent: float


DOCUMENT_ROLES = {
    "CURRENT_STATE.md": "current-truth",
    "README.md": "public-entry",
    "C_Language_Specification/GARNET_v1_0_Mini_Spec.md": "spec",
    "C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md": "conformance",
    "F_Project_Management/DOGFOOD/GARNET_v0_5_DOGFOOD_READINESS_PHASE_LOG.md": "dogfood",
}


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


def write_output_dir(pack: AssistContextPack, output_dir: Path) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    json_path = output_dir / "garnet-assist-context-pack.json"
    md_path = output_dir / "garnet-assist-context-pack.md"
    manifest_path = output_dir / "MANIFEST.sha256"

    json_path.write_text(json.dumps(asdict(pack), indent=2) + "\n", encoding="utf-8")
    md_path.write_text(render_markdown(pack), encoding="utf-8")
    entries = []
    for path in (json_path, md_path):
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
