#!/usr/bin/env python3
"""Regression tests for the tree-sitter grammar structural check (S53)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_tree_sitter_check.py")
SPEC = importlib.util.spec_from_file_location("garnet_tree_sitter_check", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
ts = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_tree_sitter_check"] = ts
SPEC.loader.exec_module(ts)


class TreeSitterCheckTests(unittest.TestCase):
    def test_grammar_file_exists(self) -> None:
        self.assertTrue(ts.GRAMMAR.is_file(), "tree-sitter-garnet/grammar.js must exist")

    def test_expected_rules_cover_headline_constructs(self) -> None:
        for needle in [
            "function_definition",
            "annotation",
            "actor_definition",
            "memory_declaration",
            "try_expression",
            "match_expression",
            "pipe_expression",
        ]:
            self.assertIn(needle, ts.EXPECTED_RULES)

    @unittest.skipUnless(__import__("shutil").which("node"), "node not installed")
    def test_grammar_loads_with_expected_name_and_rules(self) -> None:
        c = ts.read_check()
        self.assertTrue(c.node_available and c.grammar_present)
        self.assertEqual(c.grammar_name, "garnet")
        self.assertEqual(c.missing_rules, [], f"missing core rules: {c.missing_rules}")
        self.assertTrue(c.ok)

    @unittest.skipUnless(__import__("shutil").which("node"), "node not installed")
    def test_gate_passes_on_real_grammar(self) -> None:
        self.assertEqual(ts.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_honest_scope(self) -> None:
        md = ts.render_markdown(ts.read_check())
        self.assertIn("structural validation only", md)
        self.assertIn("CLI absent", md)


if __name__ == "__main__":
    unittest.main()
