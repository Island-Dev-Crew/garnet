#!/usr/bin/env python3
"""Regression tests for Windows Studio clean-VM installer proof accounting."""
from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_windows_clean_vm_installer_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_windows_clean_vm_installer_status", SCRIPT)
assert SPEC is not None
status_mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_windows_clean_vm_installer_status"] = status_mod
SPEC.loader.exec_module(status_mod)


class GarnetWindowsCleanVmInstallerStatusTests(unittest.TestCase):
    def test_target_matrix_names_architectures_without_overclaiming(self) -> None:
        status = status_mod.read_status(Path("missing-root"))
        targets = {target.id: target for target in status.package_targets}

        self.assertFalse(status.clean_vm_verified)
        self.assertEqual("x86_64-pc-windows-msvc", targets["studio-windows-x64-nsis"].rust_target)
        self.assertEqual("aarch64-pc-windows-msvc", targets["studio-windows-arm64-nsis"].rust_target)
        self.assertEqual("i686-pc-windows-msvc", targets["studio-windows-x86-nsis"].rust_target)
        self.assertEqual("first-clean-vm-target", targets["studio-windows-x64-nsis"].status)
        self.assertEqual("planned-after-x64-proof", targets["studio-windows-arm64-nsis"].status)
        self.assertEqual("deferred-until-user-demand", targets["studio-windows-x86-nsis"].status)
        self.assertIn("clean Windows VM guest identity", status.blocked_by)
        self.assertIn("winget install path is verified", status.forbidden_claims)

    def test_current_host_record_is_not_clean_vm_verified(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            installer = root / "Garnet-Studio-setup.exe"
            install_log = root / "install.log"
            smoke = root / "studio-smoke.json"
            screenshot = root / "launch.png"
            installer.write_bytes(b"fake installer")
            install_log.write_text("exit_code=0\n", encoding="utf-8")
            smoke.write_text(
                json.dumps(
                    {
                        "status": "passed",
                        "source_included": False,
                        "provider_api_called": False,
                    }
                ),
                encoding="utf-8",
            )
            screenshot.write_bytes(b"fake image")

            record = status_mod.build_proof_record(
                mode="current-host",
                installer=installer,
                vm_name="devbox",
                guest_os="Windows 11",
                guest_arch="x64",
                install_log=install_log,
                studio_smoke_json=smoke,
                screenshot=screenshot,
            )

            self.assertFalse(record.verified)
            gates = {gate.id: gate for gate in record.gates}
            self.assertEqual("blocked", gates["fresh-guest"].status)
            self.assertEqual("pass", gates["studio-smoke"].status)

    def test_clean_vm_record_writes_manifest_and_verifies_all_gates(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            bundle = root / "bundle"
            installer = root / "Garnet-Studio-setup.exe"
            install_log = root / "install.log"
            smoke = root / "studio-smoke.json"
            screenshot = root / "launch.png"
            installer.write_bytes(b"fake installer")
            install_log.write_text("exit_code=0\n", encoding="utf-8")
            smoke.write_text(
                json.dumps(
                    {
                        "status": "passed",
                        "source_included": False,
                        "provider_api_called": False,
                    }
                ),
                encoding="utf-8",
            )
            screenshot.write_bytes(b"fake image")

            record = status_mod.build_proof_record(
                mode="clean-vm",
                installer=installer,
                vm_name="garnet-win11-clean",
                guest_os="Windows 11 23H2",
                guest_arch="x64",
                install_log=install_log,
                studio_smoke_json=smoke,
                screenshot=screenshot,
            )
            path = status_mod.write_proof(record, bundle)

            self.assertTrue(record.verified)
            self.assertTrue(path.exists())
            self.assertTrue((bundle / "MANIFEST.sha256").exists())
            status = status_mod.read_status(root)
            self.assertTrue(status.clean_vm_verified)
            self.assertEqual("clean-vm-proof-verified", status.status)
            self.assertFalse(status.blocked_by)

    def test_cli_json_and_markdown_are_honest_without_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            json_output = subprocess.check_output(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--format",
                    "json",
                    "--evidence-root",
                    temp,
                ],
                text=True,
            )
            data = json.loads(json_output)
            self.assertFalse(data["clean_vm_verified"])
            self.assertIn("proof-contract-ready-clean-vm-open", data["status"])
            self.assertIn("signed Windows MSI is available", data["forbidden_claims"])

            markdown = subprocess.check_output(
                [sys.executable, str(SCRIPT), "--evidence-root", temp],
                text=True,
            )
            self.assertIn("Clean VM verified: `false`", markdown)
            self.assertIn("Windows 32-bit remains deferred", markdown)


if __name__ == "__main__":
    unittest.main()
