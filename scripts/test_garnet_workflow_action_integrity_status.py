#!/usr/bin/env python3
"""Adversarial tests for immutable external GitHub Action provenance."""
from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/garnet_workflow_action_integrity_status.py"
SPEC = importlib.util.spec_from_file_location(
    "garnet_workflow_action_integrity_status", SCRIPT
)
assert SPEC and SPEC.loader
status = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = status
SPEC.loader.exec_module(status)


CHECKOUT = "df4cb1c069e1874edd31b4311f1884172cec0e10"
CACHE = "caa296126883cff596d87d8935842f9db880ef25"
RUST_TOOLCHAIN = "2c7215f132e9ebf062739d9130488b56d53c060c"


class ActionIntegrityTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name).resolve()
        self.git("init", "-q")
        self.git("config", "user.email", "test@example.invalid")
        self.git("config", "user.name", "Test")
        (self.root / ".github/workflows").mkdir(parents=True)
        (self.root / ".github/rulesets").mkdir(parents=True)
        self.workflow = self.root / ".github/workflows/ci.yml"
        self.entries = [
            {
                "action": "actions/checkout",
                "commit": CHECKOUT,
                "resolved_at": "2026-07-17T06:07:40Z",
                "source_kind": "tag",
                "source_ref": "v6",
            },
            {
                "action": "actions/cache",
                "commit": CACHE,
                "resolved_at": "2026-07-17T06:07:40Z",
                "source_kind": "tag",
                "source_ref": "v5",
            },
        ]
        self.write_workflow(CHECKOUT, CACHE)
        self.write_manifest()

    def tearDown(self) -> None:
        self.temp.cleanup()

    def git(self, *args: str) -> bytes:
        result = subprocess.run(
            ["git", *args],
            cwd=self.root,
            capture_output=True,
            check=False,
        )
        self.assertEqual(0, result.returncode, result.stderr.decode(errors="replace"))
        return result.stdout

    def write_workflow(self, checkout: str, cache: str) -> None:
        self.workflow.write_text(
            """name: CI
on:
  pull_request:
    branches: [main]
permissions:
  contents: read
jobs:
  test:
    name: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@%s
      - uses: actions/cache@%s
        with:
          path: target
          key: cache
      - run: cargo test
"""
            % (checkout, cache),
            encoding="utf-8",
        )
        self.git("add", ".github/workflows/ci.yml")

    def write_manifest(self, *, raw: str | None = None) -> None:
        path = self.root / status.MANIFEST_PATH
        if raw is None:
            raw = json.dumps(
                {"schema": status.MANIFEST_SCHEMA, "entries": self.entries},
                ensure_ascii=False,
                indent=2,
                sort_keys=True,
            ) + "\n"
        path.write_text(raw, encoding="utf-8")

    def read(self):
        return status.read_status(self.root)

    def add_rust_toolchain(self, with_block: str) -> None:
        self.entries.append(
            {
                "action": "dtolnay/rust-toolchain",
                "commit": RUST_TOOLCHAIN,
                "resolved_at": "2026-07-17T06:07:40Z",
                "source_kind": "branch",
                "source_ref": "master",
            }
        )
        text = self.workflow.read_text(encoding="utf-8").replace(
            "      - run: cargo test",
            "      - uses: dtolnay/rust-toolchain@%s%s\n      - run: cargo test"
            % (RUST_TOOLCHAIN, with_block),
        )
        self.workflow.write_text(text, encoding="utf-8")
        self.git("add", ".github/workflows/ci.yml")
        self.write_manifest()

    def assert_red(self, fragment: str) -> None:
        result = self.read()
        self.assertFalse(result.ok)
        self.assertTrue(any(fragment in item for item in result.findings), result.findings)
        self.assertEqual(0, result.credited_occurrences)

    def test_full_sha_pins_matching_exact_manifest_pass(self) -> None:
        result = self.read()
        self.assertTrue(result.ok, result.findings)
        self.assertEqual(2, result.occurrence_count)
        self.assertEqual(2, result.credited_occurrences)
        self.assertEqual(0, result.mutable_count)

    def test_mutable_tag_is_red(self) -> None:
        self.write_workflow("v6", CACHE)
        self.assert_red("full 40-character")

    def test_wrong_full_sha_is_red(self) -> None:
        self.write_workflow("0" * 40, CACHE)
        self.assert_red("does not match the reviewed manifest")

    def test_unknown_pinned_action_is_red(self) -> None:
        self.workflow.write_text(
            self.workflow.read_text(encoding="utf-8").replace(
                f"actions/cache@{CACHE}", f"example/unknown@{'1' * 40}"
            ),
            encoding="utf-8",
        )
        self.git("add", ".github/workflows/ci.yml")
        self.assert_red("absent from the reviewed manifest")

    def test_sha_pinned_rust_toolchain_requires_explicit_channel(self) -> None:
        self.add_rust_toolchain("")
        self.assert_red("explicit toolchain input")

    def test_sha_pinned_rust_toolchain_with_explicit_channel_passes(self) -> None:
        self.add_rust_toolchain("\n        with:\n          toolchain: stable")
        result = self.read()
        self.assertTrue(result.ok, result.findings)

    def test_unused_manifest_entry_is_red(self) -> None:
        self.write_workflow(CHECKOUT, CHECKOUT)
        self.assert_red("unused manifest entry")

    def test_duplicate_manifest_action_is_red(self) -> None:
        self.entries.append(dict(self.entries[0]))
        self.write_manifest()
        self.assert_red("duplicate manifest pin")

    def test_malformed_or_unknown_manifest_fields_are_red(self) -> None:
        self.entries[0]["mutable"] = False
        self.write_manifest()
        self.assert_red("entry keys are not exact")

    def test_duplicate_json_key_is_red(self) -> None:
        self.write_manifest(
            raw=(
                '{"schema":"%s","schema":"%s","entries":[]}\n'
                % (status.MANIFEST_SCHEMA, status.MANIFEST_SCHEMA)
            )
        )
        self.assert_red("duplicate JSON key")

    @unittest.skipUnless(hasattr(os, "symlink"), "symlink API unavailable")
    def test_manifest_symlink_is_red(self) -> None:
        path = self.root / status.MANIFEST_PATH
        target = path.with_name("pins-target.json")
        path.rename(target)
        try:
            os.symlink(target, path)
        except OSError as exc:
            self.skipTest(f"symlink unavailable: {exc}")
        self.assert_red("symlink/reparse")

    def test_noncanonical_or_future_resolution_time_is_red(self) -> None:
        self.entries[0]["resolved_at"] = "2999-01-01T00:00:00Z"
        self.write_manifest()
        self.assert_red("resolution timestamp")

    def test_repository_current_workflows_are_all_immutable(self) -> None:
        result = status.read_status(ROOT)
        self.assertTrue(result.ok, result.findings)
        self.assertGreater(result.occurrence_count, 0)
        self.assertEqual(result.occurrence_count, result.credited_occurrences)


if __name__ == "__main__":
    unittest.main()
