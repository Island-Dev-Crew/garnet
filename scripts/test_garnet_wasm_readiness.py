#!/usr/bin/env python3
"""Regression tests for the evidence-backed Wasm / W-PLAY reporter."""
from __future__ import annotations

import importlib.util
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

SCRIPT = Path(__file__).with_name("garnet_wasm_readiness.py")
SPEC = importlib.util.spec_from_file_location("garnet_wasm_readiness", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
wasm = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_wasm_readiness"] = wasm
SPEC.loader.exec_module(wasm)

ROOT = Path(__file__).resolve().parents[1]


class WasmReadinessTests(unittest.TestCase):
    def test_committed_wv5_build_and_node_proof_pass(self) -> None:
        result = wasm.read_readiness()
        self.assertEqual("garnet.wasm_readiness/v3", result.schema)
        self.assertTrue(result.wasm_crate_present)
        self.assertTrue(result.windows_proof_valid)
        self.assertTrue(result.wasm_build_passed)
        self.assertTrue(result.node_execution_passed)
        self.assertTrue(result.owned_bits_ready)

    def test_committed_browser_package_and_proof_promote_browser_readiness(self) -> None:
        result = wasm.read_readiness()
        self.assertTrue(result.browser_adapter_present)
        self.assertTrue(result.browser_package_valid)
        self.assertTrue(result.browser_proof_valid)
        self.assertTrue(result.browser_ready)
        self.assertEqual([], result.blockers)

    def test_local_tools_are_observations_not_product_blockers(self) -> None:
        with mock.patch.object(wasm, "_has_wasm32_target", return_value=False), mock.patch.object(
            wasm.shutil, "which", return_value=None
        ):
            result = wasm.read_readiness()
        self.assertTrue(result.owned_bits_ready)
        self.assertFalse(result.wasm32_target_installed)
        self.assertFalse(result.wasm_pack_present)
        self.assertFalse(result.node_present)
        joined = " ".join(result.blockers).lower()
        self.assertNotIn("rustup", joined)
        self.assertNotIn("wasm-pack absent", joined)
        self.assertNotIn("miette", joined)

    def test_missing_or_invalid_proof_fails_owned_gate(self) -> None:
        with tempfile.NamedTemporaryFile(
            "w", suffix=".json", delete=False, encoding="utf-8"
        ) as handle:
            json.dump({"schema": "wrong/v1", "verdict": "pass"}, handle)
            temp_path = Path(handle.name)
        try:
            with mock.patch.object(wasm, "WV5_PROOF", temp_path):
                result = wasm.read_readiness()
            self.assertFalse(result.windows_proof_valid)
            self.assertFalse(result.wasm_build_passed)
            self.assertFalse(result.node_execution_passed)
            self.assertFalse(result.owned_bits_ready)
        finally:
            os.unlink(temp_path)

    def test_node_semantic_markers_are_required(self) -> None:
        with tempfile.NamedTemporaryFile(
            "w", suffix=".txt", delete=False, encoding="utf-8"
        ) as handle:
            handle.write("NODE_SMOKE: PASS\n")
            temp_path = Path(handle.name)
        try:
            with mock.patch.object(wasm, "WV5_NODE_LOG", temp_path):
                result = wasm.read_readiness()
            self.assertFalse(result.windows_proof_valid)
            self.assertFalse(result.node_execution_passed)
        finally:
            os.unlink(temp_path)

    def test_open_w_play_surfaces_are_named_as_blockers(self) -> None:
        result = wasm.read_readiness()
        joined = " ".join(result.blockers)
        self.assertEqual(
            not result.check_source_export_present,
            "check_source" in joined,
        )
        self.assertEqual(
            not result.caps_surface_export_present,
            "capability-surface/diff" in joined,
        )
        self.assertEqual(
            not result.browser_adapter_present,
            "browser adapter" in joined,
        )
        self.assertEqual(
            not result.browser_proof_valid,
            "Playwright" in joined,
        )

    def test_invalid_browser_proof_cannot_promote_or_pass_the_gate(self) -> None:
        with tempfile.NamedTemporaryFile(
            "w", suffix=".json", delete=False, encoding="utf-8"
        ) as handle:
            json.dump({"schema": "garnet.w-play.browser-proof/1", "verdict": "pass"}, handle)
            temp_path = Path(handle.name)
        try:
            with mock.patch.object(wasm, "BROWSER_PROOF", temp_path):
                result = wasm.read_readiness()
                self.assertFalse(result.browser_proof_valid)
                self.assertFalse(result.browser_ready)
                self.assertEqual(1, wasm.main(["--gate", "--format", "json"]))
        finally:
            os.unlink(temp_path)

    def test_markdown_separates_node_proof_from_browser_claim(self) -> None:
        markdown = wasm.render_markdown(wasm.read_readiness())
        self.assertIn("real Node execution passed: True", markdown)
        self.assertIn("does not prove live browser-page execution", markdown)
        self.assertNotIn("no wasm is built", markdown.lower())

    def test_gate_guards_committed_build_execution_and_browser_evidence(self) -> None:
        self.assertEqual(0, wasm.main(["--gate", "--format", "json"]))

    def test_hello_example_declares_no_caps(self) -> None:
        text = (ROOT / "examples" / "hello.garnet").read_text(encoding="utf-8")
        self.assertIn("@caps()", text)
        self.assertIn("Hello from Garnet!", text)


if __name__ == "__main__":
    unittest.main()
