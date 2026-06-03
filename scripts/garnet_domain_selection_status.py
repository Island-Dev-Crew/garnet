#!/usr/bin/env python3
"""Ultrapunch domain-selection status (S105 — unblocks Stage X).

S105 selects the demonstrator domains for the v0.8.1 real-world-proof finale and,
for each, the specific trust-artifact delta a non-Garnet build cannot produce —
honestly grounded in what Garnet actually produces (the 4 artifacts + the diff-caps
refusal + the enforced @caps/@max_depth trap). Each domain ships a per-OS Stage-X
proof command. This static gate asserts the selection, its honesty filter, and the
enforced-only scope stay in place.

## Honest scope (do not soften)
All domains rest ONLY on the enforced ceilings (`@caps` + `@max_depth`).
`@bounded`/memory/time/`@mailbox`/OS-sandbox remain declared-not-enforced. The
novelty is the INTEGRATION (a sealed, autonomous capability-diff gate), not any
single pillar. Domain 6 is a static `mcp-caps` report (NOT a `diff-caps` hard-fail —
`diff-caps` does not accept `.mcpcaps`).
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "F_Project_Management" / "GARNET_DOMAIN_SELECTION.md"


@dataclass
class DomainSelectionStatus:
    schema: str
    doc_present: bool
    domain_count: int
    enough_domains: bool
    enforced_only: bool
    rejected_overclaims_present: bool
    mcp_overclaim_corrected: bool
    honesty_anchor_present: bool
    stage_x_proofs_present: bool
    ok: bool = False


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def read_status() -> DomainSelectionStatus:
    doc = _read(DOC)
    present = bool(doc)
    count = len(re.findall(r"(?m)^### \d+\. ", doc))
    enough = 5 <= count <= 10
    # Enforced ceilings only: the doc must name @caps + @max_depth as enforced and
    # explicitly keep @bounded/memory/time/mailbox/OS-sandbox declared-not-enforced.
    enforced_only = (
        "declared-not-enforced" in doc
        and "not runtime fuel metering" in doc
        and "named, never faked" in doc
    )
    rejected = "Rejected overclaims" in doc
    # The diff-caps-on-.mcpcaps overclaim must be corrected (caught empirically).
    mcp_corrected = (
        "does **NOT** accept" in doc and ".mcpcaps" in doc
    ) or "diff-caps` does **NOT**" in doc
    honesty = (
        "accepted on capability + depth evidence" in doc
        and 'never "fully bounded"' in doc
    )
    stage_x = "Stage X proof" in doc and "cross-OS-complete only when all" in doc
    ok = (
        present
        and enough
        and enforced_only
        and rejected
        and mcp_corrected
        and honesty
        and stage_x
    )
    return DomainSelectionStatus(
        schema="garnet.domain_selection/v1",
        doc_present=present,
        domain_count=count,
        enough_domains=enough,
        enforced_only=enforced_only,
        rejected_overclaims_present=rejected,
        mcp_overclaim_corrected=mcp_corrected,
        honesty_anchor_present=honesty,
        stage_x_proofs_present=stage_x,
        ok=ok,
    )


def render_markdown(r: DomainSelectionStatus) -> str:
    return "\n".join([
        "# Garnet ultrapunch domain-selection status (S105)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- selection doc present: {'yes' if r.doc_present else 'NO'}",
        f"- demonstrator domains: {r.domain_count} ({'ok' if r.enough_domains else 'OUT OF RANGE'})",
        f"- enforced-only scope (@caps + @max_depth; rest declared-not-enforced): "
        f"{'yes' if r.enforced_only else 'NO'}",
        f"- rejected-overclaims (honesty filter) present: "
        f"{'yes' if r.rejected_overclaims_present else 'NO'}",
        f"- mcp `diff-caps` overclaim corrected: {'yes' if r.mcp_overclaim_corrected else 'NO'}",
        f"- honesty anchor (\"accepted on capability + depth evidence\"): "
        f"{'yes' if r.honesty_anchor_present else 'NO'}",
        f"- per-OS Stage-X proofs + cross-OS-complete rule: "
        f"{'yes' if r.stage_x_proofs_present else 'NO'}",
        "",
        "All domains rest on the enforced ceilings (`@caps` + `@max_depth`) only; the "
        "novelty is the integration, not any single pillar. v0.8.1 is research-grade.",
        "",
    ])


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless the selection doc has 5-10 domains, the honesty "
        "filter (rejected overclaims incl. the corrected mcp diff-caps claim), the "
        "enforced-only scope, the honesty anchor, and per-OS Stage-X proofs.",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    print(render_markdown(r) if args.format == "md" else json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(f"domain-selection gate FAILED: {asdict(r)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
