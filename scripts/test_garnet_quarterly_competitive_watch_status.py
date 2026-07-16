#!/usr/bin/env python3
"""Regression tests for the quarterly competitive-watch standing slice."""
from __future__ import annotations

import importlib.util
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "garnet_quarterly_competitive_watch_status.py"
SPEC = importlib.util.spec_from_file_location(
    "garnet_quarterly_competitive_watch_status", SCRIPT
)
assert SPEC is not None and SPEC.loader is not None
watch = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_quarterly_competitive_watch_status"] = watch
SPEC.loader.exec_module(watch)


class GarnetQuarterlyCompetitiveWatchStatusTests(unittest.TestCase):
    def test_current_slice_is_activated_but_first_report_is_not_claimed(self) -> None:
        status = watch.read_status(ROOT, as_of=date(2026, 7, 16))
        self.assertTrue(status.ok, status.findings)
        self.assertEqual(status.state, "planned")
        self.assertEqual(status.report_count, 0)
        self.assertEqual(status.next_due, "2026-09-30")

        proc = subprocess.run(
            [
                sys.executable,
                "-I",
                str(SCRIPT),
                "--as-of",
                "2026-07-16",
                "--gate",
            ],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn('"state": "planned"', proc.stdout)

    def test_missing_first_report_fails_after_due_date(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            contract = root / watch.CONTRACT_PATH
            contract.parent.mkdir(parents=True)
            shutil.copy2(ROOT / watch.CONTRACT_PATH, contract)
            status = watch.read_status(root, as_of=date(2026, 10, 1))
        self.assertFalse(status.ok)
        self.assertEqual(status.state, "overdue")
        self.assertTrue(any("2026-Q3" in item for item in status.findings))

    def test_placeholder_does_not_count_as_completed_report(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            contract = root / watch.CONTRACT_PATH
            contract.parent.mkdir(parents=True)
            shutil.copy2(ROOT / watch.CONTRACT_PATH, contract)
            reports = root / watch.REPORT_DIR
            reports.mkdir(parents=True)
            (reports / "2026-Q3.md").write_text(
                "# 2026 Q3\n\nStatus: planned\n", encoding="utf-8"
            )
            status = watch.read_status(root, as_of=date(2026, 10, 1))
        self.assertFalse(status.ok)
        self.assertEqual(status.report_count, 0)

    def test_contract_requires_miss_is_not_absence_rule(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            contract = root / watch.CONTRACT_PATH
            contract.parent.mkdir(parents=True)
            text = (ROOT / watch.CONTRACT_PATH).read_text(encoding="utf-8")
            contract.write_text(
                text.replace("A search miss is not evidence of absence.", ""),
                encoding="utf-8",
            )
            status = watch.read_status(root, as_of=date(2026, 7, 16))
        self.assertFalse(status.ok)
        self.assertTrue(any("miss-is-not-absence" in item for item in status.findings))

    def test_canonical_research_map_and_directives_are_complete(self) -> None:
        readme = (ROOT / "research" / "README.md").read_text(encoding="utf-8")
        for label in ("A", "B", "C", "D", "E", "F"):
            self.assertIn(f"| {label} |", readme)
        directives = (ROOT / "research" / "DIRECTIVES_LEDGER.md").read_text(
            encoding="utf-8"
        )
        rows = re.findall(
            r"^\| D([0-9]+) \| (implemented|partial|planned|research) \|",
            directives,
            flags=re.MULTILINE,
        )
        self.assertEqual([int(number) for number, _state in rows], list(range(1, 17)))
        self.assertIn("Master Plan v3.2 chapter 5", directives)
        self.assertIn("absent", directives.lower())
        self.assertIn("012021a", directives)
        self.assertIn("garnet-vm/src/caps_recheck.rs", directives)
        self.assertIn("seal-predicate integration remain open", directives)
        compatibility = (
            ROOT
            / "F_Project_Management"
            / "RESEARCH"
            / "GARNET_REASSESSMENT_2026-06-11.md"
        ).read_text(encoding="utf-8")
        self.assertIn("research/2026-06/GARNET_REASSESSMENT_2026-06-11.md", compatibility)

    def test_canonicalized_june_source_is_byte_preserved_from_lane_base(self) -> None:
        proc = subprocess.run(
            [
                "git",
                "show",
                "231aefa91985e5a0520c493c7f0fc3e54d74efc8:"
                "F_Project_Management/RESEARCH/"
                "GARNET_REASSESSMENT_2026-06-11.md",
            ],
            cwd=ROOT,
            capture_output=True,
            check=False,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr.decode(errors="replace"))
        canonical = (
            ROOT / "research" / "2026-06" / "GARNET_REASSESSMENT_2026-06-11.md"
        ).read_bytes()
        self.assertEqual(proc.stdout, canonical)


if __name__ == "__main__":
    unittest.main()
