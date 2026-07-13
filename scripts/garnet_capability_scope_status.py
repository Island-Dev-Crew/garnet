#!/usr/bin/env python3
"""Capability-claim scope fixture (S114 acceptance condition #3 + #4).

A copy/claim linter over the *public* capability-enforcement language. It does
not re-derive runtime behavior — that is the job of the Rust caps-enforcement
tests and `garnet_caps_enforcement_status.py`. This gate keeps the public
wording bounded:

  1. The capability enforcement scope table exists and names every enforcement
     class (declared/checker-only, runtime-gated, entry-gated, OS-sandboxed,
     caps-invisible) plus the "may not say" fence.
  2. `docs/why.html` still carries EXACTLY the two bounded `enforced:` claims
     (test-runner entry authority; VM/interpreter scope parity) — no more, so a
     claim cannot be added by accident (condition #3).
  3. The cited regression-test anchors exist on disk.
  4. Forbidden universal-enforcement phrases are absent from the public surfaces
     (condition #4): no "no ambient authority, ever", no "universal @caps
     runtime enforcement".

`--gate` exits 1 unless every check passes.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

SCHEMA = "garnet.capability_scope/v1"
ROOT = Path(__file__).resolve().parents[1]

SCOPE_DOC = ROOT / "C_Language_Specification" / "GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md"
WHY_HTML = ROOT / "docs" / "why.html"

# Public surfaces the forbidden-phrase fence applies to. The scope doc itself is
# NOT public copy — it discusses runtime enforcement to bound it — so it is not
# scanned for forbidden phrases.
PUBLIC_SURFACES = [
    ROOT / "README.md",
    ROOT / "docs" / "index.html",
    ROOT / "docs" / "why.html",
]

REQUIRED_SCOPE_TERMS = [
    "Declared (checker-only)",
    "Runtime-gated",
    "Entry-gated",
    "OS-sandboxed",
    "Caps-invisible",
    "may not say",
]

CITED_TEST_ANCHORS = [
    ROOT / "garnet-cli" / "tests" / "test_entry_authority.rs",
    ROOT / "garnet-vm" / "tests" / "scope_shadowing_parity.rs",
]

# Regexes that must NOT appear in any public surface (case-insensitive).
FORBIDDEN_PATTERNS = [
    r"no\s+ambient\s+authority,?\s+ever",
    r"universal\s+@?caps\s+runtime\s+enforcement",
    r"universal\s+runtime\s+enforcement",
]

# The bold marker for a published enforced claim in why.html.
ENFORCED_CLAIM_MARKER = "<b>enforced:</b>"
EXPECTED_ENFORCED_CLAIMS = 2


@dataclass
class CapabilityScopeStatus:
    schema: str
    ok: bool
    scope_doc_present: bool
    scope_terms_missing: list[str] = field(default_factory=list)
    enforced_claim_count: int = 0
    enforced_claim_expected: int = EXPECTED_ENFORCED_CLAIMS
    cited_anchors_missing: list[str] = field(default_factory=list)
    forbidden_hits: list[str] = field(default_factory=list)
    problems: list[str] = field(default_factory=list)


def _read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except OSError:
        return ""


def _rel(path: Path) -> str:
    """Repo-relative POSIX path, or the full path if it is outside the repo
    (e.g. a temp file injected by a test)."""
    try:
        return path.relative_to(ROOT).as_posix()
    except ValueError:
        return path.as_posix()


def read_status() -> CapabilityScopeStatus:
    problems: list[str] = []

    scope_text = _read(SCOPE_DOC)
    scope_present = bool(scope_text)
    if not scope_present:
        problems.append(f"missing scope doc: {_rel(SCOPE_DOC)}")
    terms_missing = [t for t in REQUIRED_SCOPE_TERMS if t not in scope_text]
    if terms_missing:
        problems.append(f"scope doc missing terms: {', '.join(terms_missing)}")

    why_text = _read(WHY_HTML)
    enforced_count = why_text.count(ENFORCED_CLAIM_MARKER)
    if enforced_count != EXPECTED_ENFORCED_CLAIMS:
        problems.append(
            f"docs/why.html has {enforced_count} '{ENFORCED_CLAIM_MARKER}' claims, "
            f"expected exactly {EXPECTED_ENFORCED_CLAIMS} (condition #3: do not add public claims)"
        )
    for anchor in ("test_entry_authority", "scope_shadowing_parity"):
        if anchor not in why_text:
            problems.append(f"docs/why.html no longer cites its test anchor: {anchor}")

    anchors_missing = [
        _rel(p) for p in CITED_TEST_ANCHORS if not p.is_file()
    ]
    if anchors_missing:
        problems.append(f"cited test anchors missing: {', '.join(anchors_missing)}")

    forbidden_hits: list[str] = []
    for surface in PUBLIC_SURFACES:
        text = _read(surface)
        for pat in FORBIDDEN_PATTERNS:
            for m in re.finditer(pat, text, flags=re.IGNORECASE):
                rel = _rel(surface)
                forbidden_hits.append(f"{rel}: '{m.group(0)}'")
    if forbidden_hits:
        problems.append(f"forbidden universal-enforcement phrasing: {'; '.join(forbidden_hits)}")

    ok = not problems
    return CapabilityScopeStatus(
        schema=SCHEMA,
        ok=ok,
        scope_doc_present=scope_present,
        scope_terms_missing=terms_missing,
        enforced_claim_count=enforced_count,
        cited_anchors_missing=anchors_missing,
        forbidden_hits=forbidden_hits,
        problems=problems,
    )


def render_markdown(s: CapabilityScopeStatus) -> str:
    lines = [
        "# Garnet capability-claim scope status (S114 condition #3 + #4)",
        "",
        f"_Schema {s.schema}._",
        "",
        f"- scope doc present: **{s.scope_doc_present}**",
        f"- published `enforced:` claims: **{s.enforced_claim_count}/{s.enforced_claim_expected}**",
        f"- cited test anchors missing: {len(s.cited_anchors_missing)}",
        f"- forbidden-phrase hits: {len(s.forbidden_hits)}",
        f"- overall: **{'ok' if s.ok else 'FAIL'}**",
    ]
    for p in s.problems:
        lines.append(f"  - PROBLEM: {p}")
    lines.append("")
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate", action="store_true", help="exit 1 unless every claim-scope check passes"
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    s = read_status()
    print(render_markdown(s) if args.format == "md" else json.dumps(asdict(s), indent=2))
    if args.gate and not s.ok:
        print(f"capability-scope gate FAILED: {len(s.problems)} problem(s)", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
