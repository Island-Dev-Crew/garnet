#!/usr/bin/env python3
"""Regression tests for the formal-verification feasibility status reporter (S75)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_formal_verification_feasibility.py")
SPEC = importlib.util.spec_from_file_location("garnet_formal_verification_feasibility", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
fv = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_formal_verification_feasibility"] = fv
SPEC.loader.exec_module(fv)


class FeasibilityTests(unittest.TestCase):
    def test_study_present_and_anchored(self) -> None:
        r = fv.read_status()
        self.assertTrue(r.study_present, f"missing anchors: {r.missing_anchors}")
        self.assertEqual(r.missing_anchors, [])

    def test_foundation_grounded_in_source(self) -> None:
        r = fv.read_status()
        self.assertTrue(r.explosive_foundation_present, "explosive.rs stance not found")
        self.assertTrue(r.safe_subset_spec_present, "safe-subset spec not found")

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(fv.main(["--gate", "--format", "json"]), 0)

    def test_study_marks_no_proof_shipped(self) -> None:
        study = fv.STUDY.read_text(encoding="utf-8")
        self.assertIn("feasibility study only", study.lower())
        self.assertIn("no verifier", study.lower())

    def test_markdown_states_feasible_increment_and_no_theorem(self) -> None:
        md = fv.render_markdown(fv.read_status())
        self.assertIn("eBPF-style", md)
        self.assertIn("no theorem ships", md)


if __name__ == "__main__":
    unittest.main()
