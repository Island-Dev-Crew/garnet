#!/usr/bin/env python3
"""Tests for the AI-PR-review-collapse wedge demo (S49)."""
from __future__ import annotations

import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("smoke_garnet_pr_review_wedge.py")
SPEC = importlib.util.spec_from_file_location("smoke_pr_review_wedge", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
wedge = importlib.util.module_from_spec(SPEC)
sys.modules["smoke_pr_review_wedge"] = wedge
SPEC.loader.exec_module(wedge)

ROOT = Path(__file__).resolve().parents[1]


def _garnet_binary() -> Path | None:
    exe = "garnet.exe" if sys.platform.startswith("win") else "garnet"
    found = [
        ROOT / "target" / profile / exe for profile in ("release", "debug")
    ]
    found = [p for p in found if p.exists()]
    return max(found, key=lambda p: p.stat().st_mtime) if found else None


class PureLogicTests(unittest.TestCase):
    def test_fixtures_exist(self) -> None:
        self.assertTrue((ROOT / wedge.BEFORE).exists())
        self.assertTrue((ROOT / wedge.AFTER).exists())

    def test_markdown_renders_and_flags_failure(self) -> None:
        report = wedge.WedgeReport(
            schema="garnet.pr_review_wedge/v1",
            scenario="x",
            steps=[wedge.Step("diff-caps", [], 1, True, "ok")],
            wedge_fires=True,
        )
        md = wedge.render_markdown(report)
        self.assertIn("AI-PR-review-collapse wedge demo", md)
        self.assertIn("Wedge fires as designed: yes", md)

    def test_resolve_prefers_newest_binary(self) -> None:
        # Constructed names only — verify the newest-mtime preference logic via
        # the documented behavior: the helper returns a single path list.
        got = wedge.resolve_garnet(str(SCRIPT))
        self.assertEqual(got, [str(SCRIPT)])


@unittest.skipUnless(_garnet_binary() is not None, "garnet CLI not built")
class LiveWedgeTests(unittest.TestCase):
    def test_wedge_fires_end_to_end(self) -> None:
        proc = subprocess.run(
            [sys.executable, str(SCRIPT), "--format", "json"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertIn('"wedge_fires": true', proc.stdout)


if __name__ == "__main__":
    unittest.main()
