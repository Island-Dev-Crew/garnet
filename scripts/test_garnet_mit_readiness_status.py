#!/usr/bin/env python3
"""Regression tests for the broader MIT-readiness objective status reporter."""
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

SCRIPT = Path(__file__).with_name("garnet_mit_readiness_status.py")
TEST_DOGFOOD_DIR = tempfile.TemporaryDirectory()
os.environ["GARNET_PROMO_VIDEO_DESKTOP_DIR"] = TEST_DOGFOOD_DIR.name
SPEC = importlib.util.spec_from_file_location("garnet_mit_readiness_status", SCRIPT)
assert SPEC is not None
status_mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_mit_readiness_status"] = status_mod
SPEC.loader.exec_module(status_mod)


class GarnetMitReadinessStatusTests(unittest.TestCase):
    @classmethod
    def tearDownClass(cls) -> None:
        TEST_DOGFOOD_DIR.cleanup()

    def test_status_distinguishes_plan_completion_from_goal_completion(self) -> None:
        status = status_mod.read_status()
        lanes = {lane.id: lane for lane in status.lanes}

        self.assertEqual("active-partial", status.overall_status)
        self.assertLess(status.completion_percent, 100.0)
        self.assertEqual("verified", lanes["tracked_implementation_plan"].status)
        self.assertEqual(100.0, lanes["tracked_implementation_plan"].completion_percent)
        self.assertEqual("blocked", lanes["developer_id_notarization"].status)
        self.assertEqual("planned", lanes["mobile_distribution"].status)
        self.assertEqual("composition-ready", lanes["promo_video"].status)
        self.assertEqual(50.0, lanes["promo_video"].completion_percent)
        self.assertEqual("active-partial", lanes["llm_assist"].status)
        self.assertLess(lanes["llm_assist"].completion_percent, 100.0)
        self.assertEqual("planned", lanes["broad_converter_frontends"].status)
        self.assertIn("windows_linux_distribution", lanes)
        self.assertEqual(
            "active-partial", lanes["windows_linux_distribution"].status
        )
        self.assertEqual(50.0, lanes["windows_linux_distribution"].completion_percent)
        self.assertLess(
            lanes["windows_linux_distribution"].completion_percent, 100.0
        )
        self.assertTrue(lanes["windows_linux_distribution"].blocked_by)

    def test_json_exposes_evidence_and_deferred_boundaries(self) -> None:
        output = subprocess.check_output(
            [sys.executable, str(SCRIPT), "--format", "json"],
            env={**os.environ, "GARNET_PROMO_VIDEO_DESKTOP_DIR": TEST_DOGFOOD_DIR.name},
            text=True,
        )
        data = json.loads(output)
        lanes = {lane["id"]: lane for lane in data["lanes"]}

        self.assertIn("tracked implementation plan is complete", data["current_truth"])
        self.assertIn("goal remains active", data["current_truth"])
        self.assertIn("APPLE_DEV_ID_APP", lanes["developer_id_notarization"]["blocked_by"])
        self.assertIn("garnet_studio_notarization_status.py", lanes["developer_id_notarization"]["evidence"])
        self.assertIn("deterministic local context pack", lanes["llm_assist"]["evidence"])
        self.assertIn("advisory-only provider option registry", lanes["llm_assist"]["evidence"])
        self.assertIn("Studio provider-options evidence action", lanes["llm_assist"]["evidence"])
        self.assertIn("provider-neutral advisory bundle", lanes["llm_assist"]["evidence"])
        self.assertIn("garnet_promo_video_status.py", lanes["promo_video"]["evidence"])
        self.assertIn("visual identity", lanes["promo_video"]["evidence"])
        self.assertIn("composition source", lanes["promo_video"]["evidence"])
        self.assertIn("rendered artifact", lanes["promo_video"]["blocked_by"])
        self.assertNotIn("HyperFrames or Remotion composition", lanes["promo_video"]["deferred"])
        self.assertIn("JavaScript", lanes["broad_converter_frontends"]["deferred"])
        self.assertIn("Android", lanes["mobile_distribution"]["deferred"])
        self.assertIn("garnet_proof_benchmark_status.py", lanes["proof_empirics"]["evidence"])
        self.assertIn("fresh benchmark measurement run", lanes["proof_empirics"]["blocked_by"])
        self.assertIn("formal RustBelt/Iris/Coq mechanization", lanes["proof_empirics"]["deferred"])
        self.assertIn("Tauri v2 shell scaffold", lanes["windows_linux_distribution"]["evidence"])
        self.assertIn("clean Windows VM", " ".join(lanes["windows_linux_distribution"]["blocked_by"]))

    def test_rendered_promo_artifacts_update_objective_blockers(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            artifact_dir = Path(temp) / "garnet-promo-video"
            artifact_dir.mkdir()
            (artifact_dir / "garnet-promo.mp4").write_bytes(b"fake-mp4")
            (artifact_dir / "garnet-promo.webm").write_bytes(b"fake-webm")

            with mock.patch.dict(os.environ, {"GARNET_PROMO_VIDEO_DESKTOP_DIR": temp}):
                status = status_mod.read_status()

        lanes = {lane.id: lane for lane in status.lanes}
        promo_lane = lanes["promo_video"]
        self.assertEqual("rendered-artifact-ready", promo_lane.status)
        self.assertEqual(65.0, promo_lane.completion_percent)
        self.assertIn("rendered MP4/WebM evidence", promo_lane.evidence)
        self.assertNotIn("rendered artifact", promo_lane.blocked_by)
        self.assertIn("visual QA verdict", promo_lane.blocked_by)

    def test_visual_qa_promo_artifacts_update_objective_blockers(self) -> None:
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

            with mock.patch.dict(os.environ, {"GARNET_PROMO_VIDEO_DESKTOP_DIR": temp}):
                status = status_mod.read_status()

        lanes = {lane.id: lane for lane in status.lanes}
        promo_lane = lanes["promo_video"]
        self.assertEqual("visual-qa-ready", promo_lane.status)
        self.assertEqual(80.0, promo_lane.completion_percent)
        self.assertNotIn("visual QA verdict", promo_lane.blocked_by)
        self.assertIn("website-ready export", promo_lane.blocked_by)

    def test_website_export_promo_artifacts_update_objective_blockers(self) -> None:
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
                status = status_mod.read_status()

        lanes = {lane.id: lane for lane in status.lanes}
        promo_lane = lanes["promo_video"]
        self.assertEqual("website-export-ready", promo_lane.status)
        self.assertEqual(90.0, promo_lane.completion_percent)
        self.assertNotIn("website-ready export", promo_lane.blocked_by)
        self.assertIn("public-site embedding and review", promo_lane.blocked_by)

    def test_repo_site_embed_updates_objective_blockers_without_full_completion(self) -> None:
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
                status = status_mod.read_status()
        lanes = {lane.id: lane for lane in status.lanes}
        promo_lane = lanes["promo_video"]

        self.assertEqual("public-site-embedded", promo_lane.status)
        self.assertEqual(95.0, promo_lane.completion_percent)
        self.assertNotIn("public-site embedding and review", promo_lane.blocked_by)
        self.assertIn("human/aesthetic acceptance review", promo_lane.blocked_by)
        self.assertLess(status.completion_percent, 100.0)
        rendered = status_mod.render_markdown(status)
        self.assertEqual(1, rendered.count("human/aesthetic acceptance review"))

    def test_markdown_is_human_readable_and_honest(self) -> None:
        rendered = subprocess.check_output(
            [sys.executable, str(SCRIPT)],
            env={**os.environ, "GARNET_PROMO_VIDEO_DESKTOP_DIR": TEST_DOGFOOD_DIR.name},
            text=True,
        )

        self.assertIn("not full MIT/productization completion", rendered)
        self.assertIn("Developer ID notarization", rendered)
        self.assertIn("Mobile distribution", rendered)
        self.assertIn("Promo video", rendered)
        self.assertIn("LLM assist", rendered)
        self.assertIn("Broad converter frontends", rendered)

    def test_public_site_surfaces_objective_accounting_without_overclaiming(self) -> None:
        docs_dir = Path(__file__).resolve().parents[1] / "docs"
        site = (docs_dir / "index.html").read_text(
            encoding="utf-8"
        )
        status_site = (docs_dir / "status.html").read_text(encoding="utf-8")

        self.assertIn("Objective accounting", site)
        self.assertIn("MIT/productization objective", site)
        self.assertIn("54.2%", site)
        self.assertNotIn("55.8%", site)
        self.assertNotIn("58.6%", site)
        self.assertIn("54.2%", status_site)
        self.assertNotIn("55.8%", status_site)
        self.assertNotIn("58.6%", status_site)
        self.assertIn("87/87 tracked slices", site)
        self.assertIn("tracked implementation plan is complete", site)
        self.assertIn("not full MIT/productization completion", site)
        self.assertIn("notarization", site)
        self.assertIn("machine-readable preflight status reporter", site)
        self.assertIn("mobile", site)
        self.assertIn("LLM assist", site)

    # ──── S0: --check-no-regression flag ────────────────────────────────

    def test_check_no_regression_passes_when_baseline_matches(self) -> None:
        # Use the live status as its own baseline: nothing can have regressed.
        status = status_mod.read_status()
        baseline = {
            "lanes": [
                {"id": lane.id, "completion_percent": lane.completion_percent}
                for lane in status.lanes
            ]
        }
        with tempfile.TemporaryDirectory() as temp:
            baseline_path = Path(temp) / "baseline.json"
            baseline_path.write_text(json.dumps(baseline), encoding="utf-8")
            regressions, missing = status_mod.check_no_regression(status, baseline_path)
        self.assertEqual([], regressions)
        self.assertEqual([], missing)

    def test_check_no_regression_detects_regression(self) -> None:
        status = status_mod.read_status()
        target_lane = status.lanes[0]
        baseline = {
            "lanes": [
                {
                    "id": target_lane.id,
                    "completion_percent": target_lane.completion_percent + 25.0,
                }
            ]
        }
        with tempfile.TemporaryDirectory() as temp:
            baseline_path = Path(temp) / "baseline.json"
            baseline_path.write_text(json.dumps(baseline), encoding="utf-8")
            regressions, missing = status_mod.check_no_regression(status, baseline_path)
        self.assertEqual(1, len(regressions))
        self.assertIn(target_lane.id, regressions[0])
        self.assertEqual([], missing)

    def test_check_no_regression_flags_missing_lane(self) -> None:
        status = status_mod.read_status()
        baseline = {
            "lanes": [
                {
                    "id": "ghost_lane_that_was_renamed_or_deleted",
                    "completion_percent": 10.0,
                }
            ]
        }
        with tempfile.TemporaryDirectory() as temp:
            baseline_path = Path(temp) / "baseline.json"
            baseline_path.write_text(json.dumps(baseline), encoding="utf-8")
            regressions, missing = status_mod.check_no_regression(status, baseline_path)
        self.assertEqual([], regressions)
        self.assertEqual(["ghost_lane_that_was_renamed_or_deleted"], missing)

    def test_check_no_regression_missing_baseline_reports_seed_instructions(self) -> None:
        status = status_mod.read_status()
        with tempfile.TemporaryDirectory() as temp:
            baseline_path = Path(temp) / "nonexistent.json"
            regressions, missing = status_mod.check_no_regression(status, baseline_path)
        self.assertEqual([], regressions)
        self.assertEqual(1, len(missing))
        self.assertIn("baseline missing", missing[0])

    def test_check_no_regression_cli_exit_code_on_regression(self) -> None:
        status = status_mod.read_status()
        target_lane = status.lanes[0]
        baseline = {
            "lanes": [
                {
                    "id": target_lane.id,
                    "completion_percent": target_lane.completion_percent + 25.0,
                }
            ]
        }
        with tempfile.TemporaryDirectory() as temp:
            baseline_path = Path(temp) / "baseline.json"
            baseline_path.write_text(json.dumps(baseline), encoding="utf-8")
            cp = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--check-no-regression",
                    "--baseline",
                    str(baseline_path),
                ],
                env={**os.environ, "GARNET_PROMO_VIDEO_DESKTOP_DIR": TEST_DOGFOOD_DIR.name},
                capture_output=True,
                text=True,
            )
        self.assertEqual(1, cp.returncode)
        self.assertIn("Readiness regression detected", cp.stderr)


if __name__ == "__main__":
    unittest.main()
