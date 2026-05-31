#!/usr/bin/env python3
"""Regression tests for the VM/interpreter parity campaign (S73)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_vm_interp_parity.py")
SPEC = importlib.util.spec_from_file_location("garnet_vm_interp_parity", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
pa = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_vm_interp_parity"] = pa
SPEC.loader.exec_module(pa)


class ParityTests(unittest.TestCase):
    def test_predicate_agrees_on_same_stdout_and_rc(self) -> None:
        self.assertTrue(pa.parity_verdict("=> 3\n", 0, "=> 3\n", 0))

    def test_predicate_rejects_stdout_mismatch(self) -> None:
        self.assertFalse(pa.parity_verdict("=> 3\n", 0, "=> 4\n", 0))

    def test_predicate_rejects_rc_mismatch(self) -> None:
        self.assertFalse(pa.parity_verdict("", 0, "", 1))

    def test_corpus_is_nonempty(self) -> None:
        r = pa.read_result(run_binary=False)
        self.assertGreater(r.corpus_size, 0)

    def test_static_gate_passes_when_binary_absent(self) -> None:
        self.assertEqual(pa.main(["--gate", "--no-run", "--format", "json"]), 0)

    def test_full_parity_when_binary_present(self) -> None:
        r = pa.read_result(run_binary=True)
        if not r.binary_available:
            self.skipTest("garnet binary not built")
        self.assertEqual(r.divergent, [], f"divergences: {r.divergent}")
        self.assertEqual(r.parity_ok, r.corpus_size)

    def test_markdown_states_corpus_based_scope(self) -> None:
        md = pa.render_markdown(pa.read_result(run_binary=False))
        self.assertIn("not a proof of total backend equivalence", md)
        self.assertIn("stdout + exit code", md)


if __name__ == "__main__":
    unittest.main()
