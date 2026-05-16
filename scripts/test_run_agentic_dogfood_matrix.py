#!/usr/bin/env python3
"""Regression tests for the agentic dogfood matrix inventory."""
from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from collections import Counter
from pathlib import Path

SCRIPT = Path(__file__).with_name("run_agentic_dogfood_matrix.py")
SPEC = importlib.util.spec_from_file_location("run_agentic_dogfood_matrix", SCRIPT)
assert SPEC is not None
matrix = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["run_agentic_dogfood_matrix"] = matrix
SPEC.loader.exec_module(matrix)


class AgenticDogfoodMatrixTests(unittest.TestCase):
    def _fake_result(self, probe: object) -> object:
        return matrix.ProbeResult(
            probe=probe,
            status="passed",
            exit_code=0,
            duration_ms=1,
            stdout_log="/tmp/stdout.log",
            stderr_log="/tmp/stderr.log",
            stdout_excerpt="",
            stderr_excerpt="",
        )

    def test_probe_inventory_includes_agent_recovery_domain(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/bin/garnet"), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["agent recovery and diagnostics"], 4)
        self.assertIn("check-malformed-agent-source", ids)
        self.assertIn("check-missing-agent-source", ids)
        self.assertIn("eval-unknown-agent-symbol", ids)
        self.assertIn("verify-missing-release-manifest", ids)

    def test_probe_inventory_includes_web_pwa_offline_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/bin/garnet"), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]

        ids = {probe.id for probe in concrete_probes}
        domains = Counter(probe.domain for probe in concrete_probes)

        self.assertEqual(domains["web/PWA productization"], 2)
        self.assertIn("smoke-web-pwa-offline-handler", ids)
        self.assertIn("smoke-web-pwa-local-readiness", ids)

    def test_domain_coverage_marks_undercovered_agentic_surfaces(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            fixtures = matrix.prepare_fixtures(work)
            probes = matrix.probe_set(Path("/bin/garnet"), work, fixtures, include_app_workbench=False)
            concrete_probes = [probe for probe in probes if isinstance(probe, matrix.Probe)]
            results = [self._fake_result(probe) for probe in concrete_probes]

        coverage = {item["domain"]: item for item in matrix.domain_coverage(results)}

        self.assertEqual(coverage["web/PWA productization"]["probe_count"], 2)
        self.assertEqual(coverage["web/PWA productization"]["target_probe_count"], 3)
        self.assertEqual(coverage["web/PWA productization"]["status"], "needs-expansion")
        self.assertEqual(coverage["agent recovery and diagnostics"]["status"], "adequate")

    def test_write_outputs_persists_domain_coverage(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            probe = matrix.Probe(
                "one-web-probe",
                "web/PWA productization",
                "one probe should still expose coverage debt",
                ["/bin/true"],
                True,
            )
            result = self._fake_result(probe)
            matrix.write_outputs(
                work,
                [result],
                {
                    "repo": "/tmp/repo",
                    "head": "abc123",
                    "branch": "test",
                    "garnet": "/bin/garnet",
                    "app_workbench": "skipped",
                    "artifact_dir": str(work),
                },
            )

            data = (work / "dogfood-readiness-data.json").read_text(encoding="utf-8")
            report = (work / "dogfood-readiness-report.md").read_text(encoding="utf-8")

        self.assertIn('"domain_coverage"', data)
        self.assertIn('"needs-expansion"', data)
        self.assertIn("## Domain Coverage Adequacy", report)
        self.assertIn("web/PWA productization", report)


if __name__ == "__main__":
    unittest.main()
