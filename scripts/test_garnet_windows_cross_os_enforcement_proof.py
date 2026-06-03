#!/usr/bin/env python3
"""Regression tests for the S106 Windows/WSL enforcement proof gate."""
from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_windows_cross_os_enforcement_proof.py")
SPEC = importlib.util.spec_from_file_location(
    "garnet_windows_cross_os_enforcement_proof", SCRIPT
)
assert SPEC is not None and SPEC.loader is not None
s106 = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_windows_cross_os_enforcement_proof"] = s106
SPEC.loader.exec_module(s106)


REQUIRED_TRAPS = [
    "@max_depth",
    "@caps(env)",
    "@caps(proc)",
    "@caps(fs)",
    "@caps(net)",
    "S92 program-entry @caps(proc)",
]


def _record(platform: str, tier: str, ok: bool = True) -> dict:
    return {
        "schema": "garnet.windows_cross_os_enforcement_proof/v1",
        "platform": platform,
        "tier": tier,
        "honesty_scope": (
            "Windows enforcement proof"
            if platform == "windows"
            else "WSL execution/portability, not enforcement"
        ),
        "required_traps": REQUIRED_TRAPS,
        "commands": [
            {"name": "s101_gate", "exit_code": 0 if ok else 1},
            {"name": "bounded_enforcement", "exit_code": 0},
            {"name": "caps_enforcement", "exit_code": 0},
        ],
        "ok": ok,
    }


def _write_fixture(root: Path, *, wsl_tier: str = "execution-portability") -> None:
    windows = root / "proofs" / "windows" / "enforcement"
    linux = root / "proofs" / "linux" / "execution"
    windows.mkdir(parents=True)
    linux.mkdir(parents=True)
    (windows / "windows-enforcement-proof.json").write_text(
        json.dumps(_record("windows", "enforcement-proof"), indent=2),
        encoding="utf-8",
    )
    (linux / "wsl-execution-portability-proof.json").write_text(
        json.dumps(_record("wsl", wsl_tier), indent=2),
        encoding="utf-8",
    )


class WindowsCrossOsEnforcementProofTests(unittest.TestCase):
    def test_committed_windows_and_wsl_records_pass(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            _write_fixture(root)

            status = s106.read_status(root)

            self.assertTrue(status.ok)
            self.assertEqual("enforcement-proof", status.windows.tier)
            self.assertEqual("execution-portability", status.wsl.tier)
            self.assertEqual(REQUIRED_TRAPS, status.windows.required_traps)

    def test_wsl_markdown_preserves_non_enforcement_boundary(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            _write_fixture(root)

            md = s106.render_markdown(s106.read_status(root))

            self.assertIn("WSL execution/portability, not enforcement", md)
            self.assertIn("not Linux seccomp", md)
            self.assertIn("not OS-sandbox enforcement", md)

    def test_gate_fails_when_windows_record_is_missing(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            linux = root / "proofs" / "linux" / "execution"
            linux.mkdir(parents=True)
            (linux / "wsl-execution-portability-proof.json").write_text(
                json.dumps(_record("wsl", "execution-portability"), indent=2),
                encoding="utf-8",
            )

            self.assertFalse(s106.read_status(root).ok)

    def test_gate_fails_if_wsl_claims_enforcement(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            _write_fixture(root, wsl_tier="enforcement-proof")

            status = s106.read_status(root)

            self.assertFalse(status.ok)
            self.assertTrue(
                any(
                    "WSL row must be execution/portability" in note
                    for note in status.wsl.notes
                )
            )


if __name__ == "__main__":
    unittest.main()
