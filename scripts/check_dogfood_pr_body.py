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

# Bound on the one subprocess this gate runs (crown D-N4). A hung git must fail
# the gate closed with a named problem, not hold the job until the runner's own
# timeout reaps it.
GIT_TIMEOUT_SECONDS = 30

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
    "### Deferred / out of scope",
)
# The evidence section may be titled either way: the legacy garnet heading or the
# `dogfood-readiness` skill's heading. At least one must be present, and the first
# one in document order must carry a checked evidence item.
EVIDENCE_HEADINGS = (
    "### Desktop dogfood bundle",
    "### Evidence bundle",
)
NEGATED_ARC_PATTERNS = (
    r"\bnot\s+production\s+ARC\s+complete\b",
    r"\bno\s+production\s+ARC\s+complete\b",
    r"\bdoes\s+not\s+claim\s+production\s+ARC\s+complete\b",
    r"\bproduction\s+ARC\s+(?:is\s+)?(?:not\s+complete|deferred|still\s+deferred)\b",
)

# --- Markdown structure --------------------------------------------------------
# An ATX heading: optional indent, one to six `#`, whitespace, the heading text.
# Trailing whitespace is normalized away and nothing else is, so a heading
# matches a contract heading only when its text is exactly the contract's
# (hardening H3-01): `### Current truth — none stated` is a different heading.
HEADING_RE = re.compile(r"^[ \t]*(#{1,6})[ \t]+(.*?)[ \t]*$")
CHECKED_ITEM_RE = re.compile(r"^[ \t]*-[ \t]+\[x\][ \t]+(\S.*?)[ \t]*$")

# --- Evidence semantics (hardening H3-01) --------------------------------------
# A checked item counts as evidence only when it carries at least one token a
# reviewer can recompute. The classes below are calibrated against the merged
# bodies of #540–#546 (2026-09-02) and the fixtures in the test file.
HARD_TOKEN_PATTERNS = tuple(
    re.compile(pattern)
    for pattern in (
        r"`[^`\n]+`",  # a command, path, or value in backticks
        r"https://\S+",  # a URL
        r"(?<![\w/])(?=[0-9a-f]*\d)[0-9a-f]{7,40}(?![\w/])",  # a 7–40 hex SHA carrying a digit
        r"(?<!\w)#\d+\b",  # a #PR / #issue reference
        r"\b\d+/\d+\b",  # 6/6
        r"\bRan \d+ tests?\b",
        r"\bexit(?: code)? \d+\b",
        r"\bok: (?:true|false)\b",
        r"\b\d+ (?:pass(?:ed|es|ing)?|fail(?:ed|s|ures?|ing)?|tests?|files?|paths?"
        r"|errors?|warnings?|findings?|problems?|contexts?)\b",
        r"\b(?:answers?|returns?|status|HTTP) \d{3}\b",
        r"\b\d+ (?:→|->) \d+\b",
    )
)
# A bare repo path (no backticks): `a/b` that exists under the repo root, or any
# `scripts/` reference. Absolute and parent-traversing tokens never count.
PATH_TOKEN_RE = re.compile(r"(?<![\w./-])((?:[\w.-]+/)+[\w.-]*)")
# Remote verification may instead name the remote check it relies on together
# with its expected or observed status: every merged body's remote line reads
# "Fresh PR checks are required to settle before handoff; no CI conclusion is
# claimed in advance." — a named check plus a status, not a placeholder.
REMOTE_CHECK_NAME_RE = re.compile(r"\b(?:CI|checks?|workflow|Actions|matrix)\b")
REMOTE_CHECK_STATUS_RE = re.compile(
    r"\b(?:required|expected|settles?|settled|green|reds?|pass(?:ed|es|ing)?"
    r"|fail(?:ed|s|ing)?|conclusion|claim(?:ed|s)?|run|ran|running|pending"
    r"|queued|completes?|completed|exercis(?:es|ed|ing))\b",
    re.IGNORECASE,
)
# The evidence bundle may instead name the artifact and where it lives, e.g.
# "The one-line diff and the cross-family record named above." (#542).
ARTIFACT_NOUN_RE = re.compile(
    r"\b(?:records?|bundle|journal|capture|diffs?|table|artifacts?|manifest"
    r"|logs?|screenshots?|transcripts?|outputs?|report|dossier)\b",
    re.IGNORECASE,
)
ARTIFACT_LOCATOR_RE = re.compile(
    r"\b(?:named above|above|in the PR|review record|folder|path|directory"
    r"|on main|at main|merged with|committed|copied|preserved|quoted|cited"
    r"|attached|recorded|under)\b",
    re.IGNORECASE,
)
EVIDENCE_TOKEN_HINT = (
    "a command/path/value in backticks, a repo path, a 7-40 hex SHA, an https:// URL, "
    "a #PR reference, or a numeric result"
)
SECTION_KIND_HINTS = {
    "local": "",
    "remote": ", or the named CI/PR check with its expected status",
    "evidence": ", or a named artifact and where it lives",
}


class ValidationResult(NamedTuple):
    """Validation result with enough detail for CI output and tests."""

    sensitive: bool
    errors: list[str]


class Heading(NamedTuple):
    """One real Markdown heading line: offset of its line start, level, normalized text."""

    offset: int
    level: int
    text: str


class EvidenceStatus(NamedTuple):
    """How many checked items a section holds and how many carry an evidence token."""

    checked: int
    evidentiary: int


def _normalize(path: str) -> str:
    # Strip leading "./" as a prefix — NOT lstrip("./"), which strips '.' and
    # '/' as a character set and eats the dot off ".github/..."-style names.
    p = path.replace("\\", "/")
    while p.startswith("./"):
        p = p[2:]
    return p


def is_sensitive_path(path: str) -> bool:
    normalized = _normalize(path)
    return normalized in SENSITIVE_FILES or normalized.startswith(SENSITIVE_PREFIXES)


def is_readiness_sensitive(paths: list[str]) -> bool:
    return any(is_sensitive_path(path) for path in paths)


def headings(body: str) -> list[Heading]:
    """Every real Markdown heading line in document order, text normalized."""
    found: list[Heading] = []
    offset = 0
    for line in body.split("\n"):
        match = HEADING_RE.match(line)
        if match:
            hashes, text = match.group(1), match.group(2)
            found.append(Heading(offset, len(hashes), f"{hashes} {text}"))
        offset += len(line) + 1
    return found


def heading_line_pos(body: str, heading: str) -> int:
    """Offset of the first line whose normalized heading text equals `heading`
    exactly (a real Markdown heading, not a prose or inline-code mention, and not
    a heading that merely starts with the contract text). Returns -1 if absent."""
    for found in headings(body):
        if found.text == heading:
            return found.offset
    return -1


def section_text(body: str, heading: str) -> str | None:
    """Text from the `heading` line up to (not including) the next heading of the
    same or higher level (crown D-1): a `## ` closes an open `### ` section just as
    the next `### ` does. A deeper heading stays inside the section."""
    found = headings(body)
    for index, current in enumerate(found):
        if current.text != heading:
            continue
        for later in found[index + 1 :]:
            if later.level <= current.level:
                return body[current.offset : later.offset]
        return body[current.offset :]
    return None


def missing_headings(body: str) -> list[str]:
    missing = [heading for heading in REQUIRED_HEADINGS if heading_line_pos(body, heading) == -1]
    if present_evidence_heading(body) is None:
        missing.append("### Evidence bundle (or ### Desktop dogfood bundle)")
    return missing


def present_evidence_heading(body: str) -> str | None:
    present: list[tuple[int, str]] = []
    for heading in EVIDENCE_HEADINGS:
        position = heading_line_pos(body, heading)
        if position != -1:
            present.append((position, heading))
    return min(present)[1] if present else None


def has_unqualified_production_arc_claim(body: str) -> bool:
    if not re.search(r"\bproduction\s+ARC\s+complete\b", body, flags=re.IGNORECASE):
        return False
    return not any(re.search(pattern, body, flags=re.IGNORECASE) for pattern in NEGATED_ARC_PATTERNS)


def checked_items(section: str) -> list[str]:
    """Checked list items in a section, each joined with its indented continuation lines."""
    items: list[str] = []
    for line in section.split("\n"):
        match = CHECKED_ITEM_RE.match(line)
        if match:
            items.append(match.group(1))
        elif items and line[:1] in (" ", "\t") and line.strip():
            items[-1] = f"{items[-1]} {line.strip()}"
    return items


def _path_token_present(text: str, root: Path) -> bool:
    for token in PATH_TOKEN_RE.findall(text):
        cleaned = token.rstrip(".,;:)")
        if not cleaned or cleaned.startswith("/") or ".." in cleaned.split("/"):
            continue
        if cleaned.startswith("scripts/"):
            return True
        try:
            if (root / cleaned).exists():
                return True
        except OSError:
            continue
    return False


def item_carries_evidence(text: str, kind: str, root: Path = ROOT) -> bool:
    """True when a checked item's text carries an evidence token. `kind` is
    "local", "remote", or "evidence"; the remote and evidence sections accept
    one additional, section-specific token class each (see the patterns above)."""
    if any(pattern.search(text) for pattern in HARD_TOKEN_PATTERNS):
        return True
    if _path_token_present(text, root):
        return True
    if kind == "remote":
        return bool(REMOTE_CHECK_NAME_RE.search(text) and REMOTE_CHECK_STATUS_RE.search(text))
    if kind == "evidence":
        return bool(ARTIFACT_NOUN_RE.search(text) and ARTIFACT_LOCATOR_RE.search(text))
    return False


def _section_kind(section_heading: str) -> str:
    if section_heading == "### Remote verification":
        return "remote"
    if section_heading in EVIDENCE_HEADINGS:
        return "evidence"
    return "local"


def evidence_status(body: str, section_heading: str, root: Path = ROOT) -> EvidenceStatus:
    section = section_text(body, section_heading)
    if section is None:
        return EvidenceStatus(checked=0, evidentiary=0)
    kind = _section_kind(section_heading)
    items = checked_items(section)
    evidentiary = sum(1 for item in items if item_carries_evidence(item, kind, root))
    return EvidenceStatus(checked=len(items), evidentiary=evidentiary)


def has_checked_evidence(body: str, section_heading: str) -> bool:
    return evidence_status(body, section_heading).evidentiary > 0


def _evidence_problems(body: str, section_heading: str, label: str) -> list[str]:
    if heading_line_pos(body, section_heading) == -1:
        return []  # reported as a missing heading instead
    status = evidence_status(body, section_heading)
    if status.checked == 0:
        return [f"{label} section must include at least one checked evidence item"]
    if status.evidentiary == 0:
        kind = _section_kind(section_heading)
        return [
            f"{label} section has {status.checked} checked item(s) but none carries evidence: "
            f"need {EVIDENCE_TOKEN_HINT}{SECTION_KIND_HINTS[kind]}"
        ]
    return []


def validate_body(body: str, changed_paths: list[str]) -> ValidationResult:
    if not is_readiness_sensitive(changed_paths):
        return ValidationResult(sensitive=False, errors=[])

    errors: list[str] = []
    for heading in missing_headings(body):
        errors.append(f"missing required heading: {heading}")

    errors.extend(_evidence_problems(body, "### Local verification", "local verification"))
    errors.extend(_evidence_problems(body, "### Remote verification", "remote verification"))

    evidence_heading = present_evidence_heading(body)
    if evidence_heading is not None:
        errors.extend(_evidence_problems(body, evidence_heading, "evidence bundle"))

    if has_unqualified_production_arc_claim(body):
        errors.append("unqualified production ARC completion claim")

    return ValidationResult(sensitive=True, errors=errors)


def read_changed_paths(base: str, head: str, root: Path = ROOT) -> list[str]:
    output = subprocess.check_output(
        ["git", "diff", "--name-only", f"{base}...{head}"],
        cwd=root,
        text=True,
        timeout=GIT_TIMEOUT_SECONDS,
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
        try:
            changed_paths = read_changed_paths(args.base, args.head)
        except subprocess.TimeoutExpired:
            print(
                f"::error::dogfood-pr-body: git diff --name-only {args.base}...{args.head} "
                f"timed out after {GIT_TIMEOUT_SECONDS}s; failing closed"
            )
            return 1
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
