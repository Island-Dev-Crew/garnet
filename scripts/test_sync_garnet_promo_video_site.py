#!/usr/bin/env python3
"""Regression checks for syncing the verified promo export into the docs site."""
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

    def test_public_site_embeds_video_with_current_boundaries(self) -> None:
        site = SITE.read_text(encoding="utf-8")

        self.assertIn('id="promo"', site)
        self.assertIn('class="promo-video"', site)
        self.assertIn('poster="assets/garnet-promo-poster.png"', site)
        self.assertIn('src="assets/garnet-promo.webm"', site)
        self.assertIn('src="assets/garnet-promo.mp4"', site)
        self.assertIn("Public-site embedded", site)
        self.assertIn("human/aesthetic acceptance", site)
        self.assertIn("not full MIT/productization completion", site)

    def test_service_worker_caches_promo_site_assets(self) -> None:
        worker = SERVICE_WORKER.read_text(encoding="utf-8")

        self.assertIn("assets/garnet-promo.mp4", worker)
        self.assertIn("assets/garnet-promo.webm", worker)
        self.assertIn("assets/garnet-promo-poster.png", worker)

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
