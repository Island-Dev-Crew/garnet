#!/usr/bin/env python3
"""Tests for the rolling S114 trust-kernel review gate."""
from __future__ import annotations

import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path
from unittest import mock

SCRIPT = Path(__file__).with_name("garnet_trust_kernel_review_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_trust_kernel_review_status", SCRIPT)
assert SPEC is not None
mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_trust_kernel_review_status"] = mod
SPEC.loader.exec_module(mod)


class ClassificationTests(unittest.TestCase):
    def test_trust_kernel_paths_are_recognized(self) -> None:
        for p in (
            "garnet-check-v0.3/src/caps_graph.rs",
            "garnet-interp-v0.3/src/eval.rs",
            "garnet-vm/src/vm.rs",
            "garnet-stdlib/src/registry.rs",
            "garnet-wasm/src/lib.rs",
            "garnet-cli/src/cmd/run.rs",
            "garnet-cli/src/bin/garnet.rs",
            "scripts/garnet_capability_scope_status.py",
            "scripts/garnet_required_context_contract.py",
            "scripts/test_garnet_required_context_contract.py",
            ".github/workflows/ci.yml",
            ".github/rulesets/garnet-main.json",
            "docs/why.html",
            "C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md",
        ):
            self.assertTrue(mod.is_trust_kernel(p), p)
            self.assertTrue(mod.is_trust_kernel(p.replace("/", "\\")), p)

    def test_non_trust_kernel_paths_are_ignored(self) -> None:
        for p in ("README.md", "docs/index.html", "ops/mission/state.json", "apps/garnet-studio/x"):
            self.assertFalse(mod.is_trust_kernel(p), p)

    def test_norm_strips_dot_slash_as_prefix_not_char_set(self) -> None:
        # Regression: lstrip("./") strips '.'/'/' as a character set, which
        # would eat the leading dot off ".github/..."-style names (the bug that
        # hid workflow files from the dogfood PR-body gate).
        self.assertEqual(".github/workflows/ci.yml", mod._norm(".github/workflows/ci.yml"))
        self.assertEqual("garnet-vm/src/vm.rs", mod._norm("./garnet-vm/src/vm.rs"))
        self.assertTrue(mod.is_trust_kernel("./garnet-interp-v0.3/src/eval.rs"))

    def test_review_companions_are_recognized(self) -> None:
        for p in (
            "proofs/independent/s114/codex-verdict-20260625/MANIFEST.sha256",
            "F_Project_Management/W_TRUST/S114_ACCEPTANCE_RECORD_2026-07-12.md",
            "F_Project_Management/VALIDATION_REPORTS/x.md",
            "F_Project_Management/LAUNCH/S114_ACCEPTANCE.json",
        ):
            self.assertTrue(mod.is_review_companion(p), p)


class GateLogicTests(unittest.TestCase):
    def test_git_diff_decomposes_renames(self) -> None:
        result = subprocess.CompletedProcess([], 0, ".github/rulesets/old.json\ndocs/new.json\n", "")
        with mock.patch.object(mod, "_git", return_value=result) as git:
            self.assertEqual(2, len(mod._changed_from_git("base", "head")))
        git.assert_called_once_with("diff", "--no-renames", "--name-only", "base...head")

    def test_trust_kernel_without_companion_fails(self) -> None:
        s = mod.read_status(changed=["garnet-interp-v0.3/src/eval.rs"], trailer=False)
        self.assertTrue(s.trust_kernel_touched)
        self.assertFalse(s.review_companion_present)
        self.assertFalse(s.ok)

    def test_trust_kernel_with_companion_file_passes(self) -> None:
        s = mod.read_status(
            changed=[
                "garnet-interp-v0.3/src/eval.rs",
                "F_Project_Management/LAUNCH/S114_ACCEPTANCE.json",
            ],
            trailer=False,
        )
        self.assertTrue(s.trust_kernel_touched)
        self.assertTrue(s.review_companion_present)
        self.assertTrue(s.ok)

    def test_trust_kernel_with_trailer_passes(self) -> None:
        s = mod.read_status(changed=["garnet-vm/src/vm.rs"], trailer=True)
        self.assertTrue(s.trust_kernel_touched)
        self.assertTrue(s.review_companion_present)
        self.assertTrue(s.ok)

    def test_non_trust_kernel_change_passes(self) -> None:
        s = mod.read_status(changed=["README.md", "ops/mission/state.json"], trailer=False)
        self.assertFalse(s.trust_kernel_touched)
        self.assertTrue(s.ok)

    def test_empty_change_set_passes(self) -> None:
        s = mod.read_status(changed=[], trailer=False)
        self.assertFalse(s.trust_kernel_touched)
        self.assertTrue(s.ok)


class CliTests(unittest.TestCase):
    def _run(self, *args: str) -> subprocess.CompletedProcess:
        return subprocess.run(
            [sys.executable, str(SCRIPT), *args], capture_output=True, text=True
        )

    def test_gate_exits_1_on_uncompanioned_trust_kernel(self) -> None:
        p = self._run(
            "--gate", "--format", "json", "--changed-file", "garnet-interp-v0.3/src/eval.rs"
        )
        self.assertEqual(1, p.returncode, p.stdout)

    def test_gate_exits_0_with_companion(self) -> None:
        p = self._run(
            "--gate",
            "--format",
            "json",
            "--changed-file",
            "garnet-interp-v0.3/src/eval.rs",
            "--changed-file",
            "F_Project_Management/LAUNCH/S114_ACCEPTANCE.json",
        )
        self.assertEqual(0, p.returncode, p.stderr)

    def test_gate_exits_0_with_assume_trailer(self) -> None:
        p = self._run(
            "--gate",
            "--changed-file",
            "garnet-vm/src/vm.rs",
            "--assume-trailer",
        )
        self.assertEqual(0, p.returncode, p.stderr)

    def test_gate_exits_0_on_non_trust_kernel(self) -> None:
        p = self._run("--gate", "--changed-file", "README.md")
        self.assertEqual(0, p.returncode, p.stderr)


if __name__ == "__main__":
    unittest.main()
