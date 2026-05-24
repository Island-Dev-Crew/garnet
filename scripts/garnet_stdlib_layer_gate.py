#!/usr/bin/env python3
"""Garnet stdlib layer + `@stability` gate (S17).

Reports the stdlib primitive surface by Layer Policy layer, the share of
primitives carrying an explicit `@stability` tier, and any deprecated
primitives with their removal target. Deterministic and source-backed: it
parses the canonical primitive table in `garnet-stdlib/src/registry.rs`
(where `Layer::` / `Stability::` are written fully-qualified precisely so this
parse is unambiguous) and makes no claim beyond that evidence.

Gate (exit non-zero on failure):
  - total primitives >= 50
  - explicit-`@stability` coverage >= 95%

See `C_Language_Specification/GARNET_STDLIB_LAYER_POLICY.md`.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "garnet-stdlib" / "src" / "registry.rs"
LAYER_POLICY_DOC = ROOT / "C_Language_Specification" / "GARNET_STDLIB_LAYER_POLICY.md"

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

COUNT_GATE = 50
STABILITY_GATE_PERCENT = 95.0

# Layer-0/1 layer names (display order) + the four stability tiers.
LAYER_ORDER = ["Core", "Std", "Package", "Community", "Open"]
STABILITY_TIERS = ["Stable", "Experimental", "Frozen", "Deprecated"]

# Matches one `p("module", "name", <arity>, <caps>, Layer::X, Stability::Y,`
# registry call. `<caps>` is a parameterless `RequiredCaps::xxx()` (no comma);
# `re.S` lets the first six args wrap across lines after rustfmt.
PRIM_RE = re.compile(
    r'p\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*\d+\s*,\s*[^,]+,\s*'
    r"Layer::(\w+)\s*,\s*Stability::(\w+)\s*,",
    re.S,
)


@dataclass(frozen=True)
class StdlibLayerStatus:
    total: int
    by_layer: dict[str, int]
    stability_breakdown: dict[str, int]
    explicit_stability_percent: float
    deprecated: list[dict[str, str]] = field(default_factory=list)
    layer_policy_doc_exists: bool = False

    @property
    def meets_count_gate(self) -> bool:
        return self.total >= COUNT_GATE

    @property
    def meets_stability_gate(self) -> bool:
        return self.explicit_stability_percent >= STABILITY_GATE_PERCENT

    @property
    def ok(self) -> bool:
        return (
            self.meets_count_gate
            and self.meets_stability_gate
            and self.layer_policy_doc_exists
        )


def parse_registry(text: str) -> list[dict[str, str]]:
    """Extract `[{module, name, layer, stability}, ...]` from registry source."""
    prims: list[dict[str, str]] = []
    for module, name, layer, stability in PRIM_RE.findall(text):
        prims.append(
            {
                "module": module,
                "name": name,
                "qualified": f"{module}::{name}",
                "layer": layer,
                "stability": stability,
            }
        )
    return prims


def summarize(prims: list[dict[str, str]], doc_exists: bool) -> StdlibLayerStatus:
    total = len(prims)
    by_layer = {layer: 0 for layer in LAYER_ORDER}
    breakdown = {tier: 0 for tier in STABILITY_TIERS}
    explicit = 0
    deprecated: list[dict[str, str]] = []
    for prim in prims:
        by_layer[prim["layer"]] = by_layer.get(prim["layer"], 0) + 1
        tier = prim["stability"]
        breakdown[tier] = breakdown.get(tier, 0) + 1
        if tier in STABILITY_TIERS:
            explicit += 1
        if tier == "Deprecated":
            deprecated.append(
                {
                    "primitive": prim["qualified"],
                    # No primitive is deprecated in v0.7; the removal target is
                    # "next major" per Layer Policy §3 when one is marked.
                    "removal_target": "next major",
                }
            )
    pct = round(explicit / total * 100.0, 1) if total else 0.0
    # Drop all-zero layers from the report for readability.
    by_layer = {k: v for k, v in by_layer.items() if v}
    return StdlibLayerStatus(
        total=total,
        by_layer=by_layer,
        stability_breakdown={k: v for k, v in breakdown.items() if v},
        explicit_stability_percent=pct,
        deprecated=deprecated,
        layer_policy_doc_exists=doc_exists,
    )


def read_status() -> StdlibLayerStatus:
    text = REGISTRY.read_text(encoding="utf-8")
    return summarize(parse_registry(text), LAYER_POLICY_DOC.exists())


def render_markdown(s: StdlibLayerStatus) -> str:
    lines = [
        "# Garnet stdlib layer + `@stability` gate",
        "",
        f"Source: `{REGISTRY.relative_to(ROOT)}`",
        "",
        f"- Total primitives: **{s.total}** "
        f"({'PASS' if s.meets_count_gate else 'FAIL'} ≥ {COUNT_GATE})",
        f"- Explicit `@stability` coverage: **{s.explicit_stability_percent:.1f}%** "
        f"({'PASS' if s.meets_stability_gate else 'FAIL'} ≥ {STABILITY_GATE_PERCENT}%)",
        f"- Layer Policy doc present: **{'yes' if s.layer_policy_doc_exists else 'no'}** "
        f"(`{LAYER_POLICY_DOC.relative_to(ROOT)}`)",
        "",
        "| Layer | Primitives |",
        "|---|---:|",
    ]
    for layer in LAYER_ORDER:
        if layer in s.by_layer:
            lines.append(f"| {layer} (L{LAYER_ORDER.index(layer)}) | {s.by_layer[layer]} |")
    lines += ["", "| Stability tier | Primitives |", "|---|---:|"]
    for tier in STABILITY_TIERS:
        if tier in s.stability_breakdown:
            lines.append(f"| {tier} | {s.stability_breakdown[tier]} |")
    if s.deprecated:
        lines += ["", "| Deprecated primitive | Removal target |", "|---|---|"]
        for d in s.deprecated:
            lines.append(f"| `{d['primitive']}` | {d['removal_target']} |")
    else:
        lines += ["", "_No deprecated primitives._"]
    lines += ["", f"Gate: **{'PASS' if s.ok else 'FAIL'}**"]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["markdown", "json"], default="markdown")
    args = parser.parse_args(argv)
    status = read_status()
    if args.format == "json":
        print(json.dumps(asdict(status), indent=2))
    else:
        print(render_markdown(status))
    return 0 if status.ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
