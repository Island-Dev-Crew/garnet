#!/usr/bin/env python3
"""Regression tests for the Windows/WSL Studio smoke recorder."""
from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from dataclasses import asdict
from pathlib import Path

SCRIPT = Path(__file__).with_name("smoke_garnet_studio_windows_wsl.py")
SPEC = importlib.util.spec_from_file_location("smoke_garnet_studio_windows_wsl", SCRIPT)
assert SPEC is not None
smoke_mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["smoke_garnet_studio_windows_wsl"] = smoke_mod
SPEC.loader.exec_module(smoke_mod)


def _write_fake_bundle(bundle: Path, *, target_platform: str) -> Path:
    bundle.mkdir(parents=True)
    command = smoke_mod.CommandRecord(
        id="studio-smoke" if target_platform == "windows" else "wsl-studio-status-json",
        display_args=(
            ["apps/garnet-studio/src-tauri/target/release/garnet-studio.exe", "--studio-smoke"]
            if target_platform == "windows"
            else [
                "wsl.exe",
                "-e",
                "bash",
                "-lc",
                "cd <repo> && python3 scripts/garnet_windows_linux_studio_status.py --format json",
            ]
        ),
        exit_code=0,
        stdout_file="commands/stdout.txt",
        stderr_file="commands/stderr.txt",
        status="passed",
    )
    (bundle / "commands").mkdir()
    (bundle / "commands" / "stdout.txt").write_text("ok\n", encoding="utf-8")
    (bundle / "commands" / "stderr.txt").write_text("", encoding="utf-8")
    data = {
        "schema": smoke_mod.SCHEMA,
        "status": "passed",
        "created_at": "2026-06-03T00:00:00+00:00",
        "target_platform": target_platform,
        "platform_tier": (
            "windows-local-tauri-studio-smoke"
            if target_platform == "windows"
            else "execution/portability, not enforcement"
        ),
        "source_included": False,
        "provider_api_called": False,
        "windows_studio_smoke_claimed": target_platform == "windows",
        "wsl_execution_portability_claimed": target_platform == "wsl",
        "linux_enforcement_claimed": False,
        "linux_desktop_gui_claimed": False,
        "signed_msi_claimed": False,
        "winget_claimed": False,
        "windows_arm64_claimed": False,
        "commands": [asdict(command)],
        "honest_scope": smoke_mod._common_scope(),
    }
    if target_platform == "windows":
        (bundle / "studio-smoke.json").write_text(
            json.dumps(
                {
                    "status": "passed",
                    "source_included": False,
                    "provider_api_called": False,
                }
            )
            + "\n",
            encoding="utf-8",
        )
        data["studio_smoke"] = {
            "bundle_path": "C:/fake",
            "bundle_found": True,
            "studio_smoke_json": "studio-smoke.json",
            "studio_smoke_sha256": smoke_mod._sha256(bundle / "studio-smoke.json"),
        }
    summary = smoke_mod._record_summary(bundle, data)
    return summary


class GarnetStudioWindowsWslSmokeTests(unittest.TestCase):
    def test_verifies_windows_and_wsl_fake_bundles(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            windows = _write_fake_bundle(
                root / "proofs" / "windows" / "studio" / "windows-studio-smoke-test",
                target_platform="windows",
            )
            wsl = _write_fake_bundle(
                root
                / "proofs"
                / "linux"
                / "execution"
                / "studio"
                / "wsl-studio-command-contract-test",
                target_platform="wsl",
            )

            self.assertTrue(smoke_mod.verify_summary(windows, expected_platform="windows"))
            self.assertTrue(smoke_mod.verify_summary(wsl, expected_platform="wsl"))
            evidence = smoke_mod.read_committed_evidence(root)

        self.assertTrue(evidence.verified)
        self.assertIn("Committed Windows Studio smoke bundle", evidence.reason)
        self.assertIn("not Linux seccomp", evidence.reason)
        self.assertIn("Linux desktop GUI", evidence.reason)

    def test_windows_summary_requires_studio_smoke_payload(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            summary = _write_fake_bundle(Path(temp) / "windows", target_platform="windows")
            (summary.parent / "studio-smoke.json").unlink()
            smoke_mod._write_manifest(summary.parent)

            self.assertFalse(smoke_mod.verify_summary(summary, expected_platform="windows"))

    def test_wsl_summary_rejects_enforcement_or_gui_claim(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            summary = _write_fake_bundle(Path(temp) / "wsl", target_platform="wsl")
            data = json.loads(summary.read_text(encoding="utf-8"))
            data["linux_desktop_gui_claimed"] = True
            summary.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
            smoke_mod._write_manifest(summary.parent)

            self.assertFalse(smoke_mod.verify_summary(summary, expected_platform="wsl"))

    def test_manifest_rejects_digest_drift(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            summary = _write_fake_bundle(Path(temp) / "windows", target_platform="windows")
            (summary.parent / "commands" / "stdout.txt").write_text("changed\n", encoding="utf-8")

            self.assertFalse(smoke_mod.verify_summary(summary, expected_platform="windows"))

    def test_missing_command_is_recorded_instead_of_crashing(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            record = smoke_mod._run_command(
                command_id="missing",
                command=["definitely-not-a-garnet-tool"],
                display_args=["definitely-not-a-garnet-tool"],
                bundle_dir=Path(temp),
            )

            self.assertEqual(127, record.exit_code)
            self.assertEqual("failed", record.status)
            self.assertIn(
                "missing executable",
                (Path(temp) / record.stderr_file).read_text(encoding="utf-8"),
            )


if __name__ == "__main__":
    unittest.main()
