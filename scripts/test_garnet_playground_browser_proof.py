#!/usr/bin/env python3
"""Regression contract for the committed W-PLAY browser proof."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_wasm_readiness.py")
SPEC = importlib.util.spec_from_file_location("garnet_wasm_readiness_proof", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
wasm = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_wasm_readiness_proof"] = wasm
SPEC.loader.exec_module(wasm)


class PlaygroundBrowserProofTests(unittest.TestCase):
    def test_proof_is_valid_and_under_thirty_seconds(self) -> None:
        proof = wasm.read_browser_proof()
        self.assertIsNotNone(proof)
        assert proof is not None
        self.assertTrue(wasm.browser_proof_valid(proof))
        self.assertLess(proof["duration_ms"], 30_000)
        self.assertEqual([], proof["network"]["external_requests"])
        self.assertEqual([], proof["network"]["untracked_requests"])

    def test_human_and_machine_diff_verdicts_are_one_decision(self) -> None:
        proof = wasm.read_browser_proof()
        self.assertIsNotNone(proof)
        assert proof is not None
        diff = proof["journeys"]["diff"]
        self.assertEqual("Authority expanded", diff["human_verdict"])
        self.assertEqual("garnet.playground.diff-caps-verdict/1", diff["machine_verdict"]["schema"])
        self.assertEqual("expanded", diff["machine_verdict"]["verdict"])
        self.assertIs(True, diff["adapter_result"]["authority_expanded"])
        self.assertIs(True, diff["machine_verdict"]["authority_expanded"])

    def test_denial_is_non_success_fatal_and_output_free(self) -> None:
        proof = wasm.read_browser_proof()
        self.assertIsNotNone(proof)
        assert proof is not None
        denial = proof["journeys"]["denial"]
        self.assertEqual("Denied", denial["ui_state"])
        self.assertEqual("runtime_error", denial["run"]["exit_class"])
        self.assertEqual("", denial["run"]["stdout"])
        self.assertTrue(denial["run"]["diagnostic"])
        self.assertIn("proc", denial["run"]["diagnostic"].lower())


if __name__ == "__main__":
    unittest.main()
