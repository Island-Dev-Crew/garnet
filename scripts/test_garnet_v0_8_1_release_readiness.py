#!/usr/bin/env python3
"""Regression tests for the v0.8.1 release-readiness gate (S119)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_v0_8_1_release_readiness.py")
SPEC = importlib.util.spec_from_file_location("garnet_v0_8_1_release_readiness", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
rr = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_v0_8_1_release_readiness"] = rr
SPEC.loader.exec_module(rr)


class ReleaseReadinessTests(unittest.TestCase):
    def test_runway_merged_and_subgates_pass(self) -> None:
        r = rr.read_readiness()
        self.assertTrue(r.runway_complete, "runway s91–s118 must be merged")
        self.assertTrue(r.sub_gates_pass, "every anti-rot sub-gate must pass")
        self.assertEqual(len(r.sub_gates), 5)

    def test_binary_strict_evidence_real(self) -> None:
        r = rr.read_readiness(binary_strict=True)
        self.assertTrue(r.cross_os_complete)
        self.assertTrue(r.integrity_ok)
        self.assertTrue(r.binary_strict)

    def test_ready_in_binary_strict_mode(self) -> None:
        self.assertTrue(rr.read_readiness(binary_strict=True).release_ready)
        self.assertEqual(rr.main(["--gate", "--format", "json"]), 0)

    def test_gate_does_not_authorize_a_tag(self) -> None:
        r = rr.read_readiness()
        self.assertIn("does NOT cut or push a tag", r.tag_note)
        self.assertIn("reserved to Jon", r.tag_note)

    def test_deferred_and_anchors_surfaced(self) -> None:
        md = rr.render_markdown(rr.read_readiness())
        self.assertIn("Deferred / out of scope", md)
        self.assertIn("not production-complete", md)
        self.assertIn("Linux-only", md)

    def test_low_confidence_slices_surfaced_not_failed(self) -> None:
        # s107 is merged with null confidence — it must be surfaced, not block READY.
        r = rr.read_readiness()
        self.assertIn("s107", r.low_confidence_slices)
        self.assertTrue(r.release_ready)


if __name__ == "__main__":
    unittest.main()
