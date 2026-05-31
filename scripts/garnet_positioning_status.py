#!/usr/bin/env python3
"""Website/deck positioning reframe status (S79).

Reframes Garnet's public messaging to lead with the **integration + agent-code
thesis** (not pillar-by-pillar novelty), make **diff-caps** the headline, and
**concede precedent honestly** — per the trajectory research. The canonical
messaging is `F_Project_Management/GARNET_POSITIONING.md`; the landing page
(`docs/index.html`) carries a matching reframed section.

This reporter is a static anti-drift gate: it verifies both the positioning doc
and the landing page carry (a) the integration thesis, (b) the diff-caps headline,
and (c) the precedent concession — so the messaging can't silently revert to
pillar-first marketing.

## Honest scope (do not soften)
A positioning claim about novelty and fit, NOT a production-readiness or 1.0
claim. Garnet remains a research-grade prototype (v0.x).
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
POSITIONING = ROOT / "F_Project_Management" / "GARNET_POSITIONING.md"
LANDING = ROOT / "docs" / "index.html"

# Each surface must carry: integration thesis, diff-caps headline, precedent concession.
REQUIRED_THEMES = {
    "integration_thesis": ("the integration, not the parts", "the integration, not the parts"),
    "diff_caps_headline": ("diff-caps", "diff-caps"),
    "precedent_concession": ("well-precedented", "well-precedented"),
    "agent_code": ("agent-authored code", "agent-authored code"),
}


@dataclass
class PositioningStatus:
    schema: str
    positioning_doc_present: bool
    landing_present: bool
    doc_missing_themes: list[str]
    landing_missing_themes: list[str]
    ok: bool = False
    notes: list[str] = field(default_factory=list)


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def read_status() -> PositioningStatus:
    doc = _read(POSITIONING)
    landing = _read(LANDING)
    doc_missing = [k for k, (d, _l) in REQUIRED_THEMES.items() if d not in doc]
    landing_missing = [k for k, (_d, l) in REQUIRED_THEMES.items() if l not in landing]
    doc_present = bool(doc)
    landing_present = bool(landing)
    ok = doc_present and landing_present and not doc_missing and not landing_missing
    return PositioningStatus(
        schema="garnet.positioning_status/v1",
        positioning_doc_present=doc_present,
        landing_present=landing_present,
        doc_missing_themes=doc_missing,
        landing_missing_themes=landing_missing,
        ok=ok,
    )


def render_markdown(r: PositioningStatus) -> str:
    lines = [
        "# Garnet positioning reframe status (S79)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- positioning doc present: {'yes' if r.positioning_doc_present else 'NO'}"
        + (f" (missing themes: {r.doc_missing_themes})" if r.doc_missing_themes else ""),
        f"- landing page present: {'yes' if r.landing_present else 'NO'}"
        + (f" (missing themes: {r.landing_missing_themes})" if r.landing_missing_themes else ""),
        "",
        "Themes enforced on both surfaces: the integration thesis (integration over "
        "pillars), the diff-caps headline, the honest precedent concession "
        "(well-precedented), and the agent-authored-code target. Honest scope: a "
        "positioning claim about novelty and fit, NOT a production/1.0 claim.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless the positioning doc AND the landing page both "
        "carry the integration thesis + diff-caps headline + precedent concession.",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    if args.format == "md":
        print(render_markdown(r))
    else:
        print(json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(
            "positioning gate FAILED: "
            f"doc={r.positioning_doc_present} landing={r.landing_present} "
            f"doc_missing={r.doc_missing_themes} landing_missing={r.landing_missing_themes}",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
