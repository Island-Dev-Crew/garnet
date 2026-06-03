#!/usr/bin/env python3
"""Tests for the WSL Linux Garnet Studio DEB install/extract proof gate."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("smoke_garnet_studio_linux_wsl_deb_install.py")
SPEC = importlib.util.spec_from_file_location("smoke_garnet_studio_linux_wsl_deb_install", SCRIPT)
assert SPEC is not None
linux_deb_install = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["smoke_garnet_studio_linux_wsl_deb_install"] = linux_deb_install
SPEC.loader.exec_module(linux_deb_install)


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
    for name in linux_deb_install.REQUIRED_COMMANDS:
        _write(bundle / "commands" / f"{name}-stdout.txt", f"{name} ok\n")
        _write(bundle / "commands" / f"{name}-stderr.txt", "")
    _write(bundle / "package" / "dpkg-info.txt", "Package: garnet-studio\nArchitecture: amd64\n")
    _write(
        bundle / "package" / "dpkg-contents.txt",
        "./usr/bin/garnet-studio\n./usr/share/applications/Garnet Studio.desktop\n",
    )
    _write(bundle / "extracted" / "studio-smoke.json", '{"status":"ok"}\n')
    data = {
        "schema": linux_deb_install.SCHEMA,
        "status": "passed",
        "platform": "linux",
        "evidence_tier": "wsl-linux-package-extract-command-smoke",
        "wsl_is_enforcement": False,
        "source_included": False,
        "provider_api_called": False,
        "desktop_gui_launch_proven": False,
        "clean_linux_install_proven": False,
        "privileged_system_install_proven": False,
        "package_extract_proven": True,
        "installed_or_extracted_binary_smoke_proven": True,
        "package": {
            "format": "deb",
            "path": "target/release/bundle/deb/Garnet Studio_0.1.0_amd64.deb",
            "sha256": "d" * 64,
            "size_bytes": 3022068,
            "architecture": "amd64",
            "contains_binary": True,
            "contains_desktop_file": True,
        },
        "extracted_binary": {
            "path": "stage/usr/bin/garnet-studio",
            "sha256": "b" * 64,
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
            for name in linux_deb_install.REQUIRED_COMMANDS
        ],
        "honest_scope": [
            "WSL is Linux package extract and command-smoke evidence only",
            "not Linux desktop GUI launch proof",
            "not Linux seccomp or OS-sandbox enforcement",
            "not clean Linux install proof",
            "not privileged system package install proof",
            "not signed, production, or v1.0 readiness",
        ],
    }
    summary = bundle / linux_deb_install.SUMMARY_NAME
    _write(summary, json.dumps(data, indent=2) + "\n")
    _write(bundle / linux_deb_install.MARKDOWN_NAME, linux_deb_install.render_markdown(data))
    _write_manifest(bundle)
    return summary


class LinuxWslDebInstallProofTests(unittest.TestCase):
    def test_valid_extract_bundle_verifies_without_overclaiming(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            summary = _valid_bundle(Path(temp))

            self.assertTrue(linux_deb_install.verify_bundle(summary))
            evidence = linux_deb_install.read_committed_evidence(root=Path(temp))

        self.assertTrue(evidence.verified)
        self.assertEqual("verified", evidence.status)
        self.assertIn("extract", evidence.reason)
        self.assertIn("not clean Linux install", " ".join(evidence.deferred))

    def test_bundle_fails_if_extracted_smoke_output_is_missing(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = Path(temp)
            summary = _valid_bundle(bundle)
            (bundle / "extracted" / "studio-smoke.json").unlink()
            _write_manifest(bundle)

            self.assertFalse(linux_deb_install.verify_bundle(summary))

    def test_bundle_fails_if_it_claims_gui_clean_install_or_enforcement(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = Path(temp)
            summary = _valid_bundle(bundle)
            data = json.loads(summary.read_text(encoding="utf-8"))
            data["desktop_gui_launch_proven"] = True
            data["clean_linux_install_proven"] = True
            data["honest_scope"].append("Linux desktop GUI launch proof verified")
            _write(summary, json.dumps(data, indent=2) + "\n")
            _write_manifest(bundle)

            self.assertFalse(linux_deb_install.verify_bundle(summary))

    def test_markdown_preserves_extract_and_boundary_truth(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            data = json.loads(_valid_bundle(Path(temp)).read_text(encoding="utf-8"))

        markdown = linux_deb_install.render_markdown(data)

        self.assertIn("Garnet Studio Linux WSL DEB Install Proof", markdown)
        self.assertIn("wsl-linux-package-extract-command-smoke", markdown)
        self.assertIn("not Linux desktop GUI launch proof", markdown)
        self.assertIn("not clean Linux install proof", markdown)

    def test_manifest_uses_posix_paths_for_linux_verification(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = Path(temp)
            _write(bundle / "commands" / "extracted-studio-smoke-stdout.txt", "ok\n")

            linux_deb_install._write_manifest(bundle)

            manifest = (bundle / linux_deb_install.MANIFEST_NAME).read_text(encoding="utf-8")

        self.assertIn("commands/extracted-studio-smoke-stdout.txt", manifest)
        self.assertNotIn("\\", manifest)


if __name__ == "__main__":
    unittest.main()
