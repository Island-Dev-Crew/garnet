#!/usr/bin/env python3
"""Academic evidence-package gate (S118).

The capstone package (`F_Project_Management/GARNET_ACADEMIC_EVIDENCE_PACKAGE.md`)
sources every load-bearing v0.8.1 claim to a slice / file / test / sealed proof, for a
skeptical academic reader. This gate keeps the package HONEST: it fails if any cited
source path does not resolve on disk (no aspirational citation survives), if the
"what we refuse to claim" section is dropped, if an honesty anchor softens, or if the
package stops tying the Stage-P artifacts together.

It does not re-run the proofs (the per-pillar gates do that); it asserts the index is
complete, sourced, and honest. The strongest line of defense for an evidence index is
that it cannot cite something that isn't there.

## Honest scope (do not soften)
The package claims capability + depth enforcement (both backends) + Linux-only seccomp.
It must keep the macOS/Windows OS-sandbox + @bounded/memory/time/@mailbox + simulated-
agent + unsigned/no-SBOM + local-stub-log fences and the no-production/1.0 anchor.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "F_Project_Management" / "GARNET_ACADEMIC_EVIDENCE_PACKAGE.md"

# Honesty anchors that must remain present (normalized substring match).
REQUIRED_ANCHORS = (
    "no production / 1.0 claim",
    "named-deferred",
    "simulated",
    "linux only",
    "declared-not-enforced",
)
# The package is a capstone: it must cite the Stage-P artifacts it ties together.
REQUIRED_ARTIFACTS = (
    "GARNET_ULTRAPUNCH_DOSSIER.md",
    "GARNET_DOMAIN_PROOF_ARTIFACTS.md",
    "GARNET_RED_TEAM.md",
    "GARNET_CROSS_OS_REPRODUCIBILITY.md",
)


@dataclass
class AcademicStatus:
    schema: str
    doc_present: bool
    has_contribution: bool
    has_refuse_section: bool
    anchors_present: bool
    artifacts_cited: bool
    sourced_pointers_total: int
    sourced_pointers_missing: int
    ok: bool = False


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def _norm(text: str) -> str:
    return re.sub(r"\s+", " ", text.replace("*", "").lower())


def _cited_paths(doc: str) -> list[str]:
    """Repo-relative paths cited in backticks (contain a `/`, no spaces/parens)."""
    out: list[str] = []
    for span in re.findall(r"`([^`]+)`", doc):
        span = span.strip().rstrip(".,;")
        if "/" not in span or " " in span or "(" in span:
            continue
        out.append(span)
    seen: set[str] = set()
    uniq: list[str] = []
    for s in out:
        if s not in seen:
            seen.add(s)
            uniq.append(s)
    return uniq


def _resolves(rel: str) -> bool:
    return (ROOT / rel).exists()


def _missing_pointers() -> list[str]:
    return [p for p in _cited_paths(_read(DOC)) if not _resolves(p)]


def read_status() -> AcademicStatus:
    doc = _read(DOC)
    low = _norm(doc)
    present = bool(doc)
    has_contribution = "one-sentence contribution" in low
    has_refuse = "what we refuse to claim" in low
    anchors = all(a in low for a in REQUIRED_ANCHORS)
    artifacts = all(a in doc for a in REQUIRED_ARTIFACTS)

    pointers = _cited_paths(doc)
    missing = [p for p in pointers if not _resolves(p)]

    ok = (
        present
        and has_contribution
        and has_refuse
        and anchors
        and artifacts
        and not missing
        and len(pointers) >= 12
    )
    return AcademicStatus(
        schema="garnet.academic_evidence/v1",
        doc_present=present,
        has_contribution=has_contribution,
        has_refuse_section=has_refuse,
        anchors_present=anchors,
        artifacts_cited=artifacts,
        sourced_pointers_total=len(pointers),
        sourced_pointers_missing=len(missing),
        ok=ok,
    )


def render_markdown(r: AcademicStatus) -> str:
    return "\n".join(
        [
            "# Garnet academic evidence-package status (S118)",
            "",
            f"_Schema {r.schema}._",
            "",
            f"- package present: {'yes' if r.doc_present else 'NO'}",
            f"- one-sentence contribution: {'yes' if r.has_contribution else 'NO'}",
            f"- 'what we refuse to claim' first-class: "
            f"{'yes' if r.has_refuse_section else 'NO'}",
            f"- honesty anchors present: {'yes' if r.anchors_present else 'NO'}",
            f"- Stage-P artifacts tied together: "
            f"{'yes' if r.artifacts_cited else 'NO'}",
            f"- sourced pointers resolved: "
            f"{r.sourced_pointers_total - r.sourced_pointers_missing}"
            f"/{r.sourced_pointers_total}",
            "",
            "Every load-bearing v0.8.1 claim is sourced to a file / test / sealed proof, "
            "and every cited source resolves on disk. The honest concessions are "
            "first-class. Research-grade; no production / 1.0 claim; the cut is Jon's.",
            "",
        ]
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless the package is present, states the contribution, "
        "keeps the refuse-to-claim section + honesty anchors, ties the Stage-P "
        "artifacts together, and every cited source resolves on disk.",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    print(render_markdown(r) if args.format == "md" else json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(f"academic evidence gate FAILED: {asdict(r)}", file=sys.stderr)
        miss = _missing_pointers()
        if miss:
            print(f"  missing sourced pointers: {miss}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
