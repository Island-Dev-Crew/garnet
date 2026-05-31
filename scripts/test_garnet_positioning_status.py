#!/usr/bin/env python3
"""Regression tests for the positioning reframe status reporter (S79)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_positioning_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_positioning_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
po = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_positioning_status"] = po
SPEC.loader.exec_module(po)


class PositioningTests(unittest.TestCase):
    def test_positioning_doc_carries_all_themes(self) -> None:
        r = po.read_status()
        self.assertTrue(r.positioning_doc_present)
        self.assertEqual(r.doc_missing_themes, [], f"doc missing: {r.doc_missing_themes}")

    def test_landing_page_carries_all_themes(self) -> None:
        r = po.read_status()
        self.assertTrue(r.landing_present)
        self.assertEqual(r.landing_missing_themes, [], f"landing missing: {r.landing_missing_themes}")

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(po.main(["--gate", "--format", "json"]), 0)

    def test_required_themes_include_diff_caps_and_precedent(self) -> None:
        self.assertIn("diff_caps_headline", po.REQUIRED_THEMES)
        self.assertIn("precedent_concession", po.REQUIRED_THEMES)

    def test_markdown_states_not_production_claim(self) -> None:
        md = po.render_markdown(po.read_status())
        self.assertIn("NOT a production/1.0 claim", md)


if __name__ == "__main__":
    unittest.main()
