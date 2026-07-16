#!/usr/bin/env python3
"""Report the standing quarterly competitive-watch cadence without faking a run."""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass, field
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACT_PATH = Path("research/QUARTERLY_COMPETITIVE_WATCH.md")
REPORT_DIR = Path("research/competitive-watch")
SCHEMA = "garnet.quarterly_competitive_watch/v1"
FIRST_QUARTER = (2026, 3)
FIRST_DUE = date(2026, 9, 30)
REQUIRED_CATEGORIES = (
    "Agent-native languages",
    "Agent sandbox and runtime systems",
    "Attestation, provenance, and evidence tooling",
    "Agent governance, standards, and regulation",
)
REQUIRED_MARKERS = (
    f"Schema: `{SCHEMA}`",
    "Cadence: quarterly",
    "First report: 2026 Q3",
    "First due: 2026-09-30",
    "Report directory: `research/competitive-watch/`",
    "A search miss is not evidence of absence.",
)
REPORT_RE = re.compile(r"^([0-9]{4})-Q([1-4])\.md$")


@dataclass
class QuarterlyWatchStatus:
    schema: str
    state: str
    as_of: str
    report_count: int
    next_due: str
    contract_present: bool
    findings: list[str] = field(default_factory=list)
    ok: bool = False


def _quarter_due(year: int, quarter: int) -> date:
    if quarter == 1:
        return date(year, 3, 31)
    if quarter == 2:
        return date(year, 6, 30)
    if quarter == 3:
        return date(year, 9, 30)
    return date(year, 12, 31)


def _next_quarter(year: int, quarter: int) -> tuple[int, int]:
    return (year + 1, 1) if quarter == 4 else (year, quarter + 1)


def _report_complete(path: Path) -> tuple[bool, list[str]]:
    findings: list[str] = []
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        return False, [f"{path.name}: unreadable report: {exc}"]
    required = (
        "Status: completed",
        "Coverage window:",
        "Search date:",
        "Source queries:",
        "Primary sources:",
        "Coverage limitations:",
        "A search miss is not evidence of absence.",
    )
    for marker in required:
        if marker not in text:
            findings.append(f"{path.name}: missing {marker!r}")
    for category in REQUIRED_CATEGORIES:
        if f"## {category}" not in text:
            findings.append(f"{path.name}: missing category {category!r}")
    return not findings, findings


def _contract_findings(root: Path) -> tuple[bool, list[str]]:
    path = root / CONTRACT_PATH
    if path.is_symlink() or not path.is_file():
        return False, [f"{CONTRACT_PATH} is not a regular file"]
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        return False, [f"{CONTRACT_PATH} is unreadable: {exc}"]
    findings = [
        f"contract missing marker {marker!r}"
        for marker in REQUIRED_MARKERS
        if marker not in text
    ]
    if "A search miss is not evidence of absence." not in text:
        findings.append("contract missing the miss-is-not-absence rule")
    for category in REQUIRED_CATEGORIES:
        if f"### {category}" not in text:
            findings.append(f"contract missing category {category!r}")
    return True, findings


def read_status(
    root: Path = ROOT,
    *,
    as_of: date | None = None,
) -> QuarterlyWatchStatus:
    observed = as_of or date.today()
    present, findings = _contract_findings(root)
    completed: set[tuple[int, int]] = set()
    report_dir = root / REPORT_DIR
    if report_dir.exists() and not report_dir.is_dir():
        findings.append(f"{REPORT_DIR} exists but is not a directory")
    elif report_dir.is_dir():
        for path in sorted(report_dir.iterdir()):
            match = REPORT_RE.fullmatch(path.name)
            if match is None:
                if path.is_file():
                    findings.append(f"unexpected watch report filename {path.name!r}")
                continue
            if path.is_symlink() or not path.is_file():
                findings.append(f"{path.name}: report must be a regular file")
                continue
            report_quarter = (int(match.group(1)), int(match.group(2)))
            if report_quarter < FIRST_QUARTER:
                findings.append(f"{path.name}: report predates the standing contract")
                continue
            valid, report_findings = _report_complete(path)
            if valid:
                completed.add(report_quarter)
            else:
                findings.extend(report_findings)

    cursor = FIRST_QUARTER
    while cursor in completed:
        cursor = _next_quarter(*cursor)
    if any(quarter > cursor for quarter in completed):
        findings.append(
            f"completed watch reports are not contiguous; {cursor[0]}-Q{cursor[1]} is missing"
        )
    next_due_date = _quarter_due(*cursor)
    if observed > next_due_date:
        state = "overdue"
        findings.append(
            f"{cursor[0]}-Q{cursor[1]} completed report is missing after "
            f"{next_due_date.isoformat()}"
        )
    elif completed:
        state = "active"
    else:
        state = "planned"
    ok = present and not findings
    return QuarterlyWatchStatus(
        schema=SCHEMA,
        state=state,
        as_of=observed.isoformat(),
        report_count=len(completed),
        next_due=next_due_date.isoformat(),
        contract_present=present,
        findings=findings,
        ok=ok,
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--as-of", type=date.fromisoformat)
    parser.add_argument("--gate", action="store_true")
    args = parser.parse_args(argv)
    status = read_status(as_of=args.as_of)
    print(json.dumps(asdict(status), indent=2))
    if args.gate and not status.ok:
        print("quarterly competitive-watch gate FAILED", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
