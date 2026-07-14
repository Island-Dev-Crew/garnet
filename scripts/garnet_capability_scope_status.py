#!/usr/bin/env python3
"""Exact current-truth fixture for S114 capability claims.

This is a copy/metadata regression guard, not a substitute for the behavioral
Rust suites. It prevents a green marker-count check from hiding semantic drift:
the two allowed public ``enforced:`` claims are hash-pinned, the accepted-S114
and independence states must coexist, the strict-default claim must retain its
high-level/raw-host boundary, and WV-5 must remain separated from browser proof.

``--gate`` exits 1 unless every check passes.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

SCHEMA = "garnet.capability_scope/v2"
ROOT = Path(__file__).resolve().parents[1]

SCOPE_DOC = ROOT / "C_Language_Specification" / "GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md"
RED_TEAM_DOC = ROOT / "C_Language_Specification" / "GARNET_RED_TEAM.md"
WHY_HTML = ROOT / "docs" / "why.html"
CURRENT_STATE = ROOT / "CURRENT_STATE.md"
WASM_TARGET = ROOT / "F_Project_Management" / "GARNET_WASM_TARGET.md"
ACCEPTANCE_JSON = ROOT / "F_Project_Management" / "LAUNCH" / "S114_ACCEPTANCE.json"
PLAYGROUND_REPORTER = ROOT / "scripts" / "garnet_playground_readiness.py"

PUBLIC_SURFACES = [
    ROOT / "README.md",
    ROOT / "docs" / "index.html",
    WHY_HTML,
]

CURRENT_TRUTH_SURFACES = [
    WHY_HTML,
    SCOPE_DOC,
    CURRENT_STATE,
    RED_TEAM_DOC,
    WASM_TARGET,
    PLAYGROUND_REPORTER,
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

FORBIDDEN_PATTERNS = [
    r"no\s+ambient\s+authority,?\s+ever",
    r"universal\s+@?caps\s+runtime\s+enforcement",
    r"universal\s+runtime\s+enforcement",
]

STALE_TRUTH_PATTERNS = [
    r"pending\s+Jon(?:'s|â€™s)\s+acceptance",
    r"S114\s+acceptance\s+remains[^.]*pending",
    r"being\s+hardened\s*\(?embedder\s+strict-by-default",
    r"no\s+wasm\s+(?:artifact\s+is\s+)?built",
    r"the\s+wasm\s+build\s+is\s+deferred",
    r"in-browser\s+execution\s+waits\s+on\s+the\s+WASM\s+build",
]

ENFORCED_CLAIM_MARKER = "<b>enforced:</b>"
EXPECTED_ENFORCED_CLAIMS = 2
EXPECTED_ENFORCED_CLAIM_HASHES = [
    "8cea8eec892bb7b908c0320fef64c9a0af02167d094141f7258aec566b5f57f0",
    "032b790318e1d10a80418f59e6f363e43671193a080c26a4803dd2beffb2a541",
]

CANONICAL_TRUTH_SNIPPETS: dict[Path, list[str]] = {
    WHY_HTML: [
        "S114 acceptance is recorded as <code>accepted-scoped</code> by Jon; the independent verdict remains <code>independently-re-verified-with-fixes</code>.",
        "WV&#8209;5 proves the Wasm build and real Node execution from a clean Windows checkout; browser&#8209;page execution remains unproven until the W&#8209;PLAY Playwright gate passes.",
    ],
    SCOPE_DOC: [
        "high-level load/eval/call methods receive the same strict default",
        "Low-level Rust host APIs are not an embedder sandbox",
        "post-acceptance delta review reopened",
        "Interpreter::new_permissive()",
    ],
    CURRENT_STATE: [
        "acceptance is recorded as `accepted-scoped` (2026-07-12)",
        "**independently-re-verified-with-fixes**",
    ],
    RED_TEAM_DOC: [
        "acceptance is recorded as `accepted-scoped` (2026-07-12)",
        "**independently-re-verified-with-fixes**",
    ],
    WASM_TARGET: [
        "WV-5 proves an interpreter compiled to real Wasm and executed through Node",
        "does not prove a live browser page",
    ],
}


@dataclass
class CapabilityScopeStatus:
    schema: str
    ok: bool
    scope_doc_present: bool
    scope_terms_missing: list[str] = field(default_factory=list)
    enforced_claim_count: int = 0
    enforced_claim_expected: int = EXPECTED_ENFORCED_CLAIMS
    enforced_claim_hashes: list[str] = field(default_factory=list)
    enforced_claim_hashes_match: bool = False
    acceptance_state: str = "missing-or-invalid"
    post_acceptance_closure_state: str = "missing-or-invalid"
    current_truth_missing: list[str] = field(default_factory=list)
    cited_anchors_missing: list[str] = field(default_factory=list)
    forbidden_hits: list[str] = field(default_factory=list)
    stale_truth_hits: list[str] = field(default_factory=list)
    problems: list[str] = field(default_factory=list)


def _read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except OSError:
        return ""


def _rel(path: Path) -> str:
    try:
        return path.relative_to(ROOT).as_posix()
    except ValueError:
        return path.as_posix()


def _normalized(text: str) -> str:
    return re.sub(r"\s+", " ", text).strip()


def _enforced_claim_hashes(why_text: str) -> list[str]:
    # Each canonical claim is deliberately a single <li> source line. Hashing
    # the normalized complete line pins meaning, evidence links, and test anchor
    # while ignoring indentation-only edits.
    hashes: list[str] = []
    for line in why_text.splitlines():
        if ENFORCED_CLAIM_MARKER in line:
            hashes.append(hashlib.sha256(_normalized(line).encode("utf-8")).hexdigest())
    return hashes


def _read_acceptance() -> dict | None:
    try:
        value = json.loads(ACCEPTANCE_JSON.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    return value if isinstance(value, dict) else None


def read_status() -> CapabilityScopeStatus:
    problems: list[str] = []
    scope_text = _read(SCOPE_DOC)
    scope_present = bool(scope_text)
    if not scope_present:
        problems.append(f"missing scope doc: {_rel(SCOPE_DOC)}")
    terms_missing = [term for term in REQUIRED_SCOPE_TERMS if term not in scope_text]
    if terms_missing:
        problems.append(f"scope doc missing terms: {', '.join(terms_missing)}")

    why_text = _read(WHY_HTML)
    enforced_count = why_text.count(ENFORCED_CLAIM_MARKER)
    if enforced_count != EXPECTED_ENFORCED_CLAIMS:
        problems.append(
            f"docs/why.html has {enforced_count} '{ENFORCED_CLAIM_MARKER}' claims, "
            f"expected exactly {EXPECTED_ENFORCED_CLAIMS}"
        )
    claim_hashes = _enforced_claim_hashes(why_text)
    hashes_match = claim_hashes == EXPECTED_ENFORCED_CLAIM_HASHES
    if not hashes_match:
        problems.append(
            "docs/why.html enforced-claim semantics changed: "
            f"actual hashes={claim_hashes}, expected={EXPECTED_ENFORCED_CLAIM_HASHES}"
        )

    for anchor in ("test_entry_authority", "scope_shadowing_parity"):
        if anchor not in why_text:
            problems.append(f"docs/why.html no longer cites its test anchor: {anchor}")
    anchors_missing = [_rel(path) for path in CITED_TEST_ANCHORS if not path.is_file()]
    if anchors_missing:
        problems.append(f"cited test anchors missing: {', '.join(anchors_missing)}")

    current_truth_missing: list[str] = []
    for path, snippets in CANONICAL_TRUTH_SNIPPETS.items():
        text = _read(path)
        for snippet in snippets:
            if snippet not in text:
                current_truth_missing.append(f"{_rel(path)}: {snippet}")
    if current_truth_missing:
        problems.append(
            "canonical current-truth snippets missing: "
            + "; ".join(current_truth_missing)
        )

    acceptance = _read_acceptance()
    acceptance_state = str(acceptance.get("state", "missing-or-invalid")) if acceptance else "missing-or-invalid"
    closure = acceptance.get("post_acceptance_closure") if acceptance else None
    closure_state = (
        str(closure.get("state", "missing-or-invalid"))
        if isinstance(closure, dict)
        else "missing-or-invalid"
    )
    condition_states = closure.get("condition_states", {}) if isinstance(closure, dict) else {}
    current_truth = closure.get("current_truth", {}) if isinstance(closure, dict) else {}
    acceptance_ok = bool(
        acceptance
        and acceptance.get("schema") == "garnet.s114_acceptance/v1"
        and acceptance_state == "accepted-scoped"
        and acceptance.get("independence_relabel") is False
        and closure_state == "condition-5-reopened-by-post-acceptance-delta-review"
        and str(condition_states.get("5", "")).startswith("reopened:")
        and current_truth.get("acceptance") == "accepted-scoped by Jon"
        and current_truth.get("independent_verdict")
        == "independently-re-verified-with-fixes"
    )
    if not acceptance_ok:
        problems.append("S114 acceptance/current-closure metadata is missing, stale, or inconsistent")

    forbidden_hits: list[str] = []
    for surface in PUBLIC_SURFACES:
        text = _read(surface)
        for pattern in FORBIDDEN_PATTERNS:
            for match in re.finditer(pattern, text, flags=re.IGNORECASE):
                forbidden_hits.append(f"{_rel(surface)}: '{match.group(0)}'")
    if forbidden_hits:
        problems.append(
            "forbidden universal-enforcement phrasing: " + "; ".join(forbidden_hits)
        )

    stale_truth_hits: list[str] = []
    for surface in CURRENT_TRUTH_SURFACES:
        text = _read(surface)
        for pattern in STALE_TRUTH_PATTERNS:
            for match in re.finditer(pattern, text, flags=re.IGNORECASE):
                stale_truth_hits.append(f"{_rel(surface)}: '{match.group(0)}'")
    if stale_truth_hits:
        problems.append("stale current-truth phrasing: " + "; ".join(stale_truth_hits))

    return CapabilityScopeStatus(
        schema=SCHEMA,
        ok=not problems,
        scope_doc_present=scope_present,
        scope_terms_missing=terms_missing,
        enforced_claim_count=enforced_count,
        enforced_claim_hashes=claim_hashes,
        enforced_claim_hashes_match=hashes_match,
        acceptance_state=acceptance_state,
        post_acceptance_closure_state=closure_state,
        current_truth_missing=current_truth_missing,
        cited_anchors_missing=anchors_missing,
        forbidden_hits=forbidden_hits,
        stale_truth_hits=stale_truth_hits,
        problems=problems,
    )


def render_markdown(status: CapabilityScopeStatus) -> str:
    lines = [
        "# Garnet capability current-truth status",
        "",
        f"_Schema {status.schema}._",
        "",
        f"- scope doc present: **{status.scope_doc_present}**",
        f"- published `enforced:` claims: **{status.enforced_claim_count}/{status.enforced_claim_expected}**",
        f"- enforced claim semantics hash-pinned: **{status.enforced_claim_hashes_match}**",
        f"- S114 acceptance: **{status.acceptance_state}**",
        f"- post-acceptance closure: **{status.post_acceptance_closure_state}**",
        f"- canonical truth snippets missing: {len(status.current_truth_missing)}",
        f"- cited test anchors missing: {len(status.cited_anchors_missing)}",
        f"- forbidden universal-claim hits: {len(status.forbidden_hits)}",
        f"- stale current-truth hits: {len(status.stale_truth_hits)}",
        f"- overall: **{'ok' if status.ok else 'FAIL'}**",
    ]
    lines.extend(f"  - PROBLEM: {problem}" for problem in status.problems)
    lines.append("")
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument("--gate", action="store_true", help="exit 1 unless every claim-scope check passes")
    args = parser.parse_args(list(argv) if argv is not None else None)

    status = read_status()
    print(render_markdown(status) if args.format == "md" else json.dumps(asdict(status), indent=2))
    if args.gate and not status.ok:
        print(f"capability-scope gate FAILED: {len(status.problems)} problem(s)", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
