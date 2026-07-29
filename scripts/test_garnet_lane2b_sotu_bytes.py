#!/usr/bin/env python3
"""Regression tests for Lane 2B Mission Control LF-only generation."""
from __future__ import annotations

import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RENDERER = ROOT / "ops" / "lane2b" / "render-sotu.mjs"


class Lane2bSotuByteTests(unittest.TestCase):
    def _fixture(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        tmp = tempfile.TemporaryDirectory()
        root = Path(tmp.name)
        shutil.copy2(RENDERER, root / "render-sotu.mjs")
        state = {
            "mission": {"name": "fixture", "status": "active"},
            "phases": [],
            "metrics": [],
            "risks": [],
            "prLog": [],
            "resume": {},
            "policies": {},
        }
        (root / "state.json").write_text(
            json.dumps(state, indent=2) + "\n",
            encoding="utf-8",
            newline="\n",
        )
        return tmp, root

    def _render(self, root: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["node", "render-sotu.mjs"],
            cwd=root,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

    def test_clean_inputs_emit_lf_only_html(self) -> None:
        tmp, root = self._fixture()
        self.addCleanup(tmp.cleanup)
        (root / "journal.md").write_text(
            "# Journal\n\n- clean\n",
            encoding="utf-8",
            newline="\n",
        )
        result = self._render(root)
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertNotIn(b"\r", (root / "state-of-the-union.html").read_bytes())

    def test_cr_bearing_state_fails_closed(self) -> None:
        tmp, root = self._fixture()
        self.addCleanup(tmp.cleanup)
        state = (root / "state.json").read_bytes().replace(b"\n", b"\r\n")
        (root / "state.json").write_bytes(state)
        result = self._render(root)
        self.assertNotEqual(0, result.returncode)
        self.assertIn("state.json contains carriage-return bytes", result.stderr)
        self.assertFalse((root / "state-of-the-union.html").exists())

    def test_cr_bearing_journal_fails_closed(self) -> None:
        tmp, root = self._fixture()
        self.addCleanup(tmp.cleanup)
        (root / "journal.md").write_bytes(b"# Journal\r\n\r\n- captured\r\n")
        result = self._render(root)
        self.assertNotEqual(0, result.returncode)
        self.assertIn("journal.md contains carriage-return bytes", result.stderr)
        self.assertFalse((root / "state-of-the-union.html").exists())

    def test_tracked_lane2b_sources_and_output_are_lf_only(self) -> None:
        for relative in (
            "ops/lane2b/render-sotu.mjs",
            "ops/lane2b/state.json",
            "ops/lane2b/journal.md",
            "ops/lane2b/state-of-the-union.html",
        ):
            with self.subTest(path=relative):
                self.assertNotIn(b"\r", (ROOT / relative).read_bytes())


if __name__ == "__main__":
    unittest.main()
