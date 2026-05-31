#!/usr/bin/env python3
"""AI-PR-review-collapse wedge demo (S49) — the launch narrative.

Composes the trust-kernel gates into one scenario: an AI-suggested PR that reads
plausibly but silently widens the program's authority from `@caps(fs)` to
`@caps(fs, net)` — an exfiltration path a human skimming a large AI diff is
liable to wave through. The demo proves the machine catches it:

1. `garnet check` is **clean on both** versions — the escalation is invisible to
   the type/safe-mode checker. (That is the point: it is not a bug, it is an
   authority change.)
2. `garnet diff-caps before after` exits non-zero with `caps GAINED: net` and
   `AUTHORITY EXPANDED` — the capability-surface diff (S37) catches it in one
   command, O(1) in the size of the surrounding diff.
3. `garnet sandbox` (S46) shows the consequence: egress flips `deny-all` →
   `allow`.

## Honest scope (do not soften)
The "human PR review collapses under AI volume" claim is the **motivating
thesis** (see `F_Project_Management/GARNET_PR_REVIEW_WEDGE.md`), **not** a
measurement made here. This harness measures only that the machine gates fire as
designed on the scenario. It is a narrative composition of existing gates
(S37/S46), not a new enforcement mechanism and not a guarantee against all AI-PR
risks.
"""
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

BEFORE = "examples/wedge_pr_review/before.garnet"
AFTER = "examples/wedge_pr_review/after.garnet"
CLEAN_MARKER = "0 diagnostics"
GAINED_MARKER = "caps GAINED:  net"
EXPANDED_MARKER = "AUTHORITY EXPANDED"


@dataclass
class Step:
    name: str
    argv: list[str]
    exit_code: int
    passed: bool
    detail: str


@dataclass
class WedgeReport:
    schema: str
    scenario: str
    steps: list[Step]
    wedge_fires: bool


def resolve_garnet(explicit: str | None) -> list[str]:
    if explicit:
        return [explicit]
    env_value = os.environ.get("GARNET_CLI")
    if env_value:
        return [env_value]
    executable = "garnet.exe" if os.name == "nt" else "garnet"
    # Prefer the most recently built binary so a stale release/ artifact never
    # shadows a fresh debug/ build (or vice versa).
    candidates = [
        ROOT / "target" / profile / executable for profile in ("release", "debug")
    ]
    existing = [c for c in candidates if c.exists()]
    if existing:
        newest = max(existing, key=lambda p: p.stat().st_mtime)
        return [str(newest)]
    installed = shutil.which("garnet")
    if installed:
        return [installed]
    raise FileNotFoundError(
        "Could not find Garnet CLI. Build it with `cargo build -p garnet-cli` "
        "or pass --garnet /path/to/garnet."
    )


def _run(argv: list[str]) -> tuple[int, str]:
    completed = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True)
    return completed.returncode, completed.stdout + completed.stderr


def _egress_mode(garnet: list[str], path: str) -> str:
    code, out = _run([*garnet, "sandbox", "--format", "json", path])
    if code != 0:
        return f"<sandbox failed: {code}>"
    try:
        return json.loads(out)["egress"]["mode"]
    except (json.JSONDecodeError, KeyError):
        return "<unparseable>"


def run_wedge(garnet: list[str]) -> WedgeReport:
    steps: list[Step] = []

    # 1. Both versions check clean — the escalation is invisible to the checker.
    for label, path in (("check-before", BEFORE), ("check-after", AFTER)):
        code, out = _run([*garnet, "check", path])
        clean = code == 0 and CLEAN_MARKER in out
        steps.append(
            Step(label, [*garnet, "check", path], code, clean,
                 "clean" if clean else "expected a clean check")
        )

    # 2. diff-caps catches the silent authority expansion.
    code, out = _run([*garnet, "diff-caps", BEFORE, AFTER])
    caught = code != 0 and GAINED_MARKER in out and EXPANDED_MARKER in out
    steps.append(
        Step("diff-caps", [*garnet, "diff-caps", BEFORE, AFTER], code, caught,
             "caught GAINED net + AUTHORITY EXPANDED" if caught
             else "diff-caps did NOT flag the escalation")
    )

    # 3. sandbox shows the egress consequence (deny-all -> allow).
    before_egress = _egress_mode(garnet, BEFORE)
    after_egress = _egress_mode(garnet, AFTER)
    flip = before_egress == "deny-all" and after_egress == "allow"
    steps.append(
        Step("sandbox-egress", [*garnet, "sandbox", AFTER], 0, flip,
             f"egress {before_egress} -> {after_egress}")
    )

    return WedgeReport(
        schema="garnet.pr_review_wedge/v1",
        scenario="silent @caps(fs) -> @caps(fs, net) escalation in an AI-suggested PR",
        steps=steps,
        wedge_fires=all(s.passed for s in steps),
    )


def render_markdown(report: WedgeReport) -> str:
    lines = [
        "# AI-PR-review-collapse wedge demo",
        "",
        f"_Schema {report.schema}._",
        "",
        f"**Scenario:** {report.scenario}",
        "",
        "| step | exit | result | detail |",
        "|---|---|---|---|",
    ]
    for s in report.steps:
        lines.append(
            f"| {s.name} | {s.exit_code} | {'✅' if s.passed else '❌'} | {s.detail} |"
        )
    lines += [
        "",
        f"**Wedge fires as designed: {'yes' if report.wedge_fires else 'NO'}.**",
        "",
        "The escalation is invisible to `garnet check` (both versions clean) yet "
        "caught by `garnet diff-caps` in one command — machine capability-review "
        "is O(1) in diff size. The 'human review collapses under AI volume' claim "
        "is the motivating thesis, not measured here.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--garnet", help="path to the garnet CLI")
    parser.add_argument("--format", choices=["json", "md"], default="md")
    args = parser.parse_args(list(argv) if argv is not None else None)

    garnet = resolve_garnet(args.garnet)
    report = run_wedge(garnet)
    if args.format == "md":
        print(render_markdown(report))
    else:
        print(json.dumps(asdict(report), indent=2))
    return 0 if report.wedge_fires else 1


if __name__ == "__main__":
    raise SystemExit(main())
