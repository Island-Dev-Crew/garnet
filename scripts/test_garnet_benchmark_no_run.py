#!/usr/bin/env python3
"""Regression tests for benchmark no-run evidence reporter."""
from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
import sys
from pathlib import Path
from typing import Sequence

SCRIPT = Path(__file__).with_name("garnet_benchmark_no_run.py")
SPEC = importlib.util.spec_from_file_location("garnet_benchmark_no_run", SCRIPT)
assert SPEC is not None
mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
if str(SCRIPT.parent) not in sys.path:
    sys.path.insert(0, str(SCRIPT.parent))
sys.modules["garnet_benchmark_no_run"] = mod
SPEC.loader.exec_module(mod)


def _runner_success(command: Sequence[str], _cwd: Path) -> tuple[int, str, str]:
    return 0, f"PASS {command}", ""


def _runner_fail(command: Sequence[str], _cwd: Path) -> tuple[int, str, str]:
    return 1, "", f"FAIL {command}"


class GarnetBenchmarkNoRunTests(unittest.TestCase):
    def test_planned_status_does_not_run_commands(self) -> None:
        status = mod._collect_status(execute=False)
        expected_count = len(mod._planned_harnesses())
        self.assertEqual("planned", status.overall_status)
        self.assertEqual("not-measured", status.measurement_status)
        self.assertEqual("not-mechanized", status.mechanized_proof_status)
        self.assertEqual(expected_count, len(status.benchmarks))
        self.assertIn("vm_parse_compile_execute", {item.id for item in status.benchmarks})
        self.assertTrue(all(item.command.endswith("--no-run") for item in status.benchmarks))
        self.assertTrue(all(item.return_code is None for item in status.benchmarks))

    def test_successful_execute_marks_compile_verified_and_writes_bundle(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            out = Path(tmpdir) / "no-run"
            status = mod._collect_status(
                execute=True,
                output_dir=out,
                run_command=_runner_success,
            )
            self.assertEqual("compile-verified", status.overall_status)
            mod.write_bundle(status, out)
            self.assertTrue((out / "garnet-benchmark-no-run.json").is_file())
            self.assertTrue((out / "garnet-benchmark-no-run.md").is_file())
            self.assertTrue((out / "garnet-benchmark-no-run.json").is_file())
            self.assertTrue((out / "MANIFEST.sha256").is_file())
            expected_count = len(mod._planned_harnesses())
            self.assertEqual(expected_count, len(list(out.glob("*.stdout.log"))))
            self.assertEqual(expected_count, len(list(out.glob("*.stderr.log"))))
            payload = json.loads((out / "garnet-benchmark-no-run.json").read_text(encoding="utf-8"))
            self.assertEqual("compile-verified", payload["overall_status"])

    def test_failing_execute_marks_failed(self) -> None:
        status = mod._collect_status(
            execute=True,
            run_command=_runner_fail,
        )
        self.assertEqual("failed", status.overall_status)
        self.assertEqual("failed", status.benchmarks[0].status)


if __name__ == "__main__":
    unittest.main()
