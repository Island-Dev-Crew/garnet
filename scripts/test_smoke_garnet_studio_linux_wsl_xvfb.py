#!/usr/bin/env python3
"""Tests for the WSL Linux Xvfb Studio runtime-start proof recorder."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("smoke_garnet_studio_linux_wsl_xvfb.py")
SPEC = importlib.util.spec_from_file_location("smoke_garnet_studio_linux_wsl_xvfb", SCRIPT)
linux_xvfb = importlib.util.module_from_spec(SPEC)
sys.modules["smoke_garnet_studio_linux_wsl_xvfb"] = linux_xvfb
SPEC.loader.exec_module(linux_xvfb)


def _write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


def _hash(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _write_manifest(bundle: Path) -> None:
    lines = []
    for path in sorted(p for p in bundle.rglob("*") if p.is_file()):
        if path.name == linux_xvfb.MANIFEST_NAME:
            continue
        lines.append(f"{_hash(path)}  {path.relative_to(bundle).as_posix()}")
    _write(bundle / linux_xvfb.MANIFEST_NAME, "\n".join(lines) + "\n")


def _valid_bundle(bundle: Path) -> Path:
    for name in linux_xvfb.REQUIRED_COMMANDS:
        _write(bundle / "commands" / f"{name}-stdout.txt", f"{name} ok\n")
        _write(bundle / "commands" / f"{name}-stderr.txt", "")
    _write(bundle / "runtime" / "xvfb-runtime-start-stdout.txt", "")
    _write(bundle / "runtime" / "xvfb-runtime-start-stderr.txt", "libEGL warning: DRI2 failed\n")
    data = {
        "schema": linux_xvfb.SCHEMA,
        "generated_at": "2026-06-03T12:00:00+00:00",
        "status": "passed",
        "platform": "linux",
        "evidence_tier": "wsl-linux-xvfb-runtime-start-smoke",
        "wsl_is_enforcement": False,
        "source_included": False,
        "provider_api_called": False,
        "linux_enforcement_proven": False,
        "desktop_gui_launch_proven": False,
        "linux_desktop_gui_launch_proven": False,
        "clean_linux_install_proven": False,
        "privileged_system_install_proven": False,
        "xvfb_runtime_start_proven": True,
        "expected_timeout_exit_code": 124,
        "timeout_seconds": 8,
        "runtime_seconds": 8.2,
        "source_package_proof": {
            "format": "rpm",
            "bundle": "proofs/linux/execution/studio-rpm-package/linux-wsl-rpm-test",
            "summary": "proofs/linux/execution/studio-rpm-package/linux-wsl-rpm-test/garnet-studio-linux-wsl-rpm.json",
        },
        "extracted_binary": {
            "path": "target/linux-wsl-rpm-stage-20260603-120000/usr/bin/garnet-studio",
            "sha256": "a" * 64,
        },
        "xvfb_tooling": {
            "xvfb-run": "/usr/bin/xvfb-run",
            "timeout": "/usr/bin/timeout",
            "DISPLAY": "",
            "WAYLAND_DISPLAY": "",
            "XDG_RUNTIME_DIR": "",
        },
        "runtime_start": {
            "exit_code": 124,
            "expected_exit_code": 124,
            "status": "passed",
            "stdout_file": "runtime/xvfb-runtime-start-stdout.txt",
            "stderr_file": "runtime/xvfb-runtime-start-stderr.txt",
        },
        "commands": [
            {
                "id": name,
                "display_args": [name],
                "exit_code": 124 if name == "xvfb-runtime-start" else 0,
                "stdout_file": f"commands/{name}-stdout.txt",
                "stderr_file": f"commands/{name}-stderr.txt",
                "status": "passed",
            }
            for name in linux_xvfb.REQUIRED_COMMANDS
        ],
        "honest_scope": [
            "WSL Xvfb runtime-start evidence only",
            "not Linux desktop GUI launch proof",
            "not Linux seccomp or OS-sandbox enforcement",
            "not clean Linux install proof",
            "not privileged system package install proof",
            "not signed, production, or v1.0 readiness",
        ],
    }
    summary = bundle / linux_xvfb.SUMMARY_NAME
    _write(summary, json.dumps(data, indent=2) + "\n")
    _write(bundle / linux_xvfb.MARKDOWN_NAME, linux_xvfb.render_markdown(data))
    _write_manifest(bundle)
    return summary


class LinuxWslXvfbRuntimeProofTests(unittest.TestCase):
    def test_valid_xvfb_bundle_verifies_timeout_runtime_start_without_overclaiming(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            summary = _valid_bundle(Path(temp))

            self.assertTrue(linux_xvfb.verify_bundle(summary))
            evidence = linux_xvfb.read_committed_evidence(root=Path(temp))

        self.assertTrue(evidence.verified)
        self.assertIn("Xvfb runtime-start", evidence.reason)
        self.assertIn("exit 124", evidence.reason)
        self.assertIn("not Linux desktop GUI launch proof", " ".join(evidence.deferred))

    def test_bundle_fails_if_runtime_command_exits_normally_or_crashes(self) -> None:
        for bad_exit in (0, 1):
            with self.subTest(exit_code=bad_exit), tempfile.TemporaryDirectory() as temp:
                bundle = Path(temp)
                summary = _valid_bundle(bundle)
                data = json.loads(summary.read_text(encoding="utf-8"))
                data["runtime_start"]["exit_code"] = bad_exit
                data["xvfb_runtime_start_proven"] = False
                for command in data["commands"]:
                    if command["id"] == "xvfb-runtime-start":
                        command["exit_code"] = bad_exit
                        command["status"] = "failed"
                _write(summary, json.dumps(data, indent=2) + "\n")
                _write_manifest(bundle)

                self.assertFalse(linux_xvfb.verify_bundle(summary))

    def test_bundle_fails_if_it_claims_desktop_clean_install_or_enforcement(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = Path(temp)
            summary = _valid_bundle(bundle)
            data = json.loads(summary.read_text(encoding="utf-8"))
            data["linux_desktop_gui_launch_proven"] = True
            data["honest_scope"].append("Linux seccomp enforced")
            _write(summary, json.dumps(data, indent=2) + "\n")
            _write_manifest(bundle)

            self.assertFalse(linux_xvfb.verify_bundle(summary))

    def test_markdown_lists_xvfb_tooling_and_honest_scope(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            data = json.loads(_valid_bundle(Path(temp)).read_text(encoding="utf-8"))

        markdown = linux_xvfb.render_markdown(data)

        self.assertIn("# Garnet Studio Linux WSL Xvfb Runtime Proof", markdown)
        self.assertIn("xvfb-run", markdown)
        self.assertIn("timeout_seconds", markdown)
        self.assertIn("not Linux desktop GUI launch proof", markdown)

    def test_manifest_paths_are_posix(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = Path(temp)
            _write(bundle / "commands" / "xvfb-tooling-stdout.txt", "ok\n")
            linux_xvfb._write_manifest(bundle)

            manifest = (bundle / linux_xvfb.MANIFEST_NAME).read_text(encoding="utf-8")

        self.assertNotIn("\\", manifest)
        self.assertIn("commands/xvfb-tooling-stdout.txt", manifest)


if __name__ == "__main__":
    unittest.main()
