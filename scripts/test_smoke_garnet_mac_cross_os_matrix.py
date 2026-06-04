#!/usr/bin/env python3
"""Tests for the S109 Mac cross-OS matrix recorder."""
from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("smoke_garnet_mac_cross_os_matrix.py")
SPEC = importlib.util.spec_from_file_location("smoke_garnet_mac_cross_os_matrix", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
s109 = importlib.util.module_from_spec(SPEC)
sys.modules["smoke_garnet_mac_cross_os_matrix"] = s109
SPEC.loader.exec_module(s109)


def _write_manifest(bundle: Path) -> None:
    lines: list[str] = []
    for path in sorted(p for p in bundle.rglob("*") if p.is_file()):
        if path.name == "MANIFEST.sha256":
            continue
        lines.append(f"{s109._sha256(path)}  {path.relative_to(bundle).as_posix()}")
    (bundle / "MANIFEST.sha256").write_text("\n".join(lines) + "\n", encoding="utf-8")


def _write_verified_bundle(bundle: Path, *, cross_os_complete: bool = False, cap_equal: bool = True) -> Path:
    commands = []
    (bundle / "commands").mkdir(parents=True)
    for command_id in ("mac-s101-gate", "mac-bounded-enforcement", "mac-caps-enforcement"):
        stdout = bundle / "commands" / f"{command_id}-stdout.txt"
        stderr = bundle / "commands" / f"{command_id}-stderr.txt"
        stdout.write_text(f"{command_id} ok\n", encoding="utf-8")
        stderr.write_text("", encoding="utf-8")
        commands.append(
            {
                "id": command_id,
                "display_args": ["test", command_id],
                "exit_code": 0,
                "stdout_file": stdout.relative_to(bundle).as_posix(),
                "stderr_file": stderr.relative_to(bundle).as_posix(),
                "status": "passed",
            }
        )
    summary = {
        "schema": "garnet.mac_cross_os_matrix.v1",
        "platform": "macos",
        "status": "passed",
        "mac_rows_complete": True,
        "cross_os_complete": cross_os_complete,
        "cross_os_complete_reason": "Independent Linux S108 enforcement row is not present.",
        "commands": commands,
        "trap_rows": [
            {
                "trap": trap,
                "status": "passed",
                "windows": {"status": True},
                "mac": {"status": True},
                "wsl": {"status": True, "tier": "execution-portability"},
            }
            for trap in ("max_depth", "caps", "diff_caps_reject")
        ],
        "byte_comparisons": [
            {
                "id": "accept_capability_manifest",
                "expected_os_independent": True,
                "byte_equal": cap_equal,
                "delta": "Must be byte-identical.",
            },
            {
                "id": "accept_transparency_log",
                "expected_os_independent": True,
                "byte_equal": True,
                "delta": "Must be byte-identical.",
            },
            {
                "id": "accept_diff_caps",
                "expected_os_independent": False,
                "byte_equal": False,
                "normalized_body_equal": True,
                "delta": "Full text includes absolute OS paths.",
            },
            {
                "id": "accept_seal",
                "expected_os_independent": False,
                "status": "passed",
                "full_json_byte_equal": False,
                "field_equal": {"source_blake3": True},
                "delta": "Full seal JSON differs because the prelude_hash field differs.",
            },
        ],
        "honest_scope": [
            "This is the Mac row for S109 consolidation, not full S109 completion.",
            "WSL remains execution/portability evidence only, not Linux seccomp enforcement.",
        ],
    }
    summary_path = bundle / "garnet-mac-cross-os-matrix.json"
    summary_path.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    (bundle / "garnet-mac-cross-os-matrix.md").write_text(
        s109.render_markdown(summary),
        encoding="utf-8",
    )
    _write_manifest(bundle)
    return summary_path


class MacCrossOsMatrixTests(unittest.TestCase):
    def test_verifier_accepts_mac_row_bundle(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            summary = _write_verified_bundle(Path(tmp))

            self.assertTrue(s109.verify_bundle(summary))

    def test_verifier_rejects_full_cross_os_completion_claim(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            summary = _write_verified_bundle(Path(tmp), cross_os_complete=True)

            self.assertFalse(s109.verify_bundle(summary))

    def test_verifier_rejects_missing_os_independent_byte_match(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            summary = _write_verified_bundle(Path(tmp), cap_equal=False)

            self.assertFalse(s109.verify_bundle(summary))


if __name__ == "__main__":
    unittest.main()
