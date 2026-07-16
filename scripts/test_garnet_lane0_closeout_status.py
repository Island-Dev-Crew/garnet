#!/usr/bin/env python3
"""Regression tests for the Lane 0 closeout and evidence-integrity gate."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("garnet_lane0_closeout_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_lane0_closeout_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
status_mod = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = status_mod
SPEC.loader.exec_module(status_mod)


def _write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def _ledger_append(path: Path, event: dict[str, object], previous: str) -> str:
    body = {
        "at": "2026-07-16T12:00:00Z",
        "prevHash": previous,
        **event,
    }
    entry_hash = hashlib.sha256(
        json.dumps(body, sort_keys=True).encode()
    ).hexdigest()
    body["entryHash"] = entry_hash
    with path.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(body) + "\n")
    return entry_hash


class Lane0CloseoutStatusTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.evidence = self.root / "ops/lane0/evidence"
        self.evidence.mkdir(parents=True)
        for name in status_mod.EXPECTED_EVIDENCE_FILES:
            if name.endswith(".json"):
                _write_json(self.evidence / name, {"fixture": name})
            else:
                (self.evidence / name).write_text(f"fixture {name}\n", encoding="utf-8")

        denominators = {
            "schema": "garnet.lane0.denominators/v1",
            "launchStatus": "HOLD",
            "denominators": list(status_mod.EXPECTED_DENOMINATORS),
        }
        _write_json(self.evidence / "10-denominators.json", denominators)

        commands = {
            "schema": "garnet.lane0.commands/v1",
            "entries": [
                {
                    "command": "python3 -I scripts/example.py",
                    "expectedExit": 0,
                    "actualExit": 0,
                    "startedAt": "2026-07-16T12:00:00Z",
                    "endedAt": "2026-07-16T12:00:01Z",
                    "output": "20-python-tests.txt",
                }
            ],
        }
        _write_json(self.evidence / "COMMANDS.json", commands)

        self.manifest = self.evidence / "MANIFEST.sha256"
        status_mod.write_manifest(self.evidence, self.manifest)

        run_id = "lane0-20260716-test"
        self.ledger = self.root / "ops/lane0/ledger.jsonl"
        previous = "0" * 64
        previous = _ledger_append(
            self.ledger,
            {"event": "session-start", "runId": run_id},
            previous,
        )
        manifest_entries = status_mod.read_manifest_entries(self.manifest)
        for rel, digest in manifest_entries:
            previous = _ledger_append(
                self.ledger,
                {
                    "event": "evidence-sealed",
                    "runId": run_id,
                    "path": rel,
                    "sha256": digest,
                },
                previous,
            )
        manifest_digests = dict(manifest_entries)
        for gate, rel in status_mod.EXPECTED_GATE_EVIDENCE.items():
            previous = _ledger_append(
                self.ledger,
                {
                    "event": "gate-evidence",
                    "runId": run_id,
                    "gate": gate,
                    "path": rel,
                    "sha256": manifest_digests[rel],
                },
                previous,
            )
        previous = _ledger_append(
            self.ledger,
            {
                "event": "loopback",
                "runId": run_id,
                "fromGate": "G4",
                "toStage": "S2",
                "reason": "Lane 2C duration proof is not current.",
            },
            previous,
        )
        previous = _ledger_append(
            self.ledger,
            {"event": "audit-band", "runId": run_id, "band": 3},
            previous,
        )
        previous = _ledger_append(
            self.ledger,
            {
                "event": "governance-verdict",
                "runId": run_id,
                "verdict": "advisory",
                "waivers": [],
            },
            previous,
        )
        previous = _ledger_append(
            self.ledger,
            {
                "event": "stage-advance",
                "runId": run_id,
                "from": "S2",
                "to": "S6",
            },
            previous,
        )
        _ledger_append(
            self.ledger,
            {
                "event": "session-close",
                "runId": run_id,
                "nextActions": list(status_mod.EXPECTED_NEXT_ACTIONS),
            },
            previous,
        )

        lane_state = {
            "mission": {"status": "complete"},
            "phases": [
                {
                    "id": f"P{phase}",
                    "status": "done",
                    "gates": [
                        {
                            "id": f"P{phase}-G1",
                            "status": "passed",
                            "evidence": "ops/lane0/evidence/20-python-tests.txt",
                            "lastRun": "2026-07-16T12:00:00Z",
                        }
                    ],
                }
                for phase in range(4)
            ],
            "metrics": [
                {
                    "label": "Definition-of-done claims verified",
                    "current": "4",
                    "target": "4",
                }
            ],
            "resume": {
                "activePhase": None,
                "nextActions": list(status_mod.EXPECTED_NEXT_ACTIONS),
            },
            "closeout": {
                "governanceVerdict": "advisory",
                "band": 3,
                "waivers": [],
            },
            "archipelago": {
                "loop": {
                    "stage": "S6",
                    "loopbacks": [
                        {
                            "fromGate": "G4",
                            "toStage": "S2",
                            "reason": "Lane 2C duration proof is not current.",
                        }
                    ],
                },
                "governance": {
                    "ledgerRunId": run_id,
                },
            },
        }
        _write_json(self.root / "ops/lane0/state.json", lane_state)

        main_state = {
            "mission": {"status": "active"},
            "readiness": {
                "launchStatus": "HOLD",
                "denominators": list(status_mod.EXPECTED_DENOMINATORS),
            },
            "lane0Closeout": {
                "audit": "ops/lane0/AUDIT.md",
                "nextActions": list(status_mod.EXPECTED_NEXT_ACTIONS),
            },
        }
        _write_json(self.root / "ops/mission/state.json", main_state)
        (self.root / "ops/lane0/AUDIT.md").write_text(
            "Band 3/5\nVerdict: advisory\nG4 -> S2\nWaivers: none\n"
            "Playwright remains pending. Lane 2C remains partial.\n",
            encoding="utf-8",
        )

    def tearDown(self) -> None:
        self.temp.cleanup()

    def test_valid_closeout_fixture_passes(self) -> None:
        status = status_mod.read_status(self.root)
        self.assertTrue(status.ok, status.findings)
        self.assertEqual(3, status.audit_band)
        self.assertEqual("advisory", status.governance_verdict)
        self.assertEqual("HOLD", status.launch_status)
        self.assertEqual(4, status.denominator_count)

    def test_manifest_rejects_extra_unsealed_file(self) -> None:
        (self.evidence / "unexpected.txt").write_text("extra\n", encoding="utf-8")
        findings, _ = status_mod.verify_manifest(self.evidence, self.manifest)
        self.assertTrue(any("exact coverage" in finding for finding in findings))

    def test_manifest_rejects_unsorted_duplicate_and_traversal_entries(self) -> None:
        lines = self.manifest.read_text(encoding="utf-8").splitlines()
        lines = list(reversed(lines))
        lines.append(lines[0])
        lines.append(f"{'0' * 64}  ../escape.txt")
        self.manifest.write_text("\n".join(lines) + "\n", encoding="utf-8")
        findings, _ = status_mod.verify_manifest(self.evidence, self.manifest)
        self.assertTrue(any("sorted" in finding for finding in findings))
        self.assertTrue(any("duplicate" in finding for finding in findings))
        self.assertTrue(any("traversal" in finding for finding in findings))

    @unittest.skipIf(os.name == "nt", "symlink creation requires elevated Windows privileges")
    def test_manifest_rejects_symlink_even_when_hash_matches(self) -> None:
        target = self.evidence / "outside.txt"
        target.write_text("outside\n", encoding="utf-8")
        victim = self.evidence / "00-environment.json"
        victim.unlink()
        victim.symlink_to(target)
        status_mod.write_manifest(self.evidence, self.manifest)
        findings, _ = status_mod.verify_manifest(self.evidence, self.manifest)
        self.assertTrue(any("symlink" in finding for finding in findings))

    def test_manifest_rejects_hash_mismatch(self) -> None:
        (self.evidence / "20-python-tests.txt").write_text(
            "changed after seal\n", encoding="utf-8"
        )
        findings, _ = status_mod.verify_manifest(self.evidence, self.manifest)
        self.assertTrue(any("hash mismatch" in finding for finding in findings))

    def test_ledger_rejects_tampered_event(self) -> None:
        lines = self.ledger.read_text(encoding="utf-8").splitlines()
        event = json.loads(lines[-1])
        event["nextActions"] = ["tampered"]
        lines[-1] = json.dumps(event)
        self.ledger.write_text("\n".join(lines) + "\n", encoding="utf-8")
        findings = status_mod.verify_ledger(
            self.ledger,
            status_mod.read_manifest_entries(self.manifest),
            "lane0-20260716-test",
        )
        self.assertTrue(any("tampered" in finding for finding in findings))

    def test_ledger_rejects_missing_gate_evidence_binding(self) -> None:
        lines = self.ledger.read_text(encoding="utf-8").splitlines()
        entries = [json.loads(line) for line in lines]
        target = next(
            entry for entry in entries if entry.get("event") == "gate-evidence"
        )
        target["gate"] = "P9-G9"
        previous = "0" * 64
        rebuilt: list[str] = []
        for entry in entries:
            entry["prevHash"] = previous
            body = {key: value for key, value in entry.items() if key != "entryHash"}
            entry["entryHash"] = hashlib.sha256(
                json.dumps(body, sort_keys=True).encode()
            ).hexdigest()
            previous = entry["entryHash"]
            rebuilt.append(json.dumps(entry))
        self.ledger.write_text("\n".join(rebuilt) + "\n", encoding="utf-8")
        findings = status_mod.verify_ledger(
            self.ledger,
            status_mod.read_manifest_entries(self.manifest),
            "lane0-20260716-test",
        )
        self.assertTrue(any("gate/evidence" in finding for finding in findings))

    def test_command_ledger_rejects_unexpected_exit(self) -> None:
        commands = json.loads(
            (self.evidence / "COMMANDS.json").read_text(encoding="utf-8")
        )
        commands["entries"][0]["actualExit"] = 1
        _write_json(self.evidence / "COMMANDS.json", commands)
        status_mod.write_manifest(self.evidence, self.manifest)
        findings = status_mod.verify_commands(self.evidence / "COMMANDS.json")
        self.assertTrue(any("exit" in finding for finding in findings))

    def test_denominators_are_derived_from_reporters_and_historical_tasks(self) -> None:
        launch = {
            "recommendation": "HOLD",
            "launch_ready": False,
            "gates": [
                {"id": "foundation_integrity", "state": "pass"},
                {"id": "native_linux", "state": "pass"},
                {"id": "s114_acceptance", "state": "accepted-scoped"},
                {"id": "static_playground", "state": "partial"},
                {"id": "live_wasm_playground", "state": "remaining"},
                {"id": "minimum_sealed_shelf", "state": "manual-deferred"},
                {"id": "promo_video", "state": "pending-human"},
                {"id": "launch_fire", "state": "jon-only"},
            ],
        }
        _write_json(self.evidence / "08-launch-readiness.json", launch)
        _write_json(
            self.evidence / "09-mit-readiness.json",
            {"source": "committed-truth", "completion_percent": 93.1},
        )
        mission_path = self.root / "ops/mission/state.json"
        mission = json.loads(mission_path.read_text(encoding="utf-8"))
        mission["phases"] = [
            {
                "id": f"P{phase}",
                "tasks": [
                    {"status": "done"}
                    for _ in range(3 if phase < 5 else 2)
                ],
            }
            for phase in range(7)
        ]
        _write_json(mission_path, mission)
        status_mod.write_denominators(self.root, "2026-07-16T12:00:00Z")
        derived = json.loads(
            (self.evidence / "10-denominators.json").read_text(encoding="utf-8")
        )
        self.assertEqual("HOLD", derived["launchStatus"])
        self.assertEqual(list(status_mod.EXPECTED_DENOMINATORS), derived["denominators"])

    def test_state_rejects_non_s6_or_non_advisory_closeout(self) -> None:
        state_path = self.root / "ops/lane0/state.json"
        state = json.loads(state_path.read_text(encoding="utf-8"))
        state["archipelago"]["loop"]["stage"] = "S5"
        state["closeout"]["governanceVerdict"] = "enforced"
        _write_json(state_path, state)
        status = status_mod.read_status(self.root)
        self.assertTrue(any("stage S6" in finding for finding in status.findings))
        self.assertTrue(any("advisory" in finding for finding in status.findings))

    def test_main_mission_rejects_fifth_denominator_or_launch_go(self) -> None:
        state_path = self.root / "ops/mission/state.json"
        state = json.loads(state_path.read_text(encoding="utf-8"))
        state["readiness"]["launchStatus"] = "GO"
        state["readiness"]["denominators"].append(
            {
                "id": "forbidden_fifth",
                "label": "Forbidden fifth",
                "numerator": 1,
                "denominator": 1,
                "percent": 100.0,
                "evidence": "ops/lane0/evidence/10-denominators.json",
            }
        )
        _write_json(state_path, state)
        status = status_mod.read_status(self.root)
        self.assertTrue(any("exactly four" in finding for finding in status.findings))
        self.assertTrue(any("HOLD" in finding for finding in status.findings))


if __name__ == "__main__":
    unittest.main()
