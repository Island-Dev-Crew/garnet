#!/usr/bin/env python3
"""Regression tests for the broader MIT-readiness objective status reporter."""
from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_mit_readiness_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_mit_readiness_status", SCRIPT)
assert SPEC is not None
status_mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_mit_readiness_status"] = status_mod
SPEC.loader.exec_module(status_mod)


class GarnetMitReadinessStatusTests(unittest.TestCase):
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

    def test_json_exposes_evidence_and_deferred_boundaries(self) -> None:
        output = subprocess.check_output(
            [sys.executable, str(SCRIPT), "--format", "json"],
            text=True,
        )
        data = json.loads(output)
        lanes = {lane["id"]: lane for lane in data["lanes"]}

        self.assertIn("tracked implementation plan is complete", data["current_truth"])
        self.assertIn("goal remains active", data["current_truth"])
        self.assertIn("APPLE_DEV_ID_APP", lanes["developer_id_notarization"]["blocked_by"])
        self.assertIn("garnet_studio_notarization_status.py", lanes["developer_id_notarization"]["evidence"])
        self.assertIn("deterministic local context pack", lanes["llm_assist"]["evidence"])
        self.assertIn("garnet_promo_video_status.py", lanes["promo_video"]["evidence"])
        self.assertIn("visual identity", lanes["promo_video"]["evidence"])
        self.assertIn("composition source", lanes["promo_video"]["evidence"])
        self.assertIn("rendered artifact", lanes["promo_video"]["blocked_by"])
        self.assertNotIn("HyperFrames or Remotion composition", lanes["promo_video"]["deferred"])
        self.assertIn("JavaScript", lanes["broad_converter_frontends"]["deferred"])
        self.assertIn("Android", lanes["mobile_distribution"]["deferred"])

    def test_markdown_is_human_readable_and_honest(self) -> None:
        rendered = subprocess.check_output([sys.executable, str(SCRIPT)], text=True)

        self.assertIn("not full MIT/productization completion", rendered)
        self.assertIn("Developer ID notarization", rendered)
        self.assertIn("Mobile distribution", rendered)
        self.assertIn("Promo video", rendered)
        self.assertIn("LLM assist", rendered)
        self.assertIn("Broad converter frontends", rendered)

    def test_public_site_surfaces_objective_accounting_without_overclaiming(self) -> None:
        site = (Path(__file__).resolve().parents[1] / "docs" / "index.html").read_text(
            encoding="utf-8"
        )

        self.assertIn("Objective accounting", site)
        self.assertIn("tracked implementation plan is complete", site)
        self.assertIn("not full MIT/productization completion", site)
        self.assertIn("notarization", site)
        self.assertIn("machine-readable preflight status reporter", site)
        self.assertIn("mobile", site)
        self.assertIn("LLM assist", site)


if __name__ == "__main__":
    unittest.main()
