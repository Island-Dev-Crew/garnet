#!/usr/bin/env python3
"""Regression tests for the Garnet MIT deck-preview browser smoke harness."""
from __future__ import annotations

import subprocess
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("smoke_garnet_mit_deck_preview_browser.mjs")


class GarnetMitDeckPreviewBrowserSmokeTests(unittest.TestCase):
    def test_harness_declares_real_browser_evidence_contract(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")

        self.assertIn("garnet_mit_deck_preview.py", source)
        self.assertIn("MANIFEST.sha256", source)
        self.assertIn("Page.captureScreenshot", source)
        self.assertIn("Emulation.setDeviceMetricsOverride", source)
        self.assertIn("externalAssets", source)
        self.assertIn("horizontalOverflow", source)
        self.assertIn("final MIT/productization acceptance", source)

    def test_harness_help_is_available_without_launching_chrome(self) -> None:
        completed = subprocess.run(
            ["node", str(SCRIPT), "--help"],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,
        )

        self.assertIn("smoke_garnet_mit_deck_preview_browser.mjs", completed.stdout)
        self.assertIn("--evidence-dir", completed.stdout)
        self.assertIn("--chrome", completed.stdout)


if __name__ == "__main__":
    unittest.main()
