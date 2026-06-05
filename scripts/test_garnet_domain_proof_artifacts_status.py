#!/usr/bin/env python3
"""Regression tests for the domain proof-artifacts gate (S116)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_domain_proof_artifacts_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_domain_proof_artifacts_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
dp = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_domain_proof_artifacts_status"] = dp
SPEC.loader.exec_module(dp)


class DomainArtifactTests(unittest.TestCase):
    def test_floor_passed_and_all_labels_present(self) -> None:
        r = dp.read_status()
        self.assertTrue(r.proof_present)
        self.assertTrue(r.floor_passed)
        self.assertEqual(r.domain_count, 6)
        self.assertEqual(r.labels_in_doc, 6)

    def test_only_accept_domain_seals(self) -> None:
        r = dp.read_status()
        self.assertTrue(r.sealed_domain_correct)
        self.assertTrue(r.refusals_unsealed_in_doc)

    def test_mcp_is_report_only(self) -> None:
        self.assertTrue(dp.read_status().mcp_enforced_false)

    def test_honest_fences_present(self) -> None:
        self.assertTrue(dp.read_status().fences_present)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(dp.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_only_accept_seals(self) -> None:
        md = dp.render_markdown(dp.read_status())
        self.assertIn("accept seals", md)
        self.assertIn("not production / 1.0", md)


if __name__ == "__main__":
    unittest.main()
