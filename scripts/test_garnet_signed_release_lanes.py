#!/usr/bin/env python3
"""Regression tests for the signed release lanes reporter (S51)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_signed_release_lanes.py")
SPEC = importlib.util.spec_from_file_location("garnet_signed_release_lanes", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
lanes = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_signed_release_lanes"] = lanes
SPEC.loader.exec_module(lanes)


class SignedReleaseLanesTests(unittest.TestCase):
    def test_three_lanes(self) -> None:
        s = lanes.read_lanes()
        ids = {l.id for l in s.lanes}
        self.assertEqual(
            ids, {"program-manifest", "release-artifact", "supply-chain-attestation"}
        )

    def test_active_lane_is_program_manifest_and_wired(self) -> None:
        s = lanes.read_lanes()
        program = next(l for l in s.lanes if l.id == "program-manifest")
        self.assertEqual(program.status, "active")
        self.assertTrue(program.owned_by_garnet)
        self.assertTrue(s.active_lane_ok, "program-manifest signing must be wired in CI")

    def test_deferred_and_partial_lanes_are_honest(self) -> None:
        s = lanes.read_lanes()
        by_id = {l.id: l for l in s.lanes}
        self.assertEqual(by_id["release-artifact"].status, "deferred")
        self.assertEqual(by_id["supply-chain-attestation"].status, "partial")
        # External-tool lanes are honestly NOT claimed as Garnet-owned.
        self.assertFalse(by_id["release-artifact"].owned_by_garnet)
        self.assertFalse(by_id["supply-chain-attestation"].owned_by_garnet)

    def test_supply_chain_lane_notes_out_flag(self) -> None:
        s = lanes.read_lanes()
        sc = next(l for l in s.lanes if l.id == "supply-chain-attestation")
        self.assertTrue(sc.present, "seal --out + cosign detection must be present")

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(lanes.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_honest_scope(self) -> None:
        md = lanes.render_markdown(lanes.read_lanes())
        self.assertIn("does not sign its own supply chain", md)
        self.assertIn("active", md)


if __name__ == "__main__":
    unittest.main()
