#!/usr/bin/env python3
"""Governance + RFC process status (S78).

Formalizes how changes land through a documented **RFC + edition process** over
ad-hoc BDFL fiat, and records the intent to donate Garnet's capability-manifest
format to a neutral body (OWASP / Linux Foundation) as RFC-0001 — while staying
honest that Garnet is a single-maintainer research-grade project.

This reporter is a static anti-overclaim gate. It verifies:
  - `GOVERNANCE.md` exists and states the honest single-maintainer status;
  - the RFC process (`rfcs/README.md` + `0000-template.md`) exists;
  - RFC-0001 exists, references the real capability-manifest standard, and marks
    the OWASP/LF donation as intent/draft (NOT an accepted standard).

## Honest scope (do not soften)
Single-maintainer governance for a research-grade prototype — no steering
committee, no foundation, no adopted standard. The donation is intent + a draft.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GOVERNANCE = ROOT / "GOVERNANCE.md"
RFC_README = ROOT / "rfcs" / "README.md"
RFC_TEMPLATE = ROOT / "rfcs" / "0000-template.md"
RFC_0001 = ROOT / "rfcs" / "0001-capability-manifest-standard.md"
CAP_STANDARD = ROOT / "C_Language_Specification" / "GARNET_CAPABILITY_TRANSPARENCY.md"


@dataclass
class GovernanceStatus:
    schema: str
    governance_present: bool
    governance_honest: bool
    rfc_process_present: bool
    rfc0001_present: bool
    rfc0001_references_standard: bool
    rfc0001_marks_intent_not_accepted: bool
    ok: bool = False
    notes: list[str] = field(default_factory=list)


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def read_status() -> GovernanceStatus:
    gov = _read(GOVERNANCE)
    rfc0001 = _read(RFC_0001)
    governance_present = bool(gov)
    # Honest status: must disclaim a steering committee / institutional permanence.
    governance_honest = "single-maintainer governance" in gov and "not" in gov.lower()
    rfc_process = RFC_README.is_file() and RFC_TEMPLATE.is_file()
    rfc0001_present = bool(rfc0001)
    refs_standard = "GARNET_CAPABILITY_TRANSPARENCY.md" in rfc0001 and CAP_STANDARD.is_file()
    # Donation must be marked intent/draft, NOT an accepted standard. Anchor on
    # the clean (non-bolded) Summary phrase so markdown emphasis can't break it.
    marks_intent = (
        "Status:** Draft" in rfc0001
        and "no external body has adopted it" in rfc0001
    )
    ok = (
        governance_present
        and governance_honest
        and rfc_process
        and rfc0001_present
        and refs_standard
        and marks_intent
    )
    return GovernanceStatus(
        schema="garnet.governance_status/v1",
        governance_present=governance_present,
        governance_honest=governance_honest,
        rfc_process_present=rfc_process,
        rfc0001_present=rfc0001_present,
        rfc0001_references_standard=refs_standard,
        rfc0001_marks_intent_not_accepted=marks_intent,
        ok=ok,
    )


def render_markdown(r: GovernanceStatus) -> str:
    lines = [
        "# Garnet governance + RFC status (S78)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- `GOVERNANCE.md` present + honest single-maintainer status: "
        f"{'yes' if r.governance_present and r.governance_honest else 'NO'}",
        f"- RFC process (README + template): {'yes' if r.rfc_process_present else 'NO'}",
        f"- RFC-0001 (cap-manifest standard) present: {'yes' if r.rfc0001_present else 'NO'}",
        f"- RFC-0001 references the real capability-manifest standard: "
        f"{'yes' if r.rfc0001_references_standard else 'NO'}",
        f"- RFC-0001 marks the OWASP/LF donation as intent/draft (not accepted): "
        f"{'yes' if r.rfc0001_marks_intent_not_accepted else 'NO'}",
        "",
        "Honest scope: single-maintainer governance for a research-grade prototype "
        "— no steering committee, no foundation, no adopted standard. The "
        "capability-manifest donation is intent + a draft (RFC-0001).",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless GOVERNANCE.md + the RFC process + RFC-0001 are "
        "present and RFC-0001 marks the donation as intent/draft (not accepted).",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    if args.format == "md":
        print(render_markdown(r))
    else:
        print(json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(f"governance gate FAILED: {asdict(r)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
