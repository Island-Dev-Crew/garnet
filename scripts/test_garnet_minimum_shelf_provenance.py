#!/usr/bin/env python3
"""Adversarial tests for Lane 2B squash-durable content provenance."""
from __future__ import annotations

import importlib.util
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


def _load(name: str, filename: str):
    script = Path(__file__).with_name(filename)
    spec = importlib.util.spec_from_file_location(name, script)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


shelf = _load("smoke_garnet_minimum_shelf", "smoke_garnet_minimum_shelf.py")
wv = _load("garnet_wv_acceptance_status", "garnet_wv_acceptance_status.py")


class SquashDurableContentProvenanceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self._git("init")
        self._git("config", "user.email", "lane2b@example.invalid")
        self._git("config", "user.name", "Lane 2B Test")
        self._write("product.txt", "reviewed product\n")
        self._write("ops/lane2b/note.txt", "mutable evidence\n")
        self._write("proofs/lane2b/proof.txt", "mutable proof\n")
        self._write(
            "F_Project_Management/W_TRUST/review.md", "mutable companion\n"
        )
        self._write(
            "scripts/smoke_garnet_minimum_shelf.py", "self excluded\n"
        )
        self._git("add", ".")
        self._git("commit", "-m", "synthetic squash result")
        self._git("branch", "-M", "main")
        self.landed = self._git("rev-parse", "HEAD")
        self._git("update-ref", "refs/remotes/origin/main", self.landed)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def _write(self, relative: str, text: str) -> None:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def _git(self, *args: str, check: bool = True) -> str:
        result = subprocess.run(
            ["git", "--no-replace-objects", *args],
            cwd=self.root,
            capture_output=True,
            text=True,
            check=False,
        )
        if check and result.returncode != 0:
            self.fail(f"git {' '.join(args)} failed: {result.stderr or result.stdout}")
        return result.stdout.strip()

    def test_any_product_blob_change_is_red(self) -> None:
        expected, _ = shelf._tracked_content_digest(self.root)
        self._write("product.txt", "tampered product\n")
        self._git("add", "product.txt")
        findings = shelf._verify_product_content(self.root, expected)
        self.assertTrue(
            any("product content digest" in item for item in findings), findings
        )

    def test_absent_branch_commits_are_green_on_fresh_main_content(self) -> None:
        for discarded in (
            "a6f0da2b81a9b181dafb83e15a17f8f313406e49",
            "e2820ce54e9c1fee030d50e9fba31be4bcdc8891",
        ):
            self.assertNotEqual(
                0,
                subprocess.run(
                    ["git", "cat-file", "-e", f"{discarded}^{{commit}}"],
                    cwd=self.root,
                    capture_output=True,
                    check=False,
                ).returncode,
            )
        expected, _ = shelf._tracked_content_digest(self.root)
        findings, landed = wv._verify_squash_durable_content(
            self.root,
            reviewed_head="a" * 40,
            reviewed_tree="b" * 40,
            expected_content_digest=expected,
            verify_git=True,
        )
        self.assertEqual([], findings)
        self.assertEqual(self.landed, landed)

    def test_mismatched_evidence_content_digest_is_red(self) -> None:
        findings, _ = wv._verify_squash_durable_content(
            self.root,
            reviewed_head="a" * 40,
            reviewed_tree="b" * 40,
            expected_content_digest="0" * 64,
            verify_git=True,
        )
        self.assertTrue(
            any("product content digest" in item for item in findings), findings
        )


if __name__ == "__main__":
    unittest.main()
