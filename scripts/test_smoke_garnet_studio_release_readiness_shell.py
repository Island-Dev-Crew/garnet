#!/usr/bin/env python3
"""Regression tests for the Studio release/readiness shell proof recorder."""
from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("smoke_garnet_studio_release_readiness_shell.py")
SPEC = importlib.util.spec_from_file_location("smoke_garnet_studio_release_readiness_shell", SCRIPT)
assert SPEC is not None
release_shell = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["smoke_garnet_studio_release_readiness_shell"] = release_shell
SPEC.loader.exec_module(release_shell)


def _write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


def _hash(path: Path) -> str:
    return release_shell.hashlib.sha256(path.read_bytes()).hexdigest()


def _write_manifest(bundle: Path) -> None:
    rows: list[str] = []
    for path in sorted(p for p in bundle.rglob("*") if p.is_file()):
        if path.name == release_shell.MANIFEST_NAME:
            continue
        rows.append(f"{_hash(path)}  {path.relative_to(bundle).as_posix()}")
    _write(bundle / release_shell.MANIFEST_NAME, "\n".join(rows) + "\n")


def make_bundle(root: Path, platform: str) -> Path:
    bundle = root / (
        "windows-release-readiness-shell-test"
        if platform == "windows"
        else "wsl-release-readiness-shell-test"
    )
    stdout_name = "studio-release-readiness-smoke-stdout.txt"
    if platform == "wsl":
        stdout_name = "wsl-studio-release-readiness-smoke-stdout.txt"
    _write(
        bundle / "commands" / stdout_name,
        "Garnet Studio release/readiness smoke passed\nevidence=/tmp/bundle\n",
    )
    _write(bundle / "commands" / stdout_name.replace("stdout", "stderr"), "")
    payload = {
        "status": "passed",
        "mode": "studio-release-readiness-smoke",
        "release_readiness_commands": [
            {
                "id": "windows-linux-studio-status",
                "success": True,
                "exit_code": 0,
                "stdout_has_expected_heading": True,
                "evidence_path": "payload/windows-linux-studio-status",
            },
            {
                "id": "objective-pulse",
                "success": True,
                "exit_code": 0,
                "stdout_has_expected_heading": True,
                "evidence_path": "payload/objective-pulse",
            },
            {
                "id": "converter-status",
                "success": True,
                "exit_code": 0,
                "stdout_has_expected_heading": True,
                "evidence_path": "payload/converter-status",
            },
            {
                "id": "windows-vm-installer-status",
                "success": True,
                "exit_code": 0,
                "stdout_has_expected_heading": True,
                "evidence_path": "payload/windows-vm-installer-status",
            },
        ],
        "source_included": False,
        "provider_api_called": False,
        "linux_enforcement_claimed": False,
        "linux_desktop_gui_claimed": False,
        "non_wsl_linux_desktop_claimed": False,
        "signed_msi_claimed": False,
        "winget_claimed": False,
        "windows_arm64_claimed": False,
    }
    _write(bundle / "studio-payload" / "release-readiness-shell-smoke.json", json.dumps(payload) + "\n")
    command = {
        "id": "studio-release-readiness-smoke"
        if platform == "windows"
        else "wsl-studio-release-readiness-smoke",
        "display_args": ["target/release/garnet-studio", "--studio-release-readiness-smoke"],
        "exit_code": 0,
        "stdout_file": f"commands/{stdout_name}",
        "stderr_file": f"commands/{stdout_name.replace('stdout', 'stderr')}",
        "status": "passed",
    }
    data = {
        "schema": release_shell.SCHEMA,
        "status": "passed",
        "target_platform": platform,
        "platform_tier": (
            "windows-local-tauri-release-readiness-shell-proof"
            if platform == "windows"
            else "execution/portability, not enforcement"
        ),
        "source_included": False,
        "provider_api_called": False,
        "release_readiness_shell_proven": True,
        "studio_command_path_proven": True,
        "wsl_execution_portability_claimed": platform == "wsl",
        "wsl_is_enforcement": False,
        "linux_enforcement_proven": False,
        "linux_desktop_gui_claimed": False,
        "clean_linux_install_proven": False,
        "non_wsl_linux_desktop_proven": False,
        "signed_msi_claimed": False,
        "winget_claimed": False,
        "windows_arm64_claimed": False,
        "studio_payload": {
            "copied": True,
            "payload_dir": "studio-payload",
            "payload_json": "studio-payload/release-readiness-shell-smoke.json",
            "payload_status": "passed",
            "payload_mode": "studio-release-readiness-smoke",
            "verified_command_ids": [
                "windows-linux-studio-status",
                "objective-pulse",
                "converter-status",
                "windows-vm-installer-status",
            ],
        },
        "commands": [command],
        "honest_scope": release_shell._common_scope(),
    }
    _write(bundle / release_shell.SUMMARY_NAME, json.dumps(data, indent=2) + "\n")
    _write(bundle / release_shell.MARKDOWN_NAME, release_shell.render_markdown(data))
    _write_manifest(bundle)
    return bundle / release_shell.SUMMARY_NAME


class StudioReleaseReadinessShellProofTests(unittest.TestCase):
    def test_windows_and_wsl_bundles_verify_and_read_as_pair(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            windows = make_bundle(root / "proofs" / "windows" / "studio-release-readiness-shell", "windows")
            wsl = make_bundle(
                root / "proofs" / "linux" / "execution" / "studio-release-readiness-shell",
                "wsl",
            )

            self.assertTrue(release_shell.verify_summary(windows, expected_platform="windows"))
            self.assertTrue(release_shell.verify_summary(wsl, expected_platform="wsl"))
            evidence = release_shell.read_committed_evidence(root)

        self.assertTrue(evidence.verified)
        self.assertIn("Release / Readiness", evidence.reason)
        self.assertIn("execution/portability", evidence.reason)

    def test_wsl_bundle_rejects_enforcement_or_desktop_overclaim(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            summary = make_bundle(Path(temp), "wsl")
            data = json.loads(summary.read_text(encoding="utf-8"))
            data["linux_enforcement_proven"] = True
            summary.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
            _write_manifest(summary.parent)

            self.assertFalse(release_shell.verify_summary(summary, expected_platform="wsl"))

    def test_bundle_rejects_missing_payload_or_unverified_reporter(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            summary = make_bundle(Path(temp), "windows")
            payload = summary.parent / "studio-payload" / "release-readiness-shell-smoke.json"
            data = json.loads(payload.read_text(encoding="utf-8"))
            data["release_readiness_commands"][0]["success"] = False
            payload.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
            _write_manifest(summary.parent)

            self.assertFalse(release_shell.verify_summary(summary, expected_platform="windows"))

    def test_markdown_keeps_non_claims_visible(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            summary = make_bundle(Path(temp), "windows")
            markdown = (summary.parent / release_shell.MARKDOWN_NAME).read_text(encoding="utf-8")

        self.assertIn("Release/readiness shell proven", markdown)
        self.assertIn("not Linux seccomp or OS-sandbox enforcement", markdown)
        self.assertIn("not clean/non-WSL Linux desktop GUI", markdown)


if __name__ == "__main__":
    unittest.main()
