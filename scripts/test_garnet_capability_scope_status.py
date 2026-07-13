#!/usr/bin/env python3
"""Tests for the capability-claim scope fixture (S114 condition #3 + #4)."""
from __future__ import annotations

import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path
from unittest import mock

SCRIPT = Path(__file__).with_name("garnet_capability_scope_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_capability_scope_status", SCRIPT)
assert SPEC is not None
mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_capability_scope_status"] = mod
SPEC.loader.exec_module(mod)


class CurrentTreeTests(unittest.TestCase):
    def test_real_tree_passes(self) -> None:
        s = mod.read_status()
        self.assertTrue(s.ok, s.problems)
        self.assertTrue(s.scope_doc_present)
        self.assertEqual(2, s.enforced_claim_count)
        self.assertEqual([], s.cited_anchors_missing)
        self.assertEqual([], s.forbidden_hits)

    def test_gate_exits_zero_on_clean_tree(self) -> None:
        proc = subprocess.run(
            [sys.executable, str(SCRIPT), "--gate", "--format", "json"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(0, proc.returncode, proc.stderr)


class FailurePathTests(unittest.TestCase):
    def test_forbidden_phrase_fails(self) -> None:
        # Inject a forbidden phrase by pointing a public surface at a temp file.
        import os
        import tempfile

        with tempfile.NamedTemporaryFile(
            "w", suffix=".html", delete=False, encoding="utf-8"
        ) as fh:
            fh.write("<p>CapCaps means no ambient authority, ever.</p>")
            tmp = fh.name
        try:
            with mock.patch.object(mod, "PUBLIC_SURFACES", [Path(tmp)]):
                s = mod.read_status()
            self.assertFalse(s.ok)
            self.assertTrue(any("no ambient authority" in h for h in s.forbidden_hits))
        finally:
            os.unlink(tmp)

    def test_extra_enforced_claim_fails(self) -> None:
        import os
        import tempfile

        with tempfile.NamedTemporaryFile(
            "w", suffix=".html", delete=False, encoding="utf-8"
        ) as fh:
            fh.write("<b>enforced:</b> a <b>enforced:</b> b <b>enforced:</b> c")
            tmp = fh.name
        try:
            with mock.patch.object(mod, "WHY_HTML", Path(tmp)):
                s = mod.read_status()
            self.assertFalse(s.ok)
            self.assertEqual(3, s.enforced_claim_count)
        finally:
            os.unlink(tmp)

    def test_missing_scope_doc_fails(self) -> None:
        with mock.patch.object(mod, "SCOPE_DOC", Path("/nonexistent/scope.md")):
            s = mod.read_status()
        self.assertFalse(s.ok)
        self.assertFalse(s.scope_doc_present)

    def test_missing_cited_anchor_fails(self) -> None:
        with mock.patch.object(
            mod, "CITED_TEST_ANCHORS", [Path("/nonexistent/test_x.rs")]
        ):
            s = mod.read_status()
        self.assertFalse(s.ok)
        self.assertTrue(s.cited_anchors_missing)


if __name__ == "__main__":
    unittest.main()
