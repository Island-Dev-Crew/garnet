#!/usr/bin/env python3
"""Regression checks for the Garnet promo visual-QA harness."""
from __future__ import annotations

import subprocess
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("qa_garnet_promo_video.mjs")


class GarnetPromoVisualQaHarnessTests(unittest.TestCase):
    def test_help_exposes_visual_qa_contract(self) -> None:
        result = subprocess.run(
            ["node", str(SCRIPT), "--help"],
            text=True,
            capture_output=True,
            check=False,
        )

        self.assertEqual(0, result.returncode, result.stderr)
        self.assertIn("qa_garnet_promo_video.mjs", result.stdout)
        self.assertIn("--input-dir", result.stdout)
        self.assertIn("--output-dir", result.stdout)
        self.assertIn("visual QA", result.stdout)
        self.assertIn("MANIFEST.sha256", result.stdout)

    def test_script_uses_ffprobe_ffmpeg_and_manifested_samples(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")

        self.assertIn("ffprobe", source)
        self.assertIn("ffmpeg", source)
        self.assertIn("garnet-promo.mp4", source)
        self.assertIn("garnet-promo.webm", source)
        self.assertIn("sample-00.png", source)
        self.assertIn("promo-visual-qa-data.json", source)
        self.assertIn("promo-visual-qa-report.md", source)
        self.assertIn("MANIFEST.sha256", source)


if __name__ == "__main__":
    unittest.main()
