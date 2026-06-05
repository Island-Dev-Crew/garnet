#!/usr/bin/env python3
"""Regression tests for the cross-OS evidence-integrity gate (S112/S113)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_evidence_integrity_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_evidence_integrity_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
ei = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_evidence_integrity_status"] = ei
SPEC.loader.exec_module(ei)


class EvidenceIntegrityTests(unittest.TestCase):
    def test_corpus_present_and_all_verify(self) -> None:
        r = ei.read_status()
        self.assertGreaterEqual(r.bundles_total, 20, "expected a non-trivial proof corpus")
        self.assertEqual(r.bundles_failed, 0, f"failing bundles: {r.failed}")
        self.assertEqual(r.bundles_ok, r.bundles_total)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(ei.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_hash_integrity_scope(self) -> None:
        md = ei.render_markdown(ei.read_status())
        self.assertIn("hash-verifies against the committed bytes", md)
        self.assertIn("hash integrity only", md)


if __name__ == "__main__":
    unittest.main()
