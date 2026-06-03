#!/usr/bin/env python3
"""Tests for the WSL Linux RPM Studio package proof recorder."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("smoke_garnet_studio_linux_wsl_rpm.py")
SPEC = importlib.util.spec_from_file_location("smoke_garnet_studio_linux_wsl_rpm", SCRIPT)
linux_rpm = importlib.util.module_from_spec(SPEC)
sys.modules["smoke_garnet_studio_linux_wsl_rpm"] = linux_rpm
SPEC.loader.exec_module(linux_rpm)


def _write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


def _hash(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _write_manifest(bundle: Path) -> None:
    lines = []
    for path in sorted(p for p in bundle.rglob("*") if p.is_file()):
        if path.name == linux_rpm.MANIFEST_NAME:
            continue
        lines.append(f"{_hash(path)}  {path.relative_to(bundle).as_posix()}")
    _write(bundle / linux_rpm.MANIFEST_NAME, "\n".join(lines) + "\n")


def _valid_bundle(bundle: Path) -> Path:
    for name in linux_rpm.REQUIRED_COMMANDS:
        _write(bundle / "commands" / f"{name}-stdout.txt", f"{name} ok\n")
        _write(bundle / "commands" / f"{name}-stderr.txt", "")
    _write(bundle / "package" / "rpm-info.txt", "Name        : garnet-studio\nArchitecture: x86_64\n")
    _write(
        bundle / "package" / "rpm-contents.txt",
        "/usr/bin/garnet-studio\n/usr/share/applications/garnet-studio.desktop\n",
    )
    _write(bundle / "extracted" / "studio-smoke.json", '{"status":"ok"}\n')
    data = {
        "schema": linux_rpm.SCHEMA,
        "generated_at": "2026-06-03T12:00:00+00:00",
        "status": "passed",
        "platform": "linux",
        "evidence_tier": "wsl-linux-rpm-package-extract-command-smoke",
        "wsl_is_enforcement": False,
        "source_included": False,
        "provider_api_called": False,
        "desktop_gui_launch_proven": False,
        "clean_linux_install_proven": False,
        "privileged_system_install_proven": False,
        "package_extract_proven": True,
        "installed_or_extracted_binary_smoke_proven": True,
        "rpm_tooling": {
            "rpmbuild": "/usr/bin/rpmbuild",
            "rpm": "/usr/bin/rpm",
            "rpm2cpio": "/usr/bin/rpm2cpio",
            "cpio": "/usr/bin/cpio",
            "installed_by_recorder": False,
        },
        "package": {
            "format": "rpm",
            "path": "target/release/bundle/rpm/garnet-studio-0.1.0-1.x86_64.rpm",
            "sha256": "b" * 64,
            "size_bytes": 12345,
            "architecture": "x86_64",
            "contains_binary": True,
            "contains_desktop_file": True,
        },
        "extracted_binary": {
            "path": "target/linux-wsl-rpm-stage-20260603-120000/usr/bin/garnet-studio",
            "sha256": "a" * 64,
            "studio_smoke_status": "passed",
            "studio_smoke_file": "extracted/studio-smoke.json",
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
            for name in linux_rpm.REQUIRED_COMMANDS
        ],
        "honest_scope": [
            "WSL is Linux RPM package extract and command-smoke evidence only",
            "not Linux desktop GUI launch proof",
            "not Linux seccomp or OS-sandbox enforcement",
            "not clean Linux install proof",
            "not privileged system package install proof",
            "not signed, production, or v1.0 readiness",
        ],
    }
    summary = bundle / linux_rpm.SUMMARY_NAME
    _write(summary, json.dumps(data, indent=2) + "\n")
    _write(bundle / linux_rpm.MARKDOWN_NAME, linux_rpm.render_markdown(data))
    _write_manifest(bundle)
    return summary


class LinuxWslRpmProofTests(unittest.TestCase):
    def test_valid_rpm_bundle_verifies_without_overclaiming(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            summary = _valid_bundle(Path(temp))

            self.assertTrue(linux_rpm.verify_bundle(summary))
            evidence = linux_rpm.read_committed_evidence(root=Path(temp))

        self.assertTrue(evidence.verified)
        self.assertIn(".rpm", evidence.reason)
        self.assertIn("not Linux desktop GUI launch proof", " ".join(evidence.deferred))

    def test_bundle_fails_if_extracted_smoke_output_is_missing(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = Path(temp)
            summary = _valid_bundle(bundle)
            (bundle / "extracted" / "studio-smoke.json").unlink()
            _write_manifest(bundle)

            self.assertFalse(linux_rpm.verify_bundle(summary))

    def test_bundle_fails_if_it_claims_gui_clean_install_or_enforcement(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = Path(temp)
            summary = _valid_bundle(bundle)
            data = json.loads(summary.read_text(encoding="utf-8"))
            data["desktop_gui_launch_proven"] = True
            data["honest_scope"].append("Linux seccomp enforced")
            _write(summary, json.dumps(data, indent=2) + "\n")
            _write_manifest(bundle)

            self.assertFalse(linux_rpm.verify_bundle(summary))

    def test_markdown_lists_rpm_tooling_and_honest_scope(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            data = json.loads(_valid_bundle(Path(temp)).read_text(encoding="utf-8"))
        markdown = linux_rpm.render_markdown(data)

        self.assertIn("# Garnet Studio Linux WSL RPM Package Proof", markdown)
        self.assertIn("rpm2cpio", markdown)
        self.assertIn("not Linux desktop GUI launch proof", markdown)

    def test_manifest_paths_are_posix(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = Path(temp)
            _write(bundle / "commands" / "rpm-contents-stdout.txt", "ok\n")
            linux_rpm._write_manifest(bundle)

            manifest = (bundle / linux_rpm.MANIFEST_NAME).read_text(encoding="utf-8")

        self.assertNotIn("\\", manifest)
        self.assertIn("commands/rpm-contents-stdout.txt", manifest)


if __name__ == "__main__":
    unittest.main()
