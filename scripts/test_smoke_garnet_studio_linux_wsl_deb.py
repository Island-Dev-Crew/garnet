#!/usr/bin/env python3
"""Tests for the WSL Linux Garnet Studio DEB package proof gate."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("smoke_garnet_studio_linux_wsl_deb.py")
SPEC = importlib.util.spec_from_file_location("smoke_garnet_studio_linux_wsl_deb", SCRIPT)
assert SPEC is not None
linux_deb = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["smoke_garnet_studio_linux_wsl_deb"] = linux_deb
SPEC.loader.exec_module(linux_deb)


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
    for name in [
        "wsl-uname",
        "npm-install",
        "npm-build",
        "tauri-build-deb",
        "studio-smoke",
        "dpkg-info",
        "dpkg-contents",
    ]:
        _write(bundle / "commands" / f"{name}-stdout.txt", f"{name} ok\n")
        _write(bundle / "commands" / f"{name}-stderr.txt", "")
    _write(bundle / "package" / "dpkg-info.txt", "Package: garnet-studio\nArchitecture: amd64\n")
    _write(
        bundle / "package" / "dpkg-contents.txt",
        "./usr/bin/garnet-studio\n./usr/share/applications/Garnet Studio.desktop\n",
    )
    data = {
        "schema": linux_deb.SCHEMA,
        "status": "passed",
        "platform": "linux",
        "evidence_tier": "wsl-linux-package-build-command-smoke",
        "wsl_is_enforcement": False,
        "source_included": False,
        "provider_api_called": False,
        "desktop_gui_launch_proven": False,
        "package_install_proven": False,
        "package": {
            "format": "deb",
            "path": "target/release/bundle/deb/Garnet Studio_0.1.0_amd64.deb",
            "sha256": "d" * 64,
            "size_bytes": 3022068,
            "architecture": "amd64",
            "contains_binary": True,
            "contains_desktop_file": True,
        },
        "binary": {
            "path": "target/release/garnet-studio",
            "sha256": "b" * 64,
            "studio_smoke_status": "passed",
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
            for name in [
                "wsl-uname",
                "npm-install",
                "npm-build",
                "tauri-build-deb",
                "studio-smoke",
                "dpkg-info",
                "dpkg-contents",
            ]
        ],
        "honest_scope": [
            "WSL is Linux package build and command-smoke evidence only",
            "not Linux desktop GUI launch proof",
            "not Linux seccomp or OS-sandbox enforcement",
            "not clean Linux install proof",
            "not signed, production, or v1.0 readiness",
        ],
    }
    summary = bundle / linux_deb.SUMMARY_NAME
    _write(summary, json.dumps(data, indent=2) + "\n")
    _write(bundle / linux_deb.MARKDOWN_NAME, linux_deb.render_markdown(data))
    _write_manifest(bundle)
    return summary


class LinuxWslDebProofTests(unittest.TestCase):
    def test_valid_manifested_deb_bundle_verifies_without_overclaiming(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            summary = _valid_bundle(Path(temp))

            self.assertTrue(linux_deb.verify_bundle(summary))
            evidence = linux_deb.read_committed_evidence(root=Path(temp))

        self.assertTrue(evidence.verified)
        self.assertEqual("verified", evidence.status)
        self.assertIn(".deb", evidence.reason)
        self.assertIn("not Linux desktop GUI", " ".join(evidence.deferred))

    def test_bundle_fails_if_required_command_log_is_missing(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = Path(temp)
            summary = _valid_bundle(bundle)
            (bundle / "commands" / "dpkg-contents-stdout.txt").unlink()
            _write_manifest(bundle)

            self.assertFalse(linux_deb.verify_bundle(summary))

    def test_bundle_fails_if_it_claims_gui_or_enforcement_completion(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = Path(temp)
            summary = _valid_bundle(bundle)
            data = json.loads(summary.read_text(encoding="utf-8"))
            data["desktop_gui_launch_proven"] = True
            data["honest_scope"].append("Linux desktop GUI launch proof verified")
            _write(summary, json.dumps(data, indent=2) + "\n")
            _write_manifest(bundle)

            self.assertFalse(linux_deb.verify_bundle(summary))

    def test_markdown_preserves_package_and_boundary_truth(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            data = json.loads(_valid_bundle(Path(temp)).read_text(encoding="utf-8"))

        markdown = linux_deb.render_markdown(data)

        self.assertIn("Garnet Studio Linux WSL DEB Package Proof", markdown)
        self.assertIn("wsl-linux-package-build-command-smoke", markdown)
        self.assertIn("not Linux desktop GUI launch proof", markdown)
        self.assertIn("not Linux seccomp or OS-sandbox enforcement", markdown)

    def test_manifest_uses_posix_paths_for_linux_verification(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = Path(temp)
            _write(bundle / "commands" / "studio-smoke-stdout.txt", "ok\n")

            linux_deb._write_manifest(bundle)

            manifest = (bundle / linux_deb.MANIFEST_NAME).read_text(encoding="utf-8")

        self.assertIn("commands/studio-smoke-stdout.txt", manifest)
        self.assertNotIn("\\", manifest)


if __name__ == "__main__":
    unittest.main()
