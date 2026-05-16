#!/usr/bin/env python3
"""Regression tests for deterministic planned-language converter assist plans."""
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_converter_assist_plan.py")


class GarnetConverterAssistPlanTests(unittest.TestCase):
    def _write_typescript_fixture(self, directory: Path) -> Path:
        source = directory / "agent_router.ts"
        source.write_text(
            """\
export class AgentRouter {
  private cache = new Map<string, string>();

  async route(endpoint: string): Promise<string> {
    const prior = this.cache.get(endpoint);
    if (prior) {
      return prior;
    }
    const response = await fetch(endpoint);
    const body = await response.text();
    this.cache.set(endpoint, body);
    return body;
  }
}
""",
            encoding="utf-8",
        )
        return source

    def _write_java_fixture(self, directory: Path) -> Path:
        source = directory / "AgentGateway.java"
        source.write_text(
            """\
import java.net.Socket;
import java.util.concurrent.CompletableFuture;

interface AgentGateway {
  CompletableFuture<String> route(Socket socket);
}
""",
            encoding="utf-8",
        )
        return source

    def test_planned_language_plan_is_advisory_and_gated(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            source = self._write_typescript_fixture(Path(temp))
            output = subprocess.check_output(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--language",
                    "typescript",
                    "--source",
                    str(source),
                    "--format",
                    "json",
                ],
                text=True,
            )

        data = json.loads(output)

        self.assertEqual("TypeScript", data["language"])
        self.assertEqual("planned", data["language_status"])
        self.assertFalse(data["conversion_active"])
        self.assertFalse(data["provider_required"])
        self.assertFalse(data["model_required"])
        self.assertFalse(data["network_required"])
        self.assertFalse(data["source_execution_allowed"])
        self.assertTrue(data["sandbox_default"])
        self.assertFalse(data["deterministic_converter_available"])

        self.assertIn("CapCaps/capability boundaries", data["analysis_targets"])
        self.assertIn("actor/orchestration mappings", data["analysis_targets"])
        self.assertIn("memory declarations", data["analysis_targets"])
        self.assertIn("lineage per emitted node", data["required_gates"])
        self.assertIn("@sandbox default", data["required_gates"])
        self.assertIn("garnet check", data["required_gates"])

        risk_titles = {risk["title"] for risk in data["risk_inventory"]}
        self.assertGreaterEqual(
            risk_titles,
            {
                "network or external capability boundary",
                "actor or async orchestration mapping",
                "type and ownership modeling",
                "memory declaration candidate",
            },
        )

    def test_active_language_plan_defers_to_deterministic_converter(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            source = Path(temp) / "route.py"
            source.write_text("def route(value):\n    return value + 1\n", encoding="utf-8")
            output = subprocess.check_output(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--language",
                    "python",
                    "--source",
                    str(source),
                    "--format",
                    "json",
                ],
                text=True,
            )

        data = json.loads(output)

        self.assertEqual("Python", data["language"])
        self.assertEqual("active", data["language_status"])
        self.assertTrue(data["deterministic_converter_available"])
        self.assertFalse(data["conversion_active"])
        self.assertIn(
            "deterministic converter output remains authoritative",
            data["current_truth"],
        )
        self.assertIn("Use `garnet convert python", "\n".join(data["next_steps"]))

    def test_java_completable_future_surfaces_orchestration_risk(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            source = self._write_java_fixture(Path(temp))
            output = subprocess.check_output(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--language",
                    "java",
                    "--source",
                    str(source),
                    "--format",
                    "json",
                ],
                text=True,
            )

        data = json.loads(output)
        risk_titles = {risk["title"] for risk in data["risk_inventory"]}

        self.assertEqual("Java", data["language"])
        self.assertEqual("planned", data["language_status"])
        self.assertFalse(data["conversion_active"])
        self.assertIn("actor or async orchestration mapping", risk_titles)
        self.assertIn("network or external capability boundary", risk_titles)

    def test_unknown_language_fails_loudly(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            source = Path(temp) / "unknown.elm"
            source.write_text("main = 1\n", encoding="utf-8")
            result = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--language",
                    "elm",
                    "--source",
                    str(source),
                ],
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )

        self.assertNotEqual(0, result.returncode)
        self.assertIn("unknown source language", result.stderr)
        self.assertIn("typescript", result.stderr)

    def test_output_dir_writes_manifested_plan(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            source = self._write_typescript_fixture(root)
            out_dir = root / "assist-plan"
            subprocess.check_call(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--language",
                    "typescript",
                    "--source",
                    str(source),
                    "--output-dir",
                    str(out_dir),
                ],
                stdout=subprocess.DEVNULL,
            )

            self.assertTrue((out_dir / "garnet-converter-assist-plan.json").exists())
            self.assertTrue((out_dir / "garnet-converter-assist-plan.md").exists())
            manifest = out_dir / "MANIFEST.sha256"
            self.assertTrue(manifest.exists())
            verify = subprocess.check_output(
                ["shasum", "-a", "256", "-c", manifest.name],
                cwd=out_dir,
                text=True,
            )

        self.assertIn("garnet-converter-assist-plan.json: OK", verify)
        self.assertIn("garnet-converter-assist-plan.md: OK", verify)

    def test_markdown_is_honest_about_non_conversion(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            source = self._write_typescript_fixture(Path(temp))
            rendered = subprocess.check_output(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--language",
                    "typescript",
                    "--source",
                    str(source),
                ],
                text=True,
            )

        self.assertIn("Garnet Converter Assist Plan", rendered)
        self.assertIn("planned", rendered)
        self.assertIn("not active conversion", rendered)
        self.assertIn("network or external capability boundary", rendered)
        self.assertIn("lineage per emitted node", rendered)
        self.assertIn("human audit before unquarantine", rendered)


if __name__ == "__main__":
    unittest.main()
