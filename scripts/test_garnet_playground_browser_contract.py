#!/usr/bin/env python3
"""Static contracts for the committed W-PLAY browser surface."""
from __future__ import annotations

import hashlib
import json
import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PAGE = ROOT / "docs" / "playground.html"
LIVE = ROOT / "docs" / "playground" / "live.js"
PACKAGE = ROOT / "docs" / "playground" / "pkg"
SMOKE = ROOT / "scripts" / "smoke_garnet_playground_browser.mjs"
PACKAGE_FILES = {"garnet_wasm.js", "garnet_wasm_bg.wasm", "provenance.json"}


class PlaygroundBrowserContractTests(unittest.TestCase):
    def test_page_wires_the_live_editor_controls(self) -> None:
        page = PAGE.read_text(encoding="utf-8")
        for marker in (
            'id="source-editor"',
            'id="baseline-editor"',
            'id="run-source"',
            'id="check-source"',
            'id="diff-caps"',
            'id="run-result"',
            'id="check-result"',
            'id="diff-verdict"',
            'id="machine-verdict"',
            'src="playground/live.js"',
        ):
            self.assertIn(marker, page)

    def test_adapter_imports_only_the_relative_committed_package(self) -> None:
        text = LIVE.read_text(encoding="utf-8")
        self.assertIn('from "./pkg/garnet_wasm.js"', text)
        self.assertIn("garnet.playground.diff-caps-verdict/1", text)
        self.assertIn("garnet.wasm.run/1", text)
        self.assertIn("garnet.wasm.check/1", text)
        self.assertIn("garnet.wasm.diff-caps/1", text)
        self.assertNotIn("http://", text)
        self.assertNotIn("https://", text)
        self.assertNotIn("eval(", text)

    def test_package_inventory_and_hashes_match_canonical_provenance(self) -> None:
        self.assertTrue(PACKAGE.is_dir())
        self.assertEqual(PACKAGE_FILES, {path.name for path in PACKAGE.iterdir()})
        raw = (PACKAGE / "provenance.json").read_bytes()
        self.assertEqual(raw, (json.dumps(json.loads(raw), sort_keys=True, separators=(",", ":")) + "\n").encode())
        provenance = json.loads(raw)
        self.assertEqual("garnet.playground.wasm-package/1", provenance["schema"])
        self.assertNotIn("build_parent_commit_observed", provenance["source"])
        for name in ("garnet_wasm.js", "garnet_wasm_bg.wasm"):
            artifact = (PACKAGE / name).read_bytes()
            self.assertEqual(len(artifact), provenance["artifacts"][name]["bytes"])
            self.assertEqual(
                hashlib.sha256(artifact).hexdigest(),
                provenance["artifacts"][name]["sha256"],
            )

    def test_every_browser_runtime_input_is_git_tracked(self) -> None:
        paths = [
            PAGE,
            LIVE,
            *(PACKAGE / name for name in sorted(PACKAGE_FILES)),
        ]
        relative = [path.relative_to(ROOT).as_posix() for path in paths]
        proc = subprocess.run(
            ["git", "ls-files", "--error-unmatch", "--", *relative],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(0, proc.returncode, proc.stderr or proc.stdout)

    def test_playwright_trap_is_fail_closed_and_time_bounded(self) -> None:
        text = SMOKE.read_text(encoding="utf-8")
        for marker in (
            'from "../apps/garnet-studio/node_modules/playwright/index.mjs"',
            'serviceWorkers: "block"',
            "externalRequests.add",
            "untrackedRequests.add",
            "durationMs < 30_000",
            'exit_class, "runtime_error"',
            'stdout, "", "denial stdout"',
            'verdict: passed ? "pass" : "fail"',
        ):
            self.assertIn(marker, text)


if __name__ == "__main__":
    unittest.main()
