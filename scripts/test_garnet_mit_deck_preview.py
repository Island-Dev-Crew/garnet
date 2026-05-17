#!/usr/bin/env python3
"""Regression tests for the Garnet MIT deck-preview artifact."""
from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("garnet_mit_deck_preview.py")
SPEC = importlib.util.spec_from_file_location("garnet_mit_deck_preview", SCRIPT)
assert SPEC is not None
preview_mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_mit_deck_preview"] = preview_mod
SPEC.loader.exec_module(preview_mod)


class GarnetMitDeckPreviewTests(unittest.TestCase):
    def test_preview_uses_outline_without_widening_claims(self) -> None:
        preview = preview_mod.read_preview()

        self.assertEqual("active-partial", preview.overall_status)
        self.assertEqual("87/87", preview.tracked_slices)
        self.assertGreaterEqual(len(preview.slides), 8)
        self.assertEqual(preview.target_slide_count, len(preview.slides))
        self.assertIn("browser-smokeable HTML preview", preview.current_truth)
        self.assertIn("final MIT/productization acceptance", preview.forbidden_claims)
        self.assertFalse(preview.claims_final_acceptance)

    def test_html_is_self_contained_review_artifact(self) -> None:
        preview = preview_mod.read_preview()
        html = preview_mod.render_html(preview)

        self.assertIn("<!doctype html>", html.lower())
        self.assertIn("Garnet MIT Deck Preview", html)
        self.assertIn(f"{preview.objective_completion_percent:.1f}%", html)
        self.assertIn("87/87", html)
        self.assertIn('data-slide-id="title-current-truth"', html)
        self.assertIn("Evidence", html)
        self.assertIn("Speaker note", html)
        self.assertIn("not full MIT/productization completion", html)
        self.assertIn("overflow-wrap: anywhere", html)
        self.assertIn("word-break: break-word", html)
        self.assertNotIn("production ready", html.lower())

    def test_output_dir_writes_manifested_html_json_and_outline(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            output_dir = Path(temp) / "deck-preview"
            subprocess.run(
                [sys.executable, str(SCRIPT), "--output-dir", str(output_dir)],
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=True,
            )

            html_path = output_dir / "garnet-mit-deck-preview.html"
            data_path = output_dir / "garnet-mit-deck-preview.json"
            outline_path = output_dir / "garnet-mit-deck-outline.md"
            manifest_path = output_dir / "MANIFEST.sha256"

            self.assertTrue(html_path.is_file())
            self.assertTrue(data_path.is_file())
            self.assertTrue(outline_path.is_file())
            self.assertTrue(manifest_path.is_file())
            data = json.loads(data_path.read_text(encoding="utf-8"))
            self.assertEqual("active-partial", data["overall_status"])
            self.assertEqual("87/87", data["tracked_slices"])
            self.assertFalse(data["claims_final_acceptance"])
            manifest = manifest_path.read_text(encoding="utf-8")
            self.assertIn("garnet-mit-deck-preview.html", manifest)
            self.assertIn("garnet-mit-deck-preview.json", manifest)
            self.assertIn("garnet-mit-deck-outline.md", manifest)

    def test_cli_stdout_defaults_to_html(self) -> None:
        rendered = subprocess.check_output([sys.executable, str(SCRIPT)], text=True)

        self.assertIn("Garnet MIT Deck Preview", rendered)
        self.assertIn("not full MIT/productization completion", rendered)
        self.assertIn("final MIT/productization acceptance", rendered)


if __name__ == "__main__":
    unittest.main()
