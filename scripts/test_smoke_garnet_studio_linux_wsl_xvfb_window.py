#!/usr/bin/env python3
"""Tests for the WSL Linux Xvfb Studio window-capture proof recorder."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("smoke_garnet_studio_linux_wsl_xvfb_window.py")
SPEC = importlib.util.spec_from_file_location("smoke_garnet_studio_linux_wsl_xvfb_window", SCRIPT)
xvfb_window = importlib.util.module_from_spec(SPEC)
sys.modules["smoke_garnet_studio_linux_wsl_xvfb_window"] = xvfb_window
SPEC.loader.exec_module(xvfb_window)


def _write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


def _hash(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _write_manifest(bundle: Path) -> None:
    lines = []
    for path in sorted(p for p in bundle.rglob("*") if p.is_file()):
        if path.name == xvfb_window.MANIFEST_NAME:
            continue
        lines.append(f"{_hash(path)}  {path.relative_to(bundle).as_posix()}")
    _write(bundle / xvfb_window.MANIFEST_NAME, "\n".join(lines) + "\n")


def _valid_bundle(bundle: Path) -> Path:
    for name in xvfb_window.REQUIRED_COMMANDS:
        _write(bundle / "commands" / f"{name}-stdout.txt", f"{name} ok\n")
        _write(bundle / "commands" / f"{name}-stderr.txt", "")
    _write(bundle / "capture" / "xwininfo.txt", "0x200001 \"Garnet Studio\": ()  1024x768+0+0\n")
    _write(bundle / "capture" / "xdpyinfo.txt", "dimensions:    1280x720 pixels\n")
    _write(bundle / "capture" / "identify.txt", "capture/screenshot.png PNG 1280x720\n")
    (bundle / "capture").mkdir(parents=True, exist_ok=True)
    (bundle / "capture" / "screenshot.png").write_bytes(b"\x89PNG\r\n\x1a\n" + b"0" * 2048)
    data = {
        "schema": xvfb_window.SCHEMA,
        "generated_at": "2026-06-03T12:00:00+00:00",
        "status": "passed",
        "platform": "linux",
        "evidence_tier": "wsl-linux-xvfb-virtual-display-window-capture",
        "wsl_is_enforcement": False,
        "source_included": False,
        "provider_api_called": False,
        "linux_enforcement_proven": False,
        "desktop_gui_launch_proven": False,
        "linux_desktop_gui_launch_proven": False,
        "clean_linux_install_proven": False,
        "privileged_system_install_proven": False,
        "virtual_display_window_capture_proven": True,
        "source_runtime_proof": {
            "summary": "proofs/linux/execution/studio-xvfb-runtime/linux-wsl-xvfb-runtime-test/garnet-studio-linux-wsl-xvfb-runtime.json"
        },
        "source_package_proof": {
            "format": "deb",
            "summary": "proofs/linux/execution/studio-package-install/linux-wsl-deb-install-test/garnet-studio-linux-wsl-deb-install.json",
        },
        "extracted_binary": {
            "path": "target/linux-wsl-deb-install-stage-test/usr/bin/garnet-studio",
            "sha256": "b" * 64,
        },
        "x11_tooling": {
            "xvfb-run": "/usr/bin/xvfb-run",
            "xwininfo": "/usr/bin/xwininfo",
            "xdpyinfo": "/usr/bin/xdpyinfo",
            "xwd": "/usr/bin/xwd",
            "convert": "/usr/bin/convert",
            "identify": "/usr/bin/identify",
        },
        "window_capture": {
            "status": "passed",
            "window_tree_file": "capture/xwininfo.txt",
            "display_info_file": "capture/xdpyinfo.txt",
            "screenshot_file": "capture/screenshot.png",
            "screenshot_sha256": _hash(bundle / "capture" / "screenshot.png"),
            "screenshot_bytes": (bundle / "capture" / "screenshot.png").stat().st_size,
            "identify_file": "capture/identify.txt",
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
            for name in xvfb_window.REQUIRED_COMMANDS
        ],
        "honest_scope": [
            "WSL Xvfb virtual-display window-capture evidence only",
            "not Linux desktop GUI launch proof",
            "not Linux seccomp or OS-sandbox enforcement",
            "not clean Linux install proof",
            "not privileged system package install proof",
            "not signed, production, or v1.0 readiness",
        ],
    }
    summary = bundle / xvfb_window.SUMMARY_NAME
    _write(summary, json.dumps(data, indent=2) + "\n")
    _write(bundle / xvfb_window.MARKDOWN_NAME, xvfb_window.render_markdown(data))
    _write_manifest(bundle)
    return summary


class LinuxWslXvfbWindowCaptureProofTests(unittest.TestCase):
    def test_valid_window_capture_bundle_verifies_without_claiming_desktop_gui(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            summary = _valid_bundle(Path(temp))

            self.assertTrue(xvfb_window.verify_bundle(summary))
            evidence = xvfb_window.read_committed_evidence(root=Path(temp))

        self.assertTrue(evidence.verified)
        self.assertIn("virtual-display window capture", evidence.reason)
        self.assertIn("not Linux desktop GUI launch proof", " ".join(evidence.deferred))

    def test_bundle_fails_without_screenshot_or_window_tree(self) -> None:
        for missing in ("capture/screenshot.png", "capture/xwininfo.txt"):
            with self.subTest(missing=missing), tempfile.TemporaryDirectory() as temp:
                bundle = Path(temp)
                summary = _valid_bundle(bundle)
                (bundle / missing).unlink()
                _write_manifest(bundle)

                self.assertFalse(xvfb_window.verify_bundle(summary))

    def test_bundle_fails_if_screenshot_is_too_small_or_hash_mismatches(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = Path(temp)
            summary = _valid_bundle(bundle)
            (bundle / "capture" / "screenshot.png").write_bytes(b"\x89PNG\r\n\x1a\n")
            _write_manifest(bundle)

            self.assertFalse(xvfb_window.verify_bundle(summary))

    def test_bundle_fails_if_it_claims_linux_desktop_or_enforcement(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = Path(temp)
            summary = _valid_bundle(bundle)
            data = json.loads(summary.read_text(encoding="utf-8"))
            data["linux_desktop_gui_launch_proven"] = True
            data["honest_scope"].append("Linux seccomp enforced")
            _write(summary, json.dumps(data, indent=2) + "\n")
            _write_manifest(bundle)

            self.assertFalse(xvfb_window.verify_bundle(summary))

    def test_markdown_lists_capture_artifacts_and_honest_scope(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            data = json.loads(_valid_bundle(Path(temp)).read_text(encoding="utf-8"))

        markdown = xvfb_window.render_markdown(data)

        self.assertIn("# Garnet Studio Linux WSL Xvfb Window Capture Proof", markdown)
        self.assertIn("screenshot.png", markdown)
        self.assertIn("xwininfo", markdown)
        self.assertIn("not Linux desktop GUI launch proof", markdown)


if __name__ == "__main__":
    unittest.main()
