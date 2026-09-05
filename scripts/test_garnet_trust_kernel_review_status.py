#!/usr/bin/env python3
"""Adversarial tests for the rolling trust-kernel review v2 gate."""
from __future__ import annotations

import hashlib
import importlib.util
import contextlib
import io
import json
import os
import re
import subprocess
import sys
import tempfile
import types
import unittest
from pathlib import Path
from unittest import mock

SCRIPT = Path(__file__).with_name("garnet_trust_kernel_review_status.py")
SPEC = importlib.util.spec_from_file_location("garnet_trust_kernel_review_status", SCRIPT)
assert SPEC is not None
mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["garnet_trust_kernel_review_status"] = mod
SPEC.loader.exec_module(mod)


def _canonical(data: object) -> bytes:
    return (json.dumps(data, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode()


def write_lf(path: Path, text: str, *, encoding: str = "utf-8") -> None:
    """Write canonical fixture bytes without platform newline translation."""
    path.write_bytes(text.encode(encoding))


def _change_digest(
    changes: list[tuple[str, str, str, bytes | None, str, bytes | None]],
) -> str:
    digest = hashlib.sha256()
    digest.update(b"garnet.trust_kernel.change/v2\0")
    for path, status, old_oid, old_blob, new_oid, new_blob in sorted(changes):
        old_mode = "000000" if status == "A" else "100644"
        new_mode = "000000" if status == "D" else "100644"
        digest.update(status.encode("ascii"))
        digest.update(b"\0")
        digest.update(path.encode("utf-8"))
        digest.update(b"\0")
        digest.update(old_mode.encode("ascii"))
        digest.update(b"\0")
        digest.update(old_oid.encode("ascii"))
        digest.update(b"\0")
        digest.update(
            (hashlib.sha256(old_blob).hexdigest() if old_blob is not None else "-").encode(
                "ascii"
            )
        )
        digest.update(b"\0")
        digest.update(new_mode.encode("ascii"))
        digest.update(b"\0")
        digest.update(new_oid.encode("ascii"))
        digest.update(b"\0")
        digest.update(
            (hashlib.sha256(new_blob).hexdigest() if new_blob is not None else "-").encode(
                "ascii"
            )
        )
        digest.update(b"\0")
    return "sha256:" + digest.hexdigest()


class GitRepoFixture(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name).resolve()
        self._git("init")
        self._git("config", "core.autocrlf", "false")
        self._git("config", "user.email", "author@example.invalid")
        self._git("config", "user.name", "Author")
        write_lf(self.root / "README.md", "base\n")
        # The surface in force at a landing is a matter of provenance: the era
        # stone of the current surface sits at the fixture base, so every
        # landing after it is in the current era.
        self._write_stone(mod.CURRENT_TRUST_SURFACE)
        registry = self.root / "F_Project_Management/W_TRUST/LANDED_REVIEW_MARKERS.json"
        registry.parent.mkdir(parents=True, exist_ok=True)
        registry.write_bytes(
            _canonical(
                {
                    "markers": [],
                    "schema": "garnet.trust_kernel_landed_review_registry/v1",
                }
            )
        )
        self._git("add", "README.md", mod.era_stone_path(mod.CURRENT_TRUST_SURFACE), str(registry.relative_to(self.root)))
        self._git("commit", "-m", "base")
        self.base = self._git("rev-parse", "HEAD")
        self._git("branch", "-M", "main")
        self._git("update-ref", "refs/remotes/origin/main", self.base)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def _write_stone(self, version: str, pull: int = 559) -> str:
        path = self.root / mod.era_stone_path(version)
        path.parent.mkdir(parents=True, exist_ok=True)
        digest = mod.surface_digest(version) if version in mod.TRUST_SURFACES else "0" * 64
        path.write_bytes(_canonical({"introduced_by_pull_request": pull, "schema": mod.ERA_STONE_SCHEMA, "surface_sha256": digest, "version": version}))
        return mod.era_stone_path(version)

    def _git(self, *args: str, check: bool = True) -> str:
        result = subprocess.run(
            ["git", *args],
            cwd=self.root,
            capture_output=True,
            text=True,
            check=False,
        )
        if check and result.returncode != 0:
            self.fail(f"git {' '.join(args)} failed: {result.stderr or result.stdout}")
        return result.stdout.strip()

    def _commit_file(self, relative: str, content: bytes, message: str) -> str:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        self._git("add", relative)
        self._git("commit", "-m", message)
        return self._git("rev-parse", "HEAD")


class ClassificationTests(unittest.TestCase):
    def test_trust_kernel_paths_are_recognized(self) -> None:
        for path in (
            "garnet-check-v0.3/src/caps_graph.rs",
            "garnet-interp-v0.3/src/eval.rs",
            "garnet-vm/src/vm.rs",
            "garnet-stdlib/src/registry.rs",
            "garnet-wasm/src/lib.rs",
            "garnet-cli/src/cmd/run.rs",
            "garnet-cli/src/cmd/add.rs",
            "garnet-cli/src/cmd/mod.rs",
            "garnet-cli/src/bound_source.rs",
            "garnet-cli/src/lib.rs",
            "garnet-cli/Cargo.toml",
            "Cargo.lock",
            ".github/CODEOWNERS",
            "scripts/garnet_required_context_contract.py",
            "scripts/test_garnet_required_context_contract.py",
            ".github/workflows/ci.yml",
            ".github/rulesets/garnet-main.json",
            "F_Project_Management/W_TRUST/LANDED_REVIEW_MARKERS.json",
            "F_Project_Management/W_TRUST/landed/LANE1.landed-review.json",
            "docs/why.html",
        ):
            self.assertTrue(mod.is_trust_kernel(path), path)
            self.assertTrue(mod.is_trust_kernel(path.replace("/", "\\")), path)

    def test_non_trust_kernel_paths_are_ignored(self) -> None:
        for path in ("README.md", "ops/mission/state.json", "apps/garnet-studio/x"):
            self.assertFalse(mod.is_trust_kernel(path), path)

    def test_only_canonical_w_trust_json_name_is_a_record(self) -> None:
        self.assertTrue(
            mod.is_review_record("F_Project_Management/W_TRUST/LANE1_ITEM2.review.json")
        )
        self.assertFalse(mod.is_review_record("F_Project_Management/W_TRUST/note.md"))
        self.assertFalse(mod.is_review_record("proofs/independent/s114/review.json"))


class RawDiffParserTests(unittest.TestCase):
    def test_well_formed_nul_status_is_parsed(self) -> None:
        old = "1" * 40
        new = "2" * 40
        payload = f":100644 100644 {old} {new} M\0scripts/garnet_a.py\0".encode()
        entries, errors = mod._parse_raw_z(payload)
        self.assertEqual(["scripts/garnet_a.py"], [entry.path for entry in entries])
        self.assertEqual("M", entries[0].status)
        self.assertEqual([], errors)

    def test_empty_payload_is_a_legitimate_empty_diff(self) -> None:
        self.assertEqual(([], []), mod._parse_raw_z(b""))

    def test_non_nul_terminated_status_is_red(self) -> None:
        _, errors = mod._parse_raw_z(
            f":100644 100644 {'1' * 40} {'2' * 40} M\0scripts/garnet_a.py".encode()
        )
        self.assertTrue(any("NUL" in item for item in errors))

    def test_odd_status_path_arity_is_red(self) -> None:
        _, errors = mod._parse_raw_z(
            f":100644 100644 {'1' * 40} {'2' * 40} M\0".encode()
        )
        self.assertTrue(any("header/path" in item for item in errors))

    def test_unknown_status_is_red(self) -> None:
        _, errors = mod._parse_raw_z(
            f":100644 100644 {'1' * 40} {'2' * 40} X\0scripts/garnet_a.py\0".encode()
        )
        self.assertTrue(any("unsupported" in item for item in errors))

    def test_exact_deletion_tombstone_is_preserved(self) -> None:
        payload = (
            f":100644 000000 {'1' * 40} {'0' * 40} D\0scripts/garnet_a.py\0"
        ).encode()
        entries, errors = mod._parse_raw_z(payload)
        self.assertEqual([], errors)
        self.assertEqual("D", entries[0].status)
        self.assertEqual("1" * 40, entries[0].old_oid)
        self.assertEqual("0" * 40, entries[0].new_oid)

    def test_ambiguous_deletion_identity_is_red(self) -> None:
        payload = (
            f":100644 100644 {'1' * 40} {'2' * 40} D\0scripts/garnet_a.py\0"
        ).encode()
        _, errors = mod._parse_raw_z(payload)
        self.assertTrue(any("ambiguous deletion" in item for item in errors))

    def test_invalid_utf8_path_is_red(self) -> None:
        payload = (
            f":100644 100644 {'1' * 40} {'2' * 40} M\0".encode()
            + b"scripts/\xff.py\0"
        )
        _, errors = mod._parse_raw_z(payload)
        self.assertTrue(any("UTF-8" in item for item in errors))


class DiscoveryTests(GitRepoFixture):
    def test_legitimate_empty_diff_is_green_and_distinct_from_failure(self) -> None:
        result = mod.discover_changes(base="HEAD", head="HEAD", root=self.root)
        self.assertTrue(result.ok, result.problems)
        self.assertEqual([], result.paths)
        self.assertTrue(result.empty)

    def test_missing_origin_main_is_red(self) -> None:
        self._git("update-ref", "-d", "refs/remotes/origin/main")
        result = mod.discover_changes(base=None, head="HEAD", root=self.root)
        self.assertFalse(result.ok)
        self.assertTrue(any("origin/main" in item for item in result.problems))

    def test_non_commit_head_is_red(self) -> None:
        result = mod.discover_changes(base=self.base, head="README.md", root=self.root)
        self.assertFalse(result.ok)
        self.assertTrue(any("head ref does not name a commit" in item for item in result.problems))

    def test_merge_base_empty_output_is_red(self) -> None:
        ok_head = mod.GitResult(0, ("a" * 40 + "\n").encode(), b"")
        ok_main = mod.GitResult(0, ("b" * 40 + "\n").encode(), b"")
        empty = mod.GitResult(0, b"", b"")
        with mock.patch.object(mod, "_git_bytes", side_effect=[ok_head, ok_main, empty]):
            result = mod.discover_changes(base=None, head="HEAD", root=self.root)
        self.assertFalse(result.ok)
        self.assertTrue(any("merge-base returned no commit" in item for item in result.problems))

    def test_diff_failure_is_red(self) -> None:
        ok_head = mod.GitResult(0, ("a" * 40 + "\n").encode(), b"")
        ok_base = mod.GitResult(0, ("b" * 40 + "\n").encode(), b"")
        failed = mod.GitResult(128, b"", b"fatal: diff failed")
        with mock.patch.object(mod, "_git_bytes", side_effect=[ok_head, ok_base, failed]):
            result = mod.discover_changes(base="base", head="head", root=self.root)
        self.assertFalse(result.ok)
        self.assertTrue(any("diff enumeration failed" in item for item in result.problems))

    def test_diff_timeout_is_red(self) -> None:
        ok_head = mod.GitResult(0, ("a" * 40 + "\n").encode(), b"")
        ok_base = mod.GitResult(0, ("b" * 40 + "\n").encode(), b"")
        timeout = mod.GitResult(124, b"", b"", timed_out=True)
        with mock.patch.object(mod, "_git_bytes", side_effect=[ok_head, ok_base, timeout]):
            result = mod.discover_changes(base="base", head="head", root=self.root)
        self.assertFalse(result.ok)
        self.assertTrue(any("timed out" in item for item in result.problems))

    def test_valid_but_partial_raw_diff_is_red_against_name_crosscheck(self) -> None:
        raw = mod.GitResult(
            0,
            (
                f":000000 100644 {'0' * 40} {'1' * 40} A\0"
                "scripts/garnet_a.py\0"
            ).encode(),
            b"",
        )
        names = mod.GitResult(
            0,
            b"scripts/garnet_a.py\0scripts/garnet_b.py\0",
            b"",
        )
        with mock.patch.object(mod, "_git_bytes", side_effect=[raw, names]):
            _, problems = mod._diff_entries(self.root, "base..head")
        self.assertTrue(any("partial" in item for item in problems))

    def test_exact_deleted_trust_path_is_enumerated_with_tombstone(self) -> None:
        self._commit_file("scripts/garnet_deleted.py", b"x\n", "add trust path")
        added = self._git("rev-parse", "HEAD")
        self._git("update-ref", "refs/remotes/origin/main", added)
        (self.root / "scripts/garnet_deleted.py").unlink()
        self._git("add", "-u")
        self._git("commit", "-m", "delete trust path")
        result = mod.discover_changes(base=added, head="HEAD", root=self.root)
        self.assertTrue(result.ok, result.problems)
        self.assertEqual(["scripts/garnet_deleted.py"], result.paths)
        self.assertEqual("D", result.entries[0].status)

    def test_identically_truncated_diff_presentations_are_red_against_tree_objects(self) -> None:
        self._commit_file("scripts/garnet_tree_truth.py", b"truth = 1\n", "tree truth")
        head = self._git("rev-parse", "HEAD")
        new_oid = self._git("rev-parse", f"{head}:scripts/garnet_tree_truth.py")
        complete = [
            mod.ChangeEntry(
                "scripts/garnet_tree_truth.py",
                "A",
                "000000",
                "100644",
                "0" * 40,
                new_oid,
            )
        ]
        with (
            mock.patch.object(mod, "_diff_entries", return_value=([], [])),
            mock.patch.object(mod, "_independent_tree_diff", return_value=(complete, [])),
        ):
            result = mod.discover_changes(base=self.base, head=head, root=self.root)
        self.assertFalse(result.ok)
        self.assertTrue(any("tree-object" in p and "partial" in p for p in result.problems))


class RecordSuccessionOrderingTests(unittest.TestCase):
    def test_aligned_linear_succession_selects_tip_most_record(self) -> None:
        records = {
            "predecessor.review.json": ("intro-1", "reviewed-1"),
            "successor.review.json": ("intro-2", "reviewed-2"),
        }
        strict_ancestors = {
            ("intro-1", "intro-2"),
            ("reviewed-1", "reviewed-2"),
        }

        selected, findings = mod._select_linear_record_path(records, strict_ancestors)

        self.assertEqual("successor.review.json", selected)
        self.assertEqual([], findings)

    def test_forked_reviewed_heads_fail_closed(self) -> None:
        records = {
            "predecessor.review.json": ("intro-1", "reviewed-left"),
            "successor.review.json": ("intro-2", "reviewed-right"),
        }
        strict_ancestors = {("intro-1", "intro-2")}

        selected, findings = mod._select_linear_record_path(records, strict_ancestors)

        self.assertIsNone(selected)
        self.assertTrue(any("reviewed_heads must be strictly ordered" in p for p in findings))

    def test_later_record_cannot_bind_an_older_reviewed_head(self) -> None:
        records = {
            "newer.review.json": ("intro-1", "reviewed-2"),
            "stale-terminal.review.json": ("intro-2", "reviewed-1"),
        }
        strict_ancestors = {
            ("intro-1", "intro-2"),
            ("reviewed-1", "reviewed-2"),
        }

        selected, findings = mod._select_linear_record_path(records, strict_ancestors)

        self.assertIsNone(selected)
        self.assertTrue(
            any("tip-most record must bind the newest reviewed_head" in p for p in findings)
        )


class PreMergeReviewRecordTests(GitRepoFixture):
    RECORD = "F_Project_Management/W_TRUST/LANE1_ITEM2.review.json"
    TRUST_BLOBS = {
        "scripts/garnet_alpha.py": b"alpha = 1\n",
        "scripts/test_garnet_alpha.py": b"assert True\n",
    }

    def setUp(self) -> None:
        super().setUp()
        self.TRUST_BLOBS = dict(self.TRUST_BLOBS)
        for relative, content in self.TRUST_BLOBS.items():
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
            self._git("add", relative)
        self._git("commit", "-m", "trust change")
        self.reviewed_head = self._git("rev-parse", "HEAD")
        self.reviewed_tree = self._git("rev-parse", "HEAD^{tree}")

    def _changes(self) -> list[tuple[str, str, str, bytes | None, str, bytes | None]]:
        return [
            (
                path,
                "A",
                "0" * 40,
                None,
                self._git("rev-parse", f"{self.reviewed_head}:{path}"),
                blob,
            )
            for path, blob in self.TRUST_BLOBS.items()
        ]

    def _record(self) -> dict[str, object]:
        return {
            "author_emails": ["email:author@example.invalid"],
            "author_ids": [101],
            "base_commit": self.base,
            "blocking_findings": [],
            "content_digest": _change_digest(self._changes()),
            "head_repository": "Navigata1/garnet",
            "head_repository_id": 6006,
            "pull_request_id": 7007,
            "pull_request_number": 77,
            "repository": "Island-Dev-Crew/garnet",
            "repository_id": 5005,
            "review_scope": (
                "Independent review ended at reviewed_head; content proof does not "
                "extend or backdate review coverage."
            ),
            "reviewed_head": self.reviewed_head,
            "reviewed_tree": self.reviewed_tree,
            "review_state": "APPROVED",
            "reviewer_id": 202,
            "reviewer_login": "independent-reviewer",
            "schema": "garnet.trust_kernel_review_record/v2",
            "state": "premerge",
            "touched_paths": sorted(self.TRUST_BLOBS),
            "verdict": "pass",
        }

    def _commit_record(self, record: dict[str, object] | None = None, raw: bytes | None = None) -> None:
        payload = raw if raw is not None else _canonical(record or self._record())
        self._commit_file(self.RECORD, payload, "review record")

    def _commit_named_record(
        self,
        path: str,
        record: dict[str, object],
        message: str,
    ) -> None:
        self._commit_file(path, _canonical(record), message)

    def _advance_reviewed_head(
        self,
        path: str,
        content: bytes,
        message: str = "later trust change",
    ) -> None:
        self.TRUST_BLOBS[path] = content
        self._commit_file(path, content, message)
        self.reviewed_head = self._git("rev-parse", "HEAD")
        self.reviewed_tree = self._git("rev-parse", "HEAD^{tree}")

    def _transport(self, **overrides: object):
        head = self._git("rev-parse", "HEAD")
        commit_ids = self._git("rev-list", "--reverse", f"{self.base}..{head}").splitlines()
        review = {
            "id": 9009,
            "user": {"id": 202, "login": "independent-reviewer"},
            "state": "APPROVED",
            "commit_id": head,
        }
        collections = {
            "pulls/77/reviews": types.SimpleNamespace(
                rows=(review,), problems=(), page_count=1, byte_count=1
            ),
            "pulls/77/commits": types.SimpleNamespace(
                rows=tuple(
                    {
                        "sha": commit,
                        "author": {"id": 101, "login": "author"},
                        "committer": {"id": 101, "login": "author"},
                    }
                    for commit in commit_ids
                ),
                problems=(),
                page_count=1,
                byte_count=1,
            ),
        }
        objects = {
            "pulls/77": types.SimpleNamespace(
                value={
                    "id": 7007,
                    "number": 77,
                    "head": {
                        "sha": head,
                        "repo": {"id": 6006, "full_name": "Navigata1/garnet"},
                    },
                    "base": {
                        "repo": {"id": 5005, "full_name": "Island-Dev-Crew/garnet"},
                    },
                },
                problems=(),
                byte_count=1,
            ),
            "pulls/77/reviews/9009": types.SimpleNamespace(
                value=review, problems=(), byte_count=1
            ),
        }
        collections.update(overrides.get("collections", {}))
        objects.update(overrides.get("objects", {}))

        class FakeTransport:
            def get_collection(self, path: str, **_: object):
                return collections[path]

            def get_object(self, path: str):
                return objects[path]

        return FakeTransport()

    def _status(self, transport: object | None = None):
        return mod.read_status(
            base=self.base,
            head="HEAD",
            root=self.root,
            github_transport=self._transport() if transport is None else transport,
            repository="Island-Dev-Crew/garnet",
            pull_request=77,
        )

    def test_exact_canonical_content_bound_record_passes(self) -> None:
        self._commit_record()
        status = self._status()
        self.assertTrue(status.ok, status.problems)
        self.assertEqual(sorted(self.TRUST_BLOBS), status.touched_paths)
        self.assertEqual(202, status.reviewer_id)

    def test_authenticated_approval_must_bind_the_exact_current_candidate_head(self) -> None:
        self._commit_record()
        stale = {
            "id": 9009,
            "user": {"id": 202, "login": "independent-reviewer"},
            "state": "APPROVED",
            "commit_id": self.reviewed_head,
        }
        reviews = types.SimpleNamespace(
            rows=(stale,), problems=(), page_count=1, byte_count=1
        )
        status = self._status(
            self._transport(collections={"pulls/77/reviews": reviews})
        )
        self.assertFalse(status.ok)
        self.assertTrue(
            any("exact current candidate head" in problem for problem in status.problems),
            status.problems,
        )

    def test_fork_head_and_upstream_base_repositories_are_bound_separately(self) -> None:
        self._commit_record()
        transport = self._transport()
        original = transport.get_object("pulls/77").value
        pull = dict(original)
        pull["head"] = dict(original["head"])
        pull["head"]["repo"] = {"id": 6006, "full_name": "Navigata1/garnet"}
        fork_pr = types.SimpleNamespace(value=pull, problems=(), byte_count=1)
        status = self._status(
            self._transport(objects={"pulls/77": fork_pr})
        )
        self.assertTrue(status.ok, status.problems)

    def test_authenticated_pull_request_number_requires_exact_integer_type(self) -> None:
        self._commit_record()
        transport = self._transport()
        pull = dict(transport.get_object("pulls/77").value)
        pull["number"] = 77.0
        status = self._status(
            self._transport(
                objects={
                    "pulls/77": types.SimpleNamespace(
                        value=pull, problems=(), byte_count=1
                    )
                }
            )
        )
        self.assertFalse(status.ok, status.problems)
        self.assertTrue(
            any("authenticated pull request number is malformed" in p for p in status.problems),
            status.problems,
        )

    def test_authenticated_pull_request_id_rejects_boolean_integer_alias(self) -> None:
        record = self._record()
        record["pull_request_id"] = 1
        self._commit_record(record)
        transport = self._transport()
        pull = dict(transport.get_object("pulls/77").value)
        pull["id"] = True
        status = self._status(
            self._transport(
                objects={
                    "pulls/77": types.SimpleNamespace(
                        value=pull, problems=(), byte_count=1
                    )
                }
            )
        )
        self.assertFalse(status.ok, status.problems)
        self.assertTrue(
            any("authenticated pull request id is malformed" in p for p in status.problems),
            status.problems,
        )

    def test_authenticated_head_repository_id_rejects_boolean_integer_alias(self) -> None:
        record = self._record()
        record["head_repository_id"] = 1
        self._commit_record(record)
        transport = self._transport()
        pull = dict(transport.get_object("pulls/77").value)
        pull["head"] = dict(pull["head"])
        pull["head"]["repo"] = dict(pull["head"]["repo"])
        pull["head"]["repo"]["id"] = True
        status = self._status(
            self._transport(
                objects={
                    "pulls/77": types.SimpleNamespace(
                        value=pull, problems=(), byte_count=1
                    )
                }
            )
        )
        self.assertFalse(status.ok, status.problems)
        self.assertTrue(
            any("authenticated head repository id is malformed" in p for p in status.problems),
            status.problems,
        )

    def test_authenticated_base_repository_id_rejects_boolean_integer_alias(self) -> None:
        record = self._record()
        record["repository_id"] = 1
        self._commit_record(record)
        transport = self._transport()
        pull = dict(transport.get_object("pulls/77").value)
        pull["base"] = dict(pull["base"])
        pull["base"]["repo"] = dict(pull["base"]["repo"])
        pull["base"]["repo"]["id"] = True
        status = self._status(
            self._transport(
                objects={
                    "pulls/77": types.SimpleNamespace(
                        value=pull, problems=(), byte_count=1
                    )
                }
            )
        )
        self.assertFalse(status.ok, status.problems)
        self.assertTrue(
            any("authenticated base repository id is malformed" in p for p in status.problems),
            status.problems,
        )

    def test_missing_authenticated_transport_is_red(self) -> None:
        self._commit_record()
        status = mod.read_status(
            base=self.base,
            head="HEAD",
            root=self.root,
            github_transport=None,
            repository="Island-Dev-Crew/garnet",
            pull_request=77,
        )
        self.assertFalse(status.ok)
        self.assertTrue(any("authenticated GitHub review transport" in p for p in status.problems))

    def test_unreachable_or_forbidden_review_enumeration_is_red(self) -> None:
        self._commit_record()
        failed = types.SimpleNamespace(
            rows=(),
            problems=(types.SimpleNamespace(code="transport-failure"),),
            page_count=0,
            byte_count=0,
        )
        status = self._status(
            self._transport(collections={"pulls/77/reviews": failed})
        )
        self.assertTrue(any("review enumeration" in p for p in status.problems))

    def test_partial_pagination_is_red(self) -> None:
        self._commit_record()
        partial = types.SimpleNamespace(
            rows=(),
            problems=(types.SimpleNamespace(code="pagination"),),
            page_count=1,
            byte_count=1,
        )
        status = self._status(
            self._transport(collections={"pulls/77/reviews": partial})
        )
        self.assertTrue(any("pagination" in p for p in status.problems))

    def test_duplicate_review_id_is_red(self) -> None:
        self._commit_record()
        review = {
            "id": 9009,
            "user": {"id": 202, "login": "independent-reviewer"},
            "state": "APPROVED",
            "commit_id": self._git("rev-parse", "HEAD"),
        }
        duplicate = types.SimpleNamespace(
            rows=(review, dict(review)), problems=(), page_count=1, byte_count=1
        )
        status = self._status(
            self._transport(collections={"pulls/77/reviews": duplicate})
        )
        self.assertTrue(any("duplicate review id" in p for p in status.problems))

    def test_later_adverse_review_from_same_reviewer_invalidates_older_approval(self) -> None:
        self._commit_record()
        head = self._git("rev-parse", "HEAD")
        approved = {
            "id": 9009,
            "user": {"id": 202, "login": "independent-reviewer"},
            "state": "APPROVED",
            "commit_id": head,
        }
        for adverse_state in ("CHANGES_REQUESTED", "DISMISSED"):
            with self.subTest(state=adverse_state):
                adverse = {
                    "id": 9010,
                    "user": {"id": 202, "login": "independent-reviewer"},
                    "state": adverse_state,
                    "commit_id": head,
                }
                reviews = types.SimpleNamespace(
                    rows=(approved, adverse), problems=(), page_count=1, byte_count=1
                )
                direct = types.SimpleNamespace(value=adverse, problems=(), byte_count=1)
                status = self._status(
                    self._transport(
                        collections={"pulls/77/reviews": reviews},
                        objects={"pulls/77/reviews/9010": direct},
                    )
                )
                self.assertFalse(status.ok, status.problems)
                self.assertTrue(
                    any("latest decisive review" in problem for problem in status.problems),
                    status.problems,
                )

    def test_reviewer_rename_cannot_hide_a_later_adverse_decision(self) -> None:
        self._commit_record()
        head = self._git("rev-parse", "HEAD")
        approved = {
            "id": 9009,
            "user": {"id": 202, "login": "independent-reviewer"},
            "state": "APPROVED",
            "commit_id": head,
        }
        adverse = {
            "id": 9010,
            "user": {"id": 202, "login": "renamed-reviewer"},
            "state": "CHANGES_REQUESTED",
            "commit_id": head,
        }
        reviews = types.SimpleNamespace(
            rows=(approved, adverse), problems=(), page_count=1, byte_count=1
        )
        direct = types.SimpleNamespace(value=adverse, problems=(), byte_count=1)
        status = self._status(
            self._transport(
                collections={"pulls/77/reviews": reviews},
                objects={"pulls/77/reviews/9010": direct},
            )
        )
        self.assertFalse(status.ok, status.problems)
        self.assertTrue(
            any("latest decisive review" in problem for problem in status.problems),
            status.problems,
        )
        self.assertTrue(
            any("reviewer identity" in problem for problem in status.problems),
            status.problems,
        )

    def test_review_must_be_approved_at_exact_current_candidate_head(self) -> None:
        self._commit_record()
        review = {
            "id": 9009,
            "user": {"id": 202, "login": "independent-reviewer"},
            "state": "CHANGES_REQUESTED",
            "commit_id": "f" * 40,
        }
        reviews = types.SimpleNamespace(
            rows=(review,), problems=(), page_count=1, byte_count=1
        )
        direct = types.SimpleNamespace(value=review, problems=(), byte_count=1)
        status = self._status(
            self._transport(
                collections={"pulls/77/reviews": reviews},
                objects={"pulls/77/reviews/9009": direct},
            )
        )
        self.assertTrue(any("APPROVED" in p for p in status.problems))
        self.assertTrue(any("exact current candidate head" in p for p in status.problems))

    def test_authenticated_commit_subset_is_red(self) -> None:
        self._commit_record()
        head = self._git("rev-parse", "HEAD")
        subset = types.SimpleNamespace(
            rows=(
                {
                    "sha": head,
                    "author": {"id": 101, "login": "author"},
                    "committer": {"id": 101, "login": "author"},
                },
            ),
            problems=(),
            page_count=1,
            byte_count=1,
        )
        status = self._status(
            self._transport(collections={"pulls/77/commits": subset})
        )
        self.assertTrue(any("commit enumeration is partial" in p for p in status.problems))

    def test_reviewer_committer_overlap_is_red(self) -> None:
        record = self._record()
        record["reviewer_id"] = 303
        record["reviewer_login"] = "commit-committer"
        self._commit_record(record)
        head = self._git("rev-parse", "HEAD")
        commits = self._git("rev-list", "--reverse", f"{self.base}..{head}").splitlines()
        rows = types.SimpleNamespace(
            rows=tuple(
                {
                    "sha": commit,
                    "author": {"id": 101, "login": "author"},
                    "committer": {"id": 303, "login": "commit-committer"},
                }
                for commit in commits
            ),
            problems=(),
            page_count=1,
            byte_count=1,
        )
        review = {
            "id": 9009,
            "user": {"id": 303, "login": "commit-committer"},
            "state": "APPROVED",
            "commit_id": head,
        }
        reviews = types.SimpleNamespace(
            rows=(review,), problems=(), page_count=1, byte_count=1
        )
        direct = types.SimpleNamespace(value=review, problems=(), byte_count=1)
        status = self._status(
            self._transport(
                collections={
                    "pulls/77/commits": rows,
                    "pulls/77/reviews": reviews,
                },
                objects={"pulls/77/reviews/9009": direct},
            )
        )
        self.assertFalse(status.ok, status.problems)
        self.assertTrue(
            any("reviewer identity overlaps an authenticated commit principal" in p for p in status.problems),
            status.problems,
        )

    def test_malformed_authenticated_committer_identity_is_red(self) -> None:
        self._commit_record()
        transport = self._transport()
        original = transport.get_collection("pulls/77/commits")
        rows = tuple(dict(row, committer=None) for row in original.rows)
        malformed = types.SimpleNamespace(
            rows=rows,
            problems=(),
            page_count=1,
            byte_count=1,
        )
        status = self._status(
            self._transport(collections={"pulls/77/commits": malformed})
        )
        self.assertFalse(status.ok, status.problems)
        self.assertTrue(
            any("malformed committer identity" in p for p in status.problems),
            status.problems,
        )

    def test_author_ids_bind_author_and_committer_identity_union(self) -> None:
        record = self._record()
        record["author_ids"] = [101, 303]
        self._commit_record(record)
        transport = self._transport()
        original = transport.get_collection("pulls/77/commits")
        rows = tuple(
            dict(row, committer={"id": 303, "login": "commit-committer"})
            for row in original.rows
        )
        authenticated = types.SimpleNamespace(
            rows=rows,
            problems=(),
            page_count=1,
            byte_count=1,
        )
        status = self._status(
            self._transport(collections={"pulls/77/commits": authenticated})
        )
        self.assertTrue(status.ok, status.problems)

    def test_repository_and_pull_request_are_immutably_bound(self) -> None:
        record = self._record()
        record["repository_id"] = 9999
        record["pull_request_id"] = 8888
        self._commit_record(record)
        status = self._status()
        self.assertTrue(any("repository id" in p for p in status.problems))
        self.assertTrue(any("pull request id" in p for p in status.problems))

    def test_missing_record_is_red(self) -> None:
        status = self._status()
        self.assertFalse(status.ok)
        self.assertTrue(any("structured review record is missing" in p for p in status.problems))

    def test_markdown_presence_is_not_a_record(self) -> None:
        self._commit_file(
            "F_Project_Management/W_TRUST/LANE1_ITEM2.md",
            b"Trust-Kernel-Review: someone\n",
            "bare prose",
        )
        status = self._status()
        self.assertFalse(status.ok)
        self.assertTrue(any("non-JSON" in p for p in status.problems))

    def test_multiple_structured_records_are_red(self) -> None:
        first = self.root / self.RECORD
        second = self.root / "F_Project_Management/W_TRUST/LANE1_DUPLICATE.review.json"
        first.parent.mkdir(parents=True, exist_ok=True)
        first.write_bytes(_canonical(self._record()))
        second.write_bytes(_canonical(self._record()))
        self._git("add", str(first.relative_to(self.root)), str(second.relative_to(self.root)))
        self._git("commit", "-m", "ambiguous review records")
        self.assertTrue(
            any("reviewed_heads must be strictly ordered" in p for p in self._status().problems)
        )

    def test_linear_record_succession_selects_tip_record_for_full_range(self) -> None:
        predecessor = self._record()
        self._commit_record(predecessor)
        self._advance_reviewed_head("scripts/garnet_beta.py", b"beta = 2\n")
        successor_path = "F_Project_Management/W_TRUST/LANE1_SUCCESSOR.review.json"
        successor = self._record()
        self._commit_named_record(successor_path, successor, "successor review record")

        status = self._status()

        self.assertTrue(status.ok, status.problems)
        self.assertEqual(successor_path, status.review_record_path)
        self.assertEqual(successor["reviewed_head"], status.reviewed_head)
        self.assertEqual(sorted(self.TRUST_BLOBS), status.touched_paths)

    def test_forked_record_reviewed_heads_are_red(self) -> None:
        predecessor = self._record()
        self._commit_record(predecessor)
        predecessor_tip = self._git("rev-parse", "HEAD")

        self._git("switch", "--detach", self.base)
        self._commit_file("scripts/garnet_fork.py", b"fork = True\n", "fork trust change")
        fork_head = self._git("rev-parse", "HEAD")
        fork_tree = self._git("rev-parse", "HEAD^{tree}")
        self._git("switch", "--detach", predecessor_tip)
        self._git("merge", "--no-ff", fork_head, "-m", "merge forked trust history")

        successor = dict(predecessor)
        successor["reviewed_head"] = fork_head
        successor["reviewed_tree"] = fork_tree
        self._commit_named_record(
            "F_Project_Management/W_TRUST/LANE1_FORKED.review.json",
            successor,
            "forked review record",
        )

        status = self._status()
        self.assertFalse(status.ok)
        self.assertTrue(
            any("reviewed_heads must be strictly ordered" in p for p in status.problems),
            status.problems,
        )

    def test_modified_predecessor_record_stays_red_under_succession(self) -> None:
        predecessor = self._record()
        self._commit_record(predecessor)
        self._advance_reviewed_head("scripts/garnet_beta.py", b"beta = 2\n")
        successor_path = "F_Project_Management/W_TRUST/LANE1_SUCCESSOR.review.json"
        successor = self._record()
        modified = dict(predecessor)
        modified["review_scope"] = (
            "Independent review ended at reviewed_head; content proof does not "
            "extend or backdate review coverage. Modified later."
        )
        (self.root / self.RECORD).write_bytes(_canonical(modified))
        successor_file = self.root / successor_path
        successor_file.write_bytes(_canonical(successor))
        self._git("add", self.RECORD, successor_path)
        self._git("commit", "-m", "modify predecessor and add successor")

        status = self._status()
        self.assertFalse(status.ok)
        self.assertTrue(
            any("record changed or was removed" in p for p in status.problems),
            status.problems,
        )

    def test_malformed_predecessor_record_stays_red_under_succession(self) -> None:
        predecessor = self._record()
        predecessor["schema"] = "garnet.invalid/v1"
        self._commit_record(predecessor)
        self._advance_reviewed_head("scripts/garnet_beta.py", b"beta = 2\n")
        self._commit_named_record(
            "F_Project_Management/W_TRUST/LANE1_SUCCESSOR.review.json",
            self._record(),
            "successor review record",
        )

        status = self._status()
        self.assertFalse(status.ok)
        self.assertTrue(
            any("predecessor schema" in p for p in status.problems), status.problems
        )

    def test_deleted_predecessor_record_stays_red_under_succession(self) -> None:
        self._commit_record()
        self._advance_reviewed_head("scripts/garnet_beta.py", b"beta = 2\n")
        successor_path = "F_Project_Management/W_TRUST/LANE1_SUCCESSOR.review.json"
        successor = self._record()
        self._git("rm", self.RECORD)
        successor_file = self.root / successor_path
        successor_file.write_bytes(_canonical(successor))
        self._git("add", successor_path)
        self._git("commit", "-m", "delete predecessor and add successor")

        status = self._status()
        self.assertFalse(status.ok)
        self.assertTrue(
            any("record changed or was removed" in p for p in status.problems),
            status.problems,
        )

    def test_tip_record_missing_full_range_touched_path_is_red(self) -> None:
        self._commit_record()
        self._advance_reviewed_head("scripts/garnet_beta.py", b"beta = 2\n")
        successor = self._record()
        successor["touched_paths"] = sorted(self.TRUST_BLOBS)[:-1]
        self._commit_named_record(
            "F_Project_Management/W_TRUST/LANE1_SUCCESSOR.review.json",
            successor,
            "incomplete successor review record",
        )

        status = self._status()
        self.assertFalse(status.ok)
        self.assertTrue(
            any("missing touched path" in p for p in status.problems), status.problems
        )

    def test_tip_record_bound_to_non_tip_reviewed_head_is_red(self) -> None:
        stale_terminal = self._record()
        self._advance_reviewed_head("scripts/garnet_beta.py", b"beta = 2\n")
        newest = self._record()
        self._commit_record(newest)
        self._commit_named_record(
            "F_Project_Management/W_TRUST/LANE1_STALE_TERMINAL.review.json",
            stale_terminal,
            "stale terminal review record",
        )

        status = self._status()
        self.assertFalse(status.ok)
        self.assertTrue(
            any("tip-most record must bind the newest reviewed_head" in p for p in status.problems),
            status.problems,
        )

    def test_missing_touched_path_is_red(self) -> None:
        record = self._record()
        record["touched_paths"] = [sorted(self.TRUST_BLOBS)[0]]
        self._commit_record(record)
        self.assertTrue(any("missing touched path" in p for p in self._status().problems))

    def test_extra_touched_path_is_red(self) -> None:
        record = self._record()
        record["touched_paths"] = sorted(self.TRUST_BLOBS) + ["scripts/garnet_not_changed.py"]
        self._commit_record(record)
        self.assertTrue(any("extra touched path" in p for p in self._status().problems))

    def test_empty_touched_paths_is_red(self) -> None:
        record = self._record()
        record["touched_paths"] = []
        self._commit_record(record)
        self.assertTrue(any("must not be empty" in p for p in self._status().problems))

    def test_digest_mismatch_is_red(self) -> None:
        record = self._record()
        record["content_digest"] = "sha256:" + "0" * 64
        self._commit_record(record)
        self.assertTrue(any("content digest mismatch" in p for p in self._status().problems))

    def test_non_pass_verdict_is_red(self) -> None:
        record = self._record()
        record["verdict"] = "needs-work"
        self._commit_record(record)
        self.assertTrue(any("verdict must be pass" in p for p in self._status().problems))

    def test_blocking_findings_are_red(self) -> None:
        record = self._record()
        record["blocking_findings"] = ["Important: fix it"]
        self._commit_record(record)
        self.assertTrue(any("blocking_findings must be empty" in p for p in self._status().problems))

    def test_malformed_reviewer_identity_is_red(self) -> None:
        record = self._record()
        record["reviewer_id"] = -1
        self._commit_record(record)
        self.assertTrue(any("reviewer identity" in p for p in self._status().problems))

    def test_reviewer_author_overlap_is_red(self) -> None:
        record = self._record()
        record["reviewer_id"] = 101
        self._commit_record(record)
        self.assertTrue(any("overlaps an author" in p for p in self._status().problems))

    def test_wrong_author_set_is_red(self) -> None:
        record = self._record()
        record["author_ids"] = [303]
        self._commit_record(record)
        self.assertTrue(any("authors do not match" in p for p in self._status().problems))

    def test_omitted_author_set_is_red(self) -> None:
        record = self._record()
        record["author_ids"] = []
        self._commit_record(record)
        self.assertTrue(any("authors" in p for p in self._status().problems))

    def test_unknown_key_is_red(self) -> None:
        record = self._record()
        record["blessing"] = True
        self._commit_record(record)
        self.assertTrue(any("unknown key" in p for p in self._status().problems))

    def test_duplicate_key_is_red(self) -> None:
        raw = _canonical(self._record()).replace(
            b'  "verdict": "pass"\n',
            b'  "verdict": "pass",\n  "verdict": "pass"\n',
        )
        self._commit_record(raw=raw)
        self.assertTrue(any("duplicate key" in p for p in self._status().problems))

    def test_noncanonical_json_is_red(self) -> None:
        raw = (json.dumps(self._record(), sort_keys=False) + "\n").encode()
        self._commit_record(raw=raw)
        self.assertTrue(any("canonical JSON" in p for p in self._status().problems))

    def test_reviewed_tree_must_match_reviewed_head(self) -> None:
        record = self._record()
        record["reviewed_tree"] = "0" * 40
        self._commit_record(record)
        self.assertTrue(any("reviewed_tree mismatch" in p for p in self._status().problems))

    def test_backslash_alias_is_not_an_exact_git_path(self) -> None:
        record = self._record()
        record["touched_paths"] = [
            "scripts/garnet_alpha.py",
            "scripts\\test_garnet_alpha.py",
        ]
        self._commit_record(record)
        status = self._status()
        self.assertFalse(status.ok, status.problems)
        self.assertTrue(
            any("canonical Git path syntax" in problem for problem in status.problems),
            status.problems,
        )

    def test_reviewed_head_must_precede_current_head(self) -> None:
        record = self._record()
        record["reviewed_head"] = "a" * 40
        self._commit_record(record)
        self.assertTrue(any("reviewed_head does not name a commit" in p for p in self._status().problems))

    def test_record_must_bind_the_discovered_base(self) -> None:
        record = self._record()
        record["base_commit"] = "0" * 40
        self._commit_record(record)
        self.assertTrue(any("base_commit does not match" in p for p in self._status().problems))

    def test_record_cannot_claim_a_premerge_merged_commit(self) -> None:
        record = self._record()
        record["merged_commit"] = self.reviewed_head
        self._commit_record(record)
        self.assertTrue(any("unknown key" in p for p in self._status().problems))

    def test_trust_content_changed_after_reviewed_head_is_red(self) -> None:
        write_lf(self.root / "scripts/garnet_alpha.py", "alpha = 2\n")
        self._git("commit", "-am", "post-review trust mutation")
        self._commit_record(self._record())
        self.assertTrue(any("content digest mismatch" in p for p in self._status().problems))

    def test_edit_then_revert_after_reviewed_head_is_still_red(self) -> None:
        path = self.root / "scripts/garnet_alpha.py"
        write_lf(path, "alpha = 2\n")
        self._git("commit", "-am", "post-review edit")
        path.write_bytes(self.TRUST_BLOBS["scripts/garnet_alpha.py"])
        self._git("commit", "-am", "post-review revert")
        self._commit_record(self._record())
        self.assertTrue(any("post-review trust touch" in p for p in self._status().problems))

    def test_merge_commit_trust_touch_is_checked_against_each_review_lineage_parent(self) -> None:
        self._git("checkout", "-b", "post-review-side", self.reviewed_head)
        self._commit_file("side.txt", b"side\n", "side non-trust")
        self._git("checkout", "main")
        self._commit_file("main.txt", b"main\n", "main non-trust")
        self._git("merge", "--no-ff", "--no-commit", "post-review-side")
        path = self.root / "scripts/garnet_alpha.py"
        write_lf(path, "alpha = 2\n")
        self._git("add", "scripts/garnet_alpha.py")
        self._git("commit", "-m", "merge with trust resolution")
        path.write_bytes(self.TRUST_BLOBS["scripts/garnet_alpha.py"])
        self._git("commit", "-am", "restore reviewed trust content")
        self._commit_record(self._record())
        findings = self._status().problems
        self.assertTrue(any("post-review trust touch" in p for p in findings))
        self.assertTrue(any("merge" in p for p in findings))

    def _merge_parent_outside_reviewed_lineage(self, *, mutate_trust: bool) -> str:
        self._git("checkout", "-b", "parallel-candidate", self.base)
        self._commit_file("parallel.txt", b"parallel\n", "parallel candidate")
        self._git("checkout", "main")
        self._commit_file("reviewed-lineage.txt", b"reviewed lineage\n", "reviewed lineage")
        if mutate_trust:
            self._git("merge", "--no-ff", "--no-commit", "parallel-candidate")
            write_lf(self.root / "scripts/garnet_alpha.py", "alpha = 2\n")
            self._git("add", "scripts/garnet_alpha.py")
            self._git("commit", "-m", "integration merge with one-byte trust mutation")
        else:
            self._git("merge", "--no-ff", "parallel-candidate", "-m", "integration merge")
        return self._git("rev-parse", "HEAD")

    def test_merge_parent_outside_reviewed_lineage_is_accepted_at_equal_trust_snapshot(
        self,
    ) -> None:
        head = self._merge_parent_outside_reviewed_lineage(mutate_trust=False)
        findings = mod._post_review_trust_findings(
            self.reviewed_head,
            head,
            self.root,
        )
        self.assertEqual([], findings)

    def test_merge_parent_outside_reviewed_lineage_with_mutated_trust_stays_red(
        self,
    ) -> None:
        head = self._merge_parent_outside_reviewed_lineage(mutate_trust=True)
        findings = mod._post_review_trust_findings(
            self.reviewed_head,
            head,
            self.root,
        )
        self.assertTrue(any("post-review trust touch" in finding for finding in findings))


class AuthorEnumerationTests(unittest.TestCase):
    def test_git_log_failure_is_red(self) -> None:
        failed = mod.GitResult(128, b"", b"fatal: rev-list failed")
        with mock.patch.object(mod, "_git_bytes", return_value=failed):
            authors, problems = mod.derive_author_identities("a" * 40, "b" * 40, Path("/"))
        self.assertEqual([], authors)
        self.assertTrue(any("author commit enumeration failed" in p for p in problems))

    def test_partial_git_log_output_is_red(self) -> None:
        malformed = mod.GitResult(0, (("a" * 40) + "\ntruncated\n").encode(), b"")
        with mock.patch.object(mod, "_git_bytes", return_value=malformed):
            authors, problems = mod.derive_author_identities("a" * 40, "b" * 40, Path("/"))
        self.assertEqual([], authors)
        self.assertTrue(any("malformed" in p for p in problems))

    def test_missing_author_output_is_red(self) -> None:
        commits = mod.GitResult(0, (("c" * 40) + "\n").encode(), b"")
        count = mod.GitResult(0, b"1\n", b"")
        empty_author = mod.GitResult(0, b"", b"")
        with mock.patch.object(mod, "_git_bytes", side_effect=[commits, count, empty_author]):
            authors, problems = mod.derive_author_identities("a" * 40, "b" * 40, Path("/"))
        self.assertEqual([], authors)
        self.assertTrue(any("author identity output" in p for p in problems))

    def test_valid_but_partial_commit_list_is_red_against_count(self) -> None:
        commits = mod.GitResult(0, (("c" * 40) + "\n").encode(), b"")
        count = mod.GitResult(0, b"2\n", b"")
        with mock.patch.object(mod, "_git_bytes", side_effect=[commits, count]):
            authors, problems = mod.derive_author_identities("a" * 40, "b" * 40, Path("/"))
        self.assertEqual([], authors)
        self.assertTrue(any("partial" in p for p in problems))


class CommitGraphEnumerationTests(GitRepoFixture):
    def test_identically_truncated_rev_list_and_count_are_red_against_commit_objects(self) -> None:
        self._commit_file("one.txt", b"one\n", "one")
        self._commit_file("two.txt", b"two\n", "two")
        head = self._git("rev-parse", "HEAD")
        last = head
        with mock.patch.object(mod, "_presented_commit_ids", return_value=([last], [])):
            authors, problems = mod.derive_author_identities(self.base, head, self.root)
        self.assertEqual([], authors)
        self.assertTrue(any("commit-object traversal" in p and "partial" in p for p in problems))


class GitGraphControlPlaneTests(GitRepoFixture):
    def test_default_info_grafts_cannot_hide_a_trust_change(self) -> None:
        trust_path = "scripts/garnet_hidden_by_graft.py"
        self._git("checkout", "-b", "topic", self.base)
        topic = self._commit_file(trust_path, b"hidden = True\n", "unreviewed trust change")

        self._git("checkout", "main")
        main = self._commit_file("main-advanced.txt", b"advanced\n", "advance main")
        self._git("update-ref", "refs/remotes/origin/main", main)
        self._git("checkout", "topic")

        clean = mod.read_status(root=self.root)
        self.assertFalse(clean.ok, clean.problems)
        self.assertIn(trust_path, clean.touched_paths, clean.problems)

        grafts = self.root / ".git" / "info" / "grafts"
        grafts.parent.mkdir(parents=True, exist_ok=True)
        write_lf(grafts, f"{main} {topic}\n", encoding="ascii")

        grafted = mod.read_status(root=self.root)
        self.assertFalse(grafted.ok, grafted.problems)
        self.assertIn(trust_path, grafted.touched_paths, grafted.problems)


class WorktreeStatusTests(GitRepoFixture):
    def test_clean_porcelain_v2_status_passes(self) -> None:
        self.assertEqual([], mod.check_clean_worktree(self.root))

    def test_unstaged_change_is_red(self) -> None:
        write_lf(self.root / "README.md", "dirty\n")
        self.assertTrue(any("not clean" in p for p in mod.check_clean_worktree(self.root)))

    def test_staged_change_is_red(self) -> None:
        self._commit_file("tracked.txt", b"base\n", "tracked")
        write_lf(self.root / "tracked.txt", "staged\n")
        self._git("add", "tracked.txt")
        self.assertTrue(any("not clean" in p for p in mod.check_clean_worktree(self.root)))

    def test_untracked_change_is_red(self) -> None:
        write_lf(self.root / "untracked.txt", "new\n")
        self.assertTrue(any("not clean" in p for p in mod.check_clean_worktree(self.root)))

    def test_ambient_git_repository_redirect_cannot_hide_dirty_worktree(self) -> None:
        write_lf(self.root / "untracked.txt", "dirty real checkout\n")
        with tempfile.TemporaryDirectory() as alternate_temp:
            alternate = Path(alternate_temp).resolve()
            result = subprocess.run(
                ["git", "init"],
                cwd=alternate,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(0, result.returncode, result.stderr)
            redirected = {
                "GIT_DIR": str(alternate / ".git"),
                "GIT_INDEX_FILE": str(alternate / ".git" / "index"),
                "GIT_WORK_TREE": str(alternate),
            }
            with mock.patch.dict(os.environ, redirected, clear=False):
                findings = mod.check_clean_worktree(self.root)
        self.assertTrue(any("not clean" in finding for finding in findings), findings)

    def test_malformed_or_non_nul_porcelain_is_red(self) -> None:
        _, problems = mod._parse_porcelain_v2_z(b"? unterminated")
        self.assertTrue(any("NUL" in p for p in problems))
        _, problems = mod._parse_porcelain_v2_z(b"x invalid\0")
        self.assertTrue(any("malformed" in p for p in problems))

    def test_status_failure_and_timeout_are_red(self) -> None:
        failed = mod.GitResult(128, b"", b"fatal")
        with mock.patch.object(mod, "_git_bytes", return_value=failed):
            self.assertTrue(any("status enumeration failed" in p for p in mod.check_clean_worktree(self.root)))
        timeout = mod.GitResult(124, b"", b"", timed_out=True)
        with mock.patch.object(mod, "_git_bytes", return_value=timeout):
            self.assertTrue(any("timed out" in p for p in mod.check_clean_worktree(self.root)))


class GitCredentialIsolationTests(unittest.TestCase):
    def test_git_subprocess_does_not_inherit_credential_shaped_environment(self) -> None:
        captured: dict[str, str] = {}

        def fake_run(*_: object, **kwargs: object) -> object:
            captured.update(kwargs["env"])
            return types.SimpleNamespace(returncode=0, stdout=b"", stderr=b"")

        ambient = {
            "GITHUB_TOKEN": "github-secret",
            "REVIEW_TOKEN": "review-secret",
            "GARNET_ADMIN_GITHUB_TOKEN": "admin-secret",
            "EXAMPLE_PASSWORD": "password-secret",
            "HTTP_AUTHORIZATION": "authorization-secret",
            "SESSION_COOKIE": "cookie-secret",
            "DEPLOY_PRIVATE_KEY": "private-key-secret",
            "UNRELATED_VALUE": "safe",
        }
        with mock.patch.dict(os.environ, ambient, clear=True), mock.patch.object(
            mod.subprocess, "run", side_effect=fake_run
        ):
            result = mod._git_bytes(Path("/"), "status")
        self.assertEqual(0, result.returncode)
        for name in ambient:
            self.assertNotIn(name, captured)

    def test_git_subprocess_does_not_inherit_repository_control_plane(self) -> None:
        captured: dict[str, str] = {}

        def fake_run(*_: object, **kwargs: object) -> object:
            captured.update(kwargs["env"])
            return types.SimpleNamespace(returncode=0, stdout=b"", stderr=b"")

        ambient = {
            "GIT_ALTERNATE_OBJECT_DIRECTORIES": "/alternate/objects",
            "GIT_COMMON_DIR": "/alternate/common",
            "GIT_CONFIG_COUNT": "1",
            "GIT_CONFIG_GLOBAL": "/alternate/global-config",
            "GIT_CONFIG_KEY_0": "core.worktree",
            "GIT_CONFIG_NOSYSTEM": "0",
            "GIT_CONFIG_SYSTEM": "/alternate/system-config",
            "GIT_CONFIG_VALUE_0": "/alternate/worktree",
            "GIT_DIR": "/alternate/repository",
            "GIT_GRAFT_FILE": "/alternate/grafts",
            "GIT_INDEX_FILE": "/alternate/index",
            "GIT_NAMESPACE": "alternate",
            "GIT_OBJECT_DIRECTORY": "/alternate/objects",
            "GIT_QUARANTINE_PATH": "/alternate/quarantine",
            "GIT_REPLACE_REF_BASE": "refs/alternate/replace/",
            "GIT_SHALLOW_FILE": "/alternate/shallow",
            "GIT_WORK_TREE": "/alternate/worktree",
        }
        with mock.patch.dict(os.environ, ambient, clear=True), mock.patch.object(
            mod.subprocess, "run", side_effect=fake_run
        ):
            result = mod._git_bytes(Path("/"), "status")
        self.assertEqual(0, result.returncode)
        for name, value in ambient.items():
            self.assertNotEqual(value, captured.get(name), name)
        self.assertEqual("1", captured.get("GIT_CONFIG_NOSYSTEM"))
        self.assertEqual("0", captured.get("GIT_CONFIG_COUNT"))
        self.assertEqual(os.devnull, captured.get("GIT_GRAFT_FILE"))
        self.assertEqual("1", captured.get("GIT_NO_REPLACE_OBJECTS"))

    def test_git_graft_warning_is_an_enumeration_failure(self) -> None:
        completed = types.SimpleNamespace(
            returncode=0,
            stdout=b"a" * 40 + b"\n",
            stderr=b"hint: support for .git/info/grafts is deprecated\n",
        )
        with mock.patch.object(mod.subprocess, "run", return_value=completed):
            result = mod._git_bytes(Path("/"), "rev-parse", "HEAD")
        self.assertNotEqual(0, result.returncode)
        self.assertIn(b"graft", result.stderr)


class AppendOnlyReviewRecordTests(unittest.TestCase):
    PATH = "F_Project_Management/W_TRUST/ITEM.review.json"

    def _entry(self, status: str, old_mode: str, new_mode: str) -> object:
        return mod.ChangeEntry(
            self.PATH,
            status,
            old_mode,
            new_mode,
            "0" * 40 if status == "A" else "1" * 40,
            "0" * 40 if status == "D" else "2" * 40,
        )

    def test_new_regular_nonexecutable_record_is_allowed(self) -> None:
        self.assertEqual([], mod._review_record_append_only_findings([self._entry("A", "000000", "100644")]))

    def test_historical_modify_delete_and_type_change_are_red(self) -> None:
        for entry in (
            self._entry("M", "100644", "100644"),
            self._entry("D", "100644", "000000"),
            self._entry("T", "100644", "120000"),
        ):
            with self.subTest(status=entry.status):
                findings = mod._review_record_append_only_findings([entry])
                self.assertTrue(any("append-only" in p for p in findings))

    def test_new_symlink_executable_and_gitlink_records_are_red(self) -> None:
        for mode in ("120000", "100755", "160000"):
            with self.subTest(mode=mode):
                findings = mod._review_record_append_only_findings(
                    [self._entry("A", "000000", mode)]
                )
                self.assertTrue(any("regular 100644" in p for p in findings))


class ReviewRecordHistoryTests(GitRepoFixture):
    RECORD = "F_Project_Management/W_TRUST/HISTORICAL.review.json"
    RECORD_BYTES = _canonical({"historical": True})

    def setUp(self) -> None:
        super().setUp()
        self._commit_file(self.RECORD, self.RECORD_BYTES, "add historical review record")
        self.base = self._git("rev-parse", "HEAD")
        self._git("update-ref", "refs/remotes/origin/main", self.base)

    def test_intermediate_review_record_modify_then_restore_is_red(self) -> None:
        path = self.root / self.RECORD
        path.write_bytes(_canonical({"historical": "mutated"}))
        self._git("commit", "-am", "mutate historical review record")
        path.write_bytes(self.RECORD_BYTES)
        self._git("commit", "-am", "restore historical review record")
        status = mod.read_status(base=self.base, head="HEAD", root=self.root)
        self.assertFalse(status.ok, status.problems)
        self.assertTrue(
            any("structured review record history is append-only" in p for p in status.problems),
            status.problems,
        )

    def test_intermediate_review_record_delete_then_restore_is_red(self) -> None:
        path = self.root / self.RECORD
        path.unlink()
        self._git("add", "-u")
        self._git("commit", "-m", "delete historical review record")
        path.write_bytes(self.RECORD_BYTES)
        self._git("add", self.RECORD)
        self._git("commit", "-m", "restore historical review record")
        status = mod.read_status(base=self.base, head="HEAD", root=self.root)
        self.assertFalse(status.ok, status.problems)
        self.assertTrue(
            any("structured review record history is append-only" in p for p in status.problems),
            status.problems,
        )

    def test_one_new_regular_review_record_has_clean_append_only_history(self) -> None:
        self._commit_file(
            "F_Project_Management/W_TRUST/NEW.review.json",
            _canonical({"new": True}),
            "add new review record",
        )
        self.assertEqual(
            [],
            mod._review_record_history_findings(
                self.root, self.base, self._git("rev-parse", "HEAD")
            ),
        )

    def test_parallel_duplicate_introduction_of_one_record_is_red(self) -> None:
        record = "F_Project_Management/W_TRUST/PARALLEL.review.json"
        payload = _canonical({"parallel": True})

        self._git("checkout", "-b", "left", self.base)
        self._commit_file(record, payload, "add parallel review record on left")

        self._git("checkout", "-b", "right", self.base)
        self._commit_file(record, payload, "add parallel review record on right")

        self._git("checkout", "left")
        self._git("merge", "--no-ff", "-m", "merge duplicate introductions", "right")

        status = mod.read_status(base=self.base, head="HEAD", root=self.root)
        self.assertFalse(status.ok, status.problems)
        self.assertTrue(
            any("introduced more than once" in problem for problem in status.problems),
            status.problems,
        )

    def test_record_only_non_json_addition_is_red(self) -> None:
        self._commit_file(
            "F_Project_Management/W_TRUST/FORGED.review.json",
            b"not a review record\n",
            "add forged record without trust changes",
        )

        status = mod.read_status(base=self.base, head="HEAD", root=self.root)
        self.assertFalse(status.ok, status.problems)
        self.assertTrue(
            any("not valid JSON" in problem for problem in status.problems),
            status.problems,
        )


class PreMergeDeletionRecordTests(GitRepoFixture):
    RECORD = "F_Project_Management/W_TRUST/DELETE.review.json"
    TRUST_PATH = "scripts/garnet_deleted.py"
    OLD_BYTES = b"delete_me = True\n"

    def test_exact_deletion_tombstone_can_be_reviewed(self) -> None:
        self._commit_file(self.TRUST_PATH, self.OLD_BYTES, "base trust file")
        self.base = self._git("rev-parse", "HEAD")
        old_oid = self._git("rev-parse", f"{self.base}:{self.TRUST_PATH}")
        self._git("update-ref", "refs/remotes/origin/main", self.base)
        (self.root / self.TRUST_PATH).unlink()
        self._git("add", "-u")
        self._git("commit", "-m", "reviewed deletion")
        reviewed_head = self._git("rev-parse", "HEAD")
        record = {
            "author_emails": ["email:author@example.invalid"],
            "author_ids": [101],
            "base_commit": self.base,
            "blocking_findings": [],
            "content_digest": _change_digest(
                [(self.TRUST_PATH, "D", old_oid, self.OLD_BYTES, "0" * 40, None)]
            ),
            "head_repository": "Navigata1/garnet",
            "head_repository_id": 6006,
            "pull_request_id": 7007,
            "pull_request_number": 77,
            "repository": "Island-Dev-Crew/garnet",
            "repository_id": 5005,
            "review_scope": (
                "Independent review ended at reviewed_head; content proof does not "
                "extend or backdate review coverage."
            ),
            "reviewed_head": reviewed_head,
            "reviewed_tree": self._git("rev-parse", "HEAD^{tree}"),
            "review_state": "APPROVED",
            "reviewer_id": 202,
            "reviewer_login": "independent-reviewer",
            "schema": "garnet.trust_kernel_review_record/v2",
            "state": "premerge",
            "touched_paths": [self.TRUST_PATH],
            "verdict": "pass",
        }
        self._commit_file(self.RECORD, _canonical(record), "deletion review record")
        head = self._git("rev-parse", "HEAD")
        commits = self._git("rev-list", "--reverse", f"{self.base}..{head}").splitlines()
        review = {
            "id": 9009,
            "user": {"id": 202, "login": "independent-reviewer"},
            "state": "APPROVED",
            "commit_id": head,
        }

        class Transport:
            def get_collection(inner_self, path: str, **_: object):
                rows = (
                    (review,)
                    if path.endswith("/reviews")
                    else tuple(
                        {
                            "sha": commit,
                            "author": {"id": 101, "login": "author"},
                            "committer": {"id": 101, "login": "author"},
                        }
                        for commit in commits
                    )
                )
                return types.SimpleNamespace(rows=rows, problems=(), page_count=1, byte_count=1)

            def get_object(inner_self, path: str):
                value = (
                    review
                    if "/reviews/" in path
                    else {
                        "id": 7007,
                        "number": 77,
                        "head": {
                            "sha": head,
                            "repo": {"id": 6006, "full_name": "Navigata1/garnet"},
                        },
                        "base": {
                            "repo": {"id": 5005, "full_name": "Island-Dev-Crew/garnet"},
                        },
                    }
                )
                return types.SimpleNamespace(value=value, problems=(), byte_count=1)

        status = mod.read_status(
            base=self.base,
            head="HEAD",
            root=self.root,
            github_transport=Transport(),
            repository="Island-Dev-Crew/garnet",
            pull_request=77,
        )
        self.assertTrue(status.ok, status.problems)


class DeletedReviewRecordTests(GitRepoFixture):
    RECORD = "F_Project_Management/W_TRUST/DELETED.review.json"

    def test_deleted_structured_record_is_red(self) -> None:
        self._commit_file(self.RECORD, b"{}\n", "base placeholder record")
        self.base = self._git("rev-parse", "HEAD")
        self._git("update-ref", "refs/remotes/origin/main", self.base)
        (self.root / self.RECORD).unlink()
        trust = self.root / "scripts/garnet_new.py"
        trust.parent.mkdir(parents=True, exist_ok=True)
        write_lf(trust, "new = True\n")
        self._git("add", "-A")
        self._git("commit", "-m", "trust change deletes review record")
        status = mod.read_status(base=self.base, head="HEAD", root=self.root)
        self.assertFalse(status.ok)
        self.assertTrue(any("path is missing" in p for p in status.problems))


class LandedMarkerTests(GitRepoFixture):
    TRUST_PATH = "scripts/garnet_landed.py"
    TRUST_BYTES = b"landed = True\n"
    RECORD_PATH = "F_Project_Management/W_TRUST/LANDED.review.json"

    def setUp(self) -> None:
        super().setUp()
        self._git("checkout", "-b", "topic")
        self._commit_file(self.TRUST_PATH, self.TRUST_BYTES, "reviewed topic")
        self.actual_reviewed_head = self._git("rev-parse", "HEAD")
        self.actual_reviewed_tree = self._git("rev-parse", "HEAD^{tree}")
        self.new_oid = self._git("rev-parse", f"HEAD:{self.TRUST_PATH}")
        self.digest = _change_digest(
            [(self.TRUST_PATH, "A", "0" * 40, None, self.new_oid, self.TRUST_BYTES)]
        )
        self.record = {
            "author_emails": ["email:author@example.invalid"],
            "author_ids": [101],
            "base_commit": self.base,
            "blocking_findings": [],
            "content_digest": self.digest,
            "head_repository": "Navigata1/garnet",
            "head_repository_id": 6006,
            "pull_request_id": 7007,
            "pull_request_number": 77,
            "repository": "Island-Dev-Crew/garnet",
            "repository_id": 5005,
            "review_scope": (
                "Independent review ended at reviewed_head; content proof does not "
                "extend or backdate review coverage."
            ),
            "reviewed_head": self.actual_reviewed_head,
            "reviewed_tree": self.actual_reviewed_tree,
            "review_state": "APPROVED",
            "reviewer_id": 202,
            "reviewer_login": "independent-reviewer",
            "schema": "garnet.trust_kernel_review_record/v2",
            "state": "premerge",
            "touched_paths": [self.TRUST_PATH],
            "verdict": "pass",
        }
        self.record_bytes = _canonical(self.record)
        self._git("checkout", "main")
        self._commit_file("main-advanced.txt", b"advanced\n", "unrelated main advance")
        trust = self.root / self.TRUST_PATH
        trust.parent.mkdir(parents=True, exist_ok=True)
        trust.write_bytes(self.TRUST_BYTES)
        review = self.root / self.RECORD_PATH
        review.parent.mkdir(parents=True, exist_ok=True)
        review.write_bytes(self.record_bytes)
        self._git("add", self.TRUST_PATH, self.RECORD_PATH)
        self._git("commit", "-m", "squash result")
        self.merged_commit = self._git("rev-parse", "HEAD")
        self.merged_tree = self._git("rev-parse", "HEAD^{tree}")
        self._git("update-ref", "refs/remotes/origin/main", self.merged_commit)

    def _marker(self) -> dict[str, object]:
        return {
            "author_emails": ["email:author@example.invalid"],
            "author_ids": [101],
            "base_commit": self.base,
            "blocking_findings": [],
            "content_digest": self.digest,
            "head_repository": "Navigata1/garnet",
            "head_repository_id": 6006,
            "merged_commit": self.merged_commit,
            # Optional: the surface in force at the landing comes from the era
            # ledger; a declared value may only agree with it.
            "trust_surface": mod.CURRENT_TRUST_SURFACE,
            "merged_tree": self.merged_tree,
            "pull_request_id": 7007,
            "pull_request_number": 77,
            "repository": "Island-Dev-Crew/garnet",
            "repository_id": 5005,
            "review_record_path": self.RECORD_PATH,
            "review_record_sha256": hashlib.sha256(self.record_bytes).hexdigest(),
            "review_scope": self.record["review_scope"],
            # The reviewed topic is not on main's first-parent history.  Landed
            # verification binds its committed record; it does not require ancestry.
            "reviewed_head": self.actual_reviewed_head,
            "reviewed_tree": self.actual_reviewed_tree,
            "review_state": "APPROVED",
            "reviewer_id": 202,
            "reviewer_login": "independent-reviewer",
            "schema": "garnet.trust_kernel_review_marker/v2",
            "state": "landed",
            "touched_paths": [self.TRUST_PATH],
            "verdict": "pass",
        }

    def _findings(self, marker: dict[str, object] | None = None) -> list[str]:
        return mod.verify_landed_review_marker(
            marker or self._marker(),
            root=self.root,
            main_ref="refs/remotes/origin/main",
        )

    def _commit_repository_marker(self, marker: dict[str, object]) -> str:
        marker_path = "F_Project_Management/W_TRUST/landed/LANDED.landed-review.json"
        path = self.root / marker_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(_canonical(marker))
        registry = self.root / "F_Project_Management/W_TRUST/LANDED_REVIEW_MARKERS.json"
        registry.write_bytes(
            _canonical(
                {
                    "markers": [marker_path],
                    "schema": "garnet.trust_kernel_landed_review_registry/v1",
                }
            )
        )
        self._git("add", marker_path, str(registry.relative_to(self.root)))
        self._git("commit", "-m", "register landed review marker")
        return marker_path

    def test_valid_landed_marker_does_not_require_reviewed_head_ancestry(self) -> None:
        self.assertEqual([], self._findings())

    def test_later_unrelated_main_commit_cannot_substitute_for_landing_boundary(self) -> None:
        later = self._commit_file("later.txt", b"later\n", "later unrelated main commit")
        self._git("update-ref", "refs/remotes/origin/main", later)
        marker = self._marker()
        marker["merged_commit"] = later
        marker["merged_tree"] = self._git("rev-parse", f"{later}^{{tree}}")
        findings = self._findings(marker)
        self.assertTrue(
            any("exact first-parent landing edge" in problem for problem in findings),
            findings,
        )

    def test_repository_registry_drives_production_landed_verification(self) -> None:
        self._commit_repository_marker(self._marker())
        self.assertEqual([], mod.verify_repository_landed_markers(self.root, ref="HEAD"))

    def test_repository_marker_history_is_append_only(self) -> None:
        marker_path = self._commit_repository_marker(self._marker())
        registered = self._git("rev-parse", "HEAD")
        self._git("update-ref", "refs/remotes/origin/main", registered)
        (self.root / marker_path).unlink()
        registry = self.root / "F_Project_Management/W_TRUST/LANDED_REVIEW_MARKERS.json"
        registry.write_bytes(
            _canonical(
                {
                    "markers": [],
                    "schema": "garnet.trust_kernel_landed_review_registry/v1",
                }
            )
        )
        self._git("add", "-A")
        self._git("commit", "-m", "attempt to erase landed review history")
        status = mod.read_status(base=registered, head="HEAD", root=self.root)
        self.assertFalse(status.ok, status.problems)
        self.assertTrue(
            any("landed review history is append-only" in problem for problem in status.problems),
            status.problems,
        )

    def test_landed_marker_rejects_unverified_approval_claim_fields(self) -> None:
        marker = self._marker()
        marker["approval_head"] = "a" * 40
        marker["review_id"] = 9009
        findings = self._findings(marker)
        self.assertTrue(any("unknown key" in problem for problem in findings), findings)

    def test_invalid_registered_marker_makes_normal_status_red(self) -> None:
        marker = self._marker()
        marker["merged_tree"] = "f" * 40
        self._commit_repository_marker(marker)
        status = mod.read_status(base="HEAD", head="HEAD", root=self.root)
        self.assertFalse(status.ok)
        self.assertTrue(any("registered landed marker" in p for p in status.problems))

    def test_missing_merged_commit_is_red(self) -> None:
        marker = self._marker()
        marker.pop("merged_commit")
        self.assertTrue(any("merged_commit is missing" in p for p in self._findings(marker)))

    def test_merged_commit_absent_from_first_parent_is_red(self) -> None:
        self._git("checkout", "-b", "side", self.base)
        side = self._commit_file(self.TRUST_PATH, b"side\n", "side only")
        marker = self._marker()
        marker["merged_commit"] = side
        marker["merged_tree"] = self._git("rev-parse", f"{side}^{{tree}}")
        side_oid = self._git("rev-parse", f"{side}:{self.TRUST_PATH}")
        marker["content_digest"] = _change_digest(
            [(self.TRUST_PATH, "A", "0" * 40, None, side_oid, b"side\n")]
        )
        self.assertTrue(any("first-parent" in p for p in self._findings(marker)))

    def test_merged_tree_mismatch_is_red(self) -> None:
        marker = self._marker()
        marker["merged_tree"] = "c" * 40
        self.assertTrue(any("merged_tree mismatch" in p for p in self._findings(marker)))

    def test_landed_content_digest_mismatch_is_red(self) -> None:
        marker = self._marker()
        marker["content_digest"] = "sha256:" + "d" * 64
        self.assertTrue(any("content digest mismatch" in p for p in self._findings(marker)))

    def test_landed_exact_path_mismatch_is_red(self) -> None:
        marker = self._marker()
        marker["touched_paths"] = sorted([self.TRUST_PATH, "scripts/garnet_extra.py"])
        self.assertTrue(any("extra touched path" in p for p in self._findings(marker)))

    def test_landed_marker_cannot_invent_reviewer(self) -> None:
        marker = self._marker()
        marker["reviewer_id"] = 999
        self.assertTrue(any("does not match committed premerge record" in p for p in self._findings(marker)))

    def test_landed_marker_rejects_review_record_digest_mismatch(self) -> None:
        marker = self._marker()
        marker["review_record_sha256"] = "e" * 64
        self.assertTrue(any("review record SHA-256 mismatch" in p for p in self._findings(marker)))

    def test_landed_marker_rejects_missing_review_record(self) -> None:
        marker = self._marker()
        marker["review_record_path"] = "F_Project_Management/W_TRUST/MISSING.review.json"
        self.assertTrue(any("path is missing" in p for p in self._findings(marker)))

    def test_landed_marker_rejects_modified_historical_review_record(self) -> None:
        trust_path = "scripts/garnet_landed_modified_record.py"
        trust_bytes = b"modified_record = True\n"
        trust = self.root / trust_path
        trust.parent.mkdir(parents=True, exist_ok=True)
        trust.write_bytes(trust_bytes)
        trust_oid = self._git("hash-object", trust_path)
        digest = _change_digest(
            [(trust_path, "A", "0" * 40, None, trust_oid, trust_bytes)]
        )
        forged_record = dict(self.record)
        forged_record.update(
            {
                "base_commit": self.merged_commit,
                "content_digest": digest,
                "touched_paths": [trust_path],
            }
        )
        forged_bytes = _canonical(forged_record)
        (self.root / self.RECORD_PATH).write_bytes(forged_bytes)
        self._git("add", trust_path, self.RECORD_PATH)
        self._git("commit", "-m", "modify historical record on landing edge")
        forged_merge = self._git("rev-parse", "HEAD")
        self._git("update-ref", "refs/remotes/origin/main", forged_merge)

        marker = dict(forged_record)
        marker.update(
            {
                "merged_commit": forged_merge,
                "merged_tree": self._git("rev-parse", "HEAD^{tree}"),
                "review_record_path": self.RECORD_PATH,
                "review_record_sha256": hashlib.sha256(forged_bytes).hexdigest(),
                "schema": "garnet.trust_kernel_review_marker/v2",
                "state": "landed",
            }
        )
        findings = self._findings(marker)
        self.assertTrue(
            any("new regular 100644" in problem for problem in findings),
            findings,
        )

    def test_replace_ref_cannot_substitute_landed_tree(self) -> None:
        self._git("checkout", "-b", "replacement", self.base)
        replacement = self._commit_file(self.TRUST_PATH, b"replacement\n", "replacement")
        replacement_tree = self._git("rev-parse", f"{replacement}^{{tree}}")
        self._git("replace", self.merged_commit, replacement)
        marker = self._marker()
        marker["merged_tree"] = replacement_tree
        replacement_oid = self._git("rev-parse", f"{replacement}:{self.TRUST_PATH}")
        marker["content_digest"] = _change_digest(
            [
                (
                    self.TRUST_PATH,
                    "A",
                    "0" * 40,
                    None,
                    replacement_oid,
                    b"replacement\n",
                )
            ]
        )
        self.assertTrue(any("merged_tree mismatch" in p for p in self._findings(marker)))


class CliTests(GitRepoFixture):
    def _run(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(SCRIPT), *args],
            cwd=self.root,
            capture_output=True,
            text=True,
            check=False,
        )

    def test_gate_passes_on_legitimate_empty_authoritative_clean_git_diff(self) -> None:
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            code = mod.main(["--gate"], root=self.root)
        self.assertEqual(0, code)

    def test_explicit_base_override_is_diagnostic_not_a_gate_bypass(self) -> None:
        result = self._run("--gate", "--base", "HEAD")
        self.assertEqual(1, result.returncode, result.stdout)
        self.assertIn("authoritative merge-base", result.stdout)

    def test_explicit_changed_file_is_diagnostic_not_a_gate_bypass(self) -> None:
        result = self._run("--gate", "--changed-file", "README.md")
        self.assertEqual(1, result.returncode, result.stdout)
        self.assertIn("complete enumeration", result.stdout)

    def test_assume_trailer_cannot_bypass_v2(self) -> None:
        result = self._run(
            "--gate",
            "--changed-file",
            "scripts/garnet_alpha.py",
            "--assume-trailer",
        )
        self.assertEqual(1, result.returncode, result.stdout)


class RepositoryWiringTests(unittest.TestCase):
    def test_ci_injects_bounded_pull_request_review_transport(self) -> None:
        workflow = (mod.ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
        self.assertIn("permissions:\n  contents: read\n  pull-requests: read", workflow)
        self.assertIn(
            "ref: ${{ github.event.pull_request.head.sha || github.sha }}", workflow
        )
        self.assertIn("persist-credentials: false", workflow)
        self.assertIn("name: rolling trust-kernel review (pull request)", workflow)
        self.assertIn("if: github.event_name == 'pull_request'", workflow)
        self.assertIn("REVIEW_TOKEN: ${{ github.token }}", workflow)
        self.assertIn("REVIEW_REPO: ${{ github.repository }}", workflow)
        self.assertIn("REVIEW_PR: ${{ github.event.pull_request.number }}", workflow)
        self.assertIn("--github-repo \"$REVIEW_REPO\"", workflow)
        self.assertIn("--github-pr \"$REVIEW_PR\"", workflow)
        self.assertIn("--github-token-stdin", workflow)
        self.assertIn("review_token=\"$REVIEW_TOKEN\"", workflow)
        self.assertIn("printf '%s' \"$review_token\"", workflow)
        self.assertIn(
            "unset REVIEW_TOKEN GH_TOKEN GITHUB_TOKEN GARNET_REVIEW_GITHUB_TOKEN GARNET_ADMIN_GITHUB_TOKEN",
            workflow,
        )
        self.assertIn("name: rolling trust-kernel review (non-pull-request)", workflow)
        self.assertIn("if: github.event_name != 'pull_request'", workflow)


class TrustSurfaceCoverageTests(GitRepoFixture):
    """H3-02: the enumerated trust surface had holes in exactly the load-bearing
    places.  Three independent reviews on 2026-09-03 landed on the same spot;
    the one carrying an in-repo identifier is the Codex hardening pass, finding
    H3-02 (blocking).  The other two are attested by the mission handoff only,
    so they are not named here.

    A hand-maintained enumeration drifts.  These tests therefore do two things:
    they pin the specific paths the finding named, and they *derive* the
    required-context producer set from the repository's own machine-readable
    rulesets so the next omission fails CI instead of merging quietly.
    """

    # (b) computes or enforces capability authority, and (c) verifies a
    # signature or a digest.  The five MCP/Shelf entries are not in the
    # finding's own list: a replay of the last 60 merged PRs showed a six-file
    # widening would already have missed them.  They did NOT all land in one PR,
    # as an earlier revision of this comment said: `mcp_schema.rs` first landed
    # in #501, `mcp.rs` in #503, and `minimum_shelf.rs`, `mcp_stdio.rs` and
    # `cmd/mcp_serve.rs` in #514 (verified with git log --diff-filter=A).
    CAPABILITY_AND_SIGNATURE_PATHS = (
        "garnet-cli/src/cap_manifest.rs",
        "garnet-cli/src/cmd/diff_caps.rs",
        "garnet-cli/src/cmd/verify_gate.rs",
        "garnet-cli/src/verify_gate.rs",
        "garnet-cli/src/manifest.rs",
        "garnet-cli/src/cmd/verify.rs",
        "garnet-cli/src/seal.rs",
        "garnet-cli/src/cmd/seal.rs",
        "garnet-cli/src/sandbox.rs",
        "garnet-cli/src/cmd/sandbox.rs",
        "garnet-cli/src/cmd/caps.rs",
        "garnet-cli/src/cmd/caps_log.rs",
        "garnet-cli/src/cmd/mcp_caps.rs",
        "garnet-cli/src/cmd/agent_loop.rs",
        "garnet-cli/src/cmd/repl.rs",
        "garnet-cli/src/cmd/trust_report.rs",
        "garnet-cli/src/machine_key.rs",
        "garnet-cli/src/cmd/keygen.rs",
        "garnet-cli/src/minimum_shelf.rs",
        "garnet-cli/src/mcp.rs",
        "garnet-cli/src/mcp_schema.rs",
        "garnet-cli/src/mcp_stdio.rs",
        "garnet-cli/src/cmd/mcp_serve.rs",
    )

    # (a) produces a required CI context but escapes the `scripts/garnet_` and
    # `scripts/test_garnet_` naming prefixes.  Derived, not guessed: see
    # `test_every_required_context_producer_is_trust_kernel`.
    REQUIRED_CONTEXT_PRODUCERS = (
        "scripts/check-agent-contracts.py",
        "scripts/check_determinism_no_llm.py",
        "scripts/check_dogfood_pr_body.py",
        "scripts/package_garnet_studio_macos.sh",
        "scripts/package_garnet_vscode_extension.sh",
        "scripts/preflight_garnet_studio_notarization.sh",
        "scripts/run_agentic_dogfood_matrix.py",
        "scripts/smoke_garnet_studio_dmg.sh",
        "scripts/smoke_garnet_web_pwa.sh",
        "scripts/test_check_agent_contracts.py",
        "scripts/test_check_determinism_no_llm.py",
        "scripts/test_check_dogfood_pr_body.py",
        "scripts/test_github_actions_node24_readiness.py",
        "scripts/test_smoke_garnet_pages_pwa.py",
    )

    # The surface as it stood at origin/main ee86d063, before this widening.
    # Widening must never narrow: every one of these still classifies true.
    PREVIOUSLY_COVERED_PATHS = (
        "garnet-check-v0.3/src/caps_graph.rs",
        "garnet-interp-v0.3/src/eval.rs",
        "garnet-vm/src/vm.rs",
        "garnet-stdlib/src/registry.rs",
        "garnet-wasm/src/lib.rs",
        ".github/actions/setup-rust/action.yml",
        ".github/rulesets/garnet-main.json",
        ".github/workflows/ci.yml",
        "scripts/garnet_required_context_contract.py",
        "scripts/test_garnet_required_context_contract.py",
        "F_Project_Management/W_TRUST/landed/LANE1.landed-review.json",
        ".github/CODEOWNERS",
        "Cargo.lock",
        "garnet-cli/Cargo.toml",
        "garnet-cli/src/bound_source.rs",
        "garnet-cli/src/cmd/add.rs",
        "garnet-cli/src/cmd/mod.rs",
        "garnet-cli/src/cmd/run.rs",
        "garnet-cli/src/cmd/test.rs",
        "garnet-cli/src/cmd/eval.rs",
        "garnet-cli/src/cmd/doctest.rs",
        "garnet-cli/src/bin/garnet.rs",
        "garnet-cli/src/lib.rs",
        "scripts/garnet_launch_readiness_status.py",
        "scripts/garnet_caps_enforcement_status.py",
        "scripts/garnet_capability_scope_status.py",
        "scripts/garnet_bounded_enforcement_status.py",
        "scripts/garnet_red_team_status.py",
        "docs/why.html",
        "C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md",
        "F_Project_Management/W_TRUST/LANDED_REVIEW_MARKERS.json",
    )

    @staticmethod
    def _required_context_producer_scripts() -> set[str]:
        """Re-derive, from the repository's own rulesets, every tracked script a
        required-context workflow invokes.  This is the anti-drift half: the
        enumeration above is checked against a derivation, not trusted."""
        ruleset = json.loads(
            (mod.ROOT / ".github/rulesets/garnet-main.json").read_text(encoding="utf-8")
        )
        contexts = {
            check["context"]
            for rule in ruleset.get("rules", [])
            if rule.get("type") == "required_status_checks"
            for check in rule["parameters"]["required_status_checks"]
        }
        producers = json.loads(
            (mod.ROOT / ".github/rulesets/required-context-producers.json").read_text(
                encoding="utf-8"
            )
        )
        by_context = {entry["context"]: entry["workflow"] for entry in producers["producers"]}
        workflows = {by_context[context] for context in contexts if context in by_context}
        pattern = re.compile(r"scripts/[A-Za-z0-9_.\-/]+")
        found: set[str] = set()
        for workflow in sorted(workflows):
            text = (mod.ROOT / workflow).read_text(encoding="utf-8")
            for candidate in pattern.findall(text):
                if (mod.ROOT / candidate).is_file():
                    found.add(candidate)
        return found

    def test_capability_and_signature_paths_are_trust_kernel(self) -> None:
        for path in self.CAPABILITY_AND_SIGNATURE_PATHS:
            self.assertTrue(mod.is_trust_kernel(path), path)
            self.assertTrue(mod.is_trust_kernel(path.replace("/", "\\")), path)

    def test_required_context_producer_scripts_are_trust_kernel(self) -> None:
        for path in self.REQUIRED_CONTEXT_PRODUCERS:
            self.assertTrue(mod.is_trust_kernel(path), path)

    def test_every_required_context_producer_is_trust_kernel(self) -> None:
        derived = self._required_context_producer_scripts()
        self.assertTrue(derived, "derivation found no required-context producer scripts")
        uncovered = sorted(path for path in derived if not mod.is_trust_kernel(path))
        self.assertEqual(
            [],
            uncovered,
            "scripts that produce a required CI context but are outside the trust "
            f"surface: {uncovered}",
        )

    def test_previously_covered_paths_stay_covered(self) -> None:
        for path in self.PREVIOUSLY_COVERED_PATHS:
            self.assertTrue(mod.is_trust_kernel(path), path)

    def test_unrelated_paths_stay_outside_the_trust_surface(self) -> None:
        for path in (
            "README.md",
            "apps/garnet-studio/src/main.rs",
            "benchmarks/run.py",
            "editors/vscode/package.json",
            "garnet-cli/tests/conformance_phase_gates.rs",
        ):
            self.assertFalse(mod.is_trust_kernel(path), path)

    def _one_byte_change_touches_trust_kernel(self, relative: str) -> mod.TrustKernelReviewStatus:
        self._commit_file(relative, b"original\n", f"seed {relative}")
        base = self._git("rev-parse", "HEAD")
        self._commit_file(relative, b"originaL\n", f"one byte of {relative}")
        return mod.read_status(base=base, head="HEAD", root=self.root)

    def test_one_byte_change_to_dogfood_body_checker_is_trust_kernel(self) -> None:
        # The script PRODUCES the required "PR dogfood evidence" context; a
        # change to it changes what that context proves.
        status = self._one_byte_change_touches_trust_kernel("scripts/check_dogfood_pr_body.py")
        self.assertTrue(status.trust_kernel_touched)
        self.assertEqual(["scripts/check_dogfood_pr_body.py"], status.touched_paths)

    def test_one_byte_change_to_capability_walk_is_trust_kernel(self) -> None:
        # The walk that decides whether authority widened.
        status = self._one_byte_change_touches_trust_kernel("garnet-cli/src/cap_manifest.rs")
        self.assertTrue(status.trust_kernel_touched)
        self.assertEqual(["garnet-cli/src/cap_manifest.rs"], status.touched_paths)

    def test_one_byte_change_to_manifest_verifier_is_trust_kernel(self) -> None:
        status = self._one_byte_change_touches_trust_kernel("garnet-cli/src/manifest.rs")
        self.assertTrue(status.trust_kernel_touched)
        self.assertEqual(["garnet-cli/src/manifest.rs"], status.touched_paths)


class TrustSurfaceVersionTests(unittest.TestCase):
    """Widening the surface must not retroactively invalidate a sealed landed
    marker.  A marker's ``touched_paths`` and ``content_digest`` are computed
    over the trust subset of its landing edge, so a wider surface makes an
    already-sealed marker report missing paths and a digest mismatch — and the
    append-only rule forbids editing the marker to say otherwise.  The surface
    is therefore versioned, and a marker is verified under the version it was
    sealed under.  Found while gating the H3-02 widening (2026-09-03).
    """

    def test_surfaces_are_registered_and_current_is_the_live_constant(self) -> None:
        self.assertIn("v1", mod.TRUST_SURFACES)
        self.assertIn("v2", mod.TRUST_SURFACES)
        self.assertEqual("v2", mod.CURRENT_TRUST_SURFACE)
        self.assertEqual(
            (tuple(mod.TRUST_KERNEL_PREFIXES), tuple(mod.TRUST_KERNEL_FILES)),
            mod.TRUST_SURFACES[mod.CURRENT_TRUST_SURFACE],
        )

    def test_v1_is_the_pre_h3_02_surface(self) -> None:
        prefixes, files = mod.TRUST_SURFACES["v1"]
        self.assertNotIn("garnet-cli/src/", prefixes)
        self.assertNotIn("garnet-cli/src/manifest.rs", files)
        self.assertIn("garnet-cli/src/lib.rs", files)
        self.assertNotIn("scripts/check_dogfood_pr_body.py", files)

    def test_v1_predicate_classifies_the_old_way(self) -> None:
        old = mod.trust_surface_predicate("v1")
        self.assertTrue(old("garnet-cli/src/lib.rs"))
        self.assertFalse(old("garnet-cli/src/manifest.rs"))
        self.assertFalse(old("scripts/check_dogfood_pr_body.py"))

    def test_current_predicate_classifies_the_new_way(self) -> None:
        now = mod.trust_surface_predicate(mod.CURRENT_TRUST_SURFACE)
        self.assertTrue(now("garnet-cli/src/manifest.rs"))
        self.assertTrue(now("scripts/check_dogfood_pr_body.py"))

    def test_every_registered_version_after_v1_has_a_stone_and_the_live_surface_is_the_latest(self) -> None:
        stones, findings = mod.era_stones_in_tree(mod.ROOT, "HEAD")
        self.assertEqual([], findings)
        self.assertEqual(set(mod.TRUST_SURFACES) - {"v1"}, set(stones))
        self.assertEqual(mod.CURRENT_TRUST_SURFACE, mod.latest_era(stones))
        self.assertEqual(559, stones["v2"]["introduced_by_pull_request"])

    def test_the_cli_entry_reports_the_live_surface_it_applies(self) -> None:
        # Review v4 (R559-v4-1): an imported-module check misses a rebinding at
        # the __main__ entry. The actual entry reports what it applies, and it
        # must be the latest stone.
        proc = subprocess.run(
            [sys.executable, "-I", str(mod.ROOT / "scripts/garnet_trust_kernel_review_status.py"), "--print-trust-surface"],
            cwd=mod.ROOT, capture_output=True, text=True, check=True,
        )
        report = json.loads(proc.stdout)
        self.assertEqual(mod.CURRENT_TRUST_SURFACE, report["current"])
        self.assertEqual(report["current"], report["latest_era_stone"])
        self.assertEqual([], report["problems"])

    def test_the_two_pre_versioning_landings_precede_every_stone(self) -> None:
        # Provenance, not identity and not a reading of any copy: both landed
        # before the v2 stone was laid, so they are v1 wherever main is.
        for merged in ("68317ae258327aade47fc2c07b7b5b580ec7c6ea", "41d6ced858684ac67683d32315920bd50a52976e"):
            self.assertEqual(("v1", []), mod.trust_surface_at_commit(mod.ROOT, merged))

    def test_era_stone_shape_is_exact(self) -> None:
        digest = mod.surface_digest("v2")
        good = _canonical({"introduced_by_pull_request": 559, "schema": mod.ERA_STONE_SCHEMA, "surface_sha256": digest, "version": "v2"})
        self.assertEqual([], mod.load_era_stone(good, "v2")[1])
        for label, payload, version in (
            ("wrong version", good, "v3"),
            ("extra key", _canonical({"introduced_by_pull_request": 559, "schema": mod.ERA_STONE_SCHEMA, "surface_sha256": digest, "version": "v2", "x": 1}), "v2"),
            ("bad schema", _canonical({"introduced_by_pull_request": 559, "schema": "other", "surface_sha256": digest, "version": "v2"}), "v2"),
            ("no pull", _canonical({"introduced_by_pull_request": 0, "schema": mod.ERA_STONE_SCHEMA, "surface_sha256": digest, "version": "v2"}), "v2"),
            ("no digest", _canonical({"introduced_by_pull_request": 559, "schema": mod.ERA_STONE_SCHEMA, "version": "v2"}), "v2"),
            ("wrong digest", _canonical({"introduced_by_pull_request": 559, "schema": mod.ERA_STONE_SCHEMA, "surface_sha256": "0" * 64, "version": "v2"}), "v2"),
            ("noncanonical", good.replace(b"\n", b"\r\n"), "v2"),
        ):
            with self.subTest(label=label):
                document, findings = mod.load_era_stone(payload, version)
                self.assertIsNone(document, label)
                self.assertTrue(findings, label)

    def test_trust_surface_is_an_allowed_marker_key(self) -> None:
        self.assertIn("trust_surface", mod.LANDED_KEYS)

    def test_registered_markers_verify_under_the_widened_surface(self) -> None:
        # The end-to-end proof: with `garnet-cli/src/` and the required-context
        # producers in the surface, both sealed markers must still verify.
        findings = mod.verify_repository_landed_markers(mod.ROOT)
        self.assertEqual([], findings)

class MarkerTrustSurfaceDerivationTests(GitRepoFixture):
    """The surface in force at a landing comes from the era ledger on main's
    first-parent history (review v4 cure): no copy of the gate script is read."""

    def _land(self, message: str = "landing") -> str:
        self._commit_file(f"noise-{message}.txt", message.encode(), message)
        commit = self._git("rev-parse", "HEAD")
        self._git("update-ref", "refs/remotes/origin/main", commit)
        return commit

    def _lay(self, version: str) -> str:
        path = self._write_stone(version)
        self._git("add", path)
        self._git("commit", "-m", f"era stone {version}")
        commit = self._git("rev-parse", "HEAD")
        self._git("update-ref", "refs/remotes/origin/main", commit)
        return commit

    def test_a_landing_after_the_current_stone_is_the_current_era(self) -> None:
        commit = self._land()
        self.assertEqual(("v2", []), mod.trust_surface_at_commit(self.root, commit))

    def test_the_base_that_laid_the_stone_is_itself_in_that_era(self) -> None:
        self.assertEqual(("v2", []), mod.trust_surface_at_commit(self.root, self.base))

    def test_a_later_stone_moves_only_later_landings(self) -> None:
        earlier = self._land("before-v3")
        wider = (tuple(mod.TRUST_KERNEL_PREFIXES) + ("future-trust/",), tuple(mod.TRUST_KERNEL_FILES))
        with mock.patch.dict(mod.TRUST_SURFACES, {"v3": wider}), mock.patch.object(mod, "CURRENT_TRUST_SURFACE", "v3"), mock.patch.object(mod, "TRUST_KERNEL_PREFIXES", wider[0]):
            stone = self._lay("v3")
            later = self._land("after-v3")
            self.assertEqual(("v2", []), mod.trust_surface_at_commit(self.root, earlier))
            self.assertEqual(("v3", []), mod.trust_surface_at_commit(self.root, stone))
            self.assertEqual(("v3", []), mod.trust_surface_at_commit(self.root, later))

    def test_a_declaration_may_only_agree_with_the_era_in_force(self) -> None:
        boundaries = {"v2": 5}
        self.assertEqual(("v2", []), mod.resolve_marker_trust_surface({}, boundaries, 3))
        self.assertEqual(("v2", []), mod.resolve_marker_trust_surface({"trust_surface": "v2"}, boundaries, 3))
        self.assertEqual(("v1", []), mod.resolve_marker_trust_surface({}, boundaries, 7))
        name, findings = mod.resolve_marker_trust_surface({"trust_surface": "v1"}, boundaries, 3)
        self.assertEqual((mod.CURRENT_TRUST_SURFACE, ["landed marker declares trust surface 'v1' but 'v2' was in force at its landing"]), (name, findings))

    def test_explicit_null_or_non_string_version_is_a_finding(self) -> None:
        for value in (None, 7, ["v2"], "v99"):
            with self.subTest(value=value):
                name, findings = mod.resolve_marker_trust_surface({"trust_surface": value}, {"v2": 5}, 3)
                self.assertEqual(mod.CURRENT_TRUST_SURFACE, name)
                self.assertEqual([f"unknown trust surface {value!r} in landed marker"], findings)

    def test_stone_history_must_be_append_only(self) -> None:
        commit = self._land()
        for label, action in (
            ("modified", lambda: self._write_stone("v2", pull=560)),
            ("deleted", lambda: (self.root / mod.era_stone_path("v2")).unlink()),
        ):
            with self.subTest(label=label):
                self._git("checkout", "-q", "-b", f"tamper-{label}", commit)
                action()
                self._git("add", "-A")
                self._git("commit", "-m", f"stone {label}")
                self._git("update-ref", "refs/remotes/origin/main", self._git("rev-parse", "HEAD"))
                name, findings = mod.trust_surface_at_commit(self.root, commit)
                self.assertIsNone(name, label)
                self.assertTrue(any("append-only" in item for item in findings), (label, findings))
                self._git("checkout", "-q", "main")
                self._git("update-ref", "refs/remotes/origin/main", commit)

    def test_a_stone_moved_away_and_back_is_not_append_only(self) -> None:
        # Review v5 (R559-v5-2): rename-out, twelve commits, rename-back showed
        # no D/M under rename detection and bisection then found the return
        # as the introduction, deriving v1 for a v2 landing.
        landing = self._land("v2-landing")
        self._git("mv", mod.era_stone_path("v2"), "F_Project_Management/W_TRUST/eras/parked.json")
        self._git("commit", "-m", "park the stone")
        for i in range(12):
            self._land(f"filler-{i}")
        self._git("mv", "F_Project_Management/W_TRUST/eras/parked.json", mod.era_stone_path("v2"))
        self._git("commit", "-m", "restore the stone")
        self._git("update-ref", "refs/remotes/origin/main", self._git("rev-parse", "HEAD"))
        name, findings = mod.trust_surface_at_commit(self.root, landing)
        self.assertIsNone(name)
        self.assertTrue(any("append-only" in item for item in findings), findings)

    def test_an_unreadable_historical_stone_is_never_absence(self) -> None:
        # Review v5 (R559-v5-2): a failed read was treated as absence.
        landing = self._land()
        real = mod._read_blob
        def failing(root, ref, path):
            if path.startswith(mod.ERA_STONE_DIR):
                return None, [f"blob lookup timed out for {path}"]
            return real(root, ref, path)
        with mock.patch.object(mod, "_read_blob", failing):
            stones, findings = mod.era_stones_in_tree(self.root, "HEAD")
            self.assertEqual({}, stones)
            self.assertTrue(any("could not be read" in item for item in findings), findings)
            self.assertFalse(any("has no era stone" in item for item in findings), findings)

    def test_a_failed_lookup_is_never_absence_at_the_git_boundary(self) -> None:
        # Review v6 (R559-v6-1): rev-parse says the same thing for a missing
        # path and a broken object. Absence is a verified empty listing; any
        # other failure, listing failures and timeouts included, propagates.
        landing = self._land()
        real = mod._git_bytes
        def failing(root, *args, **kwargs):
            if "ls-tree" in args and any(str(a).startswith(mod.ERA_STONE_DIR) or str(a).endswith("eras") for a in args):
                return mod.GitResult(returncode=128, stdout=b"", stderr=b"fatal: bad object", timed_out=False)
            return real(root, *args, **kwargs)
        with mock.patch.object(mod, "_git_bytes", failing):
            stones, findings = mod.era_stones_in_tree(self.root, "HEAD")
            self.assertEqual({}, stones)
            self.assertTrue(any("could not be listed" in item for item in findings), findings)
            self.assertFalse(any("has no era stone" in item for item in findings), findings)
            name, era_findings = mod.trust_surface_at_commit(self.root, landing)
            self.assertIsNone(name)
        def timing_out(root, *args, **kwargs):
            if "ls-tree" in args:
                return mod.GitResult(returncode=0, stdout=b"", stderr=b"", timed_out=True)
            return real(root, *args, **kwargs)
        with mock.patch.object(mod, "_git_bytes", timing_out):
            stones, findings = mod.era_stones_in_tree(self.root, "HEAD")
            self.assertEqual({}, stones)
            self.assertTrue(any("could not be listed" in item for item in findings), findings)

    def test_a_directory_listing_failure_cannot_hide_clutter(self) -> None:
        path = self.root / mod.ERA_STONE_DIR / "notes.txt"
        path.write_bytes(b"clutter\n")
        self._git("add", "-A"); self._git("commit", "-m", "clutter")
        real = mod._git_bytes
        def failing_listing(root, *args, **kwargs):
            if "ls-tree" in args and "--name-only" in args:
                return mod.GitResult(returncode=128, stdout=b"", stderr=b"", timed_out=False)
            return real(root, *args, **kwargs)
        with mock.patch.object(mod, "_git_bytes", failing_listing):
            _, findings = mod.era_stones_in_tree(self.root, "HEAD")
        self.assertTrue(any("could not be listed" in item for item in findings), findings)

    def test_only_exact_root_stones_count_and_anything_else_is_a_finding(self) -> None:
        for name in ("nested/v9.era.json", "v9.json", "notes.txt", "v2.era.json.bak"):
            path = self.root / mod.ERA_STONE_DIR / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"{}\n")
        self._git("add", "-A")
        self._git("commit", "-m", "clutter")
        stones, findings = mod.era_stones_in_tree(self.root, "HEAD")
        self.assertEqual({"v2"}, set(stones))
        for name in ("nested", "v9.json", "notes.txt", "v2.era.json.bak"):
            self.assertTrue(any(name in item and "is not a stone" in item for item in findings), (name, findings))

    def test_an_era_failure_resolves_to_the_widest_surface(self) -> None:
        name, findings = mod.resolve_marker_trust_surface({}, {}, 3, era_ok=False)
        self.assertEqual((mod.CURRENT_TRUST_SURFACE, []), (name, findings))

    def test_a_stone_for_an_unregistered_version_is_a_finding(self) -> None:
        self._lay("v9")
        stones, findings = mod.era_stones_in_tree(self.root, "HEAD")
        self.assertIn("era stone v9 names a version this gate does not register", findings)

    def test_a_registered_version_without_a_stone_is_a_finding(self) -> None:
        with mock.patch.dict(mod.TRUST_SURFACES, {"v3": mod.TRUST_SURFACES["v2"]}):
            stones, findings = mod.era_stones_in_tree(self.root, "HEAD")
            self.assertIn("registered trust surface v3 has no era stone at HEAD", findings)

    def test_a_commit_off_the_first_parent_line_has_no_era(self) -> None:
        self._git("checkout", "-q", "-b", "side")
        side = self._commit_file("side.txt", b"side\n", "side")
        self._git("checkout", "-q", "main")
        name, findings = mod.trust_surface_at_commit(self.root, side)
        self.assertIsNone(name)
        self.assertTrue(any("not on the first-parent history" in item for item in findings), findings)


class TrustSurfaceWideningTests(LandedMarkerTests):
    """End to end: a marker sealed in the v2 era stays green when a v3 stone is
    laid and the live surface widens; a landing after the v3 stone must cover
    v3; and a live surface with no stone cannot run green (review v4 cure)."""

    WIDER = (tuple(mod.TRUST_KERNEL_PREFIXES) + ("future-trust/",), tuple(mod.TRUST_KERNEL_FILES))

    def _widened_to_v3(self):
        """A real widening: the registered entry, the live label and the alias
        tuples move together (the gate compares all three inside every run)."""
        class _Widen:
            def __enter__(inner):
                inner.patches = [
                    mock.patch.dict(mod.TRUST_SURFACES, {"v3": self.WIDER}),
                    mock.patch.object(mod, "CURRENT_TRUST_SURFACE", "v3"),
                    mock.patch.object(mod, "TRUST_KERNEL_PREFIXES", self.WIDER[0]),
                ]
                for patch in inner.patches:
                    patch.start()
                return inner
            def __exit__(inner, *exc):
                for patch in reversed(inner.patches):
                    patch.stop()
        return _Widen()

    def _register(self, marker: dict[str, object]) -> None:
        self._commit_repository_marker(marker)
        self._git("update-ref", "refs/remotes/origin/main", self._git("rev-parse", "HEAD"))

    def _lay_v3(self) -> str:
        path = self._write_stone("v3", pull=600)
        self._git("add", path)
        self._git("commit", "-m", "era stone v3")
        commit = self._git("rev-parse", "HEAD")
        self._git("update-ref", "refs/remotes/origin/main", commit)
        return commit

    def test_widening_the_live_surface_keeps_a_sealed_v2_marker_green(self) -> None:
        marker = self._marker()
        del marker["trust_surface"]
        self._register(marker)
        self.assertEqual([], mod.verify_repository_landed_markers(self.root))
        with self._widened_to_v3():
            self._lay_v3()
            self.assertEqual([], mod.verify_repository_landed_markers(self.root))

    def test_a_declared_version_that_agrees_survives_the_widening_too(self) -> None:
        self._register(self._marker())
        with self._widened_to_v3():
            self._lay_v3()
            self.assertEqual([], mod.verify_repository_landed_markers(self.root))

    def test_a_live_surface_with_no_stone_cannot_run_green(self) -> None:
        # Review v4 (R559-v4-1): a copy that widens the constant at its entry
        # point without laying a stone. The check runs inside every gate run.
        self._register(self._marker())
        with mock.patch.dict(mod.TRUST_SURFACES, {"v3": self.WIDER}), mock.patch.object(mod, "CURRENT_TRUST_SURFACE", "v3"):
            findings = mod.verify_repository_landed_markers(self.root)
        self.assertIn("registered trust surface v3 has no era stone at " + self._git("rev-parse", "HEAD"), findings)
        self.assertIn("the live trust surface 'v3' is not the latest era stone 'v2' in the candidate tree", findings)

    def test_widening_the_live_classifier_without_a_version_cannot_run_green(self) -> None:
        # Review v5 (R559-v5-1): a copy that rebinds TRUST_KERNEL_PREFIXES at
        # its entry point widened the live classifier while every label still
        # said v2. The classifier reads the registered entry, and the aliases
        # are compared to it inside every run.
        self._register(self._marker())
        with mock.patch.object(mod, "TRUST_KERNEL_PREFIXES", tuple(mod.TRUST_KERNEL_PREFIXES) + ("future-v3/",)):
            self.assertFalse(mod.is_trust_kernel("future-v3/probe.txt"))
            findings = mod.verify_repository_landed_markers(self.root)
        self.assertTrue(any("live classifier tuples do not equal the registered entry" in item for item in findings), findings)

    def test_a_candidate_that_edits_a_stone_is_red_before_it_lands(self) -> None:
        # Review v5 (R559-v5-4): the ledger is on the surface and stone
        # mutation on any candidate edge is a finding, record or not.
        self.assertTrue(mod.is_trust_kernel(mod.era_stone_path("v2")))
        self._git("checkout", "-q", "-b", "poison", self.merged_commit)
        self._write_stone("v2", pull=560)
        self._git("add", "-A")
        self._git("commit", "-m", "edit the stone")
        head = self._git("rev-parse", "HEAD")
        findings = mod._era_stone_append_only_findings(self.root, self.merged_commit, head)
        self.assertTrue(any("era stones are append-only: M" in item for item in findings), findings)
        (self.root / mod.era_stone_path("v2")).unlink()
        self._git("add", "-A")
        self._git("commit", "-m", "delete the stone")
        findings = mod._era_stone_append_only_findings(self.root, self.merged_commit, self._git("rev-parse", "HEAD"))
        self.assertTrue(any("era stones are append-only: D" in item for item in findings), findings)
        self._git("checkout", "-q", "main")

    def test_a_merge_that_discards_a_side_added_stone_is_red_on_its_second_parent_edge(self) -> None:
        # Review v6 (R559-v6-2): an `ours` merge dropped a side-added stone;
        # only first-parent edges were checked.
        self._git("checkout", "-q", "-b", "side", self.merged_commit)
        with mock.patch.dict(mod.TRUST_SURFACES, {"v3": self.WIDER}):
            self._write_stone("v3", pull=600)
        self._git("add", "-A"); self._git("commit", "-m", "side adds v3 stone")
        side = self._git("rev-parse", "HEAD")
        self._git("checkout", "-q", "-b", "candidate", self.merged_commit)
        self._git("merge", "--no-ff", "-s", "ours", "-m", "discard the side stone", side)
        head = self._git("rev-parse", "HEAD")
        findings = mod._era_stone_append_only_findings(self.root, self.merged_commit, head)
        self.assertTrue(any("era stones are append-only: D" in item and "against parent" in item for item in findings), findings)
        self._git("checkout", "-q", "main")

    def test_jointly_rebinding_the_entry_and_the_aliases_disagrees_with_the_stone(self) -> None:
        # Review v6 (R559-v6-3): equality of two mutable values proved nothing;
        # the stone records the entry's digest, and the live entry must match it.
        self._register(self._marker())
        wider = self.WIDER
        with mock.patch.dict(mod.TRUST_SURFACES, {"v2": wider}), mock.patch.object(mod, "TRUST_KERNEL_PREFIXES", wider[0]):
            findings = mod.verify_repository_landed_markers(self.root)
        self.assertTrue(any("does not match the surface digest its era stone records" in item for item in findings), findings)

    def test_print_trust_surface_never_combines_with_gate(self) -> None:
        # Review v5 (R559-v5-3): the diagnostic returned 0 before gate policy.
        with self.assertRaises(SystemExit) as raised:
            mod.main(["--gate", "--print-trust-surface"], root=self.root)
        self.assertEqual(2, raised.exception.code)

    def test_a_landing_after_the_v3_stone_cannot_omit_a_v3_only_path(self) -> None:
        # The regression the contract promises: a landing in the v3 era whose
        # record and marker omit a v3-only path is reported.
        with self._widened_to_v3():
            self._lay_v3()
            self._git("checkout", "-q", "-b", "v3-landing")
            self._commit_file("future-trust/probe.txt", b"v3-only authority\n", "v3-only change")
            landed = self._git("rev-parse", "HEAD")
            self._git("update-ref", "refs/remotes/origin/main", landed)
            marker = self._marker()
            marker["merged_commit"] = landed
            marker["merged_tree"] = self._git("rev-parse", "HEAD^{tree}")
            marker["trust_surface"] = "v2"  # the laundering declaration
            findings = mod.verify_landed_review_marker(marker, root=self.root, main_ref="refs/remotes/origin/main")
            self.assertTrue(any("'v3' was in force" in item for item in findings), findings)
            self.assertTrue(any("future-trust/probe.txt" in item for item in findings), findings)
            del marker["trust_surface"]
            findings = mod.verify_landed_review_marker(marker, root=self.root, main_ref="refs/remotes/origin/main")
            self.assertTrue(any("future-trust/probe.txt" in item for item in findings), findings)
            self._git("checkout", "-q", "main")

    def test_a_merged_commit_registered_twice_is_a_finding(self) -> None:
        marker = self._marker()
        self._commit_repository_marker(marker)
        second = "F_Project_Management/W_TRUST/landed/REPLAY.landed-review.json"
        (self.root / second).write_bytes(_canonical(marker))
        registry = self.root / "F_Project_Management/W_TRUST/LANDED_REVIEW_MARKERS.json"
        registry.write_bytes(
            _canonical({"markers": ["F_Project_Management/W_TRUST/landed/LANDED.landed-review.json", second], "schema": "garnet.trust_kernel_landed_review_registry/v1"})
        )
        self._git("add", second, str(registry.relative_to(self.root)))
        self._git("commit", "-m", "replay")
        self._git("update-ref", "refs/remotes/origin/main", self._git("rev-parse", "HEAD"))
        findings = mod.verify_repository_landed_markers(self.root)
        self.assertTrue(any("registered by more than one landed marker" in item for item in findings), findings)

if __name__ == "__main__":
    unittest.main()