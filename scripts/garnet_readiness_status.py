#!/usr/bin/env python3
"""Report Garnet slice completion from tracked roadmap checkboxes."""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_PLAN = Path("F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md")

CHECKBOX_RE = re.compile(r"^- \[(?P<mark>[ xX])\] \*\*(?P<title>[^*]+)\*\*", re.MULTILINE)
HEADING_RE = re.compile(r"^(?P<hashes>#{2,3}) (?P<title>.+)$", re.MULTILINE)


@dataclass(frozen=True)
class SliceStatus:
    """One tracked implementation-plan checkbox."""

    title: str
    done: bool
    section: str


@dataclass(frozen=True)
class SectionSummary:
    """Completion statistics for one checklist section."""

    section: str
    total_slices: int
    completed_slices: int
    completion_percent: float


@dataclass(frozen=True)
class ReadinessStatus:
    """Aggregate readiness status suitable for PR bodies and dogfood bundles."""

    source: str
    total_slices: int
    completed_slices: int
    completion_percent: float
    open_slices: list[SliceStatus]
    section_summaries: list[SectionSummary]


def _section_for(text: str, offset: int) -> str:
    section = "Document"
    for match in HEADING_RE.finditer(text, 0, offset):
        section = match.group("title").strip()
    return section


def read_status(path: Path) -> ReadinessStatus:
    text = path.read_text(encoding="utf-8")
    slices: list[SliceStatus] = []
    section_totals: dict[str, list[bool]] = {}
    for match in CHECKBOX_RE.finditer(text):
        mark = match.group("mark")
        section = _section_for(text, match.start())
        done = mark.lower() == "x"
        slices.append(
            SliceStatus(
                title=match.group("title").strip(),
                done=done,
                section=section,
            )
        )
        section_totals.setdefault(section, []).append(done)

    section_summaries: list[SectionSummary] = []
    for section, done_flags in section_totals.items():
        total = len(done_flags)
        completed = sum(1 for item in done_flags if item)
        summary_pct = round((completed / total * 100.0) if total else 0.0, 1)
        section_summaries.append(
            SectionSummary(
                section=section,
                total_slices=total,
                completed_slices=completed,
                completion_percent=summary_pct,
            )
        )

    total = len(slices)
    completed = sum(1 for item in slices if item.done)
    percent = round((completed / total * 100.0) if total else 0.0, 1)
    return ReadinessStatus(
        source=str(path),
        total_slices=total,
        completed_slices=completed,
        completion_percent=percent,
        open_slices=[item for item in slices if not item.done],
        section_summaries=section_summaries,
    )


def render_markdown(status: ReadinessStatus) -> str:
    lines = [
        "# Garnet Readiness Slice Status",
        "",
        f"Source: `{status.source}`",
        "",
        (
            f"Completion: {status.completed_slices}/{status.total_slices} "
            f"slices ({status.completion_percent:.1f}%)."
        ),
        "",
        "## Open Slices",
        "",
    ]
    if not status.open_slices:
        lines.append("- None tracked in this plan.")
    else:
        for item in status.open_slices:
            lines.append(f"- `{item.section}` - {item.title}")

    lines.extend(["", "## Section Completion"])
    for section in status.section_summaries:
        lines.append(
            f"- {section.section}: {section.completed_slices}/{section.total_slices} "
            f"({section.completion_percent:.1f}%)"
        )
    return "\n".join(lines) + "\n"


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--plan",
        default=str(DEFAULT_PLAN),
        help="implementation plan path, relative to the repo root unless absolute",
    )
    parser.add_argument(
        "--format",
        choices=("markdown", "json"),
        default="markdown",
        help="output format",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    plan = Path(args.plan)
    if not plan.is_absolute():
        plan = ROOT / plan

    status = read_status(plan)
    if args.format == "json":
        print(json.dumps(asdict(status), indent=2))
    else:
        print(render_markdown(status), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
