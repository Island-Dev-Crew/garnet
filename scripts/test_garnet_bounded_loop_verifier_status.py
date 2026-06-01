#!/usr/bin/env python3
"""Regression tests for the S93 bounded-loop verifier status gate."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_bounded_loop_verifier_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_bounded_loop_verifier_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
blv = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_bounded_loop_verifier_status"] = blv
SPEC.loader.exec_module(blv)


class BoundedLoopVerifierStatusTests(unittest.TestCase):
    def test_checker_exposes_static_loop_report(self) -> None:
        status = blv.read_status()
        self.assertTrue(status.checker_exports_report)

    def test_checker_rejects_uncheckable_safe_loops(self) -> None:
        status = blv.read_status()
        self.assertTrue(status.rejects_uncheckable_safe_loops)

    def test_cli_tests_cover_pass_and_reject(self) -> None:
        status = blv.read_status()
        self.assertTrue(status.cli_tests_present)

    def test_checker_accepts_counter_and_immediate_exit_bounds(self) -> None:
        status = blv.read_status()
        self.assertTrue(status.accepts_counter_and_exit_bounds)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(blv.main(["--gate", "--format", "json"]), 0)

    def test_markdown_preserves_no_wasmtime_boundary(self) -> None:
        md = blv.render_markdown(blv.read_status())
        self.assertIn("No Wasmtime fuel", md)
        self.assertIn("static verifier", md)


if __name__ == "__main__":
    unittest.main()
