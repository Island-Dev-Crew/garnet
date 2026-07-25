#!/usr/bin/env python3
"""Adversarial tests for the immutable workflow Git-object boundary."""
from __future__ import annotations

import importlib.util
import os
import subprocess
import sys
import tempfile
import unicodedata
import unittest
from pathlib import Path
from unittest import mock


PATH = Path(__file__).with_name("garnet_workflow_file_policy.py")
SPEC = importlib.util.spec_from_file_location("_workflow_file_policy_test", PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load {PATH}")
policy = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = policy
SPEC.loader.exec_module(policy)


class WorkflowFilePolicyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name) / "repo"
        self.root.mkdir()
        self.git("init", "-q")
        self.git("config", "user.email", "test@example.invalid")
        self.git("config", "user.name", "Test")
        self.git("config", "core.precomposeunicode", "false")
        (self.root / ".github/workflows").mkdir(parents=True)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def git(self, *args: str, data: bytes | None = None) -> bytes:
        return subprocess.run(
            ["git", "-C", str(self.root), *args],
            input=data,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,
        ).stdout

    def stage(self, name: str, data: bytes = b"jobs:\n") -> Path:
        path = self.root / ".github/workflows" / name
        path.write_bytes(data)
        self.git("add", "--", f".github/workflows/{name}")
        return path

    def cache(self, name: str, data: bytes, mode: str = "100644") -> str:
        object_id = self.git("hash-object", "-w", "--stdin", data=data).strip().decode()
        self.git(
            "update-index",
            "--add",
            "--cacheinfo",
            f"{mode},{object_id},.github/workflows/{name}",
        )
        return object_id

    def snapshot(self, root: Path | None = None):
        return policy.workflow_snapshot(root or self.root)

    def assert_closed(self, fragment: str) -> None:
        records, problems = self.snapshot()
        self.assertEqual([], records)
        self.assertTrue(any(fragment in item for item in problems), problems)

    def test_index_and_exact_commit_are_immutable_sources(self) -> None:
        path = self.stage("checks.yml", b"index\n")
        self.git("commit", "-qm", "snapshot")
        path.write_bytes(b"dirty\n")
        commit_id = self.git("rev-parse", "HEAD").strip().decode()
        index, problems = self.snapshot()
        commit, commit_problems = policy.workflow_snapshot(self.root, treeish=commit_id)
        self.assertEqual((problems, commit_problems), ([], []))
        self.assertEqual(index[0].content, b"index\n")
        self.assertEqual(commit[0].content, b"index\n")
        subtree = self.git("rev-parse", "HEAD:.github/workflows").strip().decode()
        for invalid in ("HEAD", subtree):
            rejected, rejection = policy.workflow_snapshot(self.root, treeish=invalid)
            self.assertEqual(rejected, [])
            self.assertTrue(rejection)

    @unittest.skipUnless(hasattr(os, "symlink"), "symlink API unavailable")
    def test_ancestor_symlink_or_junction_is_irrelevant(self) -> None:
        self.stage("checks.yml", b"object\n")
        alias = Path(self.temp.name) / "repo-alias"
        try:
            alias.symlink_to(self.root, target_is_directory=True)
        except OSError as exc:
            self.skipTest(f"cannot create directory indirection: {exc}")
        records, problems = self.snapshot(alias)
        self.assertEqual(problems, [])
        self.assertEqual(records[0].content, b"object\n")

    def test_mode_120000_fails_even_when_core_symlinks_is_false(self) -> None:
        self.git("config", "core.symlinks", "false")
        self.cache("link.yml", b"elsewhere.yml", "120000")
        self.assert_closed("not a regular Git blob")

    def test_dirty_replacement_and_hardlink_cannot_change_index_blob(self) -> None:
        path = self.stage("checks.yml", b"trusted\n")
        other = self.root / "other"
        other.write_bytes(b"replacement\n")
        path.unlink()
        try:
            os.link(other, path)
        except OSError:
            path.write_bytes(b"replacement\n")
        records, problems = self.snapshot()
        self.assertEqual(problems, [])
        self.assertEqual(records[0].content, b"trusted\n")

    def test_git_environment_and_replace_refs_cannot_redirect_objects(self) -> None:
        original = self.cache("checks.yml", b"trusted\n")
        forged = self.git("hash-object", "-w", "--stdin", data=b"forged\n").strip().decode()
        self.git("replace", original, forged)
        redirected = str(self.root / "attacker-controlled")
        with mock.patch.dict(
            os.environ, {"GIT_DIR": redirected, "GIT_INDEX_FILE": redirected}
        ):
            records, problems = self.snapshot()
        self.assertEqual(problems, [])
        self.assertEqual((records[0].object_id, records[0].content), (original, b"trusted\n"))

    def test_post_enumeration_index_growth_cannot_change_pinned_object(self) -> None:
        path = self.stage("checks.yml", b"small\n")
        original = policy._git
        changed = False

        def race(root, *args, limit):
            nonlocal changed
            output = original(root, *args, limit=limit)
            if args[0] == "ls-files" and not changed:
                changed = True
                path.write_bytes(b"x" * (policy.MAX_WORKFLOW_BYTES + 1))
                self.git("add", "--", ".github/workflows/checks.yml")
            return output

        with mock.patch.object(policy, "_git", race):
            records, problems = self.snapshot()
        self.assertEqual(problems, [])
        self.assertEqual(records[0].content, b"small\n")

    def test_entry_and_aggregate_byte_caps_fail_all_or_zero(self) -> None:
        self.stage("one.yml", b"1234")
        self.stage("two.yaml", b"5678")
        with mock.patch.object(policy, "MAX_WORKFLOW_FILES", 1):
            self.assert_closed("workflow count")
        with mock.patch.object(policy, "MAX_WORKFLOW_TOTAL_BYTES", 7):
            self.assert_closed("byte boundary")

    def test_casefold_and_unicode_normalization_collisions_fail_closed(self) -> None:
        cases = (("Case.yml", "case.yml"), ("caf\u00e9.yml", "cafe\u0301.yml"))
        for first, second in cases:
            with self.subTest(first=first):
                self.git("read-tree", "--empty")
                self.cache(first, b"one")
                self.cache(second, b"two")
                indexed = self.git(
                    "ls-files", "-z", "--", ".github/workflows"
                ).split(b"\0")
                expected = {
                    f".github/workflows/{first}".encode(),
                    f".github/workflows/{second}".encode(),
                }
                self.assertEqual(expected, set(filter(None, indexed)))
                self.assert_closed("collide")
                self.assertEqual(
                    unicodedata.normalize("NFC", first).casefold(),
                    unicodedata.normalize("NFC", second).casefold(),
                )

    def test_invalid_utf8_payload_is_handed_off_as_exact_bytes(self) -> None:
        object_id = self.cache("bad.yml", b"\xff\xfe")
        records, problems = self.snapshot()
        self.assertEqual(problems, [])
        self.assertEqual((records[0].object_id, records[0].content), (object_id, b"\xff\xfe"))

    def test_any_bad_entry_withholds_every_consumable_record(self) -> None:
        self.stage("valid.yml")
        self.cache("bad.yaml", b"target", "120000")
        self.assert_closed("not a regular Git blob")


if __name__ == "__main__":
    unittest.main()
