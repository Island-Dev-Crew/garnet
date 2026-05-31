#!/usr/bin/env python3
"""Formal-verification feasibility study status (S75).

Assesses whether Garnet can offer a *provable* (not merely annotated) story for
termination/bounds and `@caps` authority soundness over a safe subset — the
eBPF-verifier path the trajectory research lists as a multi-year bet.
`C_Language_Specification/GARNET_FORMAL_VERIFICATION_FEASIBILITY.md` is the study.

This reporter is a static anti-overclaim gate: it verifies the study exists,
states its verdict and honest scope, and that the foundation it builds on is real
(`explosive.rs`'s undecidability stance is in tree). It does NOT run or imply any
verification.

## Honest scope (do not soften)
A feasibility study only — no verifier, no termination proof, no SMT or
proof-assistant integration, no `@caps`-soundness theorem ships. Assessment, not
implemented behavior.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
STUDY = ROOT / "C_Language_Specification" / "GARNET_FORMAL_VERIFICATION_FEASIBILITY.md"
EXPLOSIVE = ROOT / "garnet-check-v0.3" / "src" / "explosive.rs"
SAFE_SUBSET = ROOT / "C_Language_Specification" / "GARNET_SAFE_SUBSET.md"

STUDY_ANCHORS = [
    "feasibility study",
    "eBPF-verifier path",
    "linear-capability mode",
    "halting problem",
    "not feasible",  # whole-language verification verdict
]


@dataclass
class FeasibilityStatus:
    schema: str
    study_present: bool
    missing_anchors: list[str]
    explosive_foundation_present: bool
    safe_subset_spec_present: bool
    ok: bool = False
    groundings: list[str] = field(default_factory=list)


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def read_status() -> FeasibilityStatus:
    study = _read(STUDY)
    explosive = _read(EXPLOSIVE)
    missing = [a for a in STUDY_ANCHORS if a not in study]
    # The study cites explosive.rs's undecidability stance; ground it.
    explosive_found = "Static termination is undecidable" in explosive
    safe_subset_found = SAFE_SUBSET.is_file()
    study_present = bool(study) and not missing
    ok = study_present and explosive_found and safe_subset_found
    return FeasibilityStatus(
        schema="garnet.formal_verification_feasibility/v1",
        study_present=study_present,
        missing_anchors=missing,
        explosive_foundation_present=explosive_found,
        safe_subset_spec_present=safe_subset_found,
        ok=ok,
        groundings=[
            f"explosive.rs undecidability stance present: {explosive_found}",
            f"safe-subset spec (S74) present: {safe_subset_found}",
        ],
    )


def render_markdown(r: FeasibilityStatus) -> str:
    lines = [
        "# Garnet formal-verification feasibility status (S75)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- study present + anchored: {'yes' if r.study_present else 'NO'}"
        + (f" (missing: {r.missing_anchors})" if r.missing_anchors else ""),
        f"- grounded: `explosive.rs` undecidability stance present: "
        f"{'yes' if r.explosive_foundation_present else 'NO'}",
        f"- grounded: safe-subset spec (S74) present: "
        f"{'yes' if r.safe_subset_spec_present else 'NO'}",
        "",
        "Verdict: a verified bounded-loop checker for the safe subset (eBPF-style) "
        "is the feasible first provable increment; `@caps` soundness is feasible "
        "only atop the S74 linear-capability mode; whole-language verification is "
        "not feasible. Honest scope: a feasibility STUDY — no verifier, no proof, "
        "no theorem ships.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero if the study is missing/unanchored or its cited "
        "foundation is absent. No verification is run or implied.",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    if args.format == "md":
        print(render_markdown(r))
    else:
        print(json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(
            "formal-verification-feasibility gate FAILED: "
            f"study_present={r.study_present} missing={r.missing_anchors} "
            f"explosive={r.explosive_foundation_present} safe_subset={r.safe_subset_spec_present}",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
