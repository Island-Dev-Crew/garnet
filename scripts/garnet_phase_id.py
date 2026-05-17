#!/usr/bin/env python3
"""Allocate the next free Garnet phase id and guard against collisions.

Concurrent agents previously hand-picked phase letters and collided (PR #74
and PR #75 both used "Phase 4BI"). This helper derives the next free phase id
from the tracked implementation plan, the phase ownership register, and recent
git history so no agent has to guess.

Usage:
  python3 scripts/garnet_phase_id.py            # print the next free phase id
  python3 scripts/garnet_phase_id.py --check 6BT # exit 2 if 6BT is already used
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys
from collections.abc import Iterable
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

PLAN = ROOT / "F_Project_Management" / "GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md"
REGISTER = (
    ROOT
    / "F_Project_Management"
    / "ROADMAPS"
    / "GARNET_v0_5_PHASE_OWNERSHIP_REGISTER.md"
)

PHASE_TOKEN_RE = re.compile(r"[0-9]+[A-Z]+")
PHASE_IN_TEXT_RE = re.compile(r"PHASE\s+([0-9]+[A-Z]+)")
GIT_LOG_LIMIT = 250


def _col_to_int(letters: str) -> int:
    """Bijective base-26: A=1, Z=26, AA=27, AZ=52, BA=53, ZZ=702."""
    value = 0
    for ch in letters:
        value = value * 26 + (ord(ch) - ord("A") + 1)
    return value


def _int_to_col(value: int) -> str:
    """Inverse of _col_to_int."""
    letters = ""
    while value > 0:
        value, rem = divmod(value - 1, 26)
        letters = chr(ord("A") + rem) + letters
    return letters


def _key(phase_id: str) -> tuple[int, int]:
    match = re.match(r"(?P<num>[0-9]+)(?P<letters>[A-Z]+)$", phase_id)
    if match is None:
        return (0, 0)
    return (int(match.group("num")), _col_to_int(match.group("letters")))


def normalize_ids(ids: Iterable[str]) -> set[str]:
    """Extract canonical `<digits><LETTERS>` phase ids from arbitrary strings."""
    out: set[str] = set()
    for raw in ids:
        for token in PHASE_TOKEN_RE.findall(raw.upper()):
            out.add(token)
    return out


def next_phase_id(ids: Iterable[str]) -> str:
    """Return the successor of the global maximum phase id.

    The phase letter is a single shared global counter (PR titles use one
    running sequence). The successor keeps the numeric prefix and advances the
    letter column, so `6BS` -> `6BT` and `6BZ` -> `6CA`. Empty input -> `1A`.
    """
    normalized = normalize_ids(ids)
    if not normalized:
        return "1A"
    top = max(normalized, key=_key)
    match = re.match(r"(?P<num>[0-9]+)(?P<letters>[A-Z]+)$", top)
    assert match is not None
    num = match.group("num")
    next_col = _col_to_int(match.group("letters")) + 1
    return f"{num}{_int_to_col(next_col)}"


def _git_log_subjects() -> list[str]:
    try:
        proc = subprocess.run(
            ["git", "log", "--oneline", f"-{GIT_LOG_LIMIT}"],
            capture_output=True,
            text=True,
            cwd=str(ROOT),
            check=False,
        )
    except OSError:
        return []
    if proc.returncode != 0:
        return []
    return proc.stdout.splitlines()


def collect_used_ids(root: Path = ROOT) -> set[str]:
    """Gather every phase id referenced in the plan, register, and git log."""
    used: set[str] = set()
    for path in (PLAN, REGISTER):
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        used.update(PHASE_IN_TEXT_RE.findall(text.upper()))
    for subject in _git_log_subjects():
        used.update(PHASE_IN_TEXT_RE.findall(subject.upper()))
    return normalize_ids(used)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        metavar="PHASE_ID",
        help="Exit non-zero if PHASE_ID is already used anywhere tracked.",
    )
    args = parser.parse_args(argv)

    used = collect_used_ids(ROOT)

    if args.check is not None:
        candidates = normalize_ids([args.check])
        clashes = sorted(candidates & used)
        if clashes:
            print(
                f"phase-id: COLLISION — already used: {', '.join(clashes)}",
                file=sys.stderr,
            )
            return 2
        print(f"phase-id: {args.check} is free")
        return 0

    print(next_phase_id(used))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
