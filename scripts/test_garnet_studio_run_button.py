#!/usr/bin/env python3
"""Verify the local Garnet Studio build/run entrypoint contract."""
from __future__ import annotations

import stat
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RUN_SCRIPT = ROOT / "script" / "build_and_run.sh"
ENVIRONMENT = ROOT / ".codex" / "environments" / "environment.toml"


class GarnetStudioRunButtonTests(unittest.TestCase):
    def test_run_script_stages_swiftpm_gui_app_bundle(self) -> None:
        self.assertTrue(RUN_SCRIPT.exists(), "missing script/build_and_run.sh")
        mode = RUN_SCRIPT.stat().st_mode
        self.assertTrue(mode & stat.S_IXUSR, "run script must be executable")
        script = RUN_SCRIPT.read_text(encoding="utf-8")

        self.assertIn('APP_NAME="GarnetStudio"', script)
        self.assertIn('BUNDLE_ID="org.islanddevcrew.garnet.studio"', script)
        self.assertIn("swift build --package-path", script)
        self.assertIn('apps/garnet-studio-macos', script)
        self.assertIn('Garnet Studio.app', script)
        self.assertIn("/usr/bin/open -n", script)
        self.assertIn("--verify|verify", script)
        self.assertIn("--telemetry|telemetry", script)

    def test_codex_environment_exposes_run_action(self) -> None:
        self.assertTrue(ENVIRONMENT.exists(), "missing .codex environment")
        environment = ENVIRONMENT.read_text(encoding="utf-8")

        self.assertIn("version = 1", environment)
        self.assertIn('name = "Garnet"', environment)
        self.assertIn('name = "Run"', environment)
        self.assertIn('icon = "run"', environment)
        self.assertIn('command = "./script/build_and_run.sh"', environment)


if __name__ == "__main__":
    unittest.main()
