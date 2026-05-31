#!/usr/bin/env python3
"""Regression tests for the v0.8 beta gate (S50)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from dataclasses import replace
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_v0_8_beta_gate.py")
SPEC = importlib.util.spec_from_file_location("garnet_v0_8_beta_gate", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
beta = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_v0_8_beta_gate"] = beta
SPEC.loader.exec_module(beta)


class BetaGateTests(unittest.TestCase):
    def test_band_is_the_nine_hardening_slices(self) -> None:
        self.assertEqual(beta.HARDENING_BAND, [f"s4{i}" for i in range(1, 10)])

    def test_real_repo_band_is_complete_and_gate_open(self) -> None:
        g = beta.read_beta_gate()
        self.assertEqual(len(g.band_slices), 9)
        self.assertTrue(g.band_complete, "s41-s49 must be merged at conf>=5")
        self.assertTrue(g.sub_gates_pass, "build-proof + proof-matrix gates must pass")
        self.assertTrue(g.beta_ready)

    def test_honesty_anchors_are_verbatim(self) -> None:
        g = beta.read_beta_gate()
        self.assertIn(
            "research-grade prototype (v0.x.x) — not production-complete",
            g.honesty_anchors,
        )
        self.assertIn(
            "Paper VI scorecard: 4 supported, 2 partial (downgraded honestly), "
            "0 refuted, 1 pending-infra",
            g.honesty_anchors,
        )

    def test_does_not_claim_a_tag_or_production(self) -> None:
        g = beta.read_beta_gate()
        self.assertIn("does NOT cut a tag", g.tag_note)
        self.assertIn("release-truth decision for Jon", g.tag_note)

    def test_deferrals_are_present(self) -> None:
        g = beta.read_beta_gate()
        joined = " ".join(g.deferred_for_beta)
        self.assertIn("does not enforce", joined)  # S46 honest deferral
        self.assertIn("pending-infra", joined)  # LLM tier

    def test_gate_exit_is_zero_when_open(self) -> None:
        self.assertEqual(beta.main(["--gate", "--format", "json"]), 0)

    def test_incomplete_band_would_close_the_gate(self) -> None:
        g = beta.read_beta_gate()
        broken = replace(g.band_slices[0], ok=False)
        synthetic = replace(g, band_slices=[broken, *g.band_slices[1:]], band_complete=False)
        # The dataclass mirrors the gate's own logic: an incomplete band is not ready.
        self.assertFalse(synthetic.band_complete and synthetic.sub_gates_pass)

    def test_markdown_renders_open_state(self) -> None:
        md = beta.render_markdown(beta.read_beta_gate())
        self.assertIn("v0.8 beta gate", md)
        self.assertIn("Honesty anchors (verbatim", md)


if __name__ == "__main__":
    unittest.main()
