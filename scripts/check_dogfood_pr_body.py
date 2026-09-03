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
    # H3-02: this checker produces the required "PR dogfood evidence" context, so
    # a change to it (or to the tests that prove it works) must carry that
    # evidence itself.  It guards everything else; it may not skip its own gate.
    "scripts/check_dogfood_pr_body.py",
    "scripts/test_check_dogfood_pr_body.py",
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
#
# A code span is evidence only when it carries something a reviewer can
# recompute (review v1 cure). `` ` ` `` (whitespace only), `` `-` `` and
# `` `--` `` are not commands, paths, or values: content must be non-empty
# after stripping, at least two characters, and carry a letter or digit.
CODE_SPAN_RE = re.compile(r"`([^`\n]+)`")
CODE_SPAN_MIN_CONTENT = 2
CODE_SPAN_SUBSTANCE_RE = re.compile(r"[^\W_]")
# A code span that is only a verdict word is a claim, not evidence: `ok` alone
# satisfied every evidence section before this rule. Real evidence shows
# structure — a command has whitespace, a count has a digit, a path or flag has a
# separator. A bare status word has none of those, and is rejected even when it
# is long enough to clear CODE_SPAN_MIN_CONTENT.
CODE_SPAN_STATUS_ONLY = frozenset(
    {
        "clean", "complete", "completed", "done", "fine", "good", "green", "n/a",
        "na", "no", "ok", "okay", "pass", "passed", "passes", "passing", "ran",
        "success", "successful", "succeeded", "true", "verified", "yes",
    }
)
CODE_SPAN_STRUCTURE_RE = re.compile(r"[\s\d/.:=_+-]")

HARD_TOKEN_PATTERNS = tuple(
    re.compile(pattern)
    for pattern in (
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
    r"\b(?:named above|in the PR|review record|folder|path|directory"
    r"|on main|at main|merged with|committed|copied|preserved|quoted|cited"
    r"|attached|recorded|under)\b",
    re.IGNORECASE,
)
# Negation (review v1 cure). A claim that something did NOT happen is not
# evidence that it did: "No CI run.", "No report was recorded.", "no CI run
# was required." A negated clause contributes nothing, and an item whose every
# clause is negated satisfies no alternative at all — hard tokens included.
# Negation is judged per CLAUSE, not per item, because every merged body
# (#540, #542-#546) states its remote line as a positive claim followed by a
# disclaimer: "Fresh PR checks are required to settle before handoff; no CI
# conclusion is claimed in advance." Clause one is the evidence; clause two is
# the disclaimer, and a per-item guard would reject all seven real bodies.
LEADING_NEGATION_RE = re.compile(r"^\s*(?:no|none|not|never|n/?a\b|nothing)\b", re.IGNORECASE)
INTERNAL_NEGATION_RE = re.compile(
    r"\b(?:no|not|never|none)\s+"
    r"(?:ci|run|report|record|artifact|bundle|evidence|check|log|output)\b",
    re.IGNORECASE,
)
# Clause boundaries: "; " and a sentence break before a capital, "(", or a code
# span. Code spans are masked before splitting so a boundary is never found
# inside one, which keeps multi-token commands and paths intact.
CLAUSE_SPLIT_RE = re.compile(r";\s+|(?<=\.)\s+(?=[A-Z(\x00])")
CODE_SPAN_PLACEHOLDER_RE = re.compile(r"\x00(\d+)\x00")
# The two widened alternatives (a named check + its status, a named artifact +
# its locator) are prose, not recomputable tokens, so they carry two extra
# requirements: the clause must be wholly POSITIVE — a single negation word
# anywhere in it disqualifies it, which stops "The CI run was not required."
# — and it must clear a substance floor, which stops two-word stubs like
# "CI. required.", "checks green", "Bundle recorded." The floor is calibrated
# against the merged bodies: the shortest real qualifying clause is "CI matrix
# expected green before merge." at six words. This is a vacuity floor, not a
# semantic guarantee: it bounds how empty a prose claim may be, nothing more.
NEGATION_WORD_RE = re.compile(
    r"\b(?:no|not|never|none|nothing|n/?a|cannot)\b|n't\b", re.IGNORECASE
)
WIDENED_MIN_WORDS = 5
WORD_RE = re.compile(r"[^\W_]")

EVIDENCE_TOKEN_HINT = (
    "a command/path/value in backticks, a repo path, a 7-40 hex SHA, an https:// URL, "
    "a #PR reference, or a numeric result"
)
EVIDENCE_EXCLUSION_HINT = "; a blank code span and a negated claim never count"
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


def code_span_carries_content(content: str) -> bool:
    """True when a backtick span's content is substantive: non-empty after
    stripping, at least two characters, and carrying a letter or digit. A span
    that is blank, whitespace-only, or pure punctuation is not a recomputable
    command, path, or value (review v1 cure).

    Substantive is weaker than evidentiary: ``ok`` is substantive but is not
    evidence. See :func:`code_span_is_evidentiary`."""
    stripped = content.strip()
    if len(stripped) < CODE_SPAN_MIN_CONTENT:
        return False
    return bool(CODE_SPAN_SUBSTANCE_RE.search(stripped))


def code_span_is_evidentiary(content: str) -> bool:
    """True when a backtick span is evidence rather than a verdict.

    A span that is only a status word is a claim: a bare ``ok`` satisfied every
    evidence section before this rule (review v2 cure). Recomputable evidence
    shows structure — a command carries whitespace, a count carries a digit, a
    path or flag carries a separator. A bare verdict word carries none of those,
    and is rejected even when it clears CODE_SPAN_MIN_CONTENT."""
    if not code_span_carries_content(content):
        return False
    stripped = content.strip()
    if stripped.casefold() in CODE_SPAN_STATUS_ONLY:
        return False
    return bool(CODE_SPAN_STRUCTURE_RE.search(stripped))


def _code_span_present(text: str) -> bool:
    return any(code_span_is_evidentiary(m.group(1)) for m in CODE_SPAN_RE.finditer(text))


def clauses(text: str) -> list[str]:
    """Split an item into clauses on "; " and sentence breaks. Code spans are
    masked first, so a command containing a period or semicolon is never cut."""
    spans: list[str] = []

    def stash(match: re.Match[str]) -> str:
        spans.append(match.group(0))
        return f"\x00{len(spans) - 1}\x00"

    masked = CODE_SPAN_RE.sub(stash, text)
    found: list[str] = []
    for part in CLAUSE_SPLIT_RE.split(masked):
        restored = CODE_SPAN_PLACEHOLDER_RE.sub(lambda m: spans[int(m.group(1))], part).strip()
        if restored:
            found.append(restored)
    return found


def clause_supports_widened_alternative(clause: str) -> bool:
    """True when a clause is substantive enough to carry one of the two prose
    alternatives: wholly positive, and at least WIDENED_MIN_WORDS words."""
    if NEGATION_WORD_RE.search(clause):
        return False
    words = [token for token in clause.split() if WORD_RE.search(token)]
    return len(words) >= WIDENED_MIN_WORDS


def clause_is_negated(clause: str) -> bool:
    """True when a clause denies rather than reports: it opens with a negation,
    or negates one of the nouns the evidence alternatives are built on."""
    return bool(LEADING_NEGATION_RE.search(clause) or INTERNAL_NEGATION_RE.search(clause))


def item_carries_evidence(text: str, kind: str, root: Path = ROOT) -> bool:
    """True when a checked item's text carries an evidence token. `kind` is
    "local", "remote", or "evidence"; the remote and evidence sections accept
    one additional, section-specific token class each (see the patterns above).

    Only non-negated clauses are read, and the two section-specific alternatives
    must be satisfied POSITIVELY and within a SINGLE clause that carries no
    negation word and clears the substance floor: the named check and its status
    together, the named artifact and its locator together. An item with no
    surviving clause carries no evidence at all (review v1 cure)."""
    for clause in clauses(text):
        if clause_is_negated(clause):
            continue
        if _code_span_present(clause):
            return True
        if any(pattern.search(clause) for pattern in HARD_TOKEN_PATTERNS):
            return True
        if _path_token_present(clause, root):
            return True
        if not clause_supports_widened_alternative(clause):
            continue
        if kind == "remote":
            if REMOTE_CHECK_NAME_RE.search(clause) and REMOTE_CHECK_STATUS_RE.search(clause):
                return True
        if kind == "evidence":
            if ARTIFACT_NOUN_RE.search(clause) and ARTIFACT_LOCATOR_RE.search(clause):
                return True
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
            f"need {EVIDENCE_TOKEN_HINT}{SECTION_KIND_HINTS[kind]}{EVIDENCE_EXCLUSION_HINT}"
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
