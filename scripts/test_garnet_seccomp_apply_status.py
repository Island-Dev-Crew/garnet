#!/usr/bin/env python3
"""Regression tests for the OS-sandbox apply gate (UTM seccomp slice)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_seccomp_apply_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_seccomp_apply_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
sa = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_seccomp_apply_status"] = sa
SPEC.loader.exec_module(sa)


class SeccompApplyTests(unittest.TestCase):
    def test_harness_applies_generated_policy(self) -> None:
        r = sa.read_status()
        self.assertTrue(r.harness_present)
        self.assertTrue(r.applies_generated_policy)

    def test_proof_recorded_and_deterministic(self) -> None:
        r = sa.read_status()
        self.assertTrue(r.proof_recorded)
        self.assertTrue(r.proof_deterministic)

    def test_policy_driven(self) -> None:
        self.assertTrue(sa.read_status().policy_driven)

    def test_doc_and_honesty(self) -> None:
        r = sa.read_status()
        self.assertTrue(r.doc_present)
        self.assertTrue(r.honesty_anchor_present)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(sa.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_linux_only(self) -> None:
        md = sa.render_markdown(sa.read_status())
        self.assertIn("Linux seccomp only", md)
        self.assertIn("named-deferred", md)


if __name__ == "__main__":
    unittest.main()
