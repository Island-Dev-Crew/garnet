#!/usr/bin/env python3
"""Post-tag release-truth status (S83 / WIN-S80-002).

The Windows audit flagged a split truth: `v0.8.0` is cut (Jon, `cc165e8`,
2026-05-31), yet `GARNET_v0_8_0_CUT.md` still read "READY TO CUT (pending Jon)",
the CHANGELOG header read "v0.8.0 in flight", and `.dogfood/goal.json` kept `s80`
pending. S83 reconciles this in one place; this gate keeps the two truths
coexisting so they cannot drift apart again:

  1. **The tag was cut by Jon** — recorded in `GARNET_v0_8_0_CUT.md` (post-cut
     note) and `.dogfood/goal.json` (`s80` merged + a `cut_record`).
  2. **The S80 PR produced cut-readiness evidence only** — the gate is advisory;
     the cut was a separate human act. This sentence must remain alongside (1).

## Honest scope (do not soften)
Pure docs/ledger reconciliation. No tag is cut here (the tag already exists); the
full Keep-a-Changelog restructure remains a deferred decision for Jon.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CUT_DOC = ROOT / "F_Project_Management" / "GARNET_v0_8_0_CUT.md"
CHANGELOG = ROOT / "CHANGELOG.md"
# S91 re-initializes the active goal ledger for S91-S110. The finished v0.8.0
# release-truth gates read the archived S31-S80 ledger so their evidence remains
# reproducible after the active ledger advances to v0.8.1 work.
LEDGER = ROOT / ".dogfood" / "v0_8_goal.json"
EXPECTED_COMMIT = "cc165e8eaa8b2118f5da8d5e7bb80baa93a02fc5"


@dataclass
class ReleaseTruthStatus:
    schema: str
    cut_doc_records_tag_cut: bool
    cut_doc_keeps_readiness_only_truth: bool
    changelog_records_cut: bool
    ledger_s80_merged_with_cut_record: bool
    ok: bool = False


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def read_status() -> ReleaseTruthStatus:
    cut = _read(CUT_DOC)
    changelog = _read(CHANGELOG)
    ledger = json.loads(LEDGER.read_text(encoding="utf-8")) if LEDGER.is_file() else {"slices": []}
    s80 = next((x for x in ledger.get("slices", []) if x.get("id") == "s80"), {})

    # Truth 1: the tag was cut by Jon (doc + changelog).
    cut_records = "`v0.8.0` IS cut" in cut and "Jon Isaac tagged" in cut
    changelog_cut = "`v0.8.0` is cut" in changelog and "S83" in changelog
    # Truth 2: the S80 PR produced cut-readiness evidence only (must coexist).
    keeps_evidence_only = "cut-readiness *evidence*" in cut or "cut-readiness evidence only" in changelog
    # Ledger: s80 merged + cut_record pinning the tag commit.
    cr = s80.get("cut_record") or {}
    ledger_ok = (
        s80.get("status") == "merged"
        and (s80.get("merge_confidence") or 0) >= 5
        and cr.get("tag") == "v0.8.0"
        and cr.get("commit") == EXPECTED_COMMIT
    )
    ok = cut_records and changelog_cut and keeps_evidence_only and ledger_ok
    return ReleaseTruthStatus(
        schema="garnet.release_truth/v1",
        cut_doc_records_tag_cut=cut_records,
        cut_doc_keeps_readiness_only_truth=keeps_evidence_only,
        changelog_records_cut=changelog_cut,
        ledger_s80_merged_with_cut_record=ledger_ok,
        ok=ok,
    )


def render_markdown(r: ReleaseTruthStatus) -> str:
    return "\n".join([
        "# Garnet post-tag release-truth status (S83)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- cut doc records the tag was cut by Jon: {'yes' if r.cut_doc_records_tag_cut else 'NO'}",
        f"- the 'S80 PR = readiness evidence only' truth coexists: "
        f"{'yes' if r.cut_doc_keeps_readiness_only_truth else 'NO'}",
        f"- CHANGELOG records the cut (→ v0.8.1 runway): {'yes' if r.changelog_records_cut else 'NO'}",
        f"- ledger `s80` merged + `cut_record` pinning cc165e8: "
        f"{'yes' if r.ledger_s80_merged_with_cut_record else 'NO'}",
        "",
        "Both truths in one place: v0.8.0 was cut by Jon AND the S80 PR produced "
        "cut-readiness evidence only. Pure docs/ledger reconciliation; no tag cut here.",
        "",
    ])


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless the cut-by-Jon truth and the S80-readiness-only "
        "truth coexist (doc + CHANGELOG + ledger cut_record).",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    print(render_markdown(r) if args.format == "md" else json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(f"release-truth gate FAILED: {asdict(r)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
