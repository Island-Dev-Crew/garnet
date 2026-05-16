#!/usr/bin/env python3
"""Regression tests for the converter advisory review gate."""
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

BUNDLE_SCRIPT = Path(__file__).with_name("garnet_converter_advisory_bundle.py")
REVIEW_SCRIPT = Path(__file__).with_name("garnet_converter_advisory_review.py")


class GarnetConverterAdvisoryReviewTests(unittest.TestCase):
    def _write_typescript_fixture(self, directory: Path) -> Path:
        source = directory / "agent_router.ts"
        source.write_text(
            """\
export async function route(endpoint: string): Promise<string> {
  const response = await fetch(endpoint);
  return await response.text();
}
""",
            encoding="utf-8",
        )
        return source

    def _write_bundle(self, directory: Path, *, include_source: bool = False) -> Path:
        source = self._write_typescript_fixture(directory)
        bundle = directory / "advisory-bundle"
        command = [
            sys.executable,
            str(BUNDLE_SCRIPT),
            "--language",
            "typescript",
            "--source",
            str(source),
            "--output-dir",
            str(bundle),
        ]
        if include_source:
            command.append("--include-source")
        subprocess.check_call(command, stdout=subprocess.DEVNULL)
        return bundle

    def test_review_json_accepts_manifested_no_source_bundle(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = self._write_bundle(Path(temp))
            output = subprocess.check_output(
                [
                    sys.executable,
                    str(REVIEW_SCRIPT),
                    "--bundle-dir",
                    str(bundle),
                    "--format",
                    "json",
                ],
                text=True,
            )

        data = json.loads(output)

        self.assertEqual("ready-for-human-advisory-review", data["status"])
        self.assertTrue(data["manifest_verified"])
        self.assertTrue(data["source_privacy_passed"])
        self.assertTrue(data["provider_boundary_passed"])
        self.assertTrue(data["conversion_boundary_passed"])
        self.assertTrue(data["human_review_required"])
        self.assertFalse(data["provider_backed_conversion_allowed"])
        self.assertIn("run garnet check on any candidate output", data["required_before_unquarantine"])
        self.assertIn("attach dogfood evidence", data["required_before_unquarantine"])

    def test_review_markdown_keeps_provider_conversion_blocked(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = self._write_bundle(Path(temp))
            rendered = subprocess.check_output(
                [sys.executable, str(REVIEW_SCRIPT), "--bundle-dir", str(bundle)],
                text=True,
            )

        self.assertIn("Garnet Converter Advisory Review Gate", rendered)
        self.assertIn("ready-for-human-advisory-review", rendered)
        self.assertIn("Provider-backed conversion allowed: false", rendered)
        self.assertIn("Source privacy passed: true", rendered)
        self.assertIn("human audit before unquarantine", rendered)

    def test_review_blocks_source_included_bundle_without_explicit_approval(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = self._write_bundle(Path(temp), include_source=True)
            result = subprocess.run(
                [
                    sys.executable,
                    str(REVIEW_SCRIPT),
                    "--bundle-dir",
                    str(bundle),
                    "--format",
                    "json",
                ],
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )

        self.assertEqual(1, result.returncode)
        data = json.loads(result.stdout)
        self.assertEqual("blocked-source-included", data["status"])
        self.assertFalse(data["source_privacy_passed"])
        self.assertIn("source text included", data["blockers"])

    def test_output_dir_writes_verified_review_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            bundle = self._write_bundle(root)
            output_dir = root / "review"
            subprocess.check_call(
                [
                    sys.executable,
                    str(REVIEW_SCRIPT),
                    "--bundle-dir",
                    str(bundle),
                    "--output-dir",
                    str(output_dir),
                ],
                stdout=subprocess.DEVNULL,
            )

            self.assertTrue((output_dir / "garnet-converter-advisory-review.json").exists())
            self.assertTrue((output_dir / "garnet-converter-advisory-review.md").exists())
            verify = (output_dir / "MANIFEST.verify.log").read_text(encoding="utf-8")
            self.assertIn("garnet-converter-advisory-review.json: OK", verify)
            self.assertIn("garnet-converter-advisory-review.md: OK", verify)


if __name__ == "__main__":
    unittest.main()
