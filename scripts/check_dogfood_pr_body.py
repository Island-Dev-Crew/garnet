#!/usr/bin/env python3
"""Validate dogfood readiness evidence in readiness-sensitive PR bodies."""
from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import NamedTuple

ROOT = Path(__file__).resolve().parents[1]

SENSITIVE_PREFIXES = (
    ".github/workflows/",
    "C_Language_Specification/",
    "F_Project_Management/",
    "examples/",
    "garnet-check-v0.3/",
    "garnet-cli/tests/conformance_",
    "garnet-interp-v0.3/",
    "garnet-memory-v0.3/",
    "garnet-parser-v0.3/",
)
SENSITIVE_FILES = {
    ".github/PULL_REQUEST_TEMPLATE.md",
    "Cargo.lock",
    "CURRENT_STATE.md",
}
REQUIRED_HEADINGS = (
    "## Dogfood Readiness",
    "### Current truth",
    "### Local verification",
    "### Remote verification",
    "### Desktop dogfood bundle",
    "### Deferred / out of scope",
)
NEGATED_ARC_PATTERNS = (
    r"\bnot\s+production\s+ARC\s+complete\b",
    r"\bno\s+production\s+ARC\s+complete\b",
    r"\bdoes\s+not\s+claim\s+production\s+ARC\s+complete\b",
    r"\bproduction\s+ARC\s+(?:is\s+)?(?:not\s+complete|deferred|still\s+deferred)\b",
)


class ValidationResult(NamedTuple):
    """Validation result with enough detail for CI output and tests."""

    sensitive: bool
    errors: list[str]


def is_sensitive_path(path: str) -> bool:
    normalized = path.replace("\\", "/").lstrip("./")
    return normalized in SENSITIVE_FILES or normalized.startswith(SENSITIVE_PREFIXES)


def is_readiness_sensitive(paths: list[str]) -> bool:
    return any(is_sensitive_path(path) for path in paths)


def missing_headings(body: str) -> list[str]:
    return [heading for heading in REQUIRED_HEADINGS if heading not in body]


def has_unqualified_production_arc_claim(body: str) -> bool:
    if not re.search(r"\bproduction\s+ARC\s+complete\b", body, flags=re.IGNORECASE):
        return False
    return not any(re.search(pattern, body, flags=re.IGNORECASE) for pattern in NEGATED_ARC_PATTERNS)


def has_checked_evidence(body: str, section_heading: str) -> bool:
    start = body.find(section_heading)
    if start == -1:
        return False
    next_heading = body.find("\n### ", start + len(section_heading))
    section = body[start:] if next_heading == -1 else body[start:next_heading]
    return bool(re.search(r"(?m)^\s*-\s+\[x\]\s+`?[^`\n]+`?", section))


def validate_body(body: str, changed_paths: list[str]) -> ValidationResult:
    if not is_readiness_sensitive(changed_paths):
        return ValidationResult(sensitive=False, errors=[])

    errors: list[str] = []
    for heading in missing_headings(body):
        errors.append(f"missing required heading: {heading}")

    if "### Local verification" in body and not has_checked_evidence(body, "### Local verification"):
        errors.append("local verification section must include at least one checked evidence item")

    if "### Remote verification" in body and not has_checked_evidence(body, "### Remote verification"):
        errors.append("remote verification section must include at least one checked evidence item")

    if "### Desktop dogfood bundle" in body and not has_checked_evidence(body, "### Desktop dogfood bundle"):
        errors.append("desktop dogfood bundle section must include at least one checked evidence item")

    if has_unqualified_production_arc_claim(body):
        errors.append("unqualified production ARC completion claim")

    return ValidationResult(sensitive=True, errors=errors)


def read_changed_paths(base: str, head: str, root: Path = ROOT) -> list[str]:
    output = subprocess.check_output(
        ["git", "diff", "--name-only", f"{base}...{head}"],
        cwd=root,
        text=True,
    )
    return [line.strip() for line in output.splitlines() if line.strip()]


def read_body(path: str | None) -> str:
    if path:
        return Path(path).read_text(encoding="utf-8")
    return os.environ.get("PR_BODY", "")


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", help="base ref or SHA for git diff")
    parser.add_argument("--head", help="head ref or SHA for git diff")
    parser.add_argument("--body-file", help="file containing the PR body")
    parser.add_argument(
        "--changed-file",
        action="append",
        default=[],
        help="changed file path; may be repeated, bypasses git diff when present",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    if args.changed_file:
        changed_paths = args.changed_file
    elif args.base and args.head:
        changed_paths = read_changed_paths(args.base, args.head)
    else:
        print("dogfood-pr-body: --base/--head or --changed-file is required", file=sys.stderr)
        return 2

    result = validate_body(read_body(args.body_file), changed_paths)
    if not result.sensitive:
        print("dogfood-pr-body: skipped (no readiness-sensitive files changed)")
        return 0

    if result.errors:
        for error in result.errors:
            print(f"::error::{error}")
        return 1

    print(f"dogfood-pr-body: ok ({len(changed_paths)} changed files checked)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
