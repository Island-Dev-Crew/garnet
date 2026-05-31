#!/usr/bin/env python3
"""Regression tests for the VS Code publish-readiness gate (S54)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_vscode_publish_readiness.py")
SPEC = importlib.util.spec_from_file_location("garnet_vscode_publish_readiness", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
pub = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_vscode_publish_readiness"] = pub
SPEC.loader.exec_module(pub)


class PublishReadinessTests(unittest.TestCase):
    def test_extension_is_marketplace_ready(self) -> None:
        r = pub.read_readiness()
        self.assertTrue(r.manifest_present)
        self.assertEqual(r.missing_required, [], f"missing: {r.missing_required}")
        self.assertEqual(r.missing_files, [], f"missing files: {r.missing_files}")
        self.assertTrue(r.marketplace_ready)

    def test_recommended_fields_present(self) -> None:
        # S54 added `keywords`; displayName/description/categories already present.
        self.assertEqual(pub.read_readiness().missing_recommended, [])

    def test_publish_path_defers_credentialed_publish(self) -> None:
        joined = " ".join(pub.read_readiness().publish_path)
        self.assertIn("OVSX_TOKEN", joined)
        self.assertIn("VSCE_PAT", joined)
        self.assertIn("DEFERRED", joined)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(pub.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_honest_scope(self) -> None:
        md = pub.render_markdown(pub.read_readiness())
        self.assertIn("does not publish", md)
        self.assertIn("Marketplace-ready", md)


if __name__ == "__main__":
    unittest.main()
