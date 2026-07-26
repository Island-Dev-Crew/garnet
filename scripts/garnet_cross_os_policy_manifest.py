#!/usr/bin/env python3
"""Run the four governance policy suites and emit exact-head per-OS evidence.

The runner is deliberately stricter than ``unittest``'s default exit policy:
skips and expected failures are non-passing evidence.  It also refuses to run
against a dirty or unexpected checkout, so the resulting manifest can be tied
to the exact Git head named by CI rather than to uncommitted workspace bytes.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import io
import json
import os
import platform
import re
import subprocess
import sys
import unittest
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Iterable


SCHEMA = "garnet.governance-policy-os-evidence/v1"
OID_RE = re.compile(r"^[0-9a-f]{40}(?:[0-9a-f]{24})?$")
ALLOWED_OSES = ("Linux", "macOS", "Windows")
GIT_TIMEOUT_SECONDS = 15
# Canonical SHA-256 over the ordered four-suite, 36-test, all-green outcome
# contract. It intentionally excludes the commit and OS so every matrix leg
# must reproduce this same value; the manifest binds it back to head and OS.
EXPECTED_ALL_GREEN_PARITY_SHA256 = (
    "cf3631ea6afd3443d040500c5c453c07c7609c5994975bbff74e6d9f608c8cd6"
)


@dataclass(frozen=True)
class SuiteDefinition:
    suite_id: str
    path: str


SUITES = (
    SuiteDefinition(
        "required-context-contract",
        "scripts/test_garnet_required_context_contract.py",
    ),
    SuiteDefinition(
        "workflow-file-policy",
        "scripts/test_garnet_workflow_file_policy.py",
    ),
    SuiteDefinition(
        "workflow-yaml-policy",
        "scripts/test_garnet_workflow_yaml_policy.py",
    ),
    SuiteDefinition(
        "workflow-schema-policy",
        "scripts/test_garnet_workflow_schema_policy.py",
    ),
)


def _canonical_sha256(value: object) -> str:
    payload = json.dumps(
        value,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


def _runtime_os() -> str:
    return {"Darwin": "macOS"}.get(platform.system(), platform.system())


def _git_environment() -> dict[str, str]:
    passthrough = (
        "COMSPEC",
        "ComSpec",
        "PATH",
        "PATHEXT",
        "SYSTEMROOT",
        "SystemRoot",
        "TEMP",
        "TMP",
        "TMPDIR",
        "WINDIR",
    )
    environment = {
        name: value
        for name in passthrough
        if (value := os.environ.get(name)) is not None
    }
    environment.update(
        {
            "GIT_ATTR_NOSYSTEM": "1",
            "GIT_CONFIG_COUNT": "0",
            "GIT_CONFIG_GLOBAL": os.devnull,
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_CONFIG_SYSTEM": os.devnull,
            "GIT_NO_REPLACE_OBJECTS": "1",
            "GIT_OPTIONAL_LOCKS": "0",
            "GIT_TERMINAL_PROMPT": "0",
            "LC_ALL": "C",
        }
    )
    return environment


def _git(root: Path, *arguments: str) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        ["git", "--no-replace-objects", "-C", str(root), *arguments],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
        timeout=GIT_TIMEOUT_SECONDS,
        env=_git_environment(),
    )


def _repository_state(root: Path, expected_head: str) -> tuple[str | None, bool, list[str]]:
    problems: list[str] = []
    if OID_RE.fullmatch(expected_head) is None:
        problems.append("expected head is not one canonical Git object id")

    try:
        resolved = _git(root, "rev-parse", "--verify", "--end-of-options", "HEAD^{commit}")
    except (OSError, subprocess.TimeoutExpired) as exc:
        return None, False, [f"cannot resolve repository head: {type(exc).__name__}"]
    head: str | None = None
    if resolved.returncode == 0:
        try:
            candidate = resolved.stdout.decode("ascii", errors="strict").strip()
        except UnicodeDecodeError:
            candidate = ""
        if OID_RE.fullmatch(candidate) is not None:
            head = candidate
    if head is None:
        problems.append("cannot resolve one canonical repository HEAD commit")
    elif head != expected_head:
        problems.append(f"repository HEAD {head} does not equal expected head {expected_head}")

    try:
        status = _git(root, "status", "--porcelain=v2", "-z", "--untracked-files=all")
    except (OSError, subprocess.TimeoutExpired) as exc:
        return head, False, problems + [
            f"cannot inspect working tree: {type(exc).__name__}"
        ]
    clean = status.returncode == 0 and status.stdout == b""
    if status.returncode != 0:
        problems.append("cannot inspect working tree status")
    elif not clean:
        problems.append("working tree is not clean at the exact-head evidence boundary")

    try:
        flags = _git(root, "ls-files", "-v", "-z")
    except (OSError, subprocess.TimeoutExpired) as exc:
        return head, False, problems + [
            f"cannot inspect tracked index flags: {type(exc).__name__}"
        ]
    if flags.returncode != 0:
        problems.append("cannot inspect tracked index flags")
    else:
        special = [
            entry
            for entry in flags.stdout.split(b"\0")
            if entry
            and (
                entry[:1] == b"S"
                or (b"a" <= entry[:1] <= b"z")
            )
        ]
        if special:
            clean = False
            problems.append(
                "tracked index flags contain assume-unchanged or skip-worktree paths"
            )
    return head, clean, problems


def _default_suite_loader(path: Path) -> unittest.TestSuite:
    module_name = f"_garnet_cross_os_policy_{path.stem}"
    spec = importlib.util.spec_from_file_location(module_name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot create import specification for {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    try:
        spec.loader.exec_module(module)
    except BaseException:
        sys.modules.pop(module_name, None)
        raise
    return unittest.defaultTestLoader.loadTestsFromModule(module)


def _tests(suite: unittest.TestSuite) -> Iterable[unittest.TestCase]:
    for item in suite:
        if isinstance(item, unittest.TestSuite):
            yield from _tests(item)
        else:
            yield item


def _test_id(test: object) -> str:
    identifier = getattr(test, "id", None)
    if callable(identifier):
        try:
            value = identifier()
        except BaseException:
            value = None
        if isinstance(value, str) and value:
            return value
    return type(test).__name__


def _outcomes(rows: Iterable[tuple[object, str]]) -> list[dict[str, str]]:
    return [
        {"test": _test_id(test), "detail": detail}
        for test, detail in rows
    ]


def _run_suite(
    definition: SuiteDefinition,
    root: Path,
    suite_loader: Callable[[Path], unittest.TestSuite],
) -> tuple[dict[str, object] | None, str | None]:
    path = root / definition.path
    try:
        suite = suite_loader(path)
        test_ids = [_test_id(test) for test in _tests(suite)]
    except BaseException as exc:
        return None, (
            f"cannot load {definition.suite_id} from {definition.path}: "
            f"{type(exc).__name__}: {exc}"
        )
    if not test_ids:
        return None, f"cannot load {definition.suite_id}: suite contains zero tests"

    stream = io.StringIO()
    result = unittest.TextTestRunner(stream=stream, verbosity=2).run(suite)
    transcript = stream.getvalue()
    if transcript:
        print(transcript, file=sys.stderr, end="" if transcript.endswith("\n") else "\n")
    row: dict[str, object] = {
        "id": definition.suite_id,
        "path": definition.path,
        "test_ids": test_ids,
        "tests_run": result.testsRun,
        "failures": _outcomes(result.failures),
        "errors": _outcomes(result.errors),
        "skipped": _outcomes(result.skipped),
        "expected_failures": _outcomes(result.expectedFailures),
        "unexpected_successes": [_test_id(test) for test in result.unexpectedSuccesses],
        "transcript_sha256": hashlib.sha256(transcript.encode("utf-8")).hexdigest(),
    }
    row["ok"] = (
        result.testsRun == len(test_ids)
        and not row["failures"]
        and not row["errors"]
        and not row["skipped"]
        and not row["expected_failures"]
        and not row["unexpected_successes"]
    )
    return row, None


def _empty_manifest(
    *,
    expected_head: str,
    head: str | None,
    clean: bool,
    os_name: str,
    declared_os: str,
) -> dict[str, object]:
    return {
        "schema": SCHEMA,
        "os": os_name,
        "declared_os": declared_os,
        "expected_head": expected_head,
        "head_sha": head,
        "head_exact": head == expected_head,
        "working_tree_clean": clean,
        "suite_contract_sha256": _canonical_sha256(
            [{"id": item.suite_id, "path": item.path} for item in SUITES]
        ),
        "expected_parity_sha256": EXPECTED_ALL_GREEN_PARITY_SHA256,
        "parity_sha256": None,
        "parity_exact": False,
        "evidence_sha256": None,
        "suites": [],
        "totals": {
            "tests_run": 0,
            "failures": 0,
            "errors": 0,
            "skipped": 0,
            "expected_failures": 0,
            "unexpected_successes": 0,
        },
        "problems": [],
        "ok": False,
    }


def _write_manifest(output: Path, manifest: dict[str, object]) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
        newline="\n",
    )


def run_manifest(
    *,
    root: Path,
    expected_head: str,
    output: Path,
    os_name: str,
    suite_loader: Callable[[Path], unittest.TestSuite] = _default_suite_loader,
) -> int:
    root = root.resolve()
    expected_head = expected_head.strip()
    head, clean, problems = _repository_state(root, expected_head)
    runtime_os = _runtime_os()
    manifest = _empty_manifest(
        expected_head=expected_head,
        head=head,
        clean=clean,
        os_name=runtime_os,
        declared_os=os_name,
    )
    manifest["problems"] = problems

    if os_name not in ALLOWED_OSES:
        problems.append(f"unsupported policy evidence OS {os_name!r}")
    if runtime_os != os_name:
        problems.append(
            f"runtime OS {runtime_os!r} does not equal declared OS {os_name!r}"
        )
    if not problems:
        suite_rows: list[dict[str, object]] = []
        for definition in SUITES:
            row, problem = _run_suite(definition, root, suite_loader)
            if problem is not None:
                problems.append(problem)
            elif row is not None:
                suite_rows.append(row)
                if not row["ok"]:
                    problems.append(
                        f"{definition.suite_id} did not pass every enumerated test"
                    )
        manifest["suites"] = suite_rows
        totals = manifest["totals"]
        assert isinstance(totals, dict)
        totals.update(
            {
                "tests_run": sum(int(row["tests_run"]) for row in suite_rows),
                "failures": sum(len(row["failures"]) for row in suite_rows),
                "errors": sum(len(row["errors"]) for row in suite_rows),
                "skipped": sum(len(row["skipped"]) for row in suite_rows),
                "expected_failures": sum(
                    len(row["expected_failures"]) for row in suite_rows
                ),
                "unexpected_successes": sum(
                    len(row["unexpected_successes"]) for row in suite_rows
                ),
            }
        )
        if totals["skipped"]:
            problems.append(
                f"policy evidence contains {totals['skipped']} skipped test(s)"
            )
        parity_rows = [
            {
                "id": row["id"],
                "path": row["path"],
                "test_ids": row["test_ids"],
                "tests_run": row["tests_run"],
                "failures": [item["test"] for item in row["failures"]],
                "errors": [item["test"] for item in row["errors"]],
                "skipped": [item["test"] for item in row["skipped"]],
                "expected_failures": [
                    item["test"] for item in row["expected_failures"]
                ],
                "unexpected_successes": row["unexpected_successes"],
            }
            for row in suite_rows
        ]
        manifest["parity_sha256"] = _canonical_sha256(
            {"suites": parity_rows}
        )
        manifest["parity_exact"] = (
            manifest["parity_sha256"] == EXPECTED_ALL_GREEN_PARITY_SHA256
        )
        manifest["evidence_sha256"] = _canonical_sha256(
            {
                "head_sha": head,
                "os": runtime_os,
                "parity_sha256": manifest["parity_sha256"],
            }
        )
        if not manifest["parity_exact"]:
            problems.append(
                "observed suite/test-ID outcomes do not match pinned parity contract"
            )
        final_head, final_clean, final_problems = _repository_state(
            root, expected_head
        )
        manifest["head_exact"] = bool(manifest["head_exact"]) and (
            final_head == expected_head
        )
        manifest["working_tree_clean"] = bool(
            manifest["working_tree_clean"]
        ) and final_clean
        problems.extend(
            f"repository changed while policy suites ran: {problem}"
            for problem in final_problems
        )

    manifest["ok"] = not problems and len(manifest["suites"]) == len(SUITES)
    _write_manifest(output, manifest)
    print(json.dumps(manifest, ensure_ascii=False, sort_keys=True))
    return 0 if manifest["ok"] else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--expected-head", required=True)
    parser.add_argument("--os", required=True, choices=ALLOWED_OSES, dest="os_name")
    parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()
    return run_manifest(
        root=arguments.root,
        expected_head=arguments.expected_head,
        output=arguments.output,
        os_name=arguments.os_name,
    )


if __name__ == "__main__":
    raise SystemExit(main())
