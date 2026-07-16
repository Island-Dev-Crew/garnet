#!/usr/bin/env python3
"""Regression tests for the Lane 0 archive and U-18 truth-freeze gate."""
from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "garnet_lane0_truth_freeze_status.py"


class Lane0TruthFreezeStatusTests(unittest.TestCase):
    def _run_gate(
        self,
        *,
        state: dict[str, object] | None = None,
        plan: dict[str, object] | None = None,
    ) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        with tempfile.TemporaryDirectory() as td:
            tmp = Path(td)
            state_path = tmp / "state.json"
            plan_path = tmp / "plan.lock.json"
            state_path.write_text(
                json.dumps(
                    state
                    if state is not None
                    else json.loads((ROOT / "ops/mission/state.json").read_text()),
                    indent=2,
                ),
                encoding="utf-8",
            )
            plan_path.write_text(
                json.dumps(
                    plan
                    if plan is not None
                    else json.loads((ROOT / "ops/lane0/plan.lock.json").read_text()),
                    indent=2,
                ),
                encoding="utf-8",
            )
            proc = subprocess.run(
                [
                    sys.executable,
                    "-I",
                    str(SCRIPT),
                    "--repo-root",
                    str(ROOT),
                    "--state",
                    str(state_path),
                    "--plan",
                    str(plan_path),
                    "--gate",
                ],
                cwd=ROOT,
                text=True,
                capture_output=True,
                check=False,
            )
            try:
                payload = json.loads(proc.stdout)
            except json.JSONDecodeError:
                payload = {}
            return proc, payload

    def test_real_repository_passes_gate(self) -> None:
        proc, payload = self._run_gate()
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertEqual(payload.get("archive_pr_count"), 34)
        self.assertEqual(payload.get("checkpoint_pr_count"), 35)
        self.assertEqual(payload.get("active_phase"), "P7")
        self.assertEqual(
            payload.get("action_tasks"),
            ["P7-T1", "P7-T2", "P7-T3", "P7-T4"],
        )
        self.assertIs(payload.get("adversarial_findings_resolved"), False)
        self.assertIs(payload.get("ok"), True)

    def test_gate_rejects_corrupted_recorded_sha(self) -> None:
        state = json.loads((ROOT / "ops/mission/state.json").read_text())
        checkpoint = state["mainlineCheckpoint"]
        archived = checkpoint.get("archivedFirstParentRange", checkpoint)
        mapping = archived.get("pullRequestToMainSha", archived.get("prToMainSha"))
        first_pr = next(iter(mapping))
        mapping[first_pr] = "0" * 40

        proc, payload = self._run_gate(state=state)
        self.assertNotEqual(proc.returncode, 0)
        self.assertTrue(
            any("SHA mapping" in finding for finding in payload.get("findings", [])),
            payload,
        )

    def test_gate_rejects_missing_p7_task(self) -> None:
        state = json.loads((ROOT / "ops/mission/state.json").read_text())
        p7 = next(phase for phase in state["phases"] if phase["id"] == "P7")
        p7["tasks"] = [task for task in p7["tasks"] if task["id"] != "P7-T4"]

        proc, payload = self._run_gate(state=state)
        self.assertNotEqual(proc.returncode, 0)
        self.assertTrue(
            any("P7-T4" in finding for finding in payload.get("findings", [])),
            payload,
        )

    def test_gate_rejects_stale_unmaterialized_resume_phase(self) -> None:
        state = json.loads((ROOT / "ops/mission/state.json").read_text())
        state["resume"]["nextActions"].append("P8: stale unmaterialized phase")

        proc, payload = self._run_gate(state=state)
        self.assertNotEqual(proc.returncode, 0)
        self.assertTrue(
            any("P8/P9/P10" in finding for finding in payload.get("findings", [])),
            payload,
        )

    def test_gate_rejects_resolved_adversarial_findings_claim(self) -> None:
        plan = json.loads((ROOT / "ops/lane0/plan.lock.json").read_text())
        plan["verdict"]["adversarialFindingsResolved"] = True

        proc, payload = self._run_gate(plan=plan)
        self.assertNotEqual(proc.returncode, 0)
        self.assertTrue(
            any(
                "adversarialFindingsResolved" in finding
                for finding in payload.get("findings", [])
            ),
            payload,
        )

    def test_frozen_p0_gates_invoke_durable_checker(self) -> None:
        plan = json.loads((ROOT / "ops/lane0/plan.lock.json").read_text())
        p0 = next(phase for phase in plan["phases"] if phase["id"] == "P0")
        commands = [gate["command"] for gate in p0["gates"]]
        self.assertEqual(
            commands,
            [
                "python3 -I scripts/test_garnet_lane0_truth_freeze_status.py",
                "python3 -I scripts/garnet_lane0_truth_freeze_status.py --gate",
            ],
        )

    def test_checker_is_dependency_free_and_copyable(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            copied = Path(td) / SCRIPT.name
            shutil.copy2(SCRIPT, copied)
            proc = subprocess.run(
                [sys.executable, "-I", str(copied), "--help"],
                text=True,
                capture_output=True,
                check=False,
            )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("--repo-root", proc.stdout)

    def test_sotu_distinguishes_compact_log_from_full_checkpoint(self) -> None:
        fixture = {
            "version": 1,
            "mission": {
                "name": "Renderer fixture",
                "repo": "Island-Dev-Crew/garnet",
                "status": "active",
            },
            "phases": [],
            "metrics": [],
            "risks": [],
            "prLog": [
                {"number": 472, "title": "bootstrap", "phase": "P0"},
                {"number": 473, "title": "acceptance", "phase": "P0"},
            ],
            "resume": {"activePhase": "P7", "nextActions": [], "blockers": []},
            "mainlineCheckpoint": {
                "source": "fixture",
                "archivedFirstParentRange": {
                    "baseExclusiveMainSha": "d0d4f7cc988db4d793c5c7555a4043aef9b27180",
                    "headInclusiveMainSha": "1fe74892c588f912e103742afc9d11e845e8d4e6",
                    "firstParentPullRequestCommitCount": 2,
                    "firstParentPullRequestMergeOrder": [472, 473],
                    "pullRequestToMainSha": {
                        "472": "4168582c46e60612af23f225e10483e98ddc897e",
                        "473": "155dec9c302099798b4057aabca3757d379cd25d",
                    },
                },
                "successorArchiveMerge": {
                    "pullRequest": 499,
                    "mainSha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
                    "immediatelyFollowsArchivedRange": True,
                },
                "checkpointPullRequestCommitCount": 3,
                "checkpointFirstParentPullRequestMergeOrder": [472, 473, 499],
            },
        }
        with tempfile.TemporaryDirectory() as td:
            tmp = Path(td)
            shutil.copy2(ROOT / "ops/mission/render-sotu.mjs", tmp / "render-sotu.mjs")
            (tmp / "state.json").write_text(json.dumps(fixture), encoding="utf-8")
            proc = subprocess.run(
                ["node", str(tmp / "render-sotu.mjs")],
                cwd=tmp,
                text=True,
                capture_output=True,
                check=False,
            )
            html = (tmp / "state-of-the-union.html").read_text(encoding="utf-8")

        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertIn("2 compact PR log rows", html)
        self.assertIn("3 checkpoint PR merges", html)
        self.assertIn("Mission PR Log (compact, 2 rows)", html)
        self.assertIn("Mainline Checkpoint (3 first-parent PR merges; successor #499)", html)
        self.assertIn("231aefa91985e5a0520c493c7f0fc3e54d74efc8", html)
        self.assertNotIn("2 PRs merged", html)
        self.assertIn("compact PR log rows: 2", proc.stdout)
        self.assertIn("checkpoint PR merges: 3", proc.stdout)


if __name__ == "__main__":
    unittest.main()
