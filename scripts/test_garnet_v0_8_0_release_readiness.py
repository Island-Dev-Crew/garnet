#!/usr/bin/env python3
"""Regression tests for the v0.8.0 release-readiness gate (S60)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_v0_8_0_release_readiness.py")
SPEC = importlib.util.spec_from_file_location("garnet_v0_8_0_release_readiness", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
rel = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_v0_8_0_release_readiness"] = rel
SPEC.loader.exec_module(rel)


class ReleaseReadinessTests(unittest.TestCase):
    def test_both_bands_merged_and_subgates_pass(self) -> None:
        r = rel.read_readiness()
        self.assertEqual(len(r.hardening_band), 10)  # s41..s50
        self.assertEqual(len(r.adoption_band), 9)  # s51..s59
        self.assertTrue(r.bands_complete, "S41–S59 + S50 must be merged at conf 5")
        self.assertTrue(r.sub_gates_pass, "all anti-rot sub-gates must pass")
        self.assertTrue(r.release_ready)

    def test_does_not_cut_a_tag(self) -> None:
        r = rel.read_readiness()
        self.assertIn("does NOT cut a tag", r.tag_note)
        self.assertIn("release-truth decision for Jon", r.tag_note)
        # v0.8.0 must NOT already be tagged by this slice.
        self.assertNotIn("v0.8.0", r.existing_tags)
        self.assertIn("v0.5.0", r.existing_tags)

    def test_honest_deferrals_and_anchors(self) -> None:
        r = rel.read_readiness()
        joined = " ".join(r.deferred_for_v0_8_0)
        self.assertIn("does not enforce", joined)  # S46
        self.assertIn("OVSX_TOKEN", joined)  # S54
        self.assertIn(
            "research-grade prototype (v0.x.x) — not production-complete", r.honesty_anchors
        )

    def test_gate_passes(self) -> None:
        self.assertEqual(rel.main(["--gate", "--format", "json"]), 0)

    def test_markdown_verdict_and_no_tag(self) -> None:
        md = rel.render_markdown(rel.read_readiness())
        self.assertIn("READY TO TAG", md)
        self.assertIn("NOT the tag itself", md)


if __name__ == "__main__":
    unittest.main()
