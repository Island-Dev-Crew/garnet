#!/usr/bin/env python3
"""Regression tests for the self-hosted parser seed status reporter (S72)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_self_hosted_parser_seed_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_self_hosted_parser_seed_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
sp = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_self_hosted_parser_seed_status"] = sp
SPEC.loader.exec_module(sp)


class SeedStatusTests(unittest.TestCase):
    def test_seed_present_and_well_formed(self) -> None:
        r = sp.read_status(run_binary=False)
        self.assertTrue(r.seed_present, f"missing markers: {r.missing_markers}")
        self.assertEqual(r.missing_markers, [])

    def test_gate_passes_static_when_binary_absent(self) -> None:
        # Static-only mode must pass on the real well-formed seed (mirrors the
        # python-only agent-contracts CI job where no binary is built).
        self.assertEqual(sp.main(["--gate", "--no-run", "--format", "json"]), 0)

    def test_dynamic_proof_when_binary_present(self) -> None:
        # If the compiler is built, check must be clean and run must match.
        r = sp.read_status(run_binary=True)
        if not r.binary_available:
            self.skipTest("garnet binary not built")
        self.assertTrue(r.check_clean, "garnet check was not clean")
        self.assertTrue(r.run_matches, f"missing run lines: {r.missing_run_lines}")

    def test_expected_run_lines_are_specific(self) -> None:
        self.assertIn("parsed defs: 3 managed: 1", sp.EXPECTED_RUN_LINES)
        self.assertIn("def add arity 2 caps no", sp.EXPECTED_RUN_LINES)

    def test_markdown_states_seed_not_production(self) -> None:
        md = sp.render_markdown(sp.read_status(run_binary=False))
        self.assertIn("a SEED", md)
        self.assertIn("NOT the production parser", md)


if __name__ == "__main__":
    unittest.main()
