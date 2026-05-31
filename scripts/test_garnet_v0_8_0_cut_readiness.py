#!/usr/bin/env python3
"""Regression tests for the v0.8.0 cut-readiness aggregator (S80)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_v0_8_0_cut_readiness.py")
SPEC = importlib.util.spec_from_file_location("garnet_v0_8_0_cut_readiness", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
cr = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_v0_8_0_cut_readiness"] = cr
SPEC.loader.exec_module(cr)


class CutReadinessTests(unittest.TestCase):
    def test_required_merged_covers_s31_to_s79(self) -> None:
        self.assertIn("s31", cr.REQUIRED_MERGED)
        self.assertIn("s79", cr.REQUIRED_MERGED)
        self.assertNotIn("s80", cr.REQUIRED_MERGED)

    def test_run_is_cut_ready(self) -> None:
        r = cr.read_readiness()
        self.assertTrue(r.ledger_complete, f"missing: {r.missing_merged}")
        self.assertTrue(r.release_gate.passed)
        self.assertTrue(r.runway_pass)
        self.assertTrue(r.cut_ready)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(cr.main(["--gate", "--format", "json"]), 0)

    def test_does_not_claim_to_cut_or_tag(self) -> None:
        md = cr.render_markdown(cr.read_readiness())
        self.assertIn("does NOT cut a tag", md)
        self.assertIn("release-truth decision reserved for Jon", md)

    def test_honesty_anchors_present(self) -> None:
        r = cr.read_readiness()
        self.assertTrue(any("research-grade prototype" in a for a in r.honesty_anchors))
        self.assertTrue(any("release-truth decision for Jon" in a for a in r.honesty_anchors))


if __name__ == "__main__":
    unittest.main()
