#!/usr/bin/env python3
"""Regression checks for the promo export sync script.

The script's sync contract is kept, but the public page no longer carries
the promo embed it synced: #566 replaced it with the #demonstration video.
"""
from __future__ import annotations

import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "sync_garnet_promo_video_site.mjs"
SITE = ROOT / "docs" / "index.html"
SERVICE_WORKER = ROOT / "docs" / "service-worker.js"


class GarnetPromoSiteSyncTests(unittest.TestCase):
    def test_help_exposes_public_site_sync_contract(self) -> None:
        result = subprocess.run(
            ["node", str(SCRIPT), "--help"],
            text=True,
            capture_output=True,
            check=False,
        )

        self.assertEqual(0, result.returncode, result.stderr)
        self.assertIn("sync_garnet_promo_video_site.mjs", result.stdout)
        self.assertIn("--export-dir", result.stdout)
        self.assertIn("--docs-dir", result.stdout)
        self.assertIn("public site", result.stdout)
        self.assertIn("docs/assets/garnet-promo.mp4", result.stdout)
        self.assertIn("promo-site-sync-data.json", result.stdout)
        self.assertIn("MANIFEST.sha256", result.stdout)

    def test_retired_promo_embed_is_not_claimed_on_the_page(self) -> None:
        # The May garnet-promo embed this script synced (3b21759d) was retired
        # by #566, which replaced it with the #demonstration section and
        # garnet-demonstration.mp4. The page must not claim the retired embed,
        # and the promo snapshot records that no promo embed is present.
        site = SITE.read_text(encoding="utf-8")
        worker = SERVICE_WORKER.read_text(encoding="utf-8")

        self.assertNotIn('id="promo"', site)
        self.assertNotIn("assets/garnet-promo.mp4", site)
        self.assertNotIn("assets/garnet-promo.mp4", worker)
        self.assertIn('id="demonstration"', site)

    def test_script_copies_media_assets_and_manifested_evidence(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")

        self.assertIn("promo-website-export-data.json", source)
        self.assertIn("garnet-promo.mp4", source)
        self.assertIn("garnet-promo.webm", source)
        self.assertIn("garnet-promo-poster.png", source)
        self.assertIn("docs/assets", source)
        self.assertIn("promo-site-sync-data.json", source)
        self.assertIn("promo-site-sync-report.md", source)
        self.assertIn("MANIFEST.sha256", source)


if __name__ == "__main__":
    unittest.main()
