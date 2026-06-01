#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
STATUS_PATH = ROOT / "scripts" / "garnet_cap_manifest_standard_status.py"


def load_status_module():
    spec = importlib.util.spec_from_file_location("garnet_cap_manifest_standard_status", STATUS_PATH)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)  # type: ignore[union-attr]
    return module


class CapManifestStandardStatusTests(unittest.TestCase):
    def test_status_gate_is_complete(self) -> None:
        module = load_status_module()
        status = module.read_status()
        self.assertTrue(status.ok, status)
        self.assertTrue(status.standard_doc_present)
        self.assertTrue(status.rfc_references_standard_doc)
        self.assertTrue(status.reference_impl_present)
        self.assertTrue(status.test_vectors_present)
        self.assertTrue(status.cli_gate_present)
        self.assertIn("not adopted", status.scope_summary)

    def test_markdown_keeps_honest_scope(self) -> None:
        module = load_status_module()
        md = module.render_markdown(module.read_status())
        self.assertIn("draft/reference seed", md)
        self.assertIn("No OWASP/LF adoption is claimed", md)
        self.assertIn("declared capability surface", md)

    def test_gate_cli_returns_zero(self) -> None:
        import subprocess
        import sys

        result = subprocess.run(
            [
                sys.executable,
                str(STATUS_PATH),
                "--gate",
                "--format",
                "json",
            ],
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
