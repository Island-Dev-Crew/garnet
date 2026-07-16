#!/usr/bin/env python3
"""Regression tests for the repository-wide Rust MSRV contract."""
from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "garnet_msrv_status.py"
SPEC = importlib.util.spec_from_file_location("garnet_msrv_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
msrv = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_msrv_status"] = msrv
SPEC.loader.exec_module(msrv)


class GarnetMsrvStatusTests(unittest.TestCase):
    def test_real_repository_passes(self) -> None:
        status = msrv.read_status(ROOT)
        self.assertTrue(status.ok, status.findings)
        self.assertEqual(status.msrv, "1.95")
        self.assertEqual(status.workspace_member_count, 16)
        self.assertEqual(status.workspace_members_inheriting, 16)
        self.assertEqual(status.excluded_manifests_declaring, 2)

    def test_gate_output_is_machine_readable(self) -> None:
        proc = subprocess.run(
            [sys.executable, "-I", str(SCRIPT), "--gate"],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        payload = json.loads(proc.stdout)
        self.assertEqual(payload["schema"], "garnet.msrv_status/v1")
        self.assertEqual(payload["msrv"], "1.95")
        self.assertTrue(payload["stable_tracking_preserved"])
        self.assertTrue(payload["exact_msrv_ci_check"])
        self.assertTrue(payload["studio_exact_msrv_ci_check"])
        self.assertTrue(payload["ok"])

    def test_mutated_workspace_floor_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            msrv.copy_contract_surface(ROOT, root)
            manifest = root / "garnet-parser-v0.3" / "Cargo.toml"
            text = manifest.read_text(encoding="utf-8")
            manifest.write_text(
                text.replace("rust-version.workspace = true", 'rust-version = "1.75"'),
                encoding="utf-8",
            )
            status = msrv.read_status(root)
        self.assertFalse(status.ok)
        self.assertTrue(
            any("garnet-parser-v0.3" in finding for finding in status.findings),
            status.findings,
        )

    def test_stale_current_surface_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            msrv.copy_contract_surface(ROOT, root)
            readme = root / "README.md"
            readme.write_text(
                readme.read_text(encoding="utf-8").replace("Rust 1.95+", "Rust 1.75+"),
                encoding="utf-8",
            )
            status = msrv.read_status(root)
        self.assertFalse(status.ok)
        self.assertTrue(
            any("README.md" in finding for finding in status.findings),
            status.findings,
        )


if __name__ == "__main__":
    unittest.main()
