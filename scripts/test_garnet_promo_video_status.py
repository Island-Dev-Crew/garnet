#!/usr/bin/env python3
"""Regression tests for the Garnet promo video readiness contract."""
from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_promo_video_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_promo_video_status", SCRIPT)
assert SPEC is not None
promo = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_promo_video_status"] = promo
SPEC.loader.exec_module(promo)


class GarnetPromoVideoStatusTests(unittest.TestCase):
    def test_json_contract_keeps_video_unrendered_until_evidence_exists(self) -> None:
        data = json.loads(
            subprocess.check_output(
                [sys.executable, str(SCRIPT), "--format", "json"],
                text=True,
            )
        )

        self.assertEqual("composition-ready", data["status"])
        self.assertEqual(50.0, data["completion_percent"])
        self.assertFalse(data["rendered_video_present"])
        self.assertFalse(data["website_export_present"])
        self.assertTrue(data["composition_source_present"])
        self.assertIn("HyperFrames or Remotion composition", data["completed_gates"])
        self.assertNotIn("HyperFrames or Remotion composition", data["open_gates"])
        self.assertTrue(data["visual_identity_locked"])
        self.assertTrue(data["source_surfaces_locked"])
        self.assertIn("visual identity lock", data["completed_gates"])
        self.assertIn("30-second storyboard and shot list", data["completed_gates"])
        self.assertIn("HyperFrames or Remotion composition", data["required_gates"])
        self.assertIn("visual QA verdict", data["required_gates"])
        self.assertIn("No verified rendered promo video is present.", data["current_truth"])
        self.assertIn("Do not claim a rendered promo video exists.", data["forbidden_claims"])

    def test_locked_source_packet_lists_real_repo_assets_and_surfaces(self) -> None:
        contract = promo.read_status()
        asset_ids = {asset["id"] for asset in contract.locked_assets}
        surface_ids = {surface["id"] for surface in contract.source_surfaces}

        self.assertIn("root-logo", asset_ids)
        self.assertIn("pwa-icon-512", asset_ids)
        self.assertIn("studio-logo", asset_ids)
        self.assertTrue(all(asset["sha256"] for asset in contract.locked_assets))
        self.assertIn("public-site", surface_ids)
        self.assertIn("studio-app", surface_ids)
        self.assertIn("mit-status", surface_ids)
        self.assertTrue(all(surface["exists"] for surface in contract.source_surfaces))

    def test_composition_source_is_repo_owned_and_hyperframes_compatible(self) -> None:
        contract = promo.read_status()
        composition = contract.composition_source

        self.assertTrue(contract.composition_source_present)
        self.assertEqual("docs/promo/composition.html", composition["path"])
        self.assertEqual("docs/promo/DESIGN.md", composition["design_contract_path"])
        self.assertEqual("hyperframes-html", composition["tool"])
        self.assertTrue(composition["timeline_registered"])
        self.assertTrue(composition["uses_locked_assets"])
        self.assertEqual(30, composition["duration_seconds"])
        self.assertIn("garnet-promo-main", composition["composition_id"])

    def test_storyboard_contract_is_specific_without_becoming_a_render_claim(self) -> None:
        contract = promo.read_status()
        beats = [beat["id"] for beat in contract.storyboard_beats]

        self.assertEqual(
            ["hook", "evidence", "workbench", "assist", "close"],
            beats,
        )
        self.assertTrue(all(beat["duration_seconds"] > 0 for beat in contract.storyboard_beats))
        self.assertEqual(30, sum(beat["duration_seconds"] for beat in contract.storyboard_beats))
        self.assertIn("Use current repo and Desktop dogfood evidence only.", contract.production_rules)
        self.assertIn("notarized macOS distribution", "\n".join(contract.forbidden_claims))

    def test_markdown_is_handoff_ready_and_honest(self) -> None:
        rendered = subprocess.check_output([sys.executable, str(SCRIPT)], text=True)

        self.assertIn("Garnet Promo Video Readiness Contract", rendered)
        self.assertIn("Status: **composition-ready**", rendered)
        self.assertIn("Visual Identity Lock", rendered)
        self.assertIn("Source Surface Lock", rendered)
        self.assertIn("Composition Source", rendered)
        self.assertIn("30 seconds", rendered)
        self.assertIn("HyperFrames or Remotion composition", rendered)
        self.assertIn("Do not claim a rendered promo video exists.", rendered)
        self.assertIn("No verified rendered promo video is present.", rendered)

    def test_output_dir_writes_manifested_json_and_markdown(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            output_dir = Path(temp) / "promo-status"
            output = subprocess.check_output(
                [sys.executable, str(SCRIPT), "--output-dir", str(output_dir)],
                text=True,
            )

            self.assertIn("Garnet Promo Video Readiness Contract", output)
            self.assertTrue((output_dir / "garnet-promo-video-status.json").is_file())
            self.assertTrue((output_dir / "garnet-promo-video-status.md").is_file())
            self.assertTrue((output_dir / "MANIFEST.sha256").is_file())

            verify = subprocess.run(
                ["shasum", "-a", "256", "-c", "MANIFEST.sha256"],
                cwd=output_dir,
                text=True,
                capture_output=True,
                check=False,
            )

            self.assertEqual(0, verify.returncode, verify.stderr)
            self.assertIn("garnet-promo-video-status.json: OK", verify.stdout)
            self.assertIn("garnet-promo-video-status.md: OK", verify.stdout)


if __name__ == "__main__":
    unittest.main()
