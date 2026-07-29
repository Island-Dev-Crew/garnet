#!/usr/bin/env python3
"""Tests for the enumerating CR-bearing text policy gate."""
from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from dataclasses import asdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PATH = Path(__file__).with_name("garnet_text_byte_policy_status.py")

spec = importlib.util.spec_from_file_location("garnet_text_byte_policy_status", PATH)
if spec is None or spec.loader is None:
    raise RuntimeError("could not load garnet_text_byte_policy_status")
mod = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = mod
spec.loader.exec_module(mod)


class TextBytePolicyStatusTests(unittest.TestCase):
    def _git(self, root: Path, *args: str) -> str:
        return subprocess.run(
            ["git", *args],
            cwd=root,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,
        ).stdout.strip()

    def _repo(self, files: dict[str, bytes]) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        tmp = tempfile.TemporaryDirectory()
        root = Path(tmp.name)
        self._git(root, "init", "-q")
        self._git(root, "config", "user.name", "Test")
        self._git(root, "config", "user.email", "test@example.invalid")
        self._git(root, "config", "core.autocrlf", "false")
        for name, payload in files.items():
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(payload)
        self._git(root, "add", ".")
        self._git(root, "commit", "-q", "-m", "fixture")
        return tmp, root

    def test_real_tree_has_no_unexcluded_cr_bearing_text(self) -> None:
        status = mod.read_status(root=ROOT, ref="HEAD")
        self.assertTrue(status.ok, status)
        self.assertEqual([], status.violations)

    def test_names_every_unexcluded_path_without_a_count_contract(self) -> None:
        tmp, root = self._repo(
            {
                "alpha.txt": b"alpha\r\n",
                "nested/bravo.txt": b"bravo\r",
            }
        )
        self.addCleanup(tmp.cleanup)
        status = mod.read_status(root=root, ref="HEAD")
        self.assertFalse(status.ok)
        self.assertEqual(["alpha.txt", "nested/bravo.txt"], status.violations)
        payload = asdict(status)
        self.assertFalse(any("count" in key.casefold() for key in payload))
        self.assertEqual(
            ["alpha.txt", "nested/bravo.txt"],
            json.loads(mod.render_json(status))["violations"],
        )

    def test_proofs_and_ops_evidence_are_excluded_but_other_ops_are_not(self) -> None:
        tmp, root = self._repo(
            {
                "proofs/sealed.txt": b"sealed\r\n",
                "ops/lane/evidence/run.txt": b"evidence\r\n",
                "ops/lane/state.html": b"state\r",
            }
        )
        self.addCleanup(tmp.cleanup)
        status = mod.read_status(root=root, ref="HEAD")
        self.assertEqual(["ops/lane/state.html"], status.violations)

    def test_binary_blob_is_not_classified_as_text(self) -> None:
        tmp, root = self._repo({"asset.bin": b"\x00binary\r\npayload"})
        self.addCleanup(tmp.cleanup)
        status = mod.read_status(root=root, ref="HEAD")
        self.assertTrue(status.ok, status)

    def test_missing_ref_fails_closed(self) -> None:
        tmp, root = self._repo({"clean.txt": b"clean\n"})
        self.addCleanup(tmp.cleanup)
        status = mod.read_status(root=root, ref="refs/heads/missing")
        self.assertFalse(status.ok)
        self.assertTrue(status.problems)
        self.assertEqual([], status.violations)

    def test_cured_product_text_paths_are_explicitly_pinned_to_lf(self) -> None:
        paths = [
            ".dogfood/windows-audit-goal.json",
            ".dogfood/windows-core-audit.json",
            "D_Executive_and_Presentation/garnet-website.html",
            "ops/lane2b/state-of-the-union.html",
        ]
        result = subprocess.run(
            ["git", "check-attr", "text", "eol", "--", *paths],
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,
        )
        for path in paths:
            self.assertIn(f"{path}: text: set", result.stdout)
            self.assertIn(f"{path}: eol: lf", result.stdout)


if __name__ == "__main__":
    unittest.main()
