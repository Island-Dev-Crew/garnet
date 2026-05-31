#!/usr/bin/env python3
"""Regression tests for the governance + RFC status reporter (S78)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_governance_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_governance_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
gv = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_governance_status"] = gv
SPEC.loader.exec_module(gv)


class GovernanceTests(unittest.TestCase):
    def test_governance_present_and_honest(self) -> None:
        r = gv.read_status()
        self.assertTrue(r.governance_present)
        self.assertTrue(r.governance_honest, "GOVERNANCE.md must keep the honest single-maintainer status")

    def test_rfc_process_present(self) -> None:
        self.assertTrue(gv.read_status().rfc_process_present)

    def test_rfc0001_references_real_standard(self) -> None:
        self.assertTrue(gv.read_status().rfc0001_references_standard)

    def test_rfc0001_marks_donation_as_intent_not_accepted(self) -> None:
        # The honesty anchor: no foundation has adopted anything.
        self.assertTrue(gv.read_status().rfc0001_marks_intent_not_accepted)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(gv.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_no_foundation(self) -> None:
        md = gv.render_markdown(gv.read_status())
        self.assertIn("no foundation", md)
        self.assertIn("intent + a draft", md)


if __name__ == "__main__":
    unittest.main()
