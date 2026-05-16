#!/usr/bin/env python3
"""Regression checks for the Garnet promo render harness."""
from __future__ import annotations

import subprocess
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("render_garnet_promo_video.mjs")


class GarnetPromoRenderHarnessTests(unittest.TestCase):
    def test_help_exposes_render_contract(self) -> None:
        result = subprocess.run(
            ["node", str(SCRIPT), "--help"],
            text=True,
            capture_output=True,
            check=False,
        )

        self.assertEqual(0, result.returncode, result.stderr)
        self.assertIn("render_garnet_promo_video.mjs", result.stdout)
        self.assertIn("--fps", result.stdout)
        self.assertIn("--output-dir", result.stdout)
        self.assertIn("MP4", result.stdout)
        self.assertIn("WebM", result.stdout)

    def test_script_uses_cdp_timeline_and_ffmpeg_outputs(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")

        self.assertIn("Page.captureScreenshot", source)
        self.assertIn("fileURLToPath", source)
        self.assertIn("window.__timelines", source)
        self.assertIn("garnet-promo-main", source)
        self.assertIn("ffmpeg", source)
        self.assertIn("garnet-promo.mp4", source)
        self.assertIn("garnet-promo.webm", source)
        self.assertIn("promo-render-report.md", source)
        self.assertIn("MANIFEST.sha256", source)


if __name__ == "__main__":
    unittest.main()
