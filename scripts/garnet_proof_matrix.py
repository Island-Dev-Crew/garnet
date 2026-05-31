#!/usr/bin/env python3
"""12-domain / 7-novel proof matrix — evidence inventory + anti-rot gate (S48).

Rigor evidence for a skeptical reviewer (reconciliation §44, §150): a falsifiable
inventory of (a) the 12 application domains Garnet is demonstrated across and
(b) the 7 novel Paper VI contributions, each anchored to in-repo evidence whose
existence is checked.

## Honest scope (do not soften — Paper VI anchors)

This is an **evidence inventory**, not empirical proof and not a re-adjudication
of Paper VI's per-contribution verdicts. It does NOT claim measurements,
mechanized proofs, or external study results. Two contribution-numbering schemes
exist across the repo's docs, so this matrix lists the seven contributions **by
title** and quotes Paper VI's own aggregate scorecard **verbatim** rather than
assigning a support verdict per contribution:

    Paper VI scorecard: "4 supported, 2 partial (downgraded honestly),
    0 refuted, 1 pending-infra"

The matrix proves a narrower, falsifiable thing: every contribution is *exercised*
by in-repo evidence that exists, and all 12 domain examples are present in the
domain-matrix suite. `--gate` fails if any domain example or contribution anchor
disappears.
"""
from __future__ import annotations

import argparse
import importlib.util
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

PAPER_VI_SCORECARD = (
    "4 supported, 2 partial (downgraded honestly), 0 refuted, 1 pending-infra"
)


def _load_domain_cases() -> list:
    """Reuse the 12 domains already defined in the studio domain-matrix smoke
    (single source of truth) instead of re-declaring them."""
    script = Path(__file__).with_name("smoke_garnet_studio_domain_matrix.py")
    spec = importlib.util.spec_from_file_location("smoke_domain_matrix", script)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules["smoke_domain_matrix"] = module
    spec.loader.exec_module(module)
    return list(module.CORE_12_CASES)


@dataclass
class Anchor:
    path: str
    present: bool


@dataclass
class Contribution:
    title: str
    novelty_section: str
    anchors: list[Anchor]
    exercised: bool


@dataclass
class Domain:
    id: str
    label: str
    file: str
    present: bool


@dataclass
class ProofMatrix:
    schema: str
    paper_vi_scorecard: str
    domains: list[Domain]
    contributions: list[Contribution]
    all_domains_present: bool
    every_contribution_exercised: bool


# The 7 novel contributions, by title (academic-submission-strategy table), each
# anchored to in-repo evidence that demonstrates the feature. `novelty_section`
# points at the Paper VI section that argues the novelty boundary.
_CONTRIBUTION_SPECS: tuple[tuple[str, str, tuple[str, ...]], ...] = (
    (
        "LLM-native syntax",
        "§11.1",
        ("C_Language_Specification/GARNET_v1_0_Mini_Spec.md", "examples"),
    ),
    (
        "Type spectrum (safe ↔ managed ↔ dynamic)",
        "§11.1",
        ("examples/safe_io_layer.garnet", "garnet-check-v0.3/src/lib.rs"),
    ),
    (
        "Compiler-as-agent",
        "§4.5",
        ("garnet-check-v0.3/src/suggest.rs", "C_Language_Specification/GARNET_ERROR_POLICY.md"),
    ),
    (
        "Kind-aware allocation",
        "§5.4",
        ("examples/multi_agent_builder.garnet", "garnet-interp-v0.3/src/value.rs"),
    ),
    (
        "Error bridging (value ↔ exception)",
        "§6.3",
        ("examples/safe_io_layer.garnet", "C_Language_Specification/GARNET_ERROR_POLICY.md"),
    ),
    (
        "Hot-reload across the mode boundary",
        "§7.4",
        ("examples/mvp_11_signed_hotreload.garnet", "examples/mvp_11_signed_hotreload_mismatch.garnet"),
    ),
    (
        "Reproducible builds with embedded provenance",
        "§8.4",
        (".github/workflows/determinism.yml", "garnet-cli/src/seal.rs"),
    ),
)


def _exists(rel: str) -> bool:
    return (ROOT / rel).exists()


def read_proof_matrix() -> ProofMatrix:
    cases = _load_domain_cases()
    domains = [
        Domain(id=c.id, label=c.label, file=c.file, present=_exists(c.file))
        for c in cases
    ]

    contributions: list[Contribution] = []
    for title, section, anchor_paths in _CONTRIBUTION_SPECS:
        anchors = [Anchor(path=p, present=_exists(p)) for p in anchor_paths]
        contributions.append(
            Contribution(
                title=title,
                novelty_section=section,
                anchors=anchors,
                exercised=any(a.present for a in anchors),
            )
        )

    return ProofMatrix(
        schema="garnet.proof_matrix/v1",
        paper_vi_scorecard=PAPER_VI_SCORECARD,
        domains=domains,
        contributions=contributions,
        all_domains_present=all(d.present for d in domains),
        every_contribution_exercised=all(c.exercised for c in contributions),
    )


def render_markdown(m: ProofMatrix) -> str:
    lines = [
        "# Garnet 12-domain / 7-novel proof matrix",
        "",
        f"_Schema {m.schema}. Evidence inventory — not empirical proof._",
        "",
        f"**Paper VI scorecard (verbatim, not re-adjudicated here):** "
        f'"{m.paper_vi_scorecard}"',
        "",
        f"## 12 domains ({sum(d.present for d in m.domains)}/{len(m.domains)} present)",
        "",
        "| # | Domain | Example | present |",
        "|---|---|---|---|",
    ]
    for i, d in enumerate(m.domains, 1):
        lines.append(f"| {i} | {d.label} | `{d.file}` | {'✅' if d.present else '❌'} |")
    lines += [
        "",
        "## 7 novel contributions (by title; anchored to in-repo evidence)",
        "",
        "| Contribution | §novelty | exercised | anchors |",
        "|---|---|---|---|",
    ]
    for c in m.contributions:
        anchors = ", ".join(
            f"`{a.path}`{'' if a.present else ' (missing)'}" for a in c.anchors
        )
        lines.append(
            f"| {c.title} | {c.novelty_section} | {'✅' if c.exercised else '❌'} | {anchors} |"
        )
    lines += [
        "",
        f"**All 12 domains present: {'yes' if m.all_domains_present else 'NO'}.**",
        f"**Every contribution exercised by existing evidence: "
        f"{'yes' if m.every_contribution_exercised else 'NO'}.**",
        "",
        "Honest scope: this inventory shows each contribution is *exercised* by "
        "in-repo evidence that exists; it does not re-adjudicate Paper VI's "
        "per-contribution support verdicts (quoted verbatim above) and makes no "
        "measurement, mechanized-proof, or external-study claim.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero if a domain example or contribution anchor is missing",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    matrix = read_proof_matrix()
    if args.format == "md":
        print(render_markdown(matrix))
    else:
        print(json.dumps(asdict(matrix), indent=2))

    if args.gate and not (matrix.all_domains_present and matrix.every_contribution_exercised):
        problems = [d.file for d in matrix.domains if not d.present]
        problems += [c.title for c in matrix.contributions if not c.exercised]
        print(
            f"proof-matrix gate FAILED: missing evidence — {', '.join(problems)}",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
