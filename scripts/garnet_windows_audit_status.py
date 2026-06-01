#!/usr/bin/env python3
"""Windows audit burn-down status (P0, v0.8.1 runway).

The Codex Windows audit of S1–S80 (HEAD cc165e8) recorded 14 open `WIN-*`
findings. `F_Project_Management/WINDOWS_AUDIT_S1_S80.md` is the tracked burn-down
ledger; the machine ledgers (`.dogfood/windows-core-audit.json`,
`.dogfood/windows-audit-goal.json`) are committed beside it.

This gate asserts the burn-down is honestly tracked: every open finding appears in
the doc **with an owning slice**, and the committed machine ledgers pin HEAD
`cc165e8`. It does not run any Windows command (planning lane is Mac); Windows
proofs are recorded back into the doc by the Windows lane.

## Honest scope (do not soften)
A tracking gate over imported audit evidence. It does not re-run the audit or claim
any finding fixed — it only enforces that each open finding has an owning slice.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "F_Project_Management" / "WINDOWS_AUDIT_S1_S80.md"
CORE = ROOT / ".dogfood" / "windows-core-audit.json"
GOAL = ROOT / ".dogfood" / "windows-audit-goal.json"
EXPECTED_HEAD = "cc165e8eaa8b2118f5da8d5e7bb80baa93a02fc5"

# The 14 open findings the audit recorded (the canonical set).
OPEN_FINDINGS = [
    "WIN-S33-001", "WIN-S36-001", "WIN-S37-001", "WIN-S46-001",
    "WIN-S38-001", "WIN-S80-002", "WIN-S71-001", "WIN-S73-001",
    "WIN-S80-001", "WIN-S6-001", "WIN-S31-001", "WIN-S31-002",
    "WIN-S38-002", "WIN-S39-001",
]
RESOLVED = ["WIN-S70-001"]


@dataclass
class AuditStatus:
    schema: str
    doc_present: bool
    core_ledger_present: bool
    goal_ledger_present: bool
    head_pinned: bool
    findings_without_owner: list[str] = field(default_factory=list)
    ok: bool = False


def _owning_slice(doc: str, finding: str) -> str | None:
    # Find the finding's table row and extract its `**SNN**` owning slice.
    for line in doc.splitlines():
        if finding in line:
            m = re.search(r"\*\*S\d+\*\*", line)
            if m:
                return m.group(0)
    return None


def read_status() -> AuditStatus:
    doc = DOC.read_text(encoding="utf-8") if DOC.is_file() else ""
    core = json.loads(CORE.read_text(encoding="utf-8")) if CORE.is_file() else {}
    goal = json.loads(GOAL.read_text(encoding="utf-8")) if GOAL.is_file() else {}
    head_pinned = core.get("head") == EXPECTED_HEAD and goal.get("head") == EXPECTED_HEAD
    without_owner = [f for f in OPEN_FINDINGS if _owning_slice(doc, f) is None]
    ok = (
        bool(doc)
        and CORE.is_file()
        and GOAL.is_file()
        and head_pinned
        and not without_owner
    )
    return AuditStatus(
        schema="garnet.windows_audit_status/v1",
        doc_present=bool(doc),
        core_ledger_present=CORE.is_file(),
        goal_ledger_present=GOAL.is_file(),
        head_pinned=head_pinned,
        findings_without_owner=without_owner,
        ok=ok,
    )


def render_markdown(r: AuditStatus) -> str:
    return "\n".join([
        "# Garnet Windows audit burn-down status (P0)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- tracked doc `WINDOWS_AUDIT_S1_S80.md` present: {'yes' if r.doc_present else 'NO'}",
        f"- machine ledgers committed: core={r.core_ledger_present} goal={r.goal_ledger_present}",
        f"- ledgers pin HEAD cc165e8: {'yes' if r.head_pinned else 'NO'}",
        f"- open findings ({len(OPEN_FINDINGS)}) all have an owning slice: "
        + ("yes" if not r.findings_without_owner else f"NO {r.findings_without_owner}"),
        "",
        "A tracking gate over imported audit evidence — it does not re-run the audit "
        "or claim any finding fixed. Windows proofs are recorded by the Windows lane.",
        "",
    ])


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless every open WIN-* finding has an owning slice and "
        "the committed machine ledgers pin HEAD cc165e8.",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    print(render_markdown(r) if args.format == "md" else json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(f"windows-audit gate FAILED: {asdict(r)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
