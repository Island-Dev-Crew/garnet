#!/usr/bin/env python3
"""Garnet v0.8 version-map source-of-truth check (S70).

The whole S30–S80 run is cut as ONE `v0.8.0` tag at the END of S80; S60 and S70
are readiness checkpoints, not cuts; S81+ is the runway to v0.8.1. This gate
locks that source of truth so docs cannot silently drift back to the superseded
"v0.8.0 @ S60 / v0.8.1 @ S70 / v0.8.2 decision @ S80" mapping.

It checks two things:
  1. POSITIVE — `GARNET_v0_8_VERSION_MAP.md` exists and states the corrected
     mapping anchors (single v0.8.0 cut @ S80; S60/S70 checkpoints; S81+ = v0.8.1
     runway; 1.0 held), and the operative contract points at it.
  2. NEGATIVE — the operative contract no longer carries the old **bold** band
     cells ("**v0.8.0 @ S60**", "**v0.8.1 @ S70**",
     "**v0.8.2 readiness decision @ S80**"). Historical artifacts may quote the
     old labels in prose behind a correction banner; only the bold table cells
     are forbidden, so quoted references do not false-positive.

## Honest scope (do not soften)
This gate guards documentation consistency only. It does NOT cut, push, or
authorize any tag — tagging stays a human release-truth decision.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PM = ROOT / "F_Project_Management"
VERSION_MAP = PM / "GARNET_v0_8_VERSION_MAP.md"
CONTRACT = PM / "GARNET_v0_8_SLICE_DOGFOOD.md"

# Anchors the source-of-truth doc must state (case-insensitive substring).
REQUIRED_MAP_ANCHORS = [
    "v0.8.0 cut decision @ s80",
    "checkpoint @ s60",
    "checkpoint @ s70",
    "s81+ is the runway to v0.8.1",
    "1.0 is held",
]

# The superseded **bold** band-table cells. Forbidden in the operative contract;
# quoted (non-bold) references in correction banners are allowed by design.
FORBIDDEN_BOLD_CELLS = [
    "**v0.8.0 @ S60**",
    "**v0.8.1 @ S70**",
    "**v0.8.2 readiness decision @ S80**",
]


@dataclass
class VersionMapReadiness:
    schema: str
    version_map_present: bool
    missing_map_anchors: list[str]
    contract_points_at_map: bool
    forbidden_cells_in_contract: list[str] = field(default_factory=list)
    ok: bool = False


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def read_readiness() -> VersionMapReadiness:
    vmap = _read(VERSION_MAP)
    contract = _read(CONTRACT)
    vmap_lc = vmap.lower()
    missing = [a for a in REQUIRED_MAP_ANCHORS if a not in vmap_lc]
    forbidden = [c for c in FORBIDDEN_BOLD_CELLS if c in contract]
    points_at_map = "GARNET_v0_8_VERSION_MAP.md" in contract
    ok = bool(vmap) and not missing and points_at_map and not forbidden
    return VersionMapReadiness(
        schema="garnet.version_map_check/v1",
        version_map_present=bool(vmap),
        missing_map_anchors=missing,
        contract_points_at_map=points_at_map,
        forbidden_cells_in_contract=forbidden,
        ok=ok,
    )


def render_markdown(r: VersionMapReadiness) -> str:
    lines = [
        "# Garnet v0.8 version-map check",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- source-of-truth `GARNET_v0_8_VERSION_MAP.md` present: "
        f"{'yes' if r.version_map_present else 'NO'}",
        f"- required mapping anchors: "
        + ("all present" if not r.missing_map_anchors else f"MISSING {r.missing_map_anchors}"),
        f"- operative contract points at the version map: "
        f"{'yes' if r.contract_points_at_map else 'NO'}",
        f"- superseded bold band-cells in contract: "
        + ("none" if not r.forbidden_cells_in_contract else f"FOUND {r.forbidden_cells_in_contract}"),
        "",
        "Source of truth: the whole S30–S80 run is cut as one `v0.8.0` tag at the "
        "end of S80; S60/S70 are checkpoints, not cuts; S81+ is the v0.8.1 runway. "
        "This gate guards docs only — it cuts no tag.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero if the version-map source of truth is missing/drifted",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_readiness()
    if args.format == "md":
        print(render_markdown(r))
    else:
        print(json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(
            "version-map-check gate FAILED: "
            f"present={r.version_map_present} missing_anchors={r.missing_map_anchors} "
            f"points_at_map={r.contract_points_at_map} "
            f"forbidden_cells={r.forbidden_cells_in_contract}",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
