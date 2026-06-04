#!/usr/bin/env python3
"""Regression tests for the full S109 cross-OS trap parity matrix gate."""
from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_cross_os_trap_parity_matrix.py")
SPEC = importlib.util.spec_from_file_location("garnet_cross_os_trap_parity_matrix", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
s109 = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_cross_os_trap_parity_matrix"] = s109
SPEC.loader.exec_module(s109)


def _write_manifest(bundle: Path) -> None:
    lines: list[str] = []
    for path in sorted(p for p in bundle.rglob("*") if p.is_file()):
        if path.name == "MANIFEST.sha256":
            continue
        lines.append(f"{s109._sha256(path)}  {path.relative_to(bundle).as_posix()}")
    (bundle / "MANIFEST.sha256").write_text("\n".join(lines) + "\n", encoding="utf-8")


def _write_summary(
    bundle: Path,
    *,
    linux_diff_caps: bool = True,
    wsl_not_linux: bool = True,
    cross_os_complete: bool = True,
) -> Path:
    rows = []
    for trap in ("max_depth", "caps", "diff_caps_reject"):
        linux_status = linux_diff_caps if trap == "diff_caps_reject" else True
        rows.append(
            {
                "trap": trap,
                "status": "passed" if linux_status else "failed",
                "windows": {"status": True, "tier": "enforcement"},
                "mac": {"status": True, "tier": "enforcement"},
                "linux": {"status": linux_status, "tier": "enforcement"},
                "wsl": {
                    "status": True,
                    "tier": "execution-portability",
                    "excluded_from_linux_enforcement": wsl_not_linux,
                },
            }
        )
    summary = {
        "schema": "garnet.cross_os_trap_parity_matrix.v1",
        "status": "passed" if cross_os_complete and linux_diff_caps and wsl_not_linux else "failed",
        "cross_os_complete": cross_os_complete,
        "commands": [
            {
                "id": "linux-diff-caps-reject",
                "expected_exit_code": 1,
                "exit_code": 1 if linux_diff_caps else 0,
                "status": "passed" if linux_diff_caps else "failed",
                "stdout_file": "commands/linux-diff-caps-reject-stdout.txt",
                "stderr_file": "commands/linux-diff-caps-reject-stderr.txt",
            }
        ],
        "trap_rows": rows,
        "byte_comparisons": [
            {"id": "accept_capability_manifest", "expected_os_independent": True, "byte_equal": True},
            {"id": "accept_transparency_log", "expected_os_independent": True, "byte_equal": True},
        ],
        "honest_scope": [
            "Full S109 cross-OS trap parity requires committed Windows, Mac, and Linux rows.",
            "WSL remains execution/portability evidence and is excluded from Linux enforcement.",
            "Linux seccomp is Linux-only evidence, not Windows/macOS OS-sandbox enforcement.",
            "No Wasmtime fuel, production, release, tag, S120, or v1.0 claim is made.",
        ],
    }
    (bundle / "commands").mkdir(parents=True)
    (bundle / "commands" / "linux-diff-caps-reject-stdout.txt").write_text(
        "AUTHORITY EXPANDED\n", encoding="utf-8"
    )
    (bundle / "commands" / "linux-diff-caps-reject-stderr.txt").write_text("", encoding="utf-8")
    summary_path = bundle / "garnet-cross-os-trap-parity-matrix.json"
    summary_path.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    (bundle / "garnet-cross-os-trap-parity-matrix.md").write_text(
        s109.render_markdown(summary), encoding="utf-8"
    )
    _write_manifest(bundle)
    return summary_path


class CrossOsTrapParityMatrixTests(unittest.TestCase):
    def test_verifier_accepts_full_windows_mac_linux_matrix(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            summary = _write_summary(Path(tmp))

            self.assertTrue(s109.verify_bundle(summary))

    def test_verifier_rejects_missing_linux_diff_caps_row(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            summary = _write_summary(Path(tmp), linux_diff_caps=False)

            self.assertFalse(s109.verify_bundle(summary))

    def test_verifier_rejects_wsl_as_linux_enforcement_claim(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            summary = _write_summary(Path(tmp), wsl_not_linux=False)

            self.assertFalse(s109.verify_bundle(summary))

    def test_verifier_rejects_incomplete_cross_os_claim(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            summary = _write_summary(Path(tmp), cross_os_complete=False)

            self.assertFalse(s109.verify_bundle(summary))


if __name__ == "__main__":
    unittest.main()
