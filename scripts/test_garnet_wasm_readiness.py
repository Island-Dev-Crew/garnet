#!/usr/bin/env python3
"""Regression tests for the WASM hello-world readiness reporter (S55)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_wasm_readiness.py")
SPEC = importlib.util.spec_from_file_location("garnet_wasm_readiness", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
wasm = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_wasm_readiness"] = wasm
SPEC.loader.exec_module(wasm)

ROOT = Path(__file__).resolve().parents[1]


class WasmReadinessTests(unittest.TestCase):
    def test_hello_example_and_doc_present(self) -> None:
        r = wasm.read_readiness()
        self.assertTrue(r.hello_example_present, "examples/hello.garnet must exist")
        self.assertTrue(r.target_doc_present, "GARNET_WASM_TARGET.md must exist")
        self.assertTrue(r.owned_bits_ready)

    def test_names_the_miette_fancy_blocker(self) -> None:
        # The concrete portability blocker must be surfaced, not hidden.
        r = wasm.read_readiness()
        self.assertTrue(r.miette_fancy_blocker)
        self.assertTrue(any("miette" in b for b in r.blockers))

    def test_gate_guards_owned_bits_only(self) -> None:
        # The absent toolchain is an honest deferral, not a gate failure.
        self.assertEqual(wasm.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_no_wasm_built(self) -> None:
        md = wasm.render_markdown(wasm.read_readiness())
        self.assertIn("no wasm is built", md)
        self.assertIn("DEFERRED", md)

    def test_hello_example_declares_no_caps(self) -> None:
        text = (ROOT / "examples" / "hello.garnet").read_text(encoding="utf-8")
        self.assertIn("@caps()", text)
        self.assertIn("Hello from Garnet!", text)


if __name__ == "__main__":
    unittest.main()
