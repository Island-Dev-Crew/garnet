#!/usr/bin/env python3
"""Regression tests for the interpreter large-stack status gate (S85)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_interp_stack_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_interp_stack_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
it = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_interp_stack_status"] = it
SPEC.loader.exec_module(it)


class InterpStackTests(unittest.TestCase):
    def test_interpreter_spawns_large_stack_thread(self) -> None:
        self.assertTrue(it.read_status().spawns_large_stack_thread)

    def test_routes_through_inner(self) -> None:
        self.assertTrue(it.read_status().routes_through_inner)

    def test_deep_recursion_tests_present(self) -> None:
        self.assertTrue(it.read_status().deep_recursion_test_present)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(it.main(["--gate", "--format", "json"]), 0)

    def test_markdown_notes_not_unbounded(self) -> None:
        md = it.render_markdown(it.read_status())
        self.assertIn("NOT unbounded", md)
        self.assertIn("WIN-S73-001", md)


if __name__ == "__main__":
    unittest.main()
