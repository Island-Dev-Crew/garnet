#!/usr/bin/env python3
"""Create a deterministic Garnet converter assist plan for one source file.

This reporter is an agent-facing planning surface. It reads source text, maps
obvious migration risks to the existing Garnet assist contract, and emits
gates/next steps. It does not call an LLM provider, execute source code, or
make a planned language frontend active.
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

import garnet_assist_context_pack  # noqa: E402
import garnet_converter_status  # noqa: E402


@dataclass(frozen=True)
class SourceSummary:
    path: str
    bytes: int
    lines: int
    sha256: str


@dataclass(frozen=True)
class MigrationRisk:
    title: str
    analysis_target: str
    evidence: list[str]
    recommendation: str


@dataclass(frozen=True)
class ConverterAssistPlan:
    source: str
    language: str
    language_id: str
    language_status: str
    status: str
    provider_required: bool
    model_required: bool
    network_required: bool
    enabled_by_default: bool
    conversion_active: bool
    deterministic_converter_available: bool
    source_execution_allowed: bool
    sandbox_default: bool
    current_truth: list[str]
    source_summary: SourceSummary
    context_documents: list[garnet_assist_context_pack.ContextDocument]
    analysis_targets: list[str]
    required_gates: list[str]
    risk_inventory: list[MigrationRisk]
    next_steps: list[str]


def _sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _normalise_language(value: str) -> str:
    aliases = {
        "js": "javascript",
        "ts": "typescript",
        "c++": "cpp",
        "cc": "cpp",
        "cs": "csharp",
        "c#": "csharp",
        "py": "python",
        "rb": "ruby",
        "rs": "rust",
        "golang": "go",
        "bash": "shell",
        "sh": "shell",
        "zsh": "shell",
        "sqlite": "sql",
    }
    lowered = value.strip().lower()
    return aliases.get(lowered, lowered)


def _language_maps() -> tuple[dict[str, garnet_converter_status.LanguageLane], dict[str, garnet_converter_status.LanguageLane]]:
    status = garnet_converter_status.read_status()
    active = {language.id: language for language in status.active_languages}
    planned = {language.id: language for language in status.planned_languages}
    return active, planned


def _token_hits(text: str, tokens: tuple[str, ...]) -> list[str]:
    lowered = text.lower()
    return [token for token in tokens if token.lower() in lowered]


def _risk_inventory(text: str) -> list[MigrationRisk]:
    risks: list[MigrationRisk] = []
    probes = [
        (
            "unsafe or native memory boundary",
            "safe-mode ownership candidates",
            ("unsafe", "pointer", "*mut", "*const", "malloc", "free(", "ffi", "raw"),
            "Model ownership explicitly and keep translated output quarantined until safe-mode checks pass.",
        ),
        (
            "actor or async orchestration mapping",
            "actor/orchestration mappings",
            (
                "async",
                "await",
                "promise",
                "completablefuture",
                "executor",
                "dispatchqueue",
                "actor",
                "thread",
                "std::thread",
                "spawn",
                "task",
                "goroutine",
                "channel",
            ),
            "Map concurrent control flow to Garnet orchestration constructs before emitting runnable code.",
        ),
        (
            "network or external capability boundary",
            "CapCaps/capability boundaries",
            (
                "fetch",
                "curl",
                "wget",
                "http",
                "https",
                "socket",
                "open(",
                "file",
                "readfile",
                "writefile",
                "exec",
                "spawn",
                "lwp::useragent",
                "useragent",
                "->get",
            ),
            "Declare capability boundaries and keep external effects behind explicit CapCaps.",
        ),
        (
            "type and ownership modeling",
            "safe-mode ownership candidates",
            ("class", "struct", "interface", "trait", "enum", "protocol", "record"),
            "Inventory type shapes before choosing managed-mode or safe-mode ownership boundaries.",
        ),
        (
            "memory declaration candidate",
            "memory declarations",
            (
                "cache",
                "memory",
                "remember",
                "history",
                "vector",
                "embedding",
                "map<",
                "new map",
                "create table",
                "select ",
                "insert ",
                "update ",
                "delete ",
            ),
            "Promote durable state into explicit Garnet memory declarations where appropriate.",
        ),
    ]
    for title, target, tokens, recommendation in probes:
        evidence = _token_hits(text, tokens)
        if evidence:
            risks.append(
                MigrationRisk(
                    title=title,
                    analysis_target=target,
                    evidence=evidence,
                    recommendation=recommendation,
                )
            )
    if not risks:
        risks.append(
            MigrationRisk(
                title="no obvious high-risk construct detected",
                analysis_target="migration risk inventory",
                evidence=[],
                recommendation="Still require lineage, sandboxing, garnet check, dogfood evidence, and human audit.",
            )
        )
    return risks


def _source_summary(path: Path, text: str) -> SourceSummary:
    data = path.read_bytes()
    return SourceSummary(
        path=str(path),
        bytes=len(data),
        lines=0 if text == "" else text.count("\n") + (0 if text.endswith("\n") else 1),
        sha256=_sha256_bytes(data),
    )


def read_plan(language: str, source: Path) -> ConverterAssistPlan:
    active, planned = _language_maps()
    language_id = _normalise_language(language)
    known = {**active, **planned}
    if language_id not in known:
        allowed = ", ".join(sorted(known))
        raise ValueError(f"unknown source language `{language}`; known languages: {allowed}")
    if not source.exists():
        raise FileNotFoundError(f"source file does not exist: {source}")
    if not source.is_file():
        raise ValueError(f"source path is not a file: {source}")

    converter = garnet_converter_status.read_status()
    contract = converter.intelligent_assist_contract
    pack = garnet_assist_context_pack.read_pack()
    text = source.read_text(encoding="utf-8")
    lane = known[language_id]
    is_active = language_id in active

    current_truth = [
        "not active conversion today",
        "source code is not executed during analysis",
        "deterministic converter output remains authoritative",
        "provider-backed assist remains a future gated lane",
    ]
    if is_active:
        current_truth.append(
            "this language already has a deterministic converter frontend; use it as the authoritative translation path"
        )
    else:
        current_truth.append(
            "this planned language has no deterministic frontend yet; this plan is advisory migration evidence only"
        )

    next_steps = [
        "Keep any future emitted Garnet output @sandbox by default.",
        "Preserve lineage per emitted node before treating output as conversion evidence.",
        "Emit migrate_todo evidence for unsupported constructs.",
        "Run `garnet check` before claiming translated source is valid.",
        "Preserve a dogfood readiness bundle for the attempted migration.",
        "Require human audit before unquarantine.",
    ]
    if is_active:
        next_steps.insert(0, f"Use `garnet convert {language_id} {source}` for authoritative deterministic output.")
    else:
        next_steps.insert(
            0,
            f"Treat {lane.label} as planned-only until a deterministic frontend, corpus tests, and CI gates land.",
        )

    return ConverterAssistPlan(
        source=str(ROOT),
        language=lane.label,
        language_id=language_id,
        language_status=lane.status,
        status="active-assist-plan",
        provider_required=contract.provider_required,
        model_required=contract.model_required,
        network_required=False,
        enabled_by_default=False,
        conversion_active=False,
        deterministic_converter_available=is_active,
        source_execution_allowed=converter.trust_boundaries.source_execution_allowed,
        sandbox_default=converter.trust_boundaries.sandbox_on_by_default,
        current_truth=current_truth,
        source_summary=_source_summary(source, text),
        context_documents=pack.context_documents,
        analysis_targets=contract.analysis_targets,
        required_gates=contract.required_gates,
        risk_inventory=_risk_inventory(text),
        next_steps=next_steps,
    )


def render_markdown(plan: ConverterAssistPlan) -> str:
    lines = [
        "# Garnet Converter Assist Plan",
        "",
        f"Source file: `{plan.source_summary.path}`",
        f"Language: **{plan.language}** (`{plan.language_status}`)",
        f"Status: **{plan.status}**",
        "",
        (
            "Current truth: this is not active conversion. It is deterministic, "
            "provider-optional planning evidence for Garnet-aware migration."
        ),
        "",
        "## Boundaries",
        "",
        f"- Provider required: {str(plan.provider_required).lower()}",
        f"- Model required: {str(plan.model_required).lower()}",
        f"- Network required: {str(plan.network_required).lower()}",
        f"- Source execution allowed: {str(plan.source_execution_allowed).lower()}",
        f"- Conversion active: {str(plan.conversion_active).lower()}",
        f"- Deterministic converter available: {str(plan.deterministic_converter_available).lower()}",
        "",
        "## Risk Inventory",
        "",
    ]
    for risk in plan.risk_inventory:
        evidence = ", ".join(risk.evidence) if risk.evidence else "none"
        lines.append(
            f"- **{risk.title}** -> {risk.analysis_target}. "
            f"Evidence: {evidence}. {risk.recommendation}"
        )

    lines.extend(["", "## Required Gates", ""])
    lines.extend(f"- {gate}" for gate in plan.required_gates)

    lines.extend(["", "## Next Steps", ""])
    lines.extend(f"- {step}" for step in plan.next_steps)

    lines.extend(
        [
            "",
            "Do not treat this plan as an LLM-backed converter, broad-language frontend, "
            "or replacement for lineage, sandboxing, `garnet check`, dogfood readiness, "
            "and human audit.",
        ]
    )
    return "\n".join(lines) + "\n"


def write_output_dir(plan: ConverterAssistPlan, output_dir: Path) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    json_path = output_dir / "garnet-converter-assist-plan.json"
    md_path = output_dir / "garnet-converter-assist-plan.md"
    manifest_path = output_dir / "MANIFEST.sha256"

    json_path.write_text(json.dumps(asdict(plan), indent=2) + "\n", encoding="utf-8")
    md_path.write_text(render_markdown(plan), encoding="utf-8")
    entries = []
    for path in (json_path, md_path):
        entries.append(f"{_sha256(path)}  {path.name}\n")
    manifest_path.write_text("".join(entries), encoding="utf-8")


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--language", required=True, help="source language id or common alias")
    parser.add_argument("--source", type=Path, required=True, help="source file to inspect")
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
    try:
        plan = read_plan(args.language, args.source)
    except (FileNotFoundError, ValueError, UnicodeDecodeError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    if args.output_dir:
        write_output_dir(plan, args.output_dir)

    if args.format == "json":
        print(json.dumps(asdict(plan), indent=2))
    else:
        print(render_markdown(plan), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
