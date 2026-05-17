#!/usr/bin/env python3
"""Regression tests for provider-neutral converter advisory handoff packets."""
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

BUNDLE_SCRIPT = Path(__file__).with_name("garnet_converter_advisory_bundle.py")
REVIEW_SCRIPT = Path(__file__).with_name("garnet_converter_advisory_review.py")
HANDOFF_SCRIPT = Path(__file__).with_name("garnet_converter_advisory_handoff.py")


class GarnetConverterAdvisoryHandoffTests(unittest.TestCase):
    def _source(self, directory: Path) -> Path:
        source = directory / "router.ts"
        source.write_text(
            "\n".join(
                [
                    "export class AgentRouter {",
                    "  async route(task: string) {",
                    "    const response = await fetch('/api/agent', { method: 'POST' });",
                    "    return { task, response };",
                    "  }",
                    "}",
                    "",
                ]
            ),
            encoding="utf-8",
        )
        return source

    def _bundle(self, directory: Path, *, include_source: bool = False) -> Path:
        bundle = directory / "advisory-bundle"
        command = [
            sys.executable,
            str(BUNDLE_SCRIPT),
            "--language",
            "typescript",
            "--source",
            str(self._source(directory)),
            "--output-dir",
            str(bundle),
        ]
        if include_source:
            command.insert(-2, "--include-source")
        subprocess.run(command, check=True, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        return bundle

    def _review(self, bundle: Path, directory: Path) -> Path:
        review = directory / "advisory-review"
        subprocess.run(
            [
                sys.executable,
                str(REVIEW_SCRIPT),
                "--bundle-dir",
                str(bundle),
                "--output-dir",
                str(review),
            ],
            check=False,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        return review

    def test_handoff_json_promotes_reviewed_no_source_bundle_only(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            bundle = self._bundle(root)
            review = self._review(bundle, root)

            result = subprocess.run(
                [
                    sys.executable,
                    str(HANDOFF_SCRIPT),
                    "--bundle-dir",
                    str(bundle),
                    "--review-dir",
                    str(review),
                    "--format",
                    "json",
                ],
                check=True,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )

        data = json.loads(result.stdout)
        self.assertEqual("ready-for-provider-neutral-advisory-handoff", data["status"])
        self.assertEqual("ready-for-human-advisory-review", data["review_status"])
        self.assertFalse(data["source_included"])
        self.assertFalse(data["provider_backed_conversion_allowed"])
        self.assertFalse(data["conversion_active"])
        self.assertIn("provider-neutral advisory notes only", data["allowed_handoff_use"])
        self.assertIn("lineage per emitted node", data["required_before_model_or_agent"])
        self.assertIn("Do not claim autonomous conversion", data["handoff_prompt"])
        self.assertNotIn("AgentRouter", data["handoff_prompt"])

    def test_handoff_blocks_source_included_review_without_leaking_source(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            bundle = self._bundle(root, include_source=True)
            review = self._review(bundle, root)

            result = subprocess.run(
                [
                    sys.executable,
                    str(HANDOFF_SCRIPT),
                    "--bundle-dir",
                    str(bundle),
                    "--review-dir",
                    str(review),
                    "--format",
                    "json",
                ],
                check=False,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )

        self.assertEqual(1, result.returncode)
        data = json.loads(result.stdout)
        self.assertEqual("blocked-advisory-handoff", data["status"])
        self.assertTrue(data["source_included"])
        self.assertIn("source text included", data["blockers"])
        self.assertNotIn("AgentRouter", data["handoff_prompt"])

    def test_output_dir_writes_verified_manifested_handoff(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            bundle = self._bundle(root)
            review = self._review(bundle, root)
            output_dir = root / "advisory-handoff"

            result = subprocess.run(
                [
                    sys.executable,
                    str(HANDOFF_SCRIPT),
                    "--bundle-dir",
                    str(bundle),
                    "--review-dir",
                    str(review),
                    "--output-dir",
                    str(output_dir),
                ],
                check=True,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )

            self.assertIn("Garnet Converter Advisory Handoff Packet", result.stdout)
            self.assertTrue((output_dir / "garnet-converter-advisory-handoff.json").exists())
            self.assertTrue((output_dir / "garnet-converter-advisory-handoff.md").exists())
            verify = (output_dir / "MANIFEST.verify.log").read_text(encoding="utf-8")
            self.assertIn("garnet-converter-advisory-handoff.json: OK", verify)
            self.assertIn("garnet-converter-advisory-handoff.md: OK", verify)


if __name__ == "__main__":
    unittest.main()
