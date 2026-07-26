#!/usr/bin/env python3
"""Tests for the fail-closed cross-OS governance policy manifest runner."""
from __future__ import annotations

import importlib.util
import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


ROOT = Path(__file__).resolve().parents[1]
PATH = Path(__file__).with_name("garnet_cross_os_policy_manifest.py")


def write_lf(path: Path, text: str) -> None:
    """Write canonical UTF-8 fixture bytes on every host OS."""
    path.write_bytes(text.encode("utf-8"))


def load_runner():
    if not PATH.is_file():
        raise AssertionError(f"cross-OS policy manifest runner is missing: {PATH}")
    spec = importlib.util.spec_from_file_location("_cross_os_policy_manifest_test", PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"cannot load {PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class CrossOsPolicyManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / "repo"
        self.root.mkdir()
        self.git("init", "-q")
        self.git("config", "core.autocrlf", "false")
        self.git("config", "user.email", "fixture@example.invalid")
        self.git("config", "user.name", "Fixture")
        write_lf(self.root / "tracked.txt", "tracked\n")
        self.git("add", "tracked.txt")
        self.git("commit", "-qm", "fixture")
        self.head = self.git("rev-parse", "HEAD")
        self.output = Path(self.temp.name) / "manifest.json"

    def git(self, *args: str) -> str:
        return subprocess.run(
            ["git", "-C", str(self.root), *args],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()

    @staticmethod
    def loader(case: type[unittest.TestCase]):
        def load(_path: Path) -> unittest.TestSuite:
            return unittest.defaultTestLoader.loadTestsFromTestCase(case)

        return load

    @staticmethod
    def passing_case() -> type[unittest.TestCase]:
        class PassingCase(unittest.TestCase):
            def test_passes(self) -> None:
                self.assertTrue(True)

        return PassingCase

    @staticmethod
    def skipped_case() -> type[unittest.TestCase]:
        class SkippedCase(unittest.TestCase):
            @unittest.skip("fixture skip must make policy evidence red")
            def test_skips(self) -> None:
                self.fail("skipped fixture ran")

        return SkippedCase

    @staticmethod
    def fixture_parity(runner, case: type[unittest.TestCase]) -> str:
        test_ids = [
            test.id()
            for test in unittest.defaultTestLoader.loadTestsFromTestCase(case)
        ]
        rows = [
            {
                "id": definition.suite_id,
                "path": definition.path,
                "test_ids": test_ids,
                "tests_run": len(test_ids),
                "failures": [],
                "errors": [],
                "skipped": [],
                "expected_failures": [],
                "unexpected_successes": [],
            }
            for definition in runner.SUITES
        ]
        return hashlib.sha256(
            json.dumps(
                {"suites": rows},
                ensure_ascii=False,
                separators=(",", ":"),
                sort_keys=True,
            ).encode("utf-8")
        ).hexdigest()

    def test_clean_exact_head_writes_four_suite_machine_manifest(self) -> None:
        runner = load_runner()
        case = self.passing_case()
        expected_parity = self.fixture_parity(runner, case)
        with mock.patch.object(
            runner,
            "EXPECTED_ALL_GREEN_PARITY_SHA256",
            expected_parity,
            create=True,
        ):
            exit_code = runner.run_manifest(
                root=self.root,
                expected_head=self.head,
                output=self.output,
                os_name=runner._runtime_os(),
                suite_loader=self.loader(case),
            )
        manifest = json.loads(self.output.read_text(encoding="utf-8"))
        self.assertEqual(exit_code, 0)
        self.assertEqual(manifest["schema"], runner.SCHEMA)
        self.assertEqual(manifest["head_sha"], self.head)
        self.assertTrue(manifest["head_exact"])
        self.assertTrue(manifest["working_tree_clean"])
        self.assertEqual(manifest["os"], runner._runtime_os())
        self.assertEqual(len(manifest["suites"]), 4)
        self.assertEqual(manifest["totals"]["tests_run"], 4)
        self.assertEqual(manifest["totals"]["skipped"], 0)
        self.assertRegex(manifest["suite_contract_sha256"], r"^[0-9a-f]{64}$")
        self.assertEqual(manifest["expected_parity_sha256"], expected_parity)
        self.assertEqual(manifest["parity_sha256"], expected_parity)
        self.assertTrue(manifest["parity_exact"])
        self.assertEqual(manifest["problems"], [])
        self.assertTrue(manifest["ok"])

    def test_parity_digest_is_head_independent(self) -> None:
        runner = load_runner()
        case = self.passing_case()
        expected_parity = self.fixture_parity(runner, case)
        with mock.patch.object(
            runner,
            "EXPECTED_ALL_GREEN_PARITY_SHA256",
            expected_parity,
            create=True,
        ):
            first_exit = runner.run_manifest(
                root=self.root,
                expected_head=self.head,
                output=self.output,
                os_name=runner._runtime_os(),
                suite_loader=self.loader(case),
            )
            first = json.loads(self.output.read_text(encoding="utf-8"))
            write_lf(self.root / "tracked.txt", "next head\n")
            self.git("add", "tracked.txt")
            self.git("commit", "-qm", "next head")
            next_head = self.git("rev-parse", "HEAD")
            second_exit = runner.run_manifest(
                root=self.root,
                expected_head=next_head,
                output=self.output,
                os_name=runner._runtime_os(),
                suite_loader=self.loader(case),
            )
            second = json.loads(self.output.read_text(encoding="utf-8"))
        self.assertEqual((first_exit, second_exit), (0, 0))
        self.assertNotEqual(first["head_sha"], second["head_sha"])
        self.assertEqual(first["parity_sha256"], second["parity_sha256"])

    def test_different_passing_test_set_is_red_against_pinned_parity(self) -> None:
        runner = load_runner()
        baseline = self.passing_case()
        expected_parity = self.fixture_parity(runner, baseline)

        class DifferentPassingCase(unittest.TestCase):
            def test_different_identity_passes(self) -> None:
                self.assertTrue(True)

        with mock.patch.object(
            runner,
            "EXPECTED_ALL_GREEN_PARITY_SHA256",
            expected_parity,
            create=True,
        ):
            exit_code = runner.run_manifest(
                root=self.root,
                expected_head=self.head,
                output=self.output,
                os_name=runner._runtime_os(),
                suite_loader=self.loader(DifferentPassingCase),
            )
        manifest = json.loads(self.output.read_text(encoding="utf-8"))
        self.assertEqual(exit_code, 1)
        self.assertFalse(manifest["parity_exact"])
        self.assertTrue(any("pinned parity" in item for item in manifest["problems"]))

    def test_any_skipped_test_writes_red_manifest_and_exits_nonzero(self) -> None:
        runner = load_runner()
        exit_code = runner.run_manifest(
            root=self.root,
            expected_head=self.head,
            output=self.output,
            os_name=runner._runtime_os(),
            suite_loader=self.loader(self.skipped_case()),
        )
        manifest = json.loads(self.output.read_text(encoding="utf-8"))
        self.assertEqual(exit_code, 1)
        self.assertEqual(manifest["totals"]["skipped"], 4)
        self.assertTrue(any("skipped" in problem for problem in manifest["problems"]))
        self.assertFalse(manifest["ok"])

    def test_mismatched_head_or_dirty_tree_is_red_without_running_suites(self) -> None:
        runner = load_runner()
        calls = 0

        def load(_path: Path) -> unittest.TestSuite:
            nonlocal calls
            calls += 1
            return unittest.defaultTestLoader.loadTestsFromTestCase(
                self.passing_case()
            )

        exit_code = runner.run_manifest(
            root=self.root,
            expected_head="0" * 40,
            output=self.output,
            os_name=runner._runtime_os(),
            suite_loader=load,
        )
        mismatch = json.loads(self.output.read_text(encoding="utf-8"))
        self.assertEqual((exit_code, calls, mismatch["suites"]), (1, 0, []))
        self.assertTrue(any("expected head" in item for item in mismatch["problems"]))

        write_lf(self.root / "tracked.txt", "dirty\n")
        exit_code = runner.run_manifest(
            root=self.root,
            expected_head=self.head,
            output=self.output,
            os_name=runner._runtime_os(),
            suite_loader=load,
        )
        dirty = json.loads(self.output.read_text(encoding="utf-8"))
        self.assertEqual((exit_code, calls, dirty["suites"]), (1, 0, []))
        self.assertTrue(any("working tree" in item for item in dirty["problems"]))

    def test_assume_unchanged_cannot_hide_non_head_policy_bytes(self) -> None:
        runner = load_runner()
        calls = 0

        def load(_path: Path) -> unittest.TestSuite:
            nonlocal calls
            calls += 1
            return unittest.defaultTestLoader.loadTestsFromTestCase(
                self.passing_case()
            )

        self.git("update-index", "--assume-unchanged", "tracked.txt")
        write_lf(self.root / "tracked.txt", "hidden dirty bytes\n")
        exit_code = runner.run_manifest(
            root=self.root,
            expected_head=self.head,
            output=self.output,
            os_name=runner._runtime_os(),
            suite_loader=load,
        )
        manifest = json.loads(self.output.read_text(encoding="utf-8"))
        self.assertEqual((exit_code, calls, manifest["suites"]), (1, 0, []))
        self.assertTrue(any("index flags" in item for item in manifest["problems"]))

    def test_policy_suite_mutation_invalidates_the_final_manifest(self) -> None:
        runner = load_runner()
        tracked = self.root / "tracked.txt"

        class MutatingCase(unittest.TestCase):
            def test_mutates_tracked_source(self) -> None:
                write_lf(tracked, "mutated during suite\n")

        exit_code = runner.run_manifest(
            root=self.root,
            expected_head=self.head,
            output=self.output,
            os_name=runner._runtime_os(),
            suite_loader=self.loader(MutatingCase),
        )
        manifest = json.loads(self.output.read_text(encoding="utf-8"))
        self.assertEqual(exit_code, 1)
        self.assertFalse(manifest["working_tree_clean"])
        self.assertTrue(
            any("changed while policy suites ran" in item for item in manifest["problems"])
        )

    def test_suite_loader_failure_is_manifested_and_red(self) -> None:
        runner = load_runner()

        def broken(path: Path) -> unittest.TestSuite:
            raise RuntimeError(f"cannot import {path.name}")

        exit_code = runner.run_manifest(
            root=self.root,
            expected_head=self.head,
            output=self.output,
            os_name=runner._runtime_os(),
            suite_loader=broken,
        )
        manifest = json.loads(self.output.read_text(encoding="utf-8"))
        self.assertEqual(exit_code, 1)
        self.assertTrue(any("cannot load" in item for item in manifest["problems"]))
        self.assertFalse(manifest["ok"])

    def test_declared_os_must_match_the_runtime_before_suites_run(self) -> None:
        runner = load_runner()
        calls = 0

        def load(_path: Path) -> unittest.TestSuite:
            nonlocal calls
            calls += 1
            return unittest.defaultTestLoader.loadTestsFromTestCase(
                self.passing_case()
            )

        with mock.patch.object(
            runner, "_runtime_os", return_value="Linux", create=True
        ):
            exit_code = runner.run_manifest(
                root=self.root,
                expected_head=self.head,
                output=self.output,
                os_name="macOS",
                suite_loader=load,
            )
        manifest = json.loads(self.output.read_text(encoding="utf-8"))
        self.assertEqual((exit_code, calls, manifest["suites"]), (1, 0, []))
        self.assertEqual(manifest["os"], "Linux")
        self.assertEqual(manifest["declared_os"], "macOS")
        self.assertTrue(any("runtime OS" in item for item in manifest["problems"]))

    def test_ci_runs_exact_head_manifest_and_uploads_even_on_failure(self) -> None:
        workflow = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
        test_job = workflow.split("\n  test:\n", 1)[1].split(
            "\n  canonical-examples:\n", 1
        )[0]
        self.assertIn("python -I scripts/garnet_cross_os_policy_manifest.py", test_job)
        self.assertIn("--expected-head", test_job)
        self.assertIn("github.event.pull_request.head.sha || github.sha", test_job)
        checkout = test_job.split("actions/checkout@", 1)[1].split(
            "actions/setup-python@", 1
        )[0]
        self.assertIn(
            "ref: ${{ github.event.pull_request.head.sha || github.sha }}",
            checkout,
        )
        self.assertIn("persist-credentials: false", checkout)
        self.assertIn(
            "actions/upload-artifact@b7c566a772e6b6bfb58ed0dc250532a479d7789f",
            test_job,
        )
        self.assertIn("if: always()", test_job)
        self.assertIn("governance-policy-${{ runner.os }}.json", test_job)


if __name__ == "__main__":
    unittest.main()
