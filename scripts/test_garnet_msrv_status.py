#!/usr/bin/env python3
"""Regression tests for the repository-wide Rust MSRV contract."""
from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from collections.abc import Callable
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "garnet_msrv_status.py"
SPEC = importlib.util.spec_from_file_location("garnet_msrv_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
msrv = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_msrv_status"] = msrv
SPEC.loader.exec_module(msrv)


class GarnetMsrvStatusTests(unittest.TestCase):
    def _mutated_status(
        self, relative: str, mutate: Callable[[str], str]
    ) -> object:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            msrv.copy_contract_surface(ROOT, root)
            path = root / relative
            path.write_text(mutate(path.read_text(encoding="utf-8")), encoding="utf-8")
            return msrv.read_status(root)

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
        self.assertEqual(payload["schema"], "garnet.msrv_status/v2")
        self.assertEqual(payload["msrv"], "1.95")
        self.assertTrue(payload["active_manifest_set_exact"])
        self.assertTrue(payload["workflow_projection_valid"])
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
        status = self._mutated_status(
            "README.md",
            lambda text: text.replace("Rust 1.95+", "Rust 1.94+", 1),
        )
        self.assertFalse(status.ok)
        self.assertTrue(
            any("README.md" in finding for finding in status.findings),
            status.findings,
        )

    def test_commented_exact_msrv_command_does_not_satisfy_ci(self) -> None:
        status = self._mutated_status(
            ".github/workflows/ci.yml",
            lambda text: text.replace(
                f"        run: {msrv.ROOT_CI_COMMAND}",
                f"        # run: {msrv.ROOT_CI_COMMAND}",
                1,
            ),
        )
        self.assertFalse(status.exact_msrv_ci_check)
        self.assertFalse(status.ok)

    def test_disabled_exact_msrv_step_does_not_satisfy_ci(self) -> None:
        status = self._mutated_status(
            ".github/workflows/ci.yml",
            lambda text: text.replace(
                "        if: runner.os == 'Linux'\n"
                f"        run: {msrv.ROOT_CI_COMMAND}",
                "        if: ${{ false }}\n"
                f"        run: {msrv.ROOT_CI_COMMAND}",
                1,
            ),
        )
        self.assertFalse(status.exact_msrv_ci_check)
        self.assertFalse(status.ok)

    def test_exact_command_in_wrong_job_does_not_satisfy_ci(self) -> None:
        def mutate(text: str) -> str:
            text = text.replace(
                f"        run: {msrv.ROOT_CI_COMMAND}",
                "        run: cargo check --workspace --locked",
                1,
            )
            marker = "      - run: python3 -I scripts/garnet_msrv_status.py --gate\n"
            return text.replace(
                marker,
                marker + f"      - run: {msrv.ROOT_CI_COMMAND}\n",
                1,
            )

        status = self._mutated_status(".github/workflows/ci.yml", mutate)
        self.assertFalse(status.exact_msrv_ci_check)
        self.assertFalse(status.ok)

    def test_stable_toolchain_must_be_in_required_test_job(self) -> None:
        def mutate(text: str) -> str:
            marker = (
                "  test:\n"
                "    name: cargo test (${{ matrix.os }})"
            )
            before, after = text.split(marker, 1)
            after = after.replace(
                "dtolnay/rust-toolchain@stable",
                "dtolnay/rust-toolchain@1.95.0",
                1,
            )
            return before + marker + after

        status = self._mutated_status(".github/workflows/ci.yml", mutate)
        self.assertFalse(status.stable_tracking_preserved)
        self.assertFalse(status.ok)

    def test_stable_toolchain_must_be_in_required_studio_job(self) -> None:
        status = self._mutated_status(
            ".github/workflows/macos-studio.yml",
            lambda text: text.replace(
                "dtolnay/rust-toolchain@stable",
                "dtolnay/rust-toolchain@1.95.0",
                1,
            ),
        )
        self.assertFalse(status.stable_tracking_preserved)
        self.assertFalse(status.ok)

    def test_unlisted_active_manifest_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            msrv.copy_contract_surface(ROOT, root)
            extra = root / "new-active-crate" / "Cargo.toml"
            extra.parent.mkdir(parents=True)
            extra.write_text(
                '[package]\nname = "new-active"\nversion = "0.1.0"\n'
                'edition = "2021"\nrust-version = "1.95"\n',
                encoding="utf-8",
            )
            status = msrv.read_status(root)
        self.assertFalse(status.active_manifest_set_exact)
        self.assertFalse(status.ok)
        self.assertTrue(
            any("new-active-crate/Cargo.toml" in finding for finding in status.findings),
            status.findings,
        )


if __name__ == "__main__":
    unittest.main()
