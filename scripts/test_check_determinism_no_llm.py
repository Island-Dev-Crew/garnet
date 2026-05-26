#!/usr/bin/env python3
"""Tests for the S19 determinism guard."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("check_determinism_no_llm.py")
SPEC = importlib.util.spec_from_file_location("check_determinism_no_llm", SCRIPT)
assert SPEC is not None
guard = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["check_determinism_no_llm"] = guard
SPEC.loader.exec_module(guard)


class DeterminismNoLlmTests(unittest.TestCase):
    def test_current_determinism_workflow_excludes_llm_flag(self) -> None:
        self.assertEqual([], guard.scan_path())

    def test_scanner_reports_forbidden_llm_flag(self) -> None:
        text = """
jobs:
  build:
    steps:
      - run: garnet build --deterministic --sign key examples/det.garnet
      - run: garnet check --suggest --llm anthropic examples/det.garnet
"""
        violations = guard.scan_text(text)

        self.assertEqual(1, len(violations))
        self.assertEqual(6, violations[0][0])
        self.assertIn("--llm", violations[0][1])


if __name__ == "__main__":
    unittest.main()
