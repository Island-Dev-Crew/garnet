#!/usr/bin/env python3
"""Tests for the S92 spawn/FFI authority status gate."""
from __future__ import annotations

import unittest

import garnet_spawn_ffi_authority_status as status


class SpawnFfiAuthorityStatusTests(unittest.TestCase):
    def test_subprocess_entry_guard_present(self) -> None:
        report = status.read_status()
        self.assertTrue(report.subprocess_entry_guard_present)
        self.assertEqual(report.missing_subprocess_entry_gates, [])

    def test_caps_enforcement_regression_tests_present(self) -> None:
        self.assertTrue(status.read_status().runtime_tests_present)

    def test_ffi_runtime_scope_is_honest(self) -> None:
        report = status.read_status()
        self.assertFalse(report.ffi_runtime_bridge_present)
        self.assertIn("declared/diffed/sandbox-flagged", report.ffi_scope)
        self.assertIn("no executable FFI bridge", report.ffi_scope)

    def test_gate_is_ok(self) -> None:
        self.assertTrue(status.read_status().ok)


if __name__ == "__main__":
    unittest.main()
