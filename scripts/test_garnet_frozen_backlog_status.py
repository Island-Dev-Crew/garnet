#!/usr/bin/env python3
"""Regression tests for the Lane 0 evidence-tied frozen backlog."""
from __future__ import annotations

import copy
import importlib.util
import json
import subprocess
import sys
import tempfile
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
        human = (
            ROOT
            / "F_Project_Management"
            / "LAUNCH"
            / "GARNET_L0_FROZEN_BACKLOG_2026-07-15.md"
        ).read_text(encoding="utf-8")
        self.assertIn("Current freeze context", human)
        self.assertNotIn("Implemented at the base", human)

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

    def test_tracked_directory_cannot_be_used_as_clause_evidence(self) -> None:
        document = backlog.load_document(ROOT)
        mutated = copy.deepcopy(document)
        mutated["entries"][0]["implementedClauses"][0]["evidencePaths"] = [
            "scripts"
        ]
        findings = backlog.validate_document(mutated, ROOT)
        self.assertTrue(any("regular file" in item for item in findings), findings)

    def test_symlink_cannot_be_used_as_clause_evidence(self) -> None:
        document = backlog.load_document(ROOT)
        mutated = copy.deepcopy(document)
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            evidence_dir = root / "evidence"
            evidence_dir.mkdir()
            target = evidence_dir / "real.txt"
            target.write_text("evidence\n", encoding="utf-8")
            link = evidence_dir / "link.txt"
            link.symlink_to(target.name)
            mutated["entries"][0]["implementedClauses"][0]["evidencePaths"] = [
                "evidence/link.txt"
            ]
            findings = backlog.validate_document(mutated, root, verify_git=False)
        self.assertTrue(
            any(
                "evidence/link.txt is not a regular file" in item
                for item in findings
            ),
            findings,
        )

    def test_future_destination_must_be_disjoint_from_current_evidence(self) -> None:
        document = backlog.load_document(ROOT)
        mutated = copy.deepcopy(document)
        current = mutated["entries"][0]["implementedClauses"][0]["evidencePaths"][0]
        mutated["entries"][0]["futureEvidence"][0]["path"] = current
        findings = backlog.validate_document(mutated, ROOT, verify_git=False)
        self.assertTrue(any("overlaps current evidence" in item for item in findings))

    def test_repository_authority_anchor_must_exist(self) -> None:
        document = backlog.load_document(ROOT)
        mutated = copy.deepcopy(document)
        mutated["entries"][0]["authoritySources"][0]["anchor"] = (
            "definitely-not-real"
        )
        findings = backlog.validate_document(mutated, ROOT, verify_git=False)
        self.assertTrue(any("anchor does not occur" in item for item in findings))

    def test_git_object_authority_and_anchor_are_supported(self) -> None:
        document = backlog.load_document(ROOT)
        mutated = copy.deepcopy(document)
        mutated["entries"][0]["authoritySources"][0] = {
            "kind": "repository",
            "path": (
                "git:231aefa91985e5a0520c493c7f0fc3e54d74efc8:"
                "ops/mission/state.json"
            ),
            "anchor": "P7-T1",
        }
        findings = backlog.validate_document(mutated, ROOT)
        self.assertFalse(
            any("authoritySources[0]" in item for item in findings),
            findings,
        )

    def test_clause_main_sha_must_be_full_and_ancestral(self) -> None:
        document = backlog.load_document(ROOT)
        malformed = copy.deepcopy(document)
        malformed["entries"][0]["implementedClauses"][0]["mainSha"] = "not-a-sha"
        findings = backlog.validate_document(malformed, ROOT, verify_git=False)
        self.assertTrue(any("mainSha must be one full" in item for item in findings))

        descendant = copy.deepcopy(document)
        head = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=True,
        ).stdout.strip()
        descendant["entries"][0]["implementedClauses"][0]["mainSha"] = head
        findings = backlog.validate_document(descendant, ROOT)
        self.assertTrue(any("not an ancestor" in item for item in findings))

    def test_clause_code_path_must_exist_at_main_sha(self) -> None:
        document = backlog.load_document(ROOT)
        mutated = copy.deepcopy(document)
        mutated["entries"][0]["implementedClauses"][0]["codePaths"] = [
            "research/README.md"
        ]
        findings = backlog.validate_document(mutated, ROOT)
        self.assertTrue(any("does not exist at mainSha" in item for item in findings))

    def test_clause_evidence_path_must_exist_at_exact_base(self) -> None:
        document = backlog.load_document(ROOT)
        mutated = copy.deepcopy(document)
        mutated["entries"][0]["implementedClauses"][0]["evidencePaths"] = [
            "research/README.md"
        ]
        findings = backlog.validate_document(mutated, ROOT)
        self.assertTrue(
            any("does not exist as a regular file at exact base" in item for item in findings)
        )

    def test_entry_and_clause_ids_are_unique_and_disjoint(self) -> None:
        document = backlog.load_document(ROOT)
        duplicate_entry = copy.deepcopy(document)
        duplicate_entry["entries"][1]["id"] = duplicate_entry["entries"][0]["id"]
        findings = backlog.validate_document(duplicate_entry, ROOT, verify_git=False)
        self.assertTrue(any("duplicate backlog id" in item for item in findings))

        duplicate_open = copy.deepcopy(document)
        duplicate_open["entries"][0]["openClauses"][1]["id"] = (
            duplicate_open["entries"][0]["openClauses"][0]["id"]
        )
        findings = backlog.validate_document(duplicate_open, ROOT, verify_git=False)
        self.assertTrue(any("duplicate open clause id" in item for item in findings))

        reused = copy.deepcopy(document)
        reused["entries"][0]["openClauses"][0]["id"] = (
            reused["entries"][0]["implementedClauses"][0]["id"]
        )
        findings = backlog.validate_document(reused, ROOT, verify_git=False)
        self.assertTrue(any("used by implemented and open" in item for item in findings))

    def test_planned_and_research_entries_retain_open_work(self) -> None:
        document = backlog.load_document(ROOT)
        planned = copy.deepcopy(document)
        planned_entry = next(
            entry for entry in planned["entries"] if entry["claimState"] == "planned"
        )
        planned_entry["openClauses"] = []
        findings = backlog.validate_document(planned, ROOT, verify_git=False)
        self.assertTrue(any("planned entries require openClauses" in item for item in findings))

        research = copy.deepcopy(document)
        research_entry = next(
            entry for entry in research["entries"] if entry["id"] == "WV-6"
        )
        research_entry["claimState"] = "research"
        research_entry["openClauses"] = []
        findings = backlog.validate_document(research, ROOT, verify_git=False)
        self.assertTrue(any("research entries require openClauses" in item for item in findings))


if __name__ == "__main__":
    unittest.main()
