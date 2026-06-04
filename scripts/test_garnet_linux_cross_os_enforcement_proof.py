#!/usr/bin/env python3
"""Regression tests for the S108 Linux enforcement proof gate."""
from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_linux_cross_os_enforcement_proof.py")
SPEC = importlib.util.spec_from_file_location(
    "garnet_linux_cross_os_enforcement_proof", SCRIPT
)
assert SPEC is not None and SPEC.loader is not None
s108 = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_linux_cross_os_enforcement_proof"] = s108
SPEC.loader.exec_module(s108)


def _write_manifest(bundle: Path) -> None:
    lines: list[str] = []
    for path in sorted(p for p in bundle.rglob("*") if p.is_file()):
        if path.name == "MANIFEST.sha256":
            continue
        lines.append(f"{s108._sha256(path)}  {path.relative_to(bundle).as_posix()}")
    (bundle / "MANIFEST.sha256").write_text("\n".join(lines) + "\n", encoding="utf-8")


def _write_verified_bundle(
    bundle: Path,
    *,
    seccomp_status: str = "proven",
    deterministic_runs: int = 3,
) -> Path:
    (bundle / "commands").mkdir(parents=True)
    commands = []
    for command_id in ("linux-s101-gate", "linux-bounded-enforcement", "linux-caps-enforcement"):
        stdout = bundle / "commands" / f"{command_id}-stdout.txt"
        stderr = bundle / "commands" / f"{command_id}-stderr.txt"
        stdout.write_text(f"{command_id} ok\n", encoding="utf-8")
        stderr.write_text("", encoding="utf-8")
        commands.append(
            {
                "id": command_id,
                "display_args": ["test", command_id],
                "exit_code": 0,
                "stdout_file": stdout.relative_to(bundle).as_posix(),
                "stderr_file": stderr.relative_to(bundle).as_posix(),
                "status": "passed",
            }
        )
    seccomp_stdout = bundle / "commands" / "linux-seccomp-apply-stdout.txt"
    seccomp_stderr = bundle / "commands" / "linux-seccomp-apply-stderr.txt"
    seccomp_stdout.write_text(
        "BLOCKED (trap) Operation not permitted\n"
        "ALLOWED policy-driven\n"
        "OK: generated seccomp policy is APPLIED and TRAPS\n",
        encoding="utf-8",
    )
    seccomp_stderr.write_text("", encoding="utf-8")
    commands.append(
        {
            "id": "linux-seccomp-apply",
            "display_args": ["tools/seccomp-apply/prove.sh"],
            "exit_code": 0,
            "stdout_file": seccomp_stdout.relative_to(bundle).as_posix(),
            "stderr_file": seccomp_stderr.relative_to(bundle).as_posix(),
            "status": "passed",
        }
    )
    summary = {
        "schema": "garnet.linux_cross_os_enforcement_proof.v1",
        "platform": "linux",
        "cross_os_role": "S108 Linux row for S109 consolidation",
        "status": "passed",
        "tier": "linux-enforcement-proof",
        "environment": {
            "kind": "utm-debian-12-arm64",
            "kernel": "Linux debian 6.1.0-13-arm64 aarch64",
        },
        "stage_v_traps": [
            {"trap": "max_depth", "status": "passed"},
            {"trap": "caps", "status": "passed"},
            {"trap": "s92_program_entry_proc", "status": "passed"},
        ],
        "seccomp": {
            "attempted": True,
            "status": seccomp_status,
            "denied_socket_trapped": seccomp_status == "proven",
            "allowed_socket_policy_driven": seccomp_status == "proven",
            "deterministic_denied_runs": deterministic_runs,
        },
        "commands": commands,
        "honest_scope": [
            "This is the independent Linux S108 enforcement row for S109 consolidation.",
            "Linux seccomp is Linux-only evidence, not Windows/macOS OS-sandbox enforcement.",
            "This is not full S109 completion; S109 still needs a separate consolidation gate update.",
            "No Wasmtime fuel, production, or v1.0 claim is made.",
        ],
    }
    summary_path = bundle / "garnet-linux-cross-os-enforcement-proof.json"
    summary_path.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    (bundle / "garnet-linux-cross-os-enforcement-proof.md").write_text(
        s108.render_markdown(summary),
        encoding="utf-8",
    )
    _write_manifest(bundle)
    return summary_path


class LinuxCrossOsEnforcementProofTests(unittest.TestCase):
    def test_verifier_accepts_linux_s108_bundle(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            summary = _write_verified_bundle(Path(tmp))

            self.assertTrue(s108.verify_bundle(summary))

    def test_verifier_rejects_missing_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            summary = _write_verified_bundle(Path(tmp))
            (Path(tmp) / "MANIFEST.sha256").unlink()

            self.assertFalse(s108.verify_bundle(summary))

    def test_verifier_rejects_unproven_seccomp_claim(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            summary = _write_verified_bundle(Path(tmp), seccomp_status="not-run")

            self.assertFalse(s108.verify_bundle(summary))

    def test_verifier_rejects_nondeterministic_seccomp_trap(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            summary = _write_verified_bundle(Path(tmp), deterministic_runs=1)

            self.assertFalse(s108.verify_bundle(summary))

    def test_committed_evidence_finds_verified_bundle(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            bundle = repo / "proofs" / "linux" / "enforcement" / "utm-linux-enforcement-test"
            bundle.mkdir(parents=True)
            _write_verified_bundle(bundle)

            evidence = s108.read_committed_evidence(repo)

            self.assertTrue(evidence.verified)
            self.assertIn("S108 Linux enforcement proof", evidence.reason)
            self.assertIn("not Windows/macOS", " ".join(evidence.deferred))


if __name__ == "__main__":
    unittest.main()
