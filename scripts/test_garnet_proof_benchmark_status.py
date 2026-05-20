#!/usr/bin/env python3
"""Regression tests for proof/benchmark/empirical status reporting."""
from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_proof_benchmark_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_proof_benchmark_status", SCRIPT)
assert SPEC is not None
status_mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_proof_benchmark_status"] = status_mod
SPEC.loader.exec_module(status_mod)


class GarnetProofBenchmarkStatusTests(unittest.TestCase):
    def test_status_inventories_current_benchmark_harnesses_without_claiming_results(self) -> None:
        status = status_mod.read_status()
        benches = {bench.id: bench for bench in status.benchmarks}

        self.assertEqual("active-scaffold", status.overall_status)
        self.assertEqual("not-run", status.measurement_status)
        self.assertEqual("harness-present-nightly-scheduled", status.fuzzing_status)
        self.assertEqual("not-mechanized", status.mechanized_proof_status)
        self.assertEqual("pending", status.empirical_study_status)
        self.assertEqual("parser_parse", benches["parser_parse"].id)
        self.assertEqual("garnet-parser", benches["parser_parse"].package)
        self.assertTrue(benches["parser_parse"].bench_file_exists)
        self.assertTrue(benches["parser_parse"].cargo_entry_present)
        self.assertIn("cargo bench -p garnet-parser --bench parse", benches["parser_parse"].command)
        self.assertIn("vm_parse_compile_execute", benches)
        self.assertEqual("garnet-vm", benches["vm_parse_compile_execute"].package)
        self.assertTrue(benches["vm_parse_compile_execute"].bench_file_exists)
        self.assertTrue(benches["vm_parse_compile_execute"].cargo_entry_present)
        fuzz = {harness.id: harness for harness in status.fuzz_harnesses}
        self.assertIn("parser_parse_input", fuzz)
        self.assertEqual("garnet-parser", fuzz["parser_parse_input"].package)
        self.assertEqual("parse_input", fuzz["parser_parse_input"].target_name)
        self.assertTrue(fuzz["parser_parse_input"].target_file_exists)
        self.assertTrue(fuzz["parser_parse_input"].cargo_entry_present)
        self.assertGreaterEqual(fuzz["parser_parse_input"].corpus_seed_count, 1)
        self.assertTrue(fuzz["parser_parse_input"].nightly_workflow_exists)
        self.assertIn("production native compiler", status.forbidden_claims[0])

    def test_json_preserves_blocked_and_deferred_research_boundaries(self) -> None:
        output = subprocess.check_output(
            [sys.executable, str(SCRIPT), "--format", "json"],
            text=True,
        )
        data = json.loads(output)

        self.assertEqual("active-scaffold", data["overall_status"])
        self.assertEqual("harness-present-nightly-scheduled", data["fuzzing_status"])
        self.assertIn("benchmarks compile/execution must be run separately", data["current_truth"])
        self.assertIn(
            "S5 adds a parser fuzz harness and nightly workflow without claiming accumulated fuzz hours yet.",
            data["current_truth"],
        )
        self.assertIn("mechanized proof is not present", data["blocked_by"])
        self.assertIn("nightly fuzz hours require GitHub Actions evidence after merge", data["blocked_by"])
        self.assertIn("external user study data", data["blocked_by"])
        self.assertIn("formal RustBelt/Iris/Coq mechanization", data["deferred"])
        self.assertIn("accumulated nightly fuzz hours", data["deferred"])
        self.assertIn("benchmark measurement run", data["deferred"])
        self.assertEqual(4, len(data["benchmarks"]))
        self.assertEqual(1, len(data["fuzz_harnesses"]))

    def test_markdown_is_reviewer_safe(self) -> None:
        rendered = subprocess.check_output([sys.executable, str(SCRIPT)], text=True)

        self.assertIn("Garnet Proof, Benchmark, And Empirical Status", rendered)
        self.assertIn("Criterion Benchmark Harnesses", rendered)
        self.assertIn("Fuzz Harnesses", rendered)
        self.assertIn("Not Claimed", rendered)
        self.assertIn("not production native compiler proof", rendered)
        self.assertIn("cargo bench -p garnet-memory --bench vector", rendered)
        self.assertIn("cargo bench -p garnet-vm --bench parse_compile_execute", rendered)
        self.assertIn("cargo +nightly fuzz run parse_input", rendered)

    def test_output_dir_writes_manifested_evidence_bundle(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            out = Path(temp) / "proof-benchmark-status"
            subprocess.check_call(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--format",
                    "json",
                    "--output-dir",
                    str(out),
                ],
                stdout=subprocess.DEVNULL,
            )

            self.assertTrue((out / "garnet-proof-benchmark-status.json").is_file())
            self.assertTrue((out / "garnet-proof-benchmark-status.md").is_file())
            self.assertTrue((out / "MANIFEST.sha256").is_file())
            subprocess.check_call(["shasum", "-a", "256", "-c", "MANIFEST.sha256"], cwd=out)


if __name__ == "__main__":
    unittest.main()
