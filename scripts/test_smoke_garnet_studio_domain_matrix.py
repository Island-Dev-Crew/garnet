#!/usr/bin/env python3
"""Regression tests for the Garnet Studio domain proof matrix."""
from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import textwrap
import unittest
from datetime import datetime, timezone
from pathlib import Path

SCRIPT = Path(__file__).with_name("smoke_garnet_studio_domain_matrix.py")
SPEC = importlib.util.spec_from_file_location("smoke_garnet_studio_domain_matrix", SCRIPT)
assert SPEC is not None
matrix_mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["smoke_garnet_studio_domain_matrix"] = matrix_mod
SPEC.loader.exec_module(matrix_mod)


class GarnetStudioDomainMatrixTests(unittest.TestCase):
    def test_suite_inventory_covers_core_12_and_agentic_domains(self) -> None:
        core = matrix_mod.select_cases("core12")
        agentic = matrix_mod.select_cases("agentic")
        combined = matrix_mod.select_cases("all")

        self.assertEqual(12, len(core))
        self.assertGreaterEqual(len(agentic), 7)
        self.assertEqual(len(core) + len(agentic), len(combined))
        self.assertEqual(len({case.id for case in combined}), len(combined))
        self.assertIn("mvp_11_signed_hotreload_mismatch", {case.id for case in core})
        self.assertIn("safe_io_layer", {case.id for case in agentic})

    def test_matrix_executes_fake_cli_and_writes_manifested_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            fake_cli = self._write_fake_cli(root)
            success = root / "success.garnet"
            mismatch = root / "mismatch.garnet"
            success.write_text("fn main() -> Int { 1 }\n", encoding="utf-8")
            mismatch.write_text("fn main() -> Int { 1 }\n", encoding="utf-8")
            output = root / "evidence"
            fixed = datetime(2026, 5, 25, 1, 2, 3, tzinfo=timezone.utc)

            summary = matrix_mod.run_matrix(
                "custom",
                output_dir=output,
                garnet_argv=[sys.executable, str(fake_cli)],
                cases=[
                    matrix_mod.DomainCase("success", "Success path", "test", str(success)),
                    matrix_mod.DomainCase(
                        "mismatch",
                        "Expected trust-boundary failure",
                        "test",
                        str(mismatch),
                        expected_run_failure=True,
                        expected_stderr_contains=matrix_mod.TRUST_MISMATCH_MARKER,
                    ),
                ],
                now=fixed,
                repo_root=root,
            )

            self.assertEqual("passed", summary["status"])
            self.assertEqual(2, summary["case_count"])
            self.assertEqual(6, summary["command_count"])
            self.assertFalse(summary["source_included"])
            self.assertFalse(summary["provider_api_called"])
            self.assertTrue((output / "garnet-studio-domain-matrix.json").exists())
            self.assertTrue((output / "garnet-studio-domain-matrix.md").exists())
            manifest = (output / "MANIFEST.sha256").read_text(encoding="utf-8")
            self.assertIn("garnet-studio-domain-matrix.json", manifest)
            self.assertIn("commands/mismatch-run-stderr.txt", manifest)

            recorded = json.loads((output / "garnet-studio-domain-matrix.json").read_text(encoding="utf-8"))
            mismatch_case = next(case for case in recorded["cases"] if case["id"] == "mismatch")
            run = next(command for command in mismatch_case["commands"] if command["step"] == "run")
            self.assertEqual("passed", run["status"])
            self.assertTrue(run["expected_failure"])
            self.assertNotEqual(0, run["exit_code"])

    def test_expected_failure_requires_expected_marker(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            fake_cli = self._write_fake_cli(root)
            bad = root / "badmarker.garnet"
            bad.write_text("fn main() -> Int { 1 }\n", encoding="utf-8")

            summary = matrix_mod.run_matrix(
                "custom",
                output_dir=root / "evidence",
                garnet_argv=[sys.executable, str(fake_cli)],
                cases=[
                    matrix_mod.DomainCase(
                        "badmarker",
                        "Expected failure missing marker",
                        "test",
                        str(bad),
                        expected_run_failure=True,
                        expected_stderr_contains=matrix_mod.TRUST_MISMATCH_MARKER,
                    )
                ],
                repo_root=root,
            )

        self.assertEqual("failed", summary["status"])
        self.assertEqual(1, summary["failed_cases"])
        case = summary["cases"][0]
        run = next(command for command in case["commands"] if command["step"] == "run")
        self.assertEqual("failed", run["status"])
        self.assertTrue(run["expected_failure"])

    def test_locate_garnet_prefers_release_build_before_debug_build(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            executable = "garnet.exe" if matrix_mod.os.name == "nt" else "garnet"
            release = root / "target" / "release" / executable
            debug = root / "target" / "debug" / executable
            release.parent.mkdir(parents=True)
            debug.parent.mkdir(parents=True)
            release.write_text("", encoding="utf-8")
            debug.write_text("", encoding="utf-8")

            old_root = matrix_mod.ROOT
            old_env = matrix_mod.os.environ.pop("GARNET_CLI", None)
            try:
                matrix_mod.ROOT = root
                self.assertEqual([str(release)], matrix_mod.locate_garnet())
            finally:
                matrix_mod.ROOT = old_root
                if old_env is not None:
                    matrix_mod.os.environ["GARNET_CLI"] = old_env

    def test_garnet_cli_env_accepts_windows_paths_with_spaces(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            cli = Path(temp) / "Program Files" / "Garnet" / "garnet.exe"
            cli.parent.mkdir(parents=True)
            cli.write_text("", encoding="utf-8")

            old_env = matrix_mod.os.environ.get("GARNET_CLI")
            try:
                matrix_mod.os.environ["GARNET_CLI"] = str(cli)
                self.assertEqual([str(cli)], matrix_mod.locate_garnet())

                matrix_mod.os.environ["GARNET_CLI"] = f'"{cli}"'
                self.assertEqual([str(cli)], matrix_mod.locate_garnet())
            finally:
                if old_env is None:
                    matrix_mod.os.environ.pop("GARNET_CLI", None)
                else:
                    matrix_mod.os.environ["GARNET_CLI"] = old_env

    def _write_fake_cli(self, root: Path) -> Path:
        fake_cli = root / "fake_garnet_cli.py"
        fake_cli.write_text(
            textwrap.dedent(
                f"""
                import sys
                from pathlib import Path

                step = sys.argv[1]
                source = Path(sys.argv[2])

                if step in ("parse", "check"):
                    print(f"{{step}} ok: {{source.name}}")
                    raise SystemExit(0)

                if step == "run" and "mismatch" in source.name:
                    print("blocked unsafe reload")
                    print("runtime error: exception: {matrix_mod.TRUST_MISMATCH_MARKER}", file=sys.stderr)
                    raise SystemExit(1)

                if step == "run" and "badmarker" in source.name:
                    print("blocked unsafe reload")
                    print("runtime error: exception: wrong marker", file=sys.stderr)
                    raise SystemExit(1)

                if step == "run":
                    print(f"run ok: {{source.name}}")
                    raise SystemExit(0)

                raise SystemExit(64)
                """
            ).lstrip(),
            encoding="utf-8",
        )
        return fake_cli


if __name__ == "__main__":
    unittest.main()
