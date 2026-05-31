#!/usr/bin/env python3
"""Tests for the idiomatic Garnet corpus harness (S57)."""
from __future__ import annotations

import importlib.util
import shutil
import subprocess
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_idiomatic_corpus.py")
SPEC = importlib.util.spec_from_file_location("garnet_idiomatic_corpus", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
corpus = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_idiomatic_corpus"] = corpus
SPEC.loader.exec_module(corpus)

ROOT = Path(__file__).resolve().parents[1]


def _garnet() -> Path | None:
    exe = "garnet.exe" if sys.platform.startswith("win") else "garnet"
    found = [ROOT / "target" / p / exe for p in ("release", "debug")]
    found = [p for p in found if p.exists()]
    return max(found, key=lambda p: p.stat().st_mtime) if found else None


class PureLogicTests(unittest.TestCase):
    def test_corpus_files_exist_and_are_idiomatic_dir(self) -> None:
        self.assertTrue(corpus.CORPUS)
        for c in corpus.CORPUS:
            self.assertTrue((ROOT / c.file).exists(), c.file)
            self.assertIn("examples/idiomatic/", c.file)

    def test_pass_requires_clean_check_and_expected_run(self) -> None:
        c = corpus.CORPUS[0]
        r = corpus.evaluate_case(c, 0, "2 functions checked, 0 diagnostics", 0, "handled\nfound: y")
        self.assertTrue(r.passed)

    def test_check_with_diagnostics_is_not_idiomatic(self) -> None:
        c = corpus.CORPUS[0]
        # A non-zero diagnostic count fails the idiomatic bar even if it ran.
        r = corpus.evaluate_case(c, 0, "1 diagnostics", 0, "handled\nfound: y")
        self.assertFalse(r.check_clean)
        self.assertFalse(r.passed)

    def test_wrong_run_output_fails(self) -> None:
        c = corpus.CORPUS[0]
        r = corpus.evaluate_case(c, 0, "0 diagnostics", 0, "nope")
        self.assertFalse(r.run_ok)


@unittest.skipUnless(_garnet() is not None, "garnet CLI not built")
class LiveCorpusTests(unittest.TestCase):
    def test_all_idiomatic_programs_clean_and_run(self) -> None:
        proc = subprocess.run(
            [sys.executable, str(SCRIPT), "--format", "json"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)


if __name__ == "__main__":
    unittest.main()
