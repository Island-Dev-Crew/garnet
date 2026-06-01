#!/usr/bin/env python3
"""Regression tests for the Windows audit burn-down status gate (P0)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_windows_audit_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_windows_audit_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
wa = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_windows_audit_status"] = wa
SPEC.loader.exec_module(wa)


class WindowsAuditTests(unittest.TestCase):
    def test_fourteen_open_findings_tracked(self) -> None:
        self.assertEqual(len(wa.OPEN_FINDINGS), 14)

    def test_every_open_finding_has_owning_slice(self) -> None:
        r = wa.read_status()
        self.assertEqual(r.findings_without_owner, [], f"unowned: {r.findings_without_owner}")

    def test_machine_ledgers_committed_and_head_pinned(self) -> None:
        r = wa.read_status()
        self.assertTrue(r.core_ledger_present and r.goal_ledger_present)
        self.assertTrue(r.head_pinned, "ledgers must pin HEAD cc165e8")

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(wa.main(["--gate", "--format", "json"]), 0)

    def test_resolved_finding_recorded(self) -> None:
        doc = wa.DOC.read_text(encoding="utf-8")
        self.assertIn("WIN-S70-001", doc)


if __name__ == "__main__":
    unittest.main()
