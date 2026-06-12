#!/usr/bin/env python3
"""Validate Garnet's AGENTS.md documentation-runtime contract map."""
from __future__ import annotations

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]

CONTRACT_RULES = {
    "AGENTS.md": {
        "sections": [
            "## Documentation First",
            "## Memory-Kind Mapping",
            "## Required Contract Index",
            "## Change Rules",
            "## Verification Ladder",
        ],
        "phrases": [
            "runtime documentation contract",
            "procedural memory",
            "Required Contract Index",
        ],
    },
    "C_Language_Specification/AGENTS.md": {
        "sections": [
            "## Scope",
            "## Stable Contracts",
            "## Documentation Updates",
        ],
        "phrases": [
            "normative language",
            "Mini-Spec",
            "Agent Documentation Runtime Contracts",
        ],
    },
    "F_Project_Management/AGENTS.md": {
        "sections": [
            "## Scope",
            "## Stable Contracts",
            "## Update Rules",
        ],
        "phrases": [
            "episodic memory",
            "verification evidence",
            "handoff",
        ],
    },
    "garnet-parser-v0.3/AGENTS.md": {
        "phrases": ["Mini-Spec", "lexing", "parsing", "diagnostic span"],
    },
    "garnet-interp-v0.3/AGENTS.md": {
        "phrases": ["tree-walk", "stdlib", "capability metadata", "garnet run"],
    },
    "garnet-check-v0.3/AGENTS.md": {
        "phrases": ["safe-mode", "CapCaps", "borrow", "fail closed"],
    },
    "garnet-memory-v0.3/AGENTS.md": {
        "phrases": [
            "working",
            "episodic",
            "semantic",
            "procedural",
            "machine-local key",
        ],
    },
    "garnet-actor-runtime/AGENTS.md": {
        "phrases": ["bounded mailboxes", "signed hot reload", "state migration"],
    },
    "garnet-stdlib/AGENTS.md": {
        "phrases": ["capability metadata", "file, network, process, or time"],
    },
    "garnet-cli/AGENTS.md": {
        "phrases": ["binary", "template embedding", "deterministic manifests"],
    },
    "garnet-cli/templates/AGENTS.md": {
        "phrases": ["Starter projects", "agent-orchestrator", "--agent-docs"],
    },
    "garnet-convert/AGENTS.md": {
        "phrases": ["Rust, Ruby, Python, and Go", "sandboxing", "provenance"],
    },
    "garnet-cst/AGENTS.md": {
        "phrases": ["rowan", "trivia-preserving", "Concrete Syntax Tree", "round-trip"],
    },
    "garnet-prim-macros/AGENTS.md": {
        "phrases": ["registration only", "registry-join", "garnet_primitive"],
    },
    "garnet-lsp/AGENTS.md": {
        "phrases": [
            "Language Server Protocol",
            "diagnostics",
            "hover",
            "go-to-definition",
        ],
    },
    "garnet-suggest-llm/AGENTS.md": {
        "phrases": [
            "feature-gated",
            "non-deterministic",
            "LlmClient",
            "reproducibility logs",
        ],
    },
    "garnet-vm/AGENTS.md": {
        "phrases": [
            "bytecode VM scaffold",
            "deterministic bytecode serialization",
            "fallback counts",
            "Benchmark evidence",
        ],
    },
    "garnet-registry-stub/AGENTS.md": {
        "phrases": [
            "filesystem-backed",
            "BLAKE3 per file",
            "path-traversal guard",
            "deterministic",
        ],
    },
    "garnet-parser-v0.3/fuzz/AGENTS.md": {
        "phrases": ["cargo-fuzz", "parse_input", "ParseBudget"],
    },
    "apps/garnet-studio/src-tauri/AGENTS.md": {
        "phrases": [
            "Tauri v2",
            "Windows/Linux Studio",
            "provider API",
            "source inclusion",
            "advisory",
        ],
    },
    "examples/AGENTS.md": {
        "phrases": ["MVP demonstration", "production readiness", "parser/interpreter/checker"],
    },
    "xtask/AGENTS.md": {
        "phrases": ["repository automation", "CI-friendly", "garnet-cli"],
    },
}

REQUIRED = list(CONTRACT_RULES)
DEFAULT_CHILD_SECTIONS = ["## Scope", "## Stable Contracts"]
PLACEHOLDER_PATTERNS = [
    r"\bTODO\b",
    r"\bTBD\b",
    r"\blorem ipsum\b",
    r"\bplaceholder\b",
    r"\bcoming soon\b",
    r"\bfill (?:me|this) in\b",
]
CARGO_PACKAGE_RE = re.compile(r"cargo\s+(?:test|run|check|clippy)\s+-p\s+([A-Za-z0-9_.-]+)")
MARKDOWN_LINK_RE = re.compile(r"\[[^\]]+\]\(([^)]+)\)")


class ContractError(Exception):
    """Raised when the documentation contract hierarchy is invalid."""


def fail(msg: str) -> None:
    raise ContractError(msg)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def package_names(root: Path) -> set[str]:
    names = set()
    for manifest in root.rglob("Cargo.toml"):
        if skip_path(manifest):
            continue
        text = read(manifest)
        match = re.search(r'(?m)^name\s*=\s*"([^"]+)"', text)
        if match:
            names.add(match.group(1))
    return names


def skip_path(path: Path) -> bool:
    return any(part in {".git", "target", "archive"} for part in path.parts)


def check_required_content(rel: str, text: str) -> None:
    if not text.strip().startswith("# AGENTS.md"):
        fail(f"{rel} must start with an AGENTS.md H1")

    rules = CONTRACT_RULES[rel]
    sections = rules.get("sections")
    if sections is None and rel != "AGENTS.md":
        sections = DEFAULT_CHILD_SECTIONS + ["## Required Checks"]
    for section in sections or []:
        if section not in text:
            fail(f"{rel} must include {section}")

    lowered = text.casefold()
    for pattern in PLACEHOLDER_PATTERNS:
        if re.search(pattern, text, flags=re.IGNORECASE):
            fail(f"{rel} contains placeholder text matching {pattern}")

    for phrase in rules.get("phrases", []):
        if phrase.casefold() not in lowered:
            fail(f"{rel} is missing required local contract phrase: {phrase}")


def check_internal_links(root: Path, rel: str, text: str) -> None:
    base = (root / rel).parent
    for target in MARKDOWN_LINK_RE.findall(text):
        target = target.strip()
        if (
            "://" in target
            or target.startswith("#")
            or target.startswith("mailto:")
            or target.startswith("app://")
        ):
            continue
        path_part = target.split("#", 1)[0]
        if not path_part:
            continue
        target_path = (base / path_part).resolve()
        try:
            target_path.relative_to(root.resolve())
        except ValueError:
            fail(f"{rel} links outside repo: {target}")
        if not target_path.exists():
            fail(f"{rel} has broken internal link: {target}")


def check_cargo_commands(root: Path, rel: str, text: str, packages: set[str]) -> None:
    for package in CARGO_PACKAGE_RE.findall(text):
        if package not in packages:
            fail(f"{rel} references unknown cargo package: {package}")


def check_crate_contract_coverage(root: Path) -> None:
    for manifest in root.rglob("Cargo.toml"):
        if manifest.parent == root or skip_path(manifest):
            continue
        contract = manifest.parent / "AGENTS.md"
        if not contract.exists():
            rel = manifest.parent.relative_to(root)
            fail(f"crate-like directory lacks AGENTS.md contract: {rel}")


def validate(root: Path = ROOT) -> int:
    root_doc = root / "AGENTS.md"
    if not root_doc.exists():
        fail("missing root AGENTS.md")
    root_text = read(root_doc)
    packages = package_names(root)

    seen = set()
    for rel in REQUIRED:
        path = root / rel
        if not path.exists():
            fail(f"missing required contract {rel}")
        text = read(path)
        check_required_content(rel, text)
        check_internal_links(root, rel, text)
        check_cargo_commands(root, rel, text, packages)
        indexed = f"/{rel}" if rel != "AGENTS.md" else "/AGENTS.md"
        if indexed not in root_text:
            fail(f"root AGENTS.md index omits {indexed}")
        if rel in seen:
            fail(f"duplicate required contract {rel}")
        seen.add(rel)

    actual = sorted(
        path.relative_to(root).as_posix()
        for path in root.rglob("AGENTS.md")
        if not skip_path(path)
    )
    extra = [p for p in actual if p not in REQUIRED]
    if extra:
        fail("new AGENTS.md files must be added to REQUIRED and root index: " + ", ".join(extra))

    # Catch malformed absolute index links like //foo or missing leading slash.
    index_lines = [line.strip() for line in root_text.splitlines() if line.strip().startswith("- `/")]
    indexed_paths = []
    for line in index_lines:
        match = re.search(r"`/([^`]+)`", line)
        if match:
            indexed_paths.append(match.group(1))
    missing_from_required = sorted(set(indexed_paths) - set(REQUIRED))
    if missing_from_required:
        fail("root index contains paths not in REQUIRED: " + ", ".join(missing_from_required))

    check_crate_contract_coverage(root)
    print(f"agent-contracts: ok ({len(REQUIRED)} contracts)")
    return 0


def main() -> int:
    try:
        return validate(ROOT)
    except ContractError as exc:
        print(f"agent-contracts: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
