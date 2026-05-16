#!/usr/bin/env python3
"""Regression tests for provider-neutral converter advisory bundles."""
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_converter_advisory_bundle.py")


class GarnetConverterAdvisoryBundleTests(unittest.TestCase):
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

    def test_json_bundles_context_plan_and_feasibility_without_source_by_default(self) -> None:
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

        self.assertEqual("active-advisory-bundle", data["status"])
        self.assertEqual("TypeScript", data["language"])
        self.assertFalse(data["conversion_active"])
        self.assertFalse(data["provider_required"])
        self.assertFalse(data["model_required"])
        self.assertFalse(data["network_required"])
        self.assertFalse(data["enabled_by_default"])
        self.assertFalse(data["source_included"])
        self.assertIsNone(data["source_text"])
        self.assertTrue(data["human_review_required"])
        self.assertEqual("active-context-pack", data["context_pack"]["status"])
        self.assertEqual("active-assist-plan", data["assist_plan"]["status"])
        self.assertEqual("advisory-feasible", data["llm_feasibility"]["status"])
        self.assertIn("provider-neutral advisory planning", data["recommended_first_lane"])
        self.assertIn("assist context pack JSON", data["required_inputs"])
        self.assertIn("assist plan JSON", data["required_inputs"])
        self.assertIn("converter LLM feasibility JSON", data["required_inputs"])
        self.assertIn("source file sha256", data["required_inputs"])

    def test_include_source_requires_explicit_flag_and_keeps_privacy_warning(self) -> None:
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
                    "--include-source",
                    "--format",
                    "json",
                ],
                text=True,
            )

        data = json.loads(output)

        self.assertTrue(data["source_included"])
        self.assertIn("AgentRouter", data["source_text"])
        self.assertIn("local or explicitly approved provider handoff", data["source_privacy_mode"])
        self.assertEqual(
            data["assist_plan"]["source_summary"]["sha256"],
            data["source_summary"]["sha256"],
        )

    def test_markdown_request_is_honest_about_non_conversion_boundary(self) -> None:
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

        self.assertIn("Garnet Converter Advisory Bundle", rendered)
        self.assertIn("not active conversion", rendered)
        self.assertIn("Source text included: false", rendered)
        self.assertIn("provider-neutral advisory planning", rendered)
        self.assertIn("human audit before unquarantine", rendered)
        self.assertNotIn("autonomous conversion is enabled", rendered.lower())

    def test_output_dir_writes_verified_manifested_bundle(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            source = self._write_typescript_fixture(root)
            out_dir = root / "advisory-bundle"
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

            self.assertTrue((out_dir / "garnet-converter-advisory-bundle.json").exists())
            self.assertTrue((out_dir / "garnet-converter-advisory-bundle.md").exists())
            self.assertTrue((out_dir / "garnet-converter-advisory-request.md").exists())
            self.assertTrue((out_dir / "MANIFEST.sha256").exists())
            verify_log = out_dir / "MANIFEST.verify.log"
            self.assertTrue(verify_log.exists())
            verify = verify_log.read_text(encoding="utf-8")
            self.assertIn("garnet-converter-advisory-bundle.json: OK", verify)
            self.assertIn("garnet-converter-advisory-bundle.md: OK", verify)
            self.assertIn("garnet-converter-advisory-request.md: OK", verify)

    def test_unknown_language_fails_without_creating_bundle(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            source = Path(temp) / "agent.elm"
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


if __name__ == "__main__":
    unittest.main()
