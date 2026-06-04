#!/usr/bin/env python3
"""Regression tests for the consolidated Linux/Tauri gate replay proof."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("smoke_garnet_studio_linux_gate_replay.py")
SPEC = importlib.util.spec_from_file_location("smoke_garnet_studio_linux_gate_replay", SCRIPT)
assert SPEC is not None
gate_replay = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["smoke_garnet_studio_linux_gate_replay"] = gate_replay
SPEC.loader.exec_module(gate_replay)


def _write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


def _hash(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _write_manifest(bundle: Path) -> None:
    lines = []
    for path in sorted(p for p in bundle.rglob("*") if p.is_file()):
        if path.name == gate_replay.MANIFEST_NAME:
            continue
        lines.append(f"{_hash(path)}  {path.relative_to(bundle).as_posix()}")
    _write(bundle / gate_replay.MANIFEST_NAME, "\n".join(lines) + "\n")


def _valid_bundle(bundle: Path, *, omit_gate: str | None = None, child_verified: bool = True) -> Path:
    gates = []
    for spec in gate_replay.GATE_SPECS:
        if spec.id == omit_gate:
            continue
        stdout_file = f"commands/{spec.id}-stdout.json"
        stderr_file = f"commands/{spec.id}-stderr.txt"
        _write(
            bundle / stdout_file,
            json.dumps(
                {
                    "ok": child_verified,
                    "verified": child_verified,
                    "status": "passed" if child_verified else "missing",
                }
            )
            + "\n",
        )
        _write(bundle / stderr_file, "")
        gates.append(
            {
                "id": spec.id,
                "label": spec.label,
                "script": spec.script,
                "exit_code": 0,
                "status": "passed",
                "verified": True,
                "stdout_file": stdout_file,
                "stderr_file": stderr_file,
            }
        )
    data = {
        "schema": gate_replay.SCHEMA,
        "status": "passed",
        "platform_tier": gate_replay.PLATFORM_TIER,
        "source_included": False,
        "provider_api_called": False,
        "all_current_linux_gates_replayed": True,
        "wsl_is_enforcement": False,
        "linux_enforcement_proven": False,
        "linux_seccomp_proven": False,
        "os_sandbox_enforcement_proven": False,
        "clean_linux_desktop_proven": False,
        "non_wsl_linux_desktop_proven": False,
        "signed_release_proven": False,
        "production_readiness_claimed": False,
        "v1_readiness_claimed": False,
        "gates": gates,
        "honest_scope": list(gate_replay.REQUIRED_HONEST_SCOPE),
    }
    summary = bundle / gate_replay.SUMMARY_NAME
    _write(summary, json.dumps(data, indent=2) + "\n")
    _write(bundle / gate_replay.MARKDOWN_NAME, gate_replay.render_markdown(data))
    _write_manifest(bundle)
    return summary


class LinuxGateReplayProofTests(unittest.TestCase):
    def test_valid_replay_bundle_verifies_and_reads_as_committed_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            summary = _valid_bundle(
                root / "proofs" / "linux" / "execution" / "studio-gate-replay" / "linux-gate-replay-test"
            )

            self.assertTrue(gate_replay.verify_bundle(summary))
            evidence = gate_replay.read_committed_evidence(root)

        self.assertTrue(evidence.verified)
        self.assertEqual("verified", evidence.status)
        self.assertIn("all current Linux/Tauri gates", evidence.reason)
        self.assertIn("not Linux seccomp", " ".join(evidence.deferred))

    def test_replay_bundle_fails_when_a_required_gate_is_missing(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            summary = _valid_bundle(Path(temp), omit_gate=gate_replay.GATE_SPECS[0].id)

            self.assertFalse(gate_replay.verify_bundle(summary))

    def test_replay_bundle_fails_when_child_gate_output_is_not_verified(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            summary = _valid_bundle(Path(temp), child_verified=False)

            self.assertFalse(gate_replay.verify_bundle(summary))

    def test_replay_bundle_fails_on_enforcement_or_production_overclaim(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = Path(temp)
            summary = _valid_bundle(bundle)
            data = json.loads(summary.read_text(encoding="utf-8"))
            data["linux_seccomp_proven"] = True
            data["production_readiness_claimed"] = True
            data["honest_scope"].append("Linux seccomp enforced")
            _write(summary, json.dumps(data, indent=2) + "\n")
            _write_manifest(bundle)

            self.assertFalse(gate_replay.verify_bundle(summary))

    def test_record_replay_runs_each_child_gate_and_writes_a_verifiable_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle = Path(temp)
            calls: list[list[str]] = []

            def fake_runner(args: list[str], **_: object) -> subprocess.CompletedProcess[str]:
                calls.append(args)
                return subprocess.CompletedProcess(
                    args,
                    0,
                    stdout=json.dumps({"ok": True, "verified": True, "status": "passed"}) + "\n",
                    stderr="",
                )

            summary = gate_replay.record_replay(bundle, runner=fake_runner)

            self.assertTrue(gate_replay.verify_bundle(summary))
            self.assertEqual([spec.script for spec in gate_replay.GATE_SPECS], [Path(call[1]).name for call in calls])

    def test_markdown_lists_gate_ids_and_boundaries(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            data = json.loads(_valid_bundle(Path(temp)).read_text(encoding="utf-8"))

        markdown = gate_replay.render_markdown(data)

        for spec in gate_replay.GATE_SPECS:
            self.assertIn(spec.id, markdown)
        self.assertIn("execution/portability", markdown)
        self.assertIn("not clean/non-WSL Linux desktop proof", markdown)
        self.assertIn("not Linux seccomp or OS-sandbox enforcement", markdown)


if __name__ == "__main__":
    unittest.main()
