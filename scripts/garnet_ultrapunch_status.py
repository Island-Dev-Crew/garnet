#!/usr/bin/env python3
"""Ultrapunch evidence-record status (S104, Stage U closeout).

S104 packages the ultrapunch (S102 loop + S103 demo) into a reproducible,
citeable evidence record: `C_Language_Specification/GARNET_ULTRAPUNCH.md` (the
claim, the reproduce commands, the 4 trust artifacts, the **two-level symmetry**,
and an explicit "What we refuse to claim") + `scripts/reproduce_ultrapunch.sh`
(runs accept + reject end-to-end). The reproduction itself is pinned by
`garnet-cli/tests/ultrapunch_demo.rs` under `cargo test`.

This static gate asserts the record, its honesty anchors, and the two-level
symmetry stay in place.

## Honest scope (do not soften)
The record claims acceptance on capability + depth evidence ONLY (`@caps` +
`@max_depth` enforced). `@bounded`/memory/time/`@mailbox`/OS-sandbox remain
declared-not-enforced; the agent is simulated/scripted, not a live LLM (S94).
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "C_Language_Specification" / "GARNET_ULTRAPUNCH.md"
SCRIPT = ROOT / "scripts" / "reproduce_ultrapunch.sh"
DEMO_TEST = ROOT / "garnet-cli" / "tests" / "ultrapunch_demo.rs"
DEMO_DIR = ROOT / "garnet-cli" / "tests" / "fixtures" / "ultrapunch"


@dataclass
class UltrapunchStatus:
    schema: str
    record_present: bool
    four_artifacts_named: bool
    two_level_symmetry_explicit: bool
    honesty_anchor_present: bool
    refusal_documented: bool
    reproduce_script_present: bool
    demo_pinned: bool
    ok: bool = False


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def read_status() -> UltrapunchStatus:
    doc = _read(DOC)
    script = _read(SCRIPT)
    demo_t = _read(DEMO_TEST)
    record_present = bool(doc)
    four = (
        "capability_manifest.json" in doc
        and "diff_caps.txt" in doc
        and "seal.json" in doc
        and "transparency_log.jsonl" in doc
    )
    two_level = "Two-level symmetry" in doc and "dogfoods the exact acceptance" in doc
    honesty = (
        "accepted on capability + depth evidence" in doc
        and "declared-not-enforced" in doc
        and "What we refuse to claim" in doc
    )
    refusal = "refused, never sealed" in doc or "never sealed" in doc
    script_present = (
        bool(script)
        and "agent-loop" in script
        and "reject_widen" in script
        and "caps-log --verify" in script
    )
    demo_pinned = (
        "accept_records_the_four_trust_artifacts" in demo_t
        and (DEMO_DIR / "reject_widen.garnet").is_file()
    )
    ok = (
        record_present
        and four
        and two_level
        and honesty
        and refusal
        and script_present
        and demo_pinned
    )
    return UltrapunchStatus(
        schema="garnet.ultrapunch/v1",
        record_present=record_present,
        four_artifacts_named=four,
        two_level_symmetry_explicit=two_level,
        honesty_anchor_present=honesty,
        refusal_documented=refusal,
        reproduce_script_present=script_present,
        demo_pinned=demo_pinned,
        ok=ok,
    )


def render_markdown(r: UltrapunchStatus) -> str:
    return "\n".join([
        "# Garnet ultrapunch evidence-record status (S104)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- evidence record present (`GARNET_ULTRAPUNCH.md`): {'yes' if r.record_present else 'NO'}",
        f"- the 4 trust artifacts named: {'yes' if r.four_artifacts_named else 'NO'}",
        f"- two-level symmetry explicit (Garnet dogfoods its own acceptance): "
        f"{'yes' if r.two_level_symmetry_explicit else 'NO'}",
        f"- refusal (the punch) documented: {'yes' if r.refusal_documented else 'NO'}",
        f"- honesty anchor + \"What we refuse to claim\": "
        f"{'yes' if r.honesty_anchor_present else 'NO'}",
        f"- reproduce script + demo test pinned: "
        f"{'yes' if r.reproduce_script_present and r.demo_pinned else 'NO'}",
        "",
        "Accepted on capability + depth evidence ONLY (`@caps` + `@max_depth`); "
        "`@bounded`/memory/time/`@mailbox`/OS-sandbox remain declared-not-enforced. "
        "The agent is simulated/scripted, not a live LLM (S94).",
        "",
    ])


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless the ultrapunch evidence record, the 4 named "
        "artifacts, the two-level symmetry, the honesty anchors, the documented "
        "refusal, the reproduce script, and the pinned demo are all present.",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    print(render_markdown(r) if args.format == "md" else json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(f"ultrapunch gate FAILED: {asdict(r)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
