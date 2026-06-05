#!/usr/bin/env python3
"""Regression tests for the ultrapunch dossier gate (S115)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_ultrapunch_dossier_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_ultrapunch_dossier_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
ud = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_ultrapunch_dossier_status"] = ud
SPEC.loader.exec_module(ud)


class DossierTests(unittest.TestCase):
    def test_dossier_present_and_complete(self) -> None:
        r = ud.read_status()
        self.assertTrue(r.dossier_present)
        self.assertTrue(r.has_number_one_claim)
        self.assertTrue(r.has_ranked_runners_up)
        self.assertTrue(r.has_honest_concessions)

    def test_cites_red_team_and_keeps_fences(self) -> None:
        r = ud.read_status()
        self.assertTrue(r.cites_red_team)
        self.assertTrue(r.fences_present)

    def test_every_evidence_pointer_resolves(self) -> None:
        r = ud.read_status()
        self.assertEqual(
            r.evidence_pointers_missing,
            0,
            f"dangling evidence pointers: {ud._missing_pointers()}",
        )
        self.assertGreaterEqual(r.evidence_pointers_total, 6)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(ud.main(["--gate", "--format", "json"]), 0)

    def test_markdown_carries_the_number_one_claim(self) -> None:
        md = ud.render_markdown(ud.read_status())
        self.assertIn("capability-bounded acceptance", md)
        self.assertIn("no production / 1.0 claim", md)


if __name__ == "__main__":
    unittest.main()
