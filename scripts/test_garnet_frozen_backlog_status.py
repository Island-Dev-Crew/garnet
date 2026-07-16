#!/usr/bin/env python3
"""Regression tests for the Lane 0 evidence-tied frozen backlog."""
from __future__ import annotations

import copy
import importlib.util
import json
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "garnet_frozen_backlog_status.py"
SPEC = importlib.util.spec_from_file_location("garnet_frozen_backlog_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
backlog = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_frozen_backlog_status"] = backlog
SPEC.loader.exec_module(backlog)


class GarnetFrozenBacklogStatusTests(unittest.TestCase):
    def test_current_backlog_gate_passes(self) -> None:
        status = backlog.read_status(ROOT)
        self.assertTrue(status.ok, status.findings)
        self.assertEqual(status.entry_count, 8)
        self.assertEqual(status.implemented_clause_count, 9)

        proc = subprocess.run(
            [sys.executable, "-I", str(SCRIPT), "--gate"],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)

    def test_lane_states_remain_honest(self) -> None:
        document = json.loads((ROOT / backlog.BACKLOG_PATH).read_text(encoding="utf-8"))
        states = {entry["id"]: entry["claimState"] for entry in document["entries"]}
        self.assertEqual(
            states,
            {
                "LANE-1-ITEM-1": "partial",
                "LANE-2A": "partial",
                "LANE-2B": "partial",
                "LANE-2C": "partial",
                "WV-6": "planned",
                "WV-7": "planned",
                "U-15": "planned",
                "QWATCH": "planned",
            },
        )
        lane_2c = next(row for row in document["entries"] if row["id"] == "LANE-2C")
        self.assertIn("0.03s", lane_2c["finding"])
        self.assertTrue(
            any(
                "three reproducible stress cases exceeding four minutes" in clause["claim"]
                for clause in lane_2c["openClauses"]
            )
        )

    def test_unknown_claim_state_is_rejected(self) -> None:
        document = backlog.load_document(ROOT)
        mutated = copy.deepcopy(document)
        mutated["entries"][0]["claimState"] = "approved"
        findings = backlog.validate_document(mutated, ROOT, verify_git=False)
        self.assertTrue(any("unsupported claimState" in item for item in findings))

    def test_partial_entry_requires_implemented_and_open_clauses(self) -> None:
        document = backlog.load_document(ROOT)
        mutated = copy.deepcopy(document)
        mutated["entries"][0]["openClauses"] = []
        findings = backlog.validate_document(mutated, ROOT, verify_git=False)
        self.assertTrue(
            any("partial entries require implementedClauses and openClauses" in item for item in findings)
        )

    def test_implemented_clause_requires_real_code_and_evidence_paths(self) -> None:
        document = backlog.load_document(ROOT)
        mutated = copy.deepcopy(document)
        mutated["entries"][0]["implementedClauses"][0]["evidencePaths"] = [
            "proofs/does-not-exist.json"
        ]
        findings = backlog.validate_document(mutated, ROOT, verify_git=False)
        self.assertTrue(any("does not exist" in item for item in findings))

    def test_future_destination_cannot_masquerade_as_current_evidence(self) -> None:
        document = backlog.load_document(ROOT)
        mutated = copy.deepcopy(document)
        mutated["entries"][0]["futureEvidence"][0]["status"] = "evidence"
        findings = backlog.validate_document(mutated, ROOT, verify_git=False)
        self.assertTrue(any("future-not-evidence" in item for item in findings))


if __name__ == "__main__":
    unittest.main()
