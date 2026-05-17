#!/usr/bin/env python3
"""Regression tests for the Garnet promo video readiness contract."""
from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

SCRIPT = Path(__file__).with_name("garnet_promo_video_status.py")
MATRIX_SCRIPT = Path(__file__).with_name("run_agentic_dogfood_matrix.py")
ROOT = Path(__file__).resolve().parents[1]
TEST_DOGFOOD_DIR = tempfile.TemporaryDirectory()
os.environ["GARNET_PROMO_VIDEO_DESKTOP_DIR"] = TEST_DOGFOOD_DIR.name
SPEC = importlib.util.spec_from_file_location("garnet_promo_video_status", SCRIPT)
assert SPEC is not None
promo = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_promo_video_status"] = promo
SPEC.loader.exec_module(promo)
MATRIX_SPEC = importlib.util.spec_from_file_location("run_agentic_dogfood_matrix", MATRIX_SCRIPT)
assert MATRIX_SPEC is not None
matrix = importlib.util.module_from_spec(MATRIX_SPEC)
assert MATRIX_SPEC.loader is not None
sys.modules["run_agentic_dogfood_matrix"] = matrix
MATRIX_SPEC.loader.exec_module(matrix)

def _current_dogfood_probe_count() -> int:
    with tempfile.TemporaryDirectory() as temp:
        work = Path(temp)
        fixtures = matrix.prepare_fixtures(work)
        probes = matrix.probe_set(Path("/usr/bin/true"), work, fixtures, include_app_workbench=False)
    return len(probes)


class GarnetPromoVideoStatusTests(unittest.TestCase):
    @classmethod
    def tearDownClass(cls) -> None:
        TEST_DOGFOOD_DIR.cleanup()

    def test_json_contract_keeps_video_unrendered_until_evidence_exists(self) -> None:
        env = os.environ.copy()
        env["GARNET_PROMO_VIDEO_DESKTOP_DIR"] = TEST_DOGFOOD_DIR.name
        data = json.loads(
            subprocess.check_output(
                [sys.executable, str(SCRIPT), "--format", "json"],
                env=env,
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

    def test_rendered_artifacts_promote_status_without_claiming_website_export(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            artifact_dir = Path(temp) / "garnet-promo-video"
            artifact_dir.mkdir()
            (artifact_dir / "garnet-promo.mp4").write_bytes(b"fake-mp4")
            (artifact_dir / "garnet-promo.webm").write_bytes(b"fake-webm")

            with mock.patch.dict(os.environ, {"GARNET_PROMO_VIDEO_DESKTOP_DIR": temp}):
                contract = promo.read_status()

            self.assertEqual("rendered-artifact-ready", contract.status)
            self.assertEqual(65.0, contract.completion_percent)
            self.assertTrue(contract.rendered_video_present)
            self.assertFalse(contract.website_export_present)
            self.assertIn("rendered MP4 or WebM artifact", contract.completed_gates)
            self.assertIn("visual QA verdict", contract.open_gates)
            self.assertIn("website-ready export", contract.open_gates)

    def test_visual_qa_evidence_promotes_status_without_claiming_website_export(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            artifact_dir = Path(temp) / "garnet-promo-video"
            artifact_dir.mkdir()
            (artifact_dir / "garnet-promo.mp4").write_bytes(b"fake-mp4")
            (artifact_dir / "garnet-promo.webm").write_bytes(b"fake-webm")
            qa_dir = Path(temp) / "garnet-promo-video-visual-qa"
            qa_dir.mkdir()
            (qa_dir / "promo-visual-qa-data.json").write_text(
                json.dumps(
                    {
                        "status": "visual-qa-ready",
                        "verdict": "pass",
                        "checks": [
                            {"id": "mp4-metadata", "passed": True},
                            {"id": "webm-metadata", "passed": True},
                            {"id": "sample-frames", "passed": True},
                        ],
                    }
                ),
                encoding="utf-8",
            )

            with mock.patch.dict(os.environ, {"GARNET_PROMO_VIDEO_DESKTOP_DIR": temp}):
                contract = promo.read_status()

            self.assertEqual("visual-qa-ready", contract.status)
            self.assertEqual(80.0, contract.completion_percent)
            self.assertTrue(contract.rendered_video_present)
            self.assertTrue(contract.visual_qa_present)
            self.assertFalse(contract.website_export_present)
            self.assertIn("visual QA verdict", contract.completed_gates)
            self.assertIn("website-ready export", contract.open_gates)

    def test_website_export_evidence_promotes_status_without_claiming_public_embed(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            artifact_dir = Path(temp) / "garnet-promo-video"
            artifact_dir.mkdir()
            (artifact_dir / "garnet-promo.mp4").write_bytes(b"fake-mp4")
            (artifact_dir / "garnet-promo.webm").write_bytes(b"fake-webm")
            qa_dir = Path(temp) / "garnet-promo-video-visual-qa"
            qa_dir.mkdir()
            (qa_dir / "promo-visual-qa-data.json").write_text(
                json.dumps({"status": "visual-qa-ready", "verdict": "pass", "checks": [{"passed": True}]}),
                encoding="utf-8",
            )
            export_dir = Path(temp) / "garnet-promo-video-website-export"
            export_dir.mkdir()
            (export_dir / "promo-website-export-data.json").write_text(
                json.dumps({"status": "website-export-ready", "verdict": "pass", "checks": [{"passed": True}]}),
                encoding="utf-8",
            )

            with mock.patch.dict(os.environ, {"GARNET_PROMO_VIDEO_DESKTOP_DIR": temp}):
                contract = promo.read_status()

            self.assertEqual("website-export-ready", contract.status)
            self.assertEqual(90.0, contract.completion_percent)
            self.assertTrue(contract.website_export_present)
            self.assertIn("website-ready export", contract.completed_gates)
            self.assertNotIn("website-ready export", contract.open_gates)
            self.assertIn("repo/site copy check for overclaims", contract.open_gates)

    def test_repo_site_embed_promotes_public_site_status_without_claiming_final_acceptance(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            artifact_dir = Path(temp) / "garnet-promo-video"
            artifact_dir.mkdir()
            (artifact_dir / "garnet-promo.mp4").write_bytes(b"fake-mp4")
            (artifact_dir / "garnet-promo.webm").write_bytes(b"fake-webm")
            qa_dir = Path(temp) / "garnet-promo-video-visual-qa"
            qa_dir.mkdir()
            (qa_dir / "promo-visual-qa-data.json").write_text(
                json.dumps({"status": "visual-qa-ready", "verdict": "pass", "checks": [{"passed": True}]}),
                encoding="utf-8",
            )
            export_dir = Path(temp) / "garnet-promo-video-website-export"
            export_dir.mkdir()
            (export_dir / "promo-website-export-data.json").write_text(
                json.dumps({"status": "website-export-ready", "verdict": "pass", "checks": [{"passed": True}]}),
                encoding="utf-8",
            )
            sync_dir = Path(temp) / "garnet-promo-video-site-sync"
            sync_dir.mkdir()
            (sync_dir / "promo-site-sync-data.json").write_text(
                json.dumps({"status": "public-site-embedded", "verdict": "pass", "checks": [{"passed": True}]}),
                encoding="utf-8",
            )

            with mock.patch.dict(os.environ, {"GARNET_PROMO_VIDEO_DESKTOP_DIR": temp}):
                contract = promo.read_status()

        self.assertEqual("public-site-embedded", contract.status)
        self.assertEqual(95.0, contract.completion_percent)
        self.assertTrue(contract.website_export_present)
        self.assertTrue(contract.public_site_embed_present)
        self.assertIn("repo/site copy check for overclaims", contract.completed_gates)
        self.assertIn("human/aesthetic acceptance", contract.open_gates)
        self.assertIn("Do not claim full MIT/productization completion.", contract.forbidden_claims)

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
        expected_count = _current_dogfood_probe_count()

        self.assertTrue(contract.composition_source_present)
        self.assertEqual("docs/promo/composition.html", composition["path"])
        self.assertEqual("docs/promo/DESIGN.md", composition["design_contract_path"])
        self.assertEqual("hyperframes-html", composition["tool"])
        self.assertTrue(composition["timeline_registered"])
        self.assertTrue(composition["uses_locked_assets"])
        self.assertEqual(expected_count, composition["dogfood_probe_count"])
        self.assertEqual(expected_count, composition["computed_dogfood_probe_count"])
        self.assertEqual(expected_count, composition["declared_dogfood_probe_count"])
        self.assertTrue(composition["dogfood_probe_count_matches"])
        self.assertEqual(30, composition["duration_seconds"])
        self.assertIn("garnet-promo-main", composition["composition_id"])

    def test_composition_dogfood_probe_count_matches_current_matrix_inventory(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path(sys.executable), work, fixtures, include_app_workbench=False)

        composition = (ROOT / "docs" / "promo" / "composition.html").read_text(encoding="utf-8")
        expected_count = str(len(probes))
        self.assertIn(f'data-dogfood-probes="{expected_count}"', composition)
        self.assertIn(f'<div class="proof-key">{expected_count}</div>', composition)

    def test_packaged_resource_composition_accepts_source_checkout_probe_count(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            package_root = Path(temp)
            promo_dir = package_root / "docs" / "promo"
            promo_dir.mkdir(parents=True)
            (promo_dir / "composition.html").write_text(
                (ROOT / "docs" / "promo" / "composition.html").read_text(encoding="utf-8"),
                encoding="utf-8",
            )
            (promo_dir / "DESIGN.md").write_text(
                (ROOT / "docs" / "promo" / "DESIGN.md").read_text(encoding="utf-8"),
                encoding="utf-8",
            )

        with (
            mock.patch.object(promo, "ROOT", package_root),
            mock.patch.object(promo, "_current_dogfood_probe_count", return_value=138),
        ):
            composition = promo._composition_source()
        expected_count = max(138, composition["declared_dogfood_probe_count"])

        self.assertEqual(expected_count, composition["dogfood_probe_count"])
        self.assertEqual(138, composition["computed_dogfood_probe_count"])
        if composition["declared_dogfood_probe_count"] > 0:
            self.assertEqual(composition["dogfood_probe_count"], composition["declared_dogfood_probe_count"])
            self.assertTrue(composition["dogfood_probe_count_matches"])
        else:
            self.assertFalse(composition["dogfood_probe_count_matches"])

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

            verify = (output_dir / "MANIFEST.verify.log").read_text(encoding="utf-8")

            self.assertIn("garnet-promo-video-status.json: OK", verify)
            self.assertIn("garnet-promo-video-status.md: OK", verify)


if __name__ == "__main__":
    unittest.main()
