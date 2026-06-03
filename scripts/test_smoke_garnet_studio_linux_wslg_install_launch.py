#!/usr/bin/env python3
"""Tests for the WSLg Garnet Studio privileged install/launch proof gate."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("smoke_garnet_studio_linux_wslg_install_launch.py")
SPEC = importlib.util.spec_from_file_location("smoke_garnet_studio_linux_wslg_install_launch", SCRIPT)
assert SPEC is not None
linux_wslg_install = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["smoke_garnet_studio_linux_wslg_install_launch"] = linux_wslg_install
SPEC.loader.exec_module(linux_wslg_install)


def _write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


def _hash(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _write_manifest(bundle: Path) -> None:
    lines = []
    for path in sorted(p for p in bundle.rglob("*") if p.is_file()):
        if path.name == "MANIFEST.sha256":
            continue
        lines.append(f"{_hash(path)}  {path.relative_to(bundle).as_posix()}")
    _write(bundle / "MANIFEST.sha256", "\n".join(lines) + "\n")


def _valid_bundle(bundle: Path) -> Path:
    for name in linux_wslg_install.REQUIRED_COMMANDS:
        _write(bundle / "commands" / f"{name}-stdout.txt", f"{name} ok\n")
        _write(bundle / "commands" / f"{name}-stderr.txt", "")
    _write(bundle / "package" / "dpkg-status-before.txt", "package 'garnet-studio' is not installed\n")
    _write(bundle / "package" / "dpkg-status-after-install.txt", "Package: garnet-studio\nStatus: install ok installed\n")
    _write(bundle / "package" / "dpkg-status-after-remove.txt", "package 'garnet-studio' is not installed\n")
    _write(bundle / "installed" / "studio-smoke.json", '{"status":"ok"}\n')
    _write(bundle / "launch" / "wslg-launch-stdout.txt", "")
    _write(bundle / "launch" / "wslg-launch-stderr.txt", "")
    _write(bundle / "launch" / "process.txt", "1234 /usr/bin/garnet-studio\n")
    _write(bundle / "launch" / "xwininfo.txt", '0x200001 "Garnet Studio"\n')
    data = {
        "schema": linux_wslg_install.SCHEMA,
        "status": "passed",
        "platform": "linux",
        "evidence_tier": "wslg-system-package-install-launch",
        "wsl_is_enforcement": False,
        "source_included": False,
        "provider_api_called": False,
        "desktop_gui_launch_proven": False,
        "wslg_gui_launch_proven": True,
        "clean_linux_install_proven": False,
        "privileged_system_install_proven": True,
        "linux_enforcement_proven": False,
        "package": {
            "format": "deb",
            "path": "target/release/bundle/deb/Garnet Studio_0.1.0_amd64.deb",
            "sha256": "d" * 64,
            "size_bytes": 3022068,
            "package_name": "garnet-studio",
            "architecture": "amd64",
        },
        "install": {
            "method": "dpkg -i",
            "installed_binary": "/usr/bin/garnet-studio",
            "package_absent_before_record": True,
            "package_status_before_file": "package/dpkg-status-before.txt",
            "package_status_after_install_file": "package/dpkg-status-after-install.txt",
            "package_removed_after_record": True,
            "package_status_after_remove_file": "package/dpkg-status-after-remove.txt",
        },
        "wslg": {
            "display": ":0",
            "wayland_display": "wayland-0",
            "xdg_runtime_dir": "/run/user/0/",
            "launch_status": "passed",
            "process_observed": True,
            "window_observed": True,
            "launch_stdout_file": "launch/wslg-launch-stdout.txt",
            "launch_stderr_file": "launch/wslg-launch-stderr.txt",
            "process_file": "launch/process.txt",
            "window_file": "launch/xwininfo.txt",
        },
        "commands": [
            {
                "id": name,
                "display_args": [name],
                "exit_code": 0,
                "stdout_file": f"commands/{name}-stdout.txt",
                "stderr_file": f"commands/{name}-stderr.txt",
                "status": "passed",
            }
            for name in linux_wslg_install.REQUIRED_COMMANDS
        ],
        "honest_scope": [
            "WSLg is WSL package install and GUI-launch evidence only",
            "not Linux desktop GUI proof outside WSLg",
            "not Linux seccomp or OS-sandbox enforcement",
            "not clean Linux install proof",
            "not signed, production, or v1.0 readiness",
        ],
    }
    summary = bundle / linux_wslg_install.SUMMARY_NAME
    _write(summary, json.dumps(data, indent=2) + "\n")
    _write(bundle / linux_wslg_install.MARKDOWN_NAME, linux_wslg_install.render_markdown(data))
    _write_manifest(bundle)
    return summary


class LinuxWslgInstallLaunchProofTests(unittest.TestCase):
    def test_valid_system_install_launch_bundle_verifies_without_overclaiming(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            summary = _valid_bundle(Path(temp))

            self.assertTrue(linux_wslg_install.verify_bundle(summary))
            evidence = linux_wslg_install.read_committed_evidence(root=Path(temp))

        self.assertTrue(evidence.verified)
        self.assertEqual("verified", evidence.status)
        self.assertIn("WSLg", evidence.reason)
        self.assertIn("not clean Linux install", " ".join(evidence.deferred))

    def test_bundle_fails_without_system_install_or_wslg_launch(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = Path(temp)
            summary = _valid_bundle(bundle)
            data = json.loads(summary.read_text(encoding="utf-8"))
            data["privileged_system_install_proven"] = False
            data["wslg"]["process_observed"] = False
            data["wslg"]["window_observed"] = False
            _write(summary, json.dumps(data, indent=2) + "\n")
            _write_manifest(bundle)

            self.assertFalse(linux_wslg_install.verify_bundle(summary))

    def test_bundle_fails_if_package_was_already_installed_before_record(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = Path(temp)
            summary = _valid_bundle(bundle)
            _write(bundle / "package" / "dpkg-status-before.txt", "Package: garnet-studio\nStatus: install ok installed\n")
            data = json.loads(summary.read_text(encoding="utf-8"))
            data["install"]["package_absent_before_record"] = False
            _write(summary, json.dumps(data, indent=2) + "\n")
            _write_manifest(bundle)

            self.assertFalse(linux_wslg_install.verify_bundle(summary))

    def test_bundle_fails_if_it_claims_clean_linux_enforcement_or_production(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = Path(temp)
            summary = _valid_bundle(bundle)
            data = json.loads(summary.read_text(encoding="utf-8"))
            data["clean_linux_install_proven"] = True
            data["linux_enforcement_proven"] = True
            data["honest_scope"].append("production readiness verified")
            _write(summary, json.dumps(data, indent=2) + "\n")
            _write_manifest(bundle)

            self.assertFalse(linux_wslg_install.verify_bundle(summary))

    def test_markdown_preserves_wslg_and_boundary_truth(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            data = json.loads(_valid_bundle(Path(temp)).read_text(encoding="utf-8"))

        markdown = linux_wslg_install.render_markdown(data)

        self.assertIn("Garnet Studio Linux WSLg System Install Proof", markdown)
        self.assertIn("wslg-system-package-install-launch", markdown)
        self.assertIn("not Linux desktop GUI proof outside WSLg", markdown)
        self.assertIn("not clean Linux install proof", markdown)

    def test_manifest_uses_posix_paths_for_linux_verification(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = Path(temp)
            _write(bundle / "launch" / "process.txt", "ok\n")

            linux_wslg_install._write_manifest(bundle)

            manifest = (bundle / linux_wslg_install.MANIFEST_NAME).read_text(encoding="utf-8")

        self.assertIn("launch/process.txt", manifest)
        self.assertNotIn("\\", manifest)


if __name__ == "__main__":
    unittest.main()
