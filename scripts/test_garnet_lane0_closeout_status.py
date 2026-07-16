#!/usr/bin/env python3
"""Adversarial regression tests for the Lane 0 semantic closeout gate."""
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

CANDIDATE = "d6a509e52e016a03b852d2afbc9c51baf1165201"


def _write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def _mit_lanes() -> list[dict[str, object]]:
    statuses = (
        ["verified"] * 60
        + ["active-partial"] * 3
        + ["local-registry-source-ready"]
        + ["feature-gated-source-ready"]
        + ["provider-gated-harness"]
        + ["provider-gated-5k-harness"]
        + ["planned"] * 2
        + ["blocked"]
    )
    return [
        {
            "id": f"lane-{index:02d}",
            "label": f"Lane {index}",
            "status": lane_status,
            "completion_percent": 100.0 if lane_status == "verified" else 0.0,
            "evidence": "fixture",
            "blocked_by": [],
            "deferred": [],
            "evidence_class": "committed",
        }
        for index, lane_status in enumerate(statuses, 1)
    ]


def _rechain(entries: list[dict]) -> list[dict]:
    previous = "0" * 64
    for entry in entries:
        entry["prevHash"] = previous
        body = {key: value for key, value in entry.items() if key != "entryHash"}
        entry["entryHash"] = hashlib.sha256(
            json.dumps(body, sort_keys=True).encode()
        ).hexdigest()
        previous = entry["entryHash"]
    return entries


class Lane0CloseoutStatusTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.evidence = self.root / "ops/lane0/evidence"
        self.evidence.mkdir(parents=True)
        self._write_valid_fixture()

    def tearDown(self) -> None:
        self.temp.cleanup()

    def _write_valid_fixture(self) -> None:
        _write_json(
            self.evidence / "00-environment.json",
            {
                "schema": "garnet.lane0.environment/v1",
                "capturedAt": "2026-07-16T12:00:00Z",
                "platform": {"system": "test", "release": "test", "machine": "test"},
                "versions": {
                    "rustcExactMsrv": "rustc 1.95.0 (fixture)",
                    "cargoExactMsrv": "cargo 1.95.0 (fixture)",
                },
                "dependencies": {
                    "jsonschema": {"available": True, "version": "4.26.0"},
                    "playwright": {
                        "available": False,
                        "degradation": "unavailable; no browser claim",
                    },
                },
                "credentialAndForkMainProbePerformed": False,
            },
        )
        (self.evidence / "01-archipelago-contracts.txt").write_text(
            "[PASS] idea.lock.json\n[PASS] plan.lock.json\n[PASS] state.json\n"
            "b9f7cee2823f9791503db20f33b22c9e20af7abe\n",
            encoding="utf-8",
        )
        archive_rows = "\n".join(
            f"{index:07x} fixture commit (#{471 + index})"
            for index in range(1, 35)
        )
        (self.evidence / "02-first-parent-archive.txt").write_text(
            archive_rows + "\n\n34\n", encoding="utf-8"
        )
        (self.evidence / "03-successor-pin-delta.txt").write_text(
            "231aefa ops(mission): open parallel launch convergence (#499)\n\n1\n",
            encoding="utf-8",
        )
        _write_json(
            self.evidence / "04-truth-freeze.json",
            {
                "schema": "garnet.lane0_truth_freeze/v1",
                "archive_base_exclusive": "d0d4f7cc988db4d793c5c7555a4043aef9b27180",
                "archive_head_inclusive": "1fe74892c588f912e103742afc9d11e845e8d4e6",
                "archive_pr_count": 34,
                "successor_archive_pr": 499,
                "successor_archive_sha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
                "checkpoint_pr_count": 35,
                "active_phase": "P7",
                "action_tasks": ["P7-T1", "P7-T2", "P7-T3", "P7-T4"],
                "adversarial_findings_resolved": False,
                "findings": [],
                "ok": True,
            },
        )
        _write_json(
            self.evidence / "05-msrv.json",
            {
                "schema": "garnet.msrv_status/v2",
                "msrv": "1.95",
                "workspace_member_count": 16,
                "workspace_members_inheriting": 16,
                "active_manifest_count": 18,
                "active_manifest_set_exact": True,
                "excluded_manifests_declaring": 2,
                "current_surfaces_aligned": True,
                "workflow_projection_valid": True,
                "stable_tracking_preserved": True,
                "exact_msrv_ci_check": True,
                "studio_exact_msrv_ci_check": True,
                "reporter_ci_wired": True,
                "rust_toolchain_file_absent": True,
                "procedural_contract_present": True,
                "findings": [],
                "ok": True,
            },
        )
        _write_json(
            self.evidence / "06-frozen-backlog.json",
            {
                "schema": "garnet.lane0.frozen_backlog/v1",
                "exact_base_main_sha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
                "entry_count": 8,
                "implemented_clause_count": 9,
                "states": {
                    "implemented": 0,
                    "partial": 4,
                    "planned": 4,
                    "research": 0,
                },
                "findings": [],
                "ok": True,
            },
        )
        _write_json(
            self.evidence / "07-quarterly-watch.json",
            {
                "schema": "garnet.quarterly_competitive_watch/v1",
                "state": "planned",
                "as_of": "2026-07-16",
                "report_count": 0,
                "next_due": "2026-09-30",
                "contract_present": True,
                "findings": [],
                "ok": True,
            },
        )
        _write_json(
            self.evidence / "08-launch-readiness.json",
            {
                "schema": "garnet.launch_readiness/v1",
                "recommendation": "HOLD",
                "launch_ready": False,
                "gates": [
                    {"id": identifier, "state": state}
                    for identifier, state in status_mod.EXPECTED_LAUNCH_GATES
                ],
            },
        )
        _write_json(
            self.evidence / "09-mit-readiness.json",
            {
                "source": "committed-truth",
                "overall_status": "active-partial",
                "completion_percent": 93.1,
                "current_truth": [],
                "lanes": _mit_lanes(),
            },
        )
        (self.evidence / "11-truth-check.txt").write_text(
            "truth --check: ok (6 fields vs machine truth, 4 stamped surfaces)\n",
            encoding="utf-8",
        )
        _write_json(
            self.evidence / "12-repository-evidence-integrity.json",
            {
                "schema": "garnet.evidence_integrity/v1",
                "bundles_total": 38,
                "bundles_ok": 38,
                "bundles_failed": 0,
                "failed": [],
                "ok": True,
            },
        )
        for filename, identifier in (
            ("13-wv6-pending.json", "WV-6"),
            ("14-wv7-pending.json", "WV-7"),
        ):
            _write_json(
                self.evidence / filename,
                {
                    "schema": "garnet.wv_acceptance_status/v1",
                    "wv": identifier,
                    "contract_base_main_sha": (
                        "231aefa91985e5a0520c493c7f0fc3e54d74efc8"
                    ),
                    "evidence_destination": {
                        "WV-6": (
                            "proofs/windows/launch-verification/"
                            "wv6-minimum-shelf/"
                        ),
                        "WV-7": (
                            "proofs/windows/launch-verification/"
                            "wv7-distribution/"
                        ),
                    }[identifier],
                    "candidate_main_sha": None,
                    "required_check_count": 5,
                    "passed_check_count": 0,
                    "artifact_count": 0,
                    "state": "pending",
                    "findings": ["exact-candidate evidence manifest is pending"],
                    "ok": False,
                },
            )
        for name in (
            "20-python-tests.txt",
            "21-rust-msrv-checks.txt",
            "22-workspace-tests.txt",
        ):
            (self.evidence / name).write_text("all commands passed\n", encoding="utf-8")
        (self.evidence / "23-sotu-render.txt").write_text(
            "Rendered fixture/state-of-the-union.html\n"
            "  phases: 8, tasks: 19/23, gates passed: 17/21\n",
            encoding="utf-8",
        )
        (self.evidence / "24-pr-body-validation.txt").write_text(
            "dogfood-pr-body: ok (86 changed files checked)\n",
            encoding="utf-8",
        )
        (self.evidence / "25-independent-review.md").write_text(
            "# Independent review\n\n"
            "Final integrated verdict: **APPROVED**\n"
            "Reviewer role: independent integrated reviewer\n"
            "Reviewed range: "
            "`231aefa91985e5a0520c493c7f0fc3e54d74efc8.."
            f"{CANDIDATE}`\n"
            "Reviewed at: `2026-07-16T12:00:45Z`\n"
            "Open Critical findings: 0\n"
            "Open Important findings: 0\n",
            encoding="utf-8",
        )

        tasks_per_phase = [3, 3, 3, 3, 3, 2, 2]
        main_state = {
            "mission": {"status": "active"},
            "phases": [
                {
                    "id": f"P{phase}",
                    "tasks": [
                        {"status": "done"} for _ in range(tasks_per_phase[phase])
                    ],
                }
                for phase in range(7)
            ],
            "readiness": {
                "launchStatus": "HOLD",
                "denominators": list(status_mod.EXPECTED_DENOMINATORS),
            },
            "lane0Closeout": {
                "audit": "ops/lane0/AUDIT.md",
                "finalIntegratedReview": "approved",
                "nextActions": list(status_mod.EXPECTED_NEXT_ACTIONS),
            },
        }
        _write_json(self.root / "ops/mission/state.json", main_state)
        (self.root / "ops/mission/state-of-the-union.html").write_text(
            "Mission Control &middot; State of the Union &middot; generated "
            "2026-07-16 12:00:46Z\n",
            encoding="utf-8",
        )

        phases = []
        for phase in range(4):
            phase_id = f"P{phase}"
            gates = []
            for gate_id, evidence in status_mod.EXPECTED_GATE_EVIDENCE.items():
                if gate_id.startswith(f"{phase_id}-"):
                    gates.append(
                        {
                            "id": gate_id,
                            "command": status_mod.EXPECTED_GATE_COMMANDS[gate_id],
                            "status": "passed",
                            "evidence": f"ops/lane0/evidence/{evidence}",
                            "lastRun": "2026-07-16T12:00:01Z",
                        }
                    )
            phases.append({"id": phase_id, "status": "done", "gates": gates})
        lane_state = {
            "mission": {"status": "complete"},
            "phases": phases,
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
                "finalIntegratedReview": "approved",
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
                            "at": "2026-07-16T12:00:00Z",
                        }
                    ],
                },
                "governance": {"ledgerRunId": "lane0-test"},
            },
        }
        _write_json(self.root / "ops/lane0/state.json", lane_state)
        (self.root / "ops/lane0/AUDIT.md").write_text(
            "Band 3/5\nadvisory\nG4 -> S2\nWaivers: none\n"
            "Playwright pending; Lane 2C partial.\n",
            encoding="utf-8",
        )

        commands = []
        for index, (command, expected, output) in enumerate(
            status_mod.EXPECTED_COMMANDS
        ):
            if "<CANDIDATE_SHA>" in command:
                command = command.replace("<CANDIDATE_SHA>", CANDIDATE)
            commands.append(
                {
                    "command": command,
                    "expectedExit": expected,
                    "actualExit": expected,
                    "startedAt": f"2026-07-16T12:00:{index:02d}Z",
                    "endedAt": f"2026-07-16T12:00:{index + 1:02d}Z",
                    "output": output,
                }
            )
        stdout_by_command = {
            f"git -C {status_mod.ARCHIPELAGO_TOOL} rev-parse HEAD": (
                "b9f7cee2823f9791503db20f33b22c9e20af7abe\n"
            ),
            f"python3 {status_mod.ARCHIPELAGO_TOOL}/scripts/validate_contracts.py ops/lane0/idea.lock.json": "[PASS] idea.lock.json\n",
            f"python3 {status_mod.ARCHIPELAGO_TOOL}/scripts/validate_contracts.py ops/lane0/plan.lock.json": "[PASS] plan.lock.json\n",
            f"python3 {status_mod.ARCHIPELAGO_TOOL}/scripts/validate_contracts.py ops/lane0/state.json": "[PASS] state.json\n",
            "git log --oneline --first-parent d0d4f7cc..1fe7489": (
                archive_rows + "\n"
            ),
            "git log --oneline --first-parent d0d4f7cc..1fe7489 | wc -l": "34\n",
            "git log --oneline --first-parent 1fe7489..231aefa": (
                "231aefa ops(mission): open parallel launch convergence (#499)\n"
            ),
            "git rev-list --count --first-parent 1fe7489..231aefa": "1\n",
            "cargo run -p xtask -- truth --check": (
                "truth --check: ok (6 fields vs machine truth, 4 stamped surfaces)\n"
            ),
            "node ops/mission/render-sotu.mjs": (
                "Rendered fixture/state-of-the-union.html\n"
                "  phases: 8, tasks: 19/23, gates passed: 17/21\n"
            ),
            commands[-1]["command"]: (
                "dogfood-pr-body: ok (86 changed files checked)\n"
            ),
        }
        for output in status_mod.TEXT_TRANSCRIPT_OUTPUTS:
            blocks = []
            for entry in commands:
                if entry["output"] != output:
                    continue
                command = entry["command"]
                blocks.extend(
                    [
                        f"$ {command}",
                        f"expected_exit={entry['expectedExit']}",
                        f"actual_exit={entry['actualExit']}",
                        "--- stdout ---",
                        stdout_by_command.get(command, "passed\n").rstrip(),
                        "--- stderr ---",
                        "",
                    ]
                )
            (self.evidence / output).write_text(
                "\n".join(blocks).rstrip() + "\n", encoding="utf-8"
            )
        _write_json(
            self.evidence / "COMMANDS.json",
            {"schema": "garnet.lane0.commands/v1", "entries": commands},
        )
        status_mod.write_denominators(self.root, "2026-07-16T12:00:02Z")
        status_mod.write_manifest(
            self.evidence, self.evidence / "MANIFEST.sha256"
        )
        status_mod.write_ledger(self.root, "lane0-test")

    def _status(self):
        return status_mod.read_status(self.root, verify_git=False)

    def _reseal(self) -> None:
        status_mod.write_manifest(
            self.evidence, self.evidence / "MANIFEST.sha256"
        )
        status_mod.write_ledger(self.root, "lane0-test")

    def test_valid_closeout_fixture_passes(self) -> None:
        status = self._status()
        self.assertTrue(status.ok, status.findings)
        self.assertEqual(4, status.denominator_count)
        self.assertEqual("HOLD", status.launch_status)

    def test_resealed_launch_go_is_rejected(self) -> None:
        path = self.evidence / "08-launch-readiness.json"
        data = json.loads(path.read_text(encoding="utf-8"))
        data["recommendation"] = "GO"
        data["launch_ready"] = True
        _write_json(path, data)
        self._reseal()
        status = self._status()
        self.assertTrue(any("recommendation" in item for item in status.findings))

    def test_resealed_mit_12_3_is_rejected(self) -> None:
        path = self.evidence / "09-mit-readiness.json"
        data = json.loads(path.read_text(encoding="utf-8"))
        data["completion_percent"] = 12.3
        _write_json(path, data)
        self._reseal()
        status = self._status()
        self.assertTrue(any("completion_percent" in item for item in status.findings))

    def test_resealed_accepted_wv_is_rejected(self) -> None:
        path = self.evidence / "13-wv6-pending.json"
        data = json.loads(path.read_text(encoding="utf-8"))
        data.update(
            {
                "state": "accepted",
                "ok": True,
                "passed_check_count": 5,
                "artifact_count": 5,
            }
        )
        _write_json(path, data)
        self._reseal()
        status = self._status()
        self.assertTrue(any("13-wv6" in item for item in status.findings))

    def test_one_row_command_inventory_cannot_be_resealed(self) -> None:
        path = self.evidence / "COMMANDS.json"
        data = json.loads(path.read_text(encoding="utf-8"))
        data["entries"] = data["entries"][:1]
        _write_json(path, data)
        status_mod.write_manifest(
            self.evidence, self.evidence / "MANIFEST.sha256"
        )
        with self.assertRaises(ValueError):
            status_mod.write_ledger(self.root, "lane0-test")

    def test_pending_final_review_is_the_only_review_failure(self) -> None:
        path = self.evidence / "25-independent-review.md"
        text = path.read_text(encoding="utf-8").replace(
            "**APPROVED**", "**NEEDS PATCH**"
        )
        path.write_text(text, encoding="utf-8")
        lane = json.loads((self.root / "ops/lane0/state.json").read_text())
        lane["closeout"]["finalIntegratedReview"] = "pending"
        _write_json(self.root / "ops/lane0/state.json", lane)
        main = json.loads((self.root / "ops/mission/state.json").read_text())
        main["lane0Closeout"]["finalIntegratedReview"] = "pending"
        _write_json(self.root / "ops/mission/state.json", main)
        self._reseal()
        status = self._status()
        self.assertEqual(1, len(status.findings), status.findings)
        self.assertIn("final integrated review", status.findings[0])

    def test_state_to_ledger_gate_binding_mismatch_is_rejected(self) -> None:
        path = self.root / "ops/lane0/state.json"
        data = json.loads(path.read_text(encoding="utf-8"))
        data["phases"][0]["gates"][0]["evidence"] = (
            "ops/lane0/evidence/11-truth-check.txt"
        )
        _write_json(path, data)
        status = self._status()
        self.assertTrue(any("bindings" in item for item in status.findings))

    def test_trailing_contradictory_event_is_rejected(self) -> None:
        path = self.root / "ops/lane0/ledger.jsonl"
        entries = [json.loads(line) for line in path.read_text().splitlines()]
        entries.append(
            {
                "at": "2026-07-16T12:00:11Z",
                "prevHash": entries[-1]["entryHash"],
                "event": "audit-band",
                "runId": "lane0-test",
                "band": 5,
            }
        )
        _rechain(entries)
        path.write_text(
            "\n".join(json.dumps(entry) for entry in entries) + "\n",
            encoding="utf-8",
        )
        status = self._status()
        self.assertTrue(any("sequence" in item for item in status.findings))

    def test_invalid_ledger_timestamp_is_rejected(self) -> None:
        path = self.root / "ops/lane0/ledger.jsonl"
        entries = [json.loads(line) for line in path.read_text().splitlines()]
        entries[5]["at"] = "not-a-time"
        _rechain(entries)
        path.write_text(
            "\n".join(json.dumps(entry) for entry in entries) + "\n",
            encoding="utf-8",
        )
        status = self._status()
        self.assertTrue(any("timestamp" in item for item in status.findings))

    def test_backdated_evidence_seal_is_rejected(self) -> None:
        path = self.root / "ops/lane0/ledger.jsonl"
        entries = [json.loads(line) for line in path.read_text().splitlines()]
        for entry in entries:
            if entry["event"] == "evidence-sealed":
                entry["at"] = "2026-07-16T12:00:01Z"
        _rechain(entries)
        path.write_text(
            "\n".join(json.dumps(entry) for entry in entries) + "\n",
            encoding="utf-8",
        )
        status = self._status()
        self.assertTrue(any("strictly after" in item for item in status.findings))

    def test_synthetic_truth_fraction_is_rejected(self) -> None:
        path = self.root / "ops/mission/state.json"
        data = json.loads(path.read_text(encoding="utf-8"))
        truth = data["readiness"]["denominators"][1]
        truth["numerator"] = 931
        truth["denominator"] = 1000
        truth.pop("rounded")
        _write_json(path, data)
        status = self._status()
        self.assertTrue(any("current reporter evidence" in item for item in status.findings))

    def test_manifest_rejects_extra_file_and_hash_drift(self) -> None:
        (self.evidence / "extra.txt").write_text("extra\n", encoding="utf-8")
        findings, _ = status_mod.verify_manifest(
            self.evidence, self.evidence / "MANIFEST.sha256"
        )
        self.assertTrue(any("exact coverage" in item for item in findings))
        (self.evidence / "extra.txt").unlink()
        (self.evidence / "20-python-tests.txt").write_text(
            "changed\n", encoding="utf-8"
        )
        findings, _ = status_mod.verify_manifest(
            self.evidence, self.evidence / "MANIFEST.sha256"
        )
        self.assertTrue(any("hash mismatch" in item for item in findings))

    @unittest.skipIf(os.name == "nt", "symlink creation may require elevation")
    def test_manifest_rejects_symlink_and_traversal(self) -> None:
        victim = self.evidence / "00-environment.json"
        victim.unlink()
        outside = self.root / "outside.json"
        outside.write_text("{}\n", encoding="utf-8")
        victim.symlink_to(outside)
        findings, _ = status_mod.verify_manifest(
            self.evidence, self.evidence / "MANIFEST.sha256"
        )
        self.assertTrue(any("symlink" in item for item in findings))
        manifest = self.evidence / "MANIFEST.sha256"
        with manifest.open("a", encoding="utf-8") as handle:
            handle.write(f"{'0' * 64}  ../escape.txt\n")
        findings, _ = status_mod.verify_manifest(self.evidence, manifest)
        self.assertTrue(any("traversal" in item for item in findings))


if __name__ == "__main__":
    unittest.main()
