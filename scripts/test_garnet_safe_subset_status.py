#!/usr/bin/env python3
"""Regression tests for the safe-subset spec status reporter (S74)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_safe_subset_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_safe_subset_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
ss = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_safe_subset_status"] = ss
SPEC.loader.exec_module(ss)


class SafeSubsetTests(unittest.TestCase):
    def test_spec_present_and_anchored(self) -> None:
        r = ss.read_status()
        self.assertTrue(r.spec_present, f"missing anchors: {r.missing_spec_anchors}")
        self.assertEqual(r.missing_spec_anchors, [])

    def test_implemented_claims_grounded_in_source(self) -> None:
        r = ss.read_status()
        self.assertTrue(r.fnmode_safe_in_ast, "FnMode::Safe not found in AST")
        self.assertTrue(r.boundary_audit_in_checker, "boundary audit not found in checker")

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(ss.main(["--gate", "--format", "json"]), 0)

    def test_proposed_mode_marked_not_implemented(self) -> None:
        # The honesty anchor: the linear/effect mode must be marked NOT IMPLEMENTED.
        spec = ss.SPEC.read_text(encoding="utf-8")
        self.assertIn("NOT IMPLEMENTED", spec)
        self.assertIn("linear capabilities", spec)

    def test_markdown_states_no_type_system_built(self) -> None:
        md = ss.render_markdown(ss.read_status())
        self.assertIn("PROPOSAL", md)
        self.assertIn("builds no type system", md)


if __name__ == "__main__":
    unittest.main()
