#!/usr/bin/env python3
"""Regression tests for the playground readiness + honesty gate (S56)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_playground_readiness.py")
SPEC = importlib.util.spec_from_file_location("garnet_playground_readiness", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
pg = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_playground_readiness"] = pg
SPEC.loader.exec_module(pg)


class PlaygroundReadinessTests(unittest.TestCase):
    def test_gallery_is_well_formed(self) -> None:
        r = pg.read_readiness()
        self.assertTrue(r.page_present and r.manifest_present)
        self.assertGreaterEqual(r.example_count, 3)
        self.assertTrue(r.examples_well_formed, "every example needs name/title/source/output")
        self.assertTrue(r.page_references_manifest)
        self.assertTrue(r.ok)

    def test_honesty_markers_preserved(self) -> None:
        # The page must keep its "static / not a fake editor / WebAssembly" stance.
        r = pg.read_readiness()
        self.assertEqual(r.missing_markers, [], f"lost honesty markers: {r.missing_markers}")
        self.assertTrue(r.honesty_markers_present)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(pg.main(["--gate", "--format", "json"]), 0)

    def test_markdown_keeps_preset_and_browser_authority_separate(self) -> None:
        md = pg.render_markdown(pg.read_readiness())
        self.assertIn("static presets", md)
        self.assertIn("Browser status is owned", md)


if __name__ == "__main__":
    unittest.main()
