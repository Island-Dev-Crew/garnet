#!/usr/bin/env python3
"""Regression tests for WV-6/WV-7 fail-closed acceptance reporting."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "garnet_wv_acceptance_status.py"
SPEC = importlib.util.spec_from_file_location("garnet_wv_acceptance_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
wv = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_wv_acceptance_status"] = wv
SPEC.loader.exec_module(wv)


class GarnetWvAcceptanceStatusTests(unittest.TestCase):
    def test_contracts_keep_established_meanings(self) -> None:
        contracts = wv.load_contracts(ROOT)
        self.assertEqual(
            contracts["WV-6"]["evidenceDestination"],
            "proofs/windows/launch-verification/wv6-minimum-shelf/",
        )
        self.assertIn("Core Ring Tier 1", contracts["WV-6"]["title"])
        self.assertEqual(
            contracts["WV-7"]["evidenceDestination"],
            "proofs/windows/launch-verification/wv7-distribution/",
        )
        self.assertIn("winget", contracts["WV-7"]["title"].lower())
        self.assertIn("Docker", contracts["WV-7"]["title"])

    def test_current_repository_is_pending_and_gates_red(self) -> None:
        for identifier in ("WV-6", "WV-7"):
            with self.subTest(identifier=identifier):
                status = wv.read_status(ROOT, identifier)
                self.assertEqual(status.state, "pending")
                self.assertFalse(status.ok)
                proc = subprocess.run(
                    [sys.executable, "-I", str(SCRIPT), "--wv", identifier, "--gate"],
                    cwd=ROOT,
                    text=True,
                    capture_output=True,
                    check=False,
                )
                self.assertNotEqual(proc.returncode, 0)
                payload = json.loads(proc.stdout)
                self.assertEqual(payload["state"], "pending")
                self.assertFalse(payload["ok"])

    def test_malformed_evidence_is_partial_not_accepted(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            wv.copy_contract(ROOT, root)
            contract = wv.load_contracts(root)["WV-6"]
            evidence = root / contract["evidenceDestination"]
            evidence.mkdir(parents=True)
            (evidence / wv.EVIDENCE_MANIFEST).write_text("{", encoding="utf-8")
            status = wv.read_status(root, "WV-6", verify_git=False)
        self.assertEqual(status.state, "partial")
        self.assertFalse(status.ok)

    def test_complete_hashed_evidence_can_satisfy_contract(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            wv.copy_contract(ROOT, root)
            contract = wv.load_contracts(root)["WV-6"]
            evidence = root / contract["evidenceDestination"]
            evidence.mkdir(parents=True)
            artifacts = []
            checks = []
            for item in contract["requiredChecks"]:
                relative = f"{item['id']}.txt"
                payload = f"{item['id']} passed\n".encode()
                (evidence / relative).write_bytes(payload)
                artifacts.append(
                    {"path": relative, "sha256": hashlib.sha256(payload).hexdigest()}
                )
                checks.append(
                    {
                        "id": item["id"],
                        "status": "passed",
                        "command": f"verify {item['id']}",
                        "evidence": [relative],
                    }
                )
            manifest = {
                "schema": "garnet.wv_acceptance_evidence/v1",
                "wv": "WV-6",
                "contractBaseMainSha": wv.EXPECTED_BASE_SHA,
                "candidateMainSha": "a" * 40,
                "state": "evidence_complete",
                "platform": "windows",
                "checks": checks,
                "artifacts": artifacts,
                "scopeLimitsAcknowledged": True,
                "jonOnlyActionsPerformed": [],
            }
            (evidence / wv.EVIDENCE_MANIFEST).write_text(
                json.dumps(manifest, indent=2), encoding="utf-8"
            )
            status = wv.read_status(root, "WV-6", verify_git=False)
        self.assertEqual(status.state, "accepted")
        self.assertTrue(status.ok, status.findings)

    def test_missing_check_and_hash_mismatch_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            wv.copy_contract(ROOT, root)
            contract = wv.load_contracts(root)["WV-7"]
            evidence = root / contract["evidenceDestination"]
            evidence.mkdir(parents=True)
            artifact = evidence / "only.txt"
            artifact.write_text("not enough", encoding="utf-8")
            manifest = {
                "schema": "garnet.wv_acceptance_evidence/v1",
                "wv": "WV-7",
                "contractBaseMainSha": wv.EXPECTED_BASE_SHA,
                "candidateMainSha": "b" * 40,
                "state": "evidence_complete",
                "platform": "windows",
                "checks": [],
                "artifacts": [{"path": "only.txt", "sha256": "0" * 64}],
                "scopeLimitsAcknowledged": True,
                "jonOnlyActionsPerformed": [],
            }
            (evidence / wv.EVIDENCE_MANIFEST).write_text(
                json.dumps(manifest), encoding="utf-8"
            )
            status = wv.read_status(root, "WV-7", verify_git=False)
        self.assertEqual(status.state, "partial")
        self.assertFalse(status.ok)
        self.assertTrue(any("required check" in item for item in status.findings))
        self.assertTrue(any("SHA-256" in item for item in status.findings))

    def test_nonexistent_candidate_cannot_self_promote(self) -> None:
        findings: list[str] = []
        wv._verify_candidate(ROOT, "f" * 40, findings)
        self.assertTrue(
            any("not a local commit object" in item for item in findings),
            findings,
        )


if __name__ == "__main__":
    unittest.main()
