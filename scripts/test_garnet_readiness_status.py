#!/usr/bin/env python3
"""Regression tests for the Garnet readiness status reporter."""
from __future__ import annotations

import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("garnet_readiness_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_readiness_status", SCRIPT)
assert SPEC is not None
status_mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_readiness_status"] = status_mod
SPEC.loader.exec_module(status_mod)


class GarnetReadinessStatusTests(unittest.TestCase):
    def test_counts_plan_checkboxes_and_open_slices(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            plan = Path(tmp) / "PLAN.md"
            plan.write_text(
                "\n".join(
                    [
                        "## Milestone 1",
                        "",
                        "- [x] **Step 1: Done**",
                        "- [ ] **Step 2: Open**",
                        "### Detail",
                        "- [X] **Step 3: Also done**",
                    ]
                ),
                encoding="utf-8",
            )

            result = status_mod.read_status(plan)

        self.assertEqual(3, result.total_slices)
        self.assertEqual(2, result.completed_slices)
        self.assertEqual(66.7, result.completion_percent)
        self.assertEqual(["Step 2: Open"], [item.title for item in result.open_slices])
        self.assertEqual("Milestone 1", result.open_slices[0].section)

    def test_markdown_names_open_slices(self) -> None:
        result = status_mod.ReadinessStatus(
            source="PLAN.md",
            total_slices=2,
            completed_slices=1,
            completion_percent=50.0,
            open_slices=[
                status_mod.SliceStatus(
                    title="Step 2: Open",
                    done=False,
                    section="Milestone 1",
                )
            ],
        )

        rendered = status_mod.render_markdown(result)

        self.assertIn("Completion: 1/2 slices (50.0%).", rendered)
        self.assertIn("`Milestone 1` - Step 2: Open", rendered)


if __name__ == "__main__":
    unittest.main()
