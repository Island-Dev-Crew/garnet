#!/usr/bin/env python3
"""Regression tests for the Garnet MIT deck-outline reporter."""
from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("garnet_mit_deck_outline.py")
SPEC = importlib.util.spec_from_file_location("garnet_mit_deck_outline", SCRIPT)
assert SPEC is not None
deck_mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_mit_deck_outline"] = deck_mod
SPEC.loader.exec_module(deck_mod)


class GarnetMitDeckOutlineTests(unittest.TestCase):
    def test_outline_has_reviewer_safe_slide_sequence(self) -> None:
        outline = deck_mod.read_outline()
        route = deck_mod.garnet_mit_demo_route.read_route()
        slides = {slide.id: slide for slide in outline.slides}

        self.assertEqual("active-partial", outline.overall_status)
        self.assertEqual(route.objective_completion_percent, outline.objective_completion_percent)
        self.assertEqual("87/87", outline.tracked_slices)
        self.assertGreaterEqual(len(outline.slides), 8)
        self.assertLessEqual(len(outline.slides), 12)
        self.assertIn("title-current-truth", slides)
        self.assertIn("language-hook", slides)
        self.assertIn("studio-workbench", slides)
        self.assertIn("converter-advisory", slides)
        self.assertIn("dogfood-evidence", slides)
        self.assertIn("web-pwa-surface", slides)
        self.assertIn("blocked-gates", slides)
        self.assertIn("ask-and-next-slices", slides)
        self.assertIn("not final MIT/productization acceptance", slides["title-current-truth"].speaker_note)
        self.assertIn("Rust rigor, Ruby velocity", slides["language-hook"].headline)
        self.assertIn("Demo Route", slides["studio-workbench"].speaker_note)
        self.assertIn("provider-neutral", slides["converter-advisory"].body[0])

    def test_outline_preserves_forbidden_claims_and_blocked_gates(self) -> None:
        outline = deck_mod.read_outline()
        blocked_ids = {gate.id for gate in outline.blocked_gates}

        self.assertIn("developer-id-notarization", blocked_ids)
        self.assertIn("windows-linux-studio", blocked_ids)
        self.assertIn("provider-backed-llm-conversion", blocked_ids)
        self.assertIn("native-backend-lowering", blocked_ids)
        self.assertIn("mobile-distribution", blocked_ids)
        self.assertIn("final-acceptance", blocked_ids)
        self.assertIn("Apple Developer ID notarization", outline.forbidden_claims)
        self.assertIn("provider-backed LLM conversion", outline.forbidden_claims)
        self.assertIn("final MIT/productization acceptance", outline.forbidden_claims)

    def test_markdown_is_slide_ready_without_overclaiming(self) -> None:
        outline = deck_mod.read_outline()
        rendered = subprocess.check_output([sys.executable, str(SCRIPT)], text=True)

        self.assertIn("# Garnet MIT Deck Outline", rendered)
        self.assertIn(f"{outline.objective_completion_percent:.1f}%", rendered)
        self.assertIn("87/87", rendered)
        self.assertIn("## Slide 1", rendered)
        self.assertIn("Speaker note", rendered)
        self.assertIn("provider-backed LLM conversion", rendered)
        self.assertIn("not full MIT/productization completion", rendered)
        self.assertNotIn("production ready", rendered.lower())

    def test_output_dir_writes_manifested_json_and_markdown(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            output_dir = Path(temp) / "deck-outline"
            subprocess.run(
                [sys.executable, str(SCRIPT), "--output-dir", str(output_dir)],
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=True,
            )

            data_path = output_dir / "garnet-mit-deck-outline.json"
            report_path = output_dir / "garnet-mit-deck-outline.md"
            manifest_path = output_dir / "MANIFEST.sha256"

            self.assertTrue(data_path.is_file())
            self.assertTrue(report_path.is_file())
            self.assertTrue(manifest_path.is_file())
            data = json.loads(data_path.read_text(encoding="utf-8"))
            self.assertEqual("active-partial", data["overall_status"])
            self.assertEqual("87/87", data["tracked_slices"])
            self.assertGreaterEqual(len(data["slides"]), 8)
            self.assertIn("garnet-mit-deck-outline.json", manifest_path.read_text(encoding="utf-8"))
            self.assertIn("garnet-mit-deck-outline.md", manifest_path.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
