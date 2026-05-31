#!/usr/bin/env python3
"""Regression tests for the one-line install / readme check (S52)."""
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("garnet_install_readme_check.py")
SPEC = importlib.util.spec_from_file_location("garnet_install_readme_check", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
check = importlib.util.module_from_spec(SPEC)
sys.modules["garnet_install_readme_check"] = check
SPEC.loader.exec_module(check)


class ExtractTests(unittest.TestCase):
    def test_extracts_from_readme_fence(self) -> None:
        text = "intro\n```sh\ncurl -sSf https://garnet-lang.org/install.sh | sh\n```\n"
        self.assertEqual(
            check._extract_curl(text),
            "curl -sSf https://garnet-lang.org/install.sh | sh",
        )

    def test_strips_comment_marker_and_collapses_space(self) -> None:
        text = "#   curl   -sSf   https://garnet-lang.org/install.sh | sh\n"
        self.assertEqual(
            check._extract_curl(text),
            "curl -sSf https://garnet-lang.org/install.sh | sh",
        )

    def test_returns_none_when_absent(self) -> None:
        self.assertIsNone(check._extract_curl("no install command here\n"))


class RealRepoTests(unittest.TestCase):
    def test_repo_install_docs_are_consistent(self) -> None:
        c = check.read_check()
        self.assertTrue(c.commands_match, "README vs installer command drift")
        self.assertTrue(c.url_in_readme and c.url_in_installer)
        self.assertTrue(c.consistent)

    def test_gate_passes_on_real_repo(self) -> None:
        self.assertEqual(check.main(["--gate", "--format", "json"]), 0)

    def test_markdown_states_honest_scope(self) -> None:
        md = check.render_markdown(check.read_check())
        self.assertIn("not a live network install test", md)
        self.assertIn("Install docs consistent: yes", md)


if __name__ == "__main__":
    unittest.main()
