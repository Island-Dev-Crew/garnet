#!/usr/bin/env python3
"""Enumerate CR-bearing text blobs outside Garnet's byte-exact evidence fences.

The gate reports paths at an exact Git commit and tree. It deliberately does
not pin or assert a repository-wide count: additions and removals change counts,
while the policy question is whether any violating path exists.

`proofs/**` and `ops/**/evidence/**` are excluded because those paths preserve
captured bytes. Their own manifests and the evidence-integrity gate govern them.
Git's binary classifier (`git grep -I`) separates text from binary blobs.
Exact-commit scans bind Git's attribute lookup to that same commit through
`GIT_ATTR_SOURCE`; dirty worktree attributes cannot change the classification.
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = "garnet.text_byte_policy/v1"
EXCLUSIONS = ("ops/**/evidence/**", "proofs/**")
GIT_TIMEOUT_SECONDS = 30


@dataclass
class TextBytePolicyStatus:
    schema: str = SCHEMA
    ref: str = "HEAD"
    commit: str | None = None
    tree: str | None = None
    exclusions: list[str] = field(default_factory=lambda: list(EXCLUSIONS))
    violations: list[str] = field(default_factory=list)
    problems: list[str] = field(default_factory=list)
    ok: bool = False


def _git(
    root: Path,
    *args: str,
    attr_source: str | None = None,
) -> subprocess.CompletedProcess[bytes]:
    env = os.environ.copy()
    env["GIT_NO_REPLACE_OBJECTS"] = "1"
    if attr_source is not None:
        env["GIT_ATTR_SOURCE"] = attr_source
    try:
        return subprocess.run(
            ["git", *args],
            cwd=root,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=GIT_TIMEOUT_SECONDS,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        return subprocess.CompletedProcess(
            ["git", *args],
            124,
            stdout=b"",
            stderr=str(exc).encode("utf-8", errors="replace"),
        )


def _resolve(root: Path, expression: str, label: str) -> tuple[str | None, list[str]]:
    result = _git(root, "rev-parse", "--verify", "--end-of-options", expression)
    if result.returncode != 0:
        return None, [f"{label} could not be resolved"]
    try:
        lines = result.stdout.decode("ascii", errors="strict").splitlines()
    except UnicodeDecodeError:
        return None, [f"{label} resolution was malformed"]
    if len(lines) != 1 or len(lines[0]) != 40 or any(c not in "0123456789abcdef" for c in lines[0]):
        return None, [f"{label} resolution was malformed"]
    return lines[0], []


def _valid_path(path: str) -> bool:
    candidate = PurePosixPath(path)
    return (
        bool(path)
        and "\x00" not in path
        and "\\" not in path
        and not candidate.is_absolute()
        and all(part not in ("", ".", "..") for part in candidate.parts)
        and candidate.as_posix() == path
    )


def _excluded(path: str) -> bool:
    if path.startswith("proofs/"):
        return True
    parts = PurePosixPath(path).parts
    return bool(parts and parts[0] == "ops" and "evidence" in parts[1:])


def _cr_text_paths(root: Path, commit: str) -> tuple[list[str], list[str]]:
    result = _git(
        root,
        "grep",
        "-I",
        "-l",
        "-z",
        "-e",
        "\r",
        commit,
        "--",
        attr_source=commit,
    )
    if result.returncode == 1:
        return [], []
    if result.returncode != 0:
        return [], ["Git could not enumerate CR-bearing text blobs"]
    if result.stdout and not result.stdout.endswith(b"\0"):
        return [], ["Git CR-bearing text enumeration was not NUL terminated"]
    prefix = (commit + ":").encode("ascii")
    paths: list[str] = []
    problems: list[str] = []
    for raw in result.stdout.rstrip(b"\0").split(b"\0") if result.stdout else []:
        if not raw.startswith(prefix):
            problems.append("Git CR-bearing text enumeration returned an unbound path")
            continue
        try:
            path = raw[len(prefix) :].decode("utf-8", errors="strict")
        except UnicodeDecodeError:
            problems.append("Git CR-bearing text enumeration returned a non-UTF-8 path")
            continue
        if not _valid_path(path):
            problems.append("Git CR-bearing text enumeration returned an unsafe path")
            continue
        paths.append(path)
    if paths != sorted(set(paths)):
        problems.append("Git CR-bearing text enumeration was unsorted or duplicated")
    return paths, problems


def read_status(
    *,
    root: Path = ROOT,
    ref: str = "HEAD",
) -> TextBytePolicyStatus:
    root = root.resolve()
    status = TextBytePolicyStatus(ref=ref)
    commit, commit_problems = _resolve(root, f"{ref}^{{commit}}", "scan ref")
    status.problems.extend(commit_problems)
    if commit is None:
        return status
    status.commit = commit
    tree, tree_problems = _resolve(root, f"{commit}^{{tree}}", "scan tree")
    status.problems.extend(tree_problems)
    status.tree = tree
    paths, scan_problems = _cr_text_paths(root, commit)
    status.problems.extend(scan_problems)
    status.violations = [path for path in paths if not _excluded(path)]
    status.ok = not status.problems and not status.violations
    return status


def render_json(status: TextBytePolicyStatus) -> str:
    return json.dumps(asdict(status), ensure_ascii=False, indent=2, sort_keys=True) + "\n"


def render_text(status: TextBytePolicyStatus) -> str:
    lines = [
        f"Text-byte policy at {status.commit or status.ref} / tree {status.tree or 'unresolved'}:",
    ]
    lines.extend(f"problem: {problem}" for problem in status.problems)
    lines.extend(f"CR-bearing text path: {path}" for path in status.violations)
    if status.ok:
        lines.append("PASS: no CR-bearing text paths exist outside byte-exact evidence fences")
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ref", default="HEAD", help="exact commit-ish to enumerate")
    parser.add_argument("--format", choices=("json", "text"), default="json")
    parser.add_argument("--gate", action="store_true")
    args = parser.parse_args(list(argv) if argv is not None else None)
    status = read_status(ref=args.ref)
    output = render_json(status) if args.format == "json" else render_text(status)
    sys.stdout.write(output)
    return 1 if args.gate and not status.ok else 0


if __name__ == "__main__":
    raise SystemExit(main())
