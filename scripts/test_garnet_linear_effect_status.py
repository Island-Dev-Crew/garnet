#!/usr/bin/env python3
"""Regression tests for the S96 linear/effect safe-mode status gate."""
from __future__ import annotations

import importlib.util
import io
import sys
import unittest
from contextlib import redirect_stdout
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_linear_effect_status.py")
linear_effect_status = None
LOAD_ERROR: Exception | None = None

if SCRIPT.is_file():
    SPEC = importlib.util.spec_from_file_location("garnet_linear_effect_status", SCRIPT)
    assert SPEC is not None and SPEC.loader is not None
    linear_effect_status = importlib.util.module_from_spec(SPEC)
    sys.modules["garnet_linear_effect_status"] = linear_effect_status
    try:
        SPEC.loader.exec_module(linear_effect_status)
    except Exception as exc:  # pragma: no cover - reported by the first test.
        LOAD_ERROR = exc
else:
    LOAD_ERROR = FileNotFoundError(SCRIPT)


class LinearEffectStatusTests(unittest.TestCase):
    def status(self):
        if LOAD_ERROR is not None:
            self.fail(f"could not load S96 status reporter: {LOAD_ERROR}")
        assert linear_effect_status is not None
        return linear_effect_status.read_status()

    def test_gate_passes_on_real_repo(self) -> None:
        if LOAD_ERROR is not None:
            self.fail(f"could not load S96 status reporter: {LOAD_ERROR}")
        assert linear_effect_status is not None
        with redirect_stdout(io.StringIO()):
            self.assertEqual(linear_effect_status.main(["--gate", "--format", "json"]), 0)

    def test_source_inventory_is_present(self) -> None:
        status = self.status()
        self.assertTrue(status.effects_module_present)
        self.assertTrue(status.check_error_present)
        self.assertTrue(status.focused_tests_present)
        self.assertTrue(status.readiness_lane_present)

    def test_scope_text_stays_calibrated(self) -> None:
        status = self.status()
        self.assertIn("static", status.scope_summary)
        self.assertIn("not whole-language", status.scope_summary)
        assert linear_effect_status is not None
        md = linear_effect_status.render_markdown(status)
        self.assertIn("No VM or OS sandbox enforcement is claimed", md)


if __name__ == "__main__":
    unittest.main()
