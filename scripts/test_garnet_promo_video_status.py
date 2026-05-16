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

        self.assertEqual("planned-contract", data["status"])
        self.assertEqual(25.0, data["completion_percent"])
        self.assertFalse(data["rendered_video_present"])
        self.assertFalse(data["website_export_present"])
        self.assertIn("HyperFrames or Remotion composition", data["required_gates"])
        self.assertIn("visual QA verdict", data["required_gates"])
        self.assertIn("No verified rendered promo video is present.", data["current_truth"])
        self.assertIn("Do not claim a rendered promo video exists.", data["forbidden_claims"])

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
        self.assertIn("Status: **planned-contract**", rendered)
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
