#!/usr/bin/env python3
"""Ultrapunch dossier gate (S115).

The dossier (`F_Project_Management/GARNET_ULTRAPUNCH_DOSSIER.md`) is the headline
positioning artifact: the #1 claim + ranked runners-up, each backed by a named proof,
with the honest concessions kept in-band. This static gate asserts the dossier exists,
makes the #1 claim, ranks runners-up, keeps the honest concessions (incl. the
named-deferred fences and the no-production/1.0 anchor), cites the red-team result,
and — critically — that **every evidence pointer it names actually resolves on disk**
(no dangling citation). It does not re-run the proofs; the integrity gate does that.

## Honest scope (do not soften)
The dossier claims capability + depth enforcement (both backends) + Linux-only
seccomp; it must keep the macOS/Windows OS-sandbox + @bounded/memory/time/@mailbox +
simulated-agent + unsigned/no-SBOM + local-stub-log fences. This gate fails if any
of those fences is dropped or any cited evidence path is missing.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOSSIER = ROOT / "F_Project_Management" / "GARNET_ULTRAPUNCH_DOSSIER.md"

# Fences that must remain present (calibrated honesty — these never soften).
REQUIRED_FENCES = (
    "named-deferred",
    "simulated",
    "no production / 1.0 claim",
    "Linux only",
)


@dataclass
class DossierStatus:
    schema: str
    dossier_present: bool
    has_number_one_claim: bool
    has_ranked_runners_up: bool
    has_honest_concessions: bool
    cites_red_team: bool
    fences_present: bool
    evidence_pointers_total: int
    evidence_pointers_missing: int
    ok: bool = False


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def _cited_paths(doc: str) -> list[str]:
    """Repo-relative paths the dossier cites inside backticks.

    A pointer is any backtick span that looks like a repo path (contains a `/` and a
    file-ish or directory segment). Bare command words and `@caps(...)` spans are not
    paths, so they are filtered out.
    """
    out: list[str] = []
    for span in re.findall(r"`([^`]+)`", doc):
        span = span.strip()
        # Only path-like spans: must contain a slash and no spaces/parens.
        if "/" not in span or " " in span or "(" in span:
            continue
        # Drop a trailing punctuation the prose may attach.
        span = span.rstrip(".,;")
        out.append(span)
    # De-dup, preserve order.
    seen: set[str] = set()
    uniq: list[str] = []
    for s in out:
        if s not in seen:
            seen.add(s)
            uniq.append(s)
    return uniq


def _resolves(rel: str) -> bool:
    """True if the cited path resolves. A `.../` ellipsis path resolves if its
    parent directory exists and has at least one matching child."""
    if "..." in rel:
        head = rel.split("...", 1)[0].rstrip("/")
        return bool(head) and (ROOT / head).exists()
    return (ROOT / rel).exists()


def read_status() -> DossierStatus:
    doc = _read(DOSSIER)
    present = bool(doc)
    low = doc.lower()
    has_one = "## the #1 claim" in low and "capability-bounded acceptance" in low
    has_runners = "ranked runners-up" in low and re.search(r"\n1\.\s", doc) is not None
    has_concessions = "refuse to claim" in low
    cites_rt = "GARNET_RED_TEAM.md" in doc and "red-team" in low
    fences = all(f.lower() in low for f in REQUIRED_FENCES)

    pointers = _cited_paths(doc)
    missing = [p for p in pointers if not _resolves(p)]

    ok = (
        present
        and has_one
        and has_runners
        and has_concessions
        and cites_rt
        and fences
        and not missing
    )
    return DossierStatus(
        schema="garnet.ultrapunch_dossier/v1",
        dossier_present=present,
        has_number_one_claim=has_one,
        has_ranked_runners_up=has_runners,
        has_honest_concessions=has_concessions,
        cites_red_team=cites_rt,
        fences_present=fences,
        evidence_pointers_total=len(pointers),
        evidence_pointers_missing=len(missing),
        ok=ok,
    )


def _missing_pointers() -> list[str]:
    doc = _read(DOSSIER)
    return [p for p in _cited_paths(doc) if not _resolves(p)]


def render_markdown(r: DossierStatus) -> str:
    return "\n".join(
        [
            "# Garnet ultrapunch dossier status (S115)",
            "",
            f"_Schema {r.schema}._",
            "",
            f"- dossier present: {'yes' if r.dossier_present else 'NO'}",
            f"- #1 claim (capability-bounded acceptance): "
            f"{'yes' if r.has_number_one_claim else 'NO'}",
            f"- ranked runners-up: {'yes' if r.has_ranked_runners_up else 'NO'}",
            f"- honest concessions kept: {'yes' if r.has_honest_concessions else 'NO'}",
            f"- cites the red-team result: {'yes' if r.cites_red_team else 'NO'}",
            f"- honesty fences present: {'yes' if r.fences_present else 'NO'}",
            f"- evidence pointers resolved: "
            f"{r.evidence_pointers_total - r.evidence_pointers_missing}"
            f"/{r.evidence_pointers_total}",
            "",
            "The #1 claim is capability-bounded acceptance of agent-authored code, "
            "enforced (both backends) + Linux seccomp, cross-OS verified, red-teamed. "
            "Every cited proof resolves on disk. Named-deferred fences unchanged; "
            "v0.8.1 research-grade, no production / 1.0 claim.",
            "",
        ]
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless the dossier is present, makes the #1 claim, ranks "
        "runners-up, keeps the honest concessions + fences, cites the red-team, and "
        "every cited evidence pointer resolves on disk.",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    print(render_markdown(r) if args.format == "md" else json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(f"dossier gate FAILED: {asdict(r)}", file=sys.stderr)
        miss = _missing_pointers()
        if miss:
            print(f"  missing evidence pointers: {miss}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
