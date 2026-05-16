#!/usr/bin/env python3
"""Regression tests for Garnet Studio notarization status reporting."""
from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_studio_notarization_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_studio_notarization_status", SCRIPT)
assert SPEC is not None
status_mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_studio_notarization_status"] = status_mod
SPEC.loader.exec_module(status_mod)


class GarnetStudioNotarizationStatusTests(unittest.TestCase):
    def _bundle(self, root: Path) -> Path:
        bundle = root / "garnet-studio-notarization-preflight-test"
        bundle.mkdir()
        (bundle / "checks.tsv").write_text(
            "\n".join(
                [
                    "pass\tApp bundle exists\t/tmp/Garnet Studio.app\tNone.",
                    "blocker\tDeveloper ID Application signature missing\tSignature=adhoc\tSign with APPLE_DEV_ID_APP and hardened runtime before notarization.",
                    "blocker\tAPPLE_NOTARY_PROFILE not configured\tenvironment variable is empty\tCreate a notarytool keychain profile and export its name.",
                    "warning\tDMG has no stapled notarization ticket\txcrun stapler validate failed\tExpected before notarization; must pass after notarytool submit and stapler staple.",
                ]
            )
            + "\n",
            encoding="utf-8",
        )
        (bundle / "notarization-preflight-data.env").write_text(
            "\n".join(
                [
                    "app_path=/tmp/Garnet Studio.app",
                    "dmg_path=/tmp/GarnetStudio.dmg",
                    f"output_dir={bundle}",
                    "blockers=2",
                    "warnings=1",
                    "strict=0",
                    "copy_to_desktop=1",
                ]
            )
            + "\n",
            encoding="utf-8",
        )
        (bundle / "MANIFEST.sha256").write_text("fake  ./checks.tsv\n", encoding="utf-8")
        (bundle / "MANIFEST.verify.log").write_text("./checks.tsv: OK\n", encoding="utf-8")
        return bundle

    def test_status_parses_preflight_bundle_without_claiming_notarization(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = self._bundle(Path(temp))
            status = status_mod.read_status(bundle)

        self.assertEqual("blocked", status.overall_status)
        self.assertEqual(2, status.blocker_count)
        self.assertEqual(1, status.warning_count)
        self.assertIn("preflight only", status.current_truth)
        self.assertIn("does not submit to Apple", status.current_truth)
        self.assertIn("does not claim notarization", status.current_truth)
        self.assertEqual(
            ["Developer ID Application signature missing", "APPLE_NOTARY_PROFILE not configured"],
            [item.label for item in status.blockers],
        )
        self.assertTrue(status.manifest_present)
        self.assertTrue(status.manifest_verification_log_present)

    def test_json_output_redacts_credential_values_and_lists_next_actions(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = self._bundle(Path(temp))
            output = subprocess.check_output(
                [sys.executable, str(SCRIPT), "--bundle", str(bundle), "--format", "json"],
                text=True,
            )

        data = json.loads(output)

        self.assertEqual("blocked", data["overall_status"])
        self.assertTrue(data["credential_values_redacted"])
        self.assertNotIn("APPLE_DEV_ID_APP=", output)
        self.assertNotIn("APPLE_NOTARY_PROFILE=", output)
        self.assertIn(
            "Sign with APPLE_DEV_ID_APP and hardened runtime before notarization.",
            data["next_actions"],
        )
        self.assertIn("Create a notarytool keychain profile and export its name.", data["next_actions"])

    def test_markdown_output_is_human_readable_and_boundary_safe(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = self._bundle(Path(temp))
            output = subprocess.check_output(
                [sys.executable, str(SCRIPT), "--bundle", str(bundle)],
                text=True,
            )

        self.assertIn("# Garnet Studio Notarization Status", output)
        self.assertIn("Overall status: **blocked**", output)
        self.assertIn("Developer ID Application signature missing", output)
        self.assertIn("This is not a notarization claim.", output)
        self.assertIn("Create a notarytool keychain profile", output)

    def test_missing_bundle_fails_loudly(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            missing = Path(temp) / "missing"
            completed = subprocess.run(
                [sys.executable, str(SCRIPT), "--bundle", str(missing), "--format", "json"],
                text=True,
                capture_output=True,
                check=False,
            )

        self.assertNotEqual(0, completed.returncode)
        self.assertIn("preflight bundle not found", completed.stderr)


if __name__ == "__main__":
    unittest.main()
