#!/usr/bin/env python3
"""Tests for the exact S114 capability current-truth fixture."""
from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

SCRIPT = Path(__file__).with_name("garnet_capability_scope_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_capability_scope_status", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
mod = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_capability_scope_status"] = mod
SPEC.loader.exec_module(mod)


def _temp_file(text: str, suffix: str = ".txt") -> Path:
    handle = tempfile.NamedTemporaryFile(
        "w", suffix=suffix, delete=False, encoding="utf-8"
    )
    handle.write(text)
    handle.close()
    return Path(handle.name)


class CurrentTreeTests(unittest.TestCase):
    def test_real_tree_passes(self) -> None:
        status = mod.read_status()
        self.assertTrue(status.ok, status.problems)
        self.assertEqual("garnet.capability_scope/v2", status.schema)
        self.assertEqual(2, status.enforced_claim_count)
        self.assertTrue(status.enforced_claim_hashes_match)
        self.assertEqual("accepted-scoped", status.acceptance_state)
        self.assertEqual(
            "condition-5-reopened-by-post-acceptance-delta-review",
            status.post_acceptance_closure_state,
        )
        self.assertEqual([], status.current_truth_missing)
        self.assertEqual([], status.forbidden_hits)
        self.assertEqual([], status.stale_truth_hits)

    def test_gate_exits_zero_on_clean_tree(self) -> None:
        proc = subprocess.run(
            [sys.executable, str(SCRIPT), "--gate", "--format", "json"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(0, proc.returncode, proc.stderr)


class FailurePathTests(unittest.TestCase):
    def test_forbidden_phrase_fails(self) -> None:
        path = _temp_file("<p>CapCaps means no ambient authority, ever.</p>", ".html")
        try:
            with mock.patch.object(mod, "PUBLIC_SURFACES", [path]):
                status = mod.read_status()
            self.assertFalse(status.ok)
            self.assertTrue(any("no ambient authority" in hit for hit in status.forbidden_hits))
        finally:
            os.unlink(path)

    def test_semantic_claim_change_fails_even_with_two_markers(self) -> None:
        real = mod.WHY_HTML.read_text(encoding="utf-8")
        snippets = mod.CANONICAL_TRUTH_SNIPPETS[mod.WHY_HTML]
        changed = real.replace("rejects a <code>@caps()</code> test", "permits a <code>@caps()</code> test")
        path = _temp_file(changed, ".html")
        try:
            with mock.patch.object(mod, "WHY_HTML", path), mock.patch.object(
                mod, "CANONICAL_TRUTH_SNIPPETS", {path: snippets}
            ):
                status = mod.read_status()
            self.assertEqual(2, status.enforced_claim_count)
            self.assertFalse(status.enforced_claim_hashes_match)
            self.assertFalse(status.ok)
        finally:
            os.unlink(path)

    def test_extra_enforced_claim_fails(self) -> None:
        real = mod.WHY_HTML.read_text(encoding="utf-8")
        path = _temp_file(real + "\n<li><b>enforced:</b> extra</li>\n", ".html")
        try:
            with mock.patch.object(mod, "WHY_HTML", path):
                status = mod.read_status()
            self.assertFalse(status.ok)
            self.assertEqual(3, status.enforced_claim_count)
        finally:
            os.unlink(path)

    def test_acceptance_current_state_drift_fails(self) -> None:
        real = mod.WHY_HTML.read_text(encoding="utf-8")
        snippets = mod.CANONICAL_TRUTH_SNIPPETS[mod.WHY_HTML]
        changed = real.replace(
            "S114 acceptance is recorded as <code>accepted-scoped</code>",
            "S114 acceptance remains <code>external-pending</code>",
        )
        path = _temp_file(changed, ".html")
        try:
            with mock.patch.object(mod, "WHY_HTML", path), mock.patch.object(
                mod, "CANONICAL_TRUTH_SNIPPETS", {path: snippets}
            ), mock.patch.object(mod, "CURRENT_TRUTH_SURFACES", [path]):
                status = mod.read_status()
            self.assertFalse(status.ok)
            self.assertTrue(status.current_truth_missing)
        finally:
            os.unlink(path)

    def test_stale_phrase_fails(self) -> None:
        path = _temp_file("S114 is pending Jon's acceptance", ".md")
        try:
            with mock.patch.object(mod, "CURRENT_TRUTH_SURFACES", [path]):
                status = mod.read_status()
            self.assertFalse(status.ok)
            self.assertTrue(status.stale_truth_hits)
        finally:
            os.unlink(path)

    def test_reopened_condition_metadata_is_required(self) -> None:
        payload = json.loads(mod.ACCEPTANCE_JSON.read_text(encoding="utf-8"))
        payload["post_acceptance_closure"]["condition_states"]["5"] = "closed"
        path = _temp_file(json.dumps(payload), ".json")
        try:
            with mock.patch.object(mod, "ACCEPTANCE_JSON", path):
                status = mod.read_status()
            self.assertFalse(status.ok)
            self.assertTrue(any("acceptance/current-closure" in p for p in status.problems))
        finally:
            os.unlink(path)

    def test_missing_scope_doc_fails(self) -> None:
        with mock.patch.object(mod, "SCOPE_DOC", Path("/nonexistent/scope.md")):
            status = mod.read_status()
        self.assertFalse(status.ok)
        self.assertFalse(status.scope_doc_present)

    def test_missing_cited_anchor_fails(self) -> None:
        with mock.patch.object(mod, "CITED_TEST_ANCHORS", [Path("/nonexistent/test_x.rs")]):
            status = mod.read_status()
        self.assertFalse(status.ok)
        self.assertTrue(status.cited_anchors_missing)


if __name__ == "__main__":
    unittest.main()
