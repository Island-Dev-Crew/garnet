#!/usr/bin/env python3
"""LLM-suggest v0.2 readiness + Paper VI Exp 1 prep (S69).

Garnet's compiler-as-agent advisory has two tiers: a **rules tier** (S10,
ACTIVE — deterministic, in `garnet check --suggest`) and an **LLM tier**
(provider-backed suggestions, **pending-infra**). This reporter inventories the
rules tier (verifying the shipped rule IDs exist), states the LLM tier's honest
status, and records the Paper VI Experiment 1 prep — without calling any model.

## Honest scope (do not soften)
The LLM tier is **pending-infra**: there is no LLM provider wired in this
environment, and this slice does **not** call one or add a firing advisory. It
ships the readiness/experiment-prep layer; the rules tier is the active baseline.
The Paper VI scorecard is quoted **verbatim** as an honesty anchor.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SUGGEST_RS = ROOT / "garnet-check-v0.3" / "src" / "suggest.rs"

# The S10 rules-tier rule IDs (the active baseline the LLM tier would augment).
RULES_TIER_IDS = [
    "managed-fn-missing-caps",
    "long-parameter-list",
    "empty-function-body",
]

PAPER_VI_SCORECARD = (
    "4 supported, 2 partial (downgraded honestly), 0 refuted, 1 pending-infra"
)


@dataclass
class SuggestReadiness:
    schema: str
    rules_tier_ids: list[str]
    rules_tier_present: bool
    missing_rule_ids: list[str]
    llm_tier_status: str
    exp1_protocol: list[str] = field(default_factory=list)
    paper_vi_scorecard: str = ""
    rules_tier_ready: bool = False


def read_readiness() -> SuggestReadiness:
    src = SUGGEST_RS.read_text(encoding="utf-8") if SUGGEST_RS.is_file() else ""
    missing = [rid for rid in RULES_TIER_IDS if f'"{rid}"' not in src]
    return SuggestReadiness(
        schema="garnet.llm_suggest_readiness/v1",
        rules_tier_ids=RULES_TIER_IDS,
        rules_tier_present=not missing and bool(src),
        missing_rule_ids=missing,
        llm_tier_status=(
            "pending-infra — no LLM provider wired; the rules tier is the active "
            "baseline. (Paper VI Exp 1 budget.)"
        ),
        exp1_protocol=[
            "Hold the rules tier fixed as the deterministic control.",
            "Wire a provider-backed suggester behind the same Suggestion shape.",
            "Measure suggestion precision/recall vs. a curated corpus (the idiomatic "
            "corpus S57 + the 12 domains S48) — no measurement is claimed here.",
            "Report results honestly; downgrade the contribution if unsupported.",
        ],
        paper_vi_scorecard=PAPER_VI_SCORECARD,
        rules_tier_ready=not missing and bool(src),
    )


def render_markdown(r: SuggestReadiness) -> str:
    lines = [
        "# Garnet LLM-suggest readiness (v0.2 / Paper VI Exp 1 prep)",
        "",
        f"_Schema {r.schema}._",
        "",
        "## Rules tier (S10 — ACTIVE)",
        f"- rule IDs: {', '.join(r.rules_tier_ids)}",
        f"- present in `suggest.rs`: {'yes' if r.rules_tier_present else 'NO'}"
        + (f" (missing: {r.missing_rule_ids})" if r.missing_rule_ids else ""),
        "",
        "## LLM tier",
        f"- status: {r.llm_tier_status}",
        "",
        "## Paper VI Experiment 1 — prep protocol",
    ]
    for step in r.exp1_protocol:
        lines.append(f"- {step}")
    lines += [
        "",
        f'**Paper VI scorecard (verbatim): "{r.paper_vi_scorecard}"**',
        "",
        "Honest scope: the LLM tier is pending-infra — no model is called and no "
        "firing advisory is added here; the rules tier is the active baseline.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero if the rules-tier rule IDs are missing (the LLM tier is NOT gated)",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_readiness()
    if args.format == "md":
        print(render_markdown(r))
    else:
        print(json.dumps(asdict(r), indent=2))

    if args.gate and not r.rules_tier_ready:
        print(
            f"llm-suggest-readiness gate FAILED: rules-tier IDs missing: {r.missing_rule_ids} "
            "(the LLM tier is pending-infra and is NOT gated)",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
