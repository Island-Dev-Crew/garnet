#!/usr/bin/env python3
"""Tree-sitter grammar structural check + gate (S53).

Loads `tree-sitter-garnet/grammar.js` with Node (a tiny `grammar()` shim, so the
rule thunks are never executed) and asserts the grammar's name and that every
expected CORE rule is present. This validates the grammar's *structure* without
the tree-sitter CLI.

## Honest scope (do not soften)
This does **not** compile the grammar (`tree-sitter generate`) or run corpus
tests — that requires the tree-sitter CLI, which is not present in this build
environment. It checks that `grammar.js` is loadable and declares the expected
core rules. If Node is unavailable, the check reports `node_available: false`
and the gate is a no-op (it cannot run), reported honestly.
"""
from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GRAMMAR = ROOT / "tree-sitter-garnet" / "grammar.js"

# Core rules the grammar must declare for editor highlighting to be meaningful.
EXPECTED_RULES = [
    "source_file",
    "function_definition",
    "annotation",
    "parameter_list",
    "struct_definition",
    "enum_definition",
    "actor_definition",
    "memory_declaration",
    "block",
    "let_declaration",
    "if_expression",
    "match_expression",
    "try_expression",
    "rescue_clause",
    "call_expression",
    "binary_expression",
    "pipe_expression",
    "identifier",
    "line_comment",
    "doc_comment",
]

_NODE_SCRIPT = """
global.grammar = (cfg) => cfg;
const g = require(process.argv[1]);
process.stdout.write(JSON.stringify({ name: g.name, rules: Object.keys(g.rules || {}) }));
"""


@dataclass
class TreeSitterCheck:
    schema: str
    node_available: bool
    grammar_present: bool
    grammar_name: str | None
    declared_rules: list[str]
    missing_rules: list[str]
    ok: bool


def read_check() -> TreeSitterCheck:
    node = shutil.which("node")
    if node is None:
        return TreeSitterCheck(
            schema="garnet.tree_sitter_check/v1",
            node_available=False,
            grammar_present=GRAMMAR.is_file(),
            grammar_name=None,
            declared_rules=[],
            missing_rules=[],
            ok=GRAMMAR.is_file(),  # can't validate rules without node; presence only
        )
    if not GRAMMAR.is_file():
        return TreeSitterCheck(
            schema="garnet.tree_sitter_check/v1",
            node_available=True,
            grammar_present=False,
            grammar_name=None,
            declared_rules=[],
            missing_rules=list(EXPECTED_RULES),
            ok=False,
        )
    proc = subprocess.run(
        [node, "-e", _NODE_SCRIPT, str(GRAMMAR)],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        return TreeSitterCheck(
            schema="garnet.tree_sitter_check/v1",
            node_available=True,
            grammar_present=True,
            grammar_name=None,
            declared_rules=[],
            missing_rules=list(EXPECTED_RULES),
            ok=False,
        )
    data = json.loads(proc.stdout)
    declared = data.get("rules", [])
    missing = [r for r in EXPECTED_RULES if r not in declared]
    return TreeSitterCheck(
        schema="garnet.tree_sitter_check/v1",
        node_available=True,
        grammar_present=True,
        grammar_name=data.get("name"),
        declared_rules=declared,
        missing_rules=missing,
        ok=(data.get("name") == "garnet" and not missing),
    )


def render_markdown(c: TreeSitterCheck) -> str:
    return "\n".join(
        [
            "# Garnet tree-sitter grammar check",
            "",
            f"_Schema {c.schema}._",
            "",
            f"- node available: {c.node_available}",
            f"- grammar present: {c.grammar_present}",
            f"- grammar name: {c.grammar_name}",
            f"- declared rules: {len(c.declared_rules)}",
            f"- missing core rules: {c.missing_rules or 'none'}",
            "",
            f"**Grammar structurally OK: {'yes' if c.ok else 'NO'}.**",
            "",
            "Honest scope: structural validation only — not compiled "
            "(`tree-sitter generate`) or corpus-tested here (CLI absent).",
            "",
        ]
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero if the grammar is missing a core rule (when node is available)",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    check = read_check()
    if args.format == "md":
        print(render_markdown(check))
    else:
        print(json.dumps(asdict(check), indent=2))

    if args.gate and not check.ok:
        if not check.node_available:
            # Cannot validate without node; do not fail CI on a missing toolchain.
            print(
                "tree-sitter-check: node unavailable — structural validation skipped "
                "(grammar presence only)",
                file=sys.stderr,
            )
            return 0 if check.grammar_present else 1
        print(
            f"tree-sitter-check gate FAILED: name={check.grammar_name}, "
            f"missing rules={check.missing_rules}",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
