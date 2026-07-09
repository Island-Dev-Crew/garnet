#!/usr/bin/env python3
"""Regression tests for Mac-side continuation status reporting."""
from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_mac_side_continuation_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_mac_side_continuation_status", SCRIPT)
assert SPEC is not None
status_mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_mac_side_continuation_status"] = status_mod
SPEC.loader.exec_module(status_mod)


class GarnetMacSideContinuationStatusTests(unittest.TestCase):
    def test_status_separates_actionable_mac_lanes_from_external_blockers(self) -> None:
        status = status_mod.read_status()
        lanes = {lane.id: lane for lane in status.lanes}

        self.assertEqual("published", lanes["reusable_dogfood_skill"].status)
        self.assertTrue(lanes["macos_studio_unsigned_quality"].mac_actionable)
        self.assertTrue(lanes["macos_cli_release_assets"].mac_actionable)
        self.assertEqual("release-asset-ready-local", lanes["macos_cli_release_assets"].status)
        self.assertFalse(lanes["apple_developer_id"].mac_actionable)
        self.assertEqual("blocked-external", lanes["apple_developer_id"].status)
        self.assertFalse(lanes["windows_linux_studio"].mac_actionable)
        self.assertEqual("handoff-only", lanes["windows_linux_studio"].status)

    def test_json_preserves_not_claimed_boundaries(self) -> None:
        output = subprocess.check_output(
            [sys.executable, str(SCRIPT), "--format", "json"],
            text=True,
        )
        data = json.loads(output)
        lanes = {lane["id"]: lane for lane in data["lanes"]}

        self.assertEqual(status_mod.read_status().objective_completion_percent, data["objective_completion_percent"])
        self.assertIn("must not be claimed", " ".join(data["current_truth"]))
        self.assertIn("Apple Developer Program identity verification", lanes["apple_developer_id"]["blocked_by"])
        self.assertIn("organization release smoke", lanes["macos_cli_release_assets"]["blocked_by"])
        self.assertIn("garnet-macos-cli-tarball-release-assets-20260520T135703Z", lanes["macos_cli_release_assets"]["evidence"])
        self.assertIn("do not claim Windows/Linux runtime completion", lanes["windows_linux_studio"]["next_slice"])
        self.assertIn("without calling providers", lanes["converter_advisory"]["next_slice"])
        self.assertIn("garnet_proof_benchmark_status.py", lanes["proof_benchmark_empirics"]["evidence"])

    def test_native_linux_completion_updates_target_system_boundary(self) -> None:
        status = status_mod.read_status()
        lanes = {lane.id: lane for lane in status.lanes}
        lane = lanes["windows_linux_studio"]

        self.assertNotIn("Windows/Linux runtime work are blocked", status_mod.__doc__ or "")
        self.assertIn("broader Linux distribution work is delegated", status_mod.__doc__ or "")
        self.assertFalse(lane.mac_actionable)
        self.assertIn(
            "Native ARM64 Linux CLI, seccomp, and Studio proof is committed; remaining target-system work is Windows signing/ARM64 and broader Linux distribution",
            status.current_truth,
        )
        self.assertNotIn("Linux runtime execution", lane.blocked_by)
        self.assertIn("Windows runtime execution", lane.blocked_by)

    def test_markdown_is_a_goal_prompt_friendly_pulse(self) -> None:
        rendered = subprocess.check_output([sys.executable, str(SCRIPT)], text=True)

        self.assertIn("Garnet Mac-Side Continuation Status", rendered)
        self.assertIn("Overall MIT/productization objective", rendered)
        self.assertIn("Mac-actionable", rendered)
        self.assertIn("macOS CLI release assets", rendered)
        self.assertIn("Developer ID notarization", rendered)
        self.assertIn("Windows/Linux Studio", rendered)
        self.assertIn("provider-backed LLM conversion", rendered)


if __name__ == "__main__":
    unittest.main()
