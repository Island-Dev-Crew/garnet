#!/usr/bin/env python3
"""Regression checks for the Garnet promo website-export harness."""
from __future__ import annotations

import subprocess
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("export_garnet_promo_video_site.mjs")


class GarnetPromoWebsiteExportHarnessTests(unittest.TestCase):
    def test_help_exposes_website_export_contract(self) -> None:
        result = subprocess.run(
            ["node", str(SCRIPT), "--help"],
            text=True,
            capture_output=True,
            check=False,
        )

        self.assertEqual(0, result.returncode, result.stderr)
        self.assertIn("export_garnet_promo_video_site.mjs", result.stdout)
        self.assertIn("--input-dir", result.stdout)
        self.assertIn("--qa-dir", result.stdout)
        self.assertIn("--output-dir", result.stdout)
        self.assertIn("website export", result.stdout)
        self.assertIn("embed-snippet.html", result.stdout)
        self.assertIn("promo-website-export-data.json", result.stdout)
        self.assertIn("MANIFEST.sha256", result.stdout)

    def test_script_packages_media_snippet_and_manifest_without_embedding_site(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")

        self.assertIn("garnet-promo.mp4", source)
        self.assertIn("garnet-promo.webm", source)
        self.assertIn("garnet-promo-poster.png", source)
        self.assertIn("promo-visual-qa-data.json", source)
        self.assertIn("embed-snippet.html", source)
        self.assertIn("promo-website-export-data.json", source)
        self.assertIn("promo-website-export-report.md", source)
        self.assertIn("MANIFEST.sha256", source)
        self.assertIn("not embedded on the public site", source)


if __name__ == "__main__":
    unittest.main()
