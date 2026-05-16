#!/usr/bin/env python3
"""Regression tests for the Garnet MIT demo-route reporter."""
from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("garnet_mit_demo_route.py")
SPEC = importlib.util.spec_from_file_location("garnet_mit_demo_route", SCRIPT)
assert SPEC is not None
demo_mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_mit_demo_route"] = demo_mod
SPEC.loader.exec_module(demo_mod)


class GarnetMitDemoRouteTests(unittest.TestCase):
    def test_route_has_timed_beats_for_verified_surfaces(self) -> None:
        route = demo_mod.read_route()
        beats = {beat.id: beat for beat in route.beats}

        self.assertEqual("active-partial", route.overall_status)
        self.assertEqual(58.6, route.objective_completion_percent)
        self.assertGreaterEqual(len(route.beats), 6)
        self.assertEqual(sum(beat.duration_seconds for beat in route.beats), route.total_duration_seconds)
        self.assertLessEqual(route.total_duration_seconds, 480)
        self.assertIn("objective-pulse", beats)
        self.assertIn("studio-continuation", beats)
        self.assertIn("converter-advisory", beats)
        self.assertIn("agentic-dogfood", beats)
        self.assertIn("web-pwa-live", beats)
        self.assertIn("boundaries-and-ask", beats)
        self.assertIn("garnet_mit_readiness_status.py", beats["objective-pulse"].evidence)
        self.assertIn("Continuation Pulse", beats["studio-continuation"].story)
        self.assertIn("provider-neutral", beats["converter-advisory"].story)
        self.assertIn("run_agentic_dogfood_matrix.py", beats["agentic-dogfood"].command)
        self.assertIn("garnet-lang.org", beats["web-pwa-live"].story)

    def test_route_preserves_blocked_gates_and_forbidden_claims(self) -> None:
        route = demo_mod.read_route()
        blocked = {gate.id: gate for gate in route.blocked_gates}

        self.assertIn("developer-id-notarization", blocked)
        self.assertIn("windows-linux-studio", blocked)
        self.assertIn("provider-backed-llm-conversion", blocked)
        self.assertIn("native-backend-lowering", blocked)
        self.assertIn("mobile-distribution", blocked)
        self.assertIn("final-acceptance", blocked)
        self.assertIn("Apple account-holder", blocked["developer-id-notarization"].reason)
        self.assertIn("target systems", blocked["windows-linux-studio"].reason)
        self.assertIn("provider-backed LLM conversion", route.forbidden_claims)
        self.assertIn("production-ready language", route.forbidden_claims)

    def test_markdown_is_presentation_ready_without_overclaiming(self) -> None:
        rendered = subprocess.check_output([sys.executable, str(SCRIPT)], text=True)

        self.assertIn("# Garnet MIT Demo Route", rendered)
        self.assertIn("58.6%", rendered)
        self.assertIn("87/87", rendered)
        self.assertIn("Continuation Pulse", rendered)
        self.assertIn("provider-backed LLM conversion", rendered)
        self.assertIn("not full MIT/productization completion", rendered)
        self.assertNotIn("production ready", rendered.lower())

    def test_output_dir_writes_manifested_json_and_markdown(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            output_dir = Path(temp) / "demo-route"
            subprocess.run(
                [sys.executable, str(SCRIPT), "--output-dir", str(output_dir)],
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=True,
            )

            data_path = output_dir / "garnet-mit-demo-route.json"
            report_path = output_dir / "garnet-mit-demo-route.md"
            manifest_path = output_dir / "MANIFEST.sha256"

            self.assertTrue(data_path.is_file())
            self.assertTrue(report_path.is_file())
            self.assertTrue(manifest_path.is_file())
            data = json.loads(data_path.read_text(encoding="utf-8"))
            self.assertEqual("active-partial", data["overall_status"])
            self.assertEqual(58.6, data["objective_completion_percent"])
            self.assertGreaterEqual(len(data["beats"]), 6)
            self.assertIn("garnet-mit-demo-route.json", manifest_path.read_text(encoding="utf-8"))
            self.assertIn("garnet-mit-demo-route.md", manifest_path.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
