#!/usr/bin/env python3
"""Regression tests for the ultrapunch domain-selection gate (S105)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_domain_selection_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_domain_selection_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
ds = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_domain_selection_status"] = ds
SPEC.loader.exec_module(ds)


class DomainSelectionTests(unittest.TestCase):
    def test_enough_domains(self) -> None:
        r = ds.read_status()
        self.assertTrue(r.doc_present)
        self.assertTrue(r.enough_domains, f"domain_count={r.domain_count}")

    def test_enforced_only_scope(self) -> None:
        self.assertTrue(ds.read_status().enforced_only)

    def test_honesty_filter_and_mcp_correction(self) -> None:
        r = ds.read_status()
        self.assertTrue(r.rejected_overclaims_present)
        self.assertTrue(r.mcp_overclaim_corrected)

    def test_stage_x_proofs_present(self) -> None:
        self.assertTrue(ds.read_status().stage_x_proofs_present)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(ds.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_enforced_only(self) -> None:
        md = ds.render_markdown(ds.read_status())
        self.assertIn("enforced ceilings", md)
        self.assertIn("research-grade", md)


if __name__ == "__main__":
    unittest.main()
