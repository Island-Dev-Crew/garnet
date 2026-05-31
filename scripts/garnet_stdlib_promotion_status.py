#!/usr/bin/env python3
"""Stdlib promotion-wave status (S76).

The S76 promotion wave elevates the foundational **`core::*` layer**
(iter / result / option / cmp / math) from Experimental to **Stable**, while
keeping the **`std::*`** host-authority + evolving-API utilities (env / process /
json / regex / uuid / base64 / log) **Experimental**.

## Promotion criteria (a primitive is promoted iff ALL hold)
1. it is in the `core::` layer (foundational, no host authority);
2. its semantics are universally established and API-frozen (functional
   iterators, Result/Option combinators, comparisons, basic math);
3. it is test-covered and used in the shipped corpus.

`std::*` primitives are deliberately NOT promoted: they touch host authority
(env/process) or wrap utilities whose API may change between minor releases
(regex flags, json options, base64 padding, uuid versions). Promoting them to
silence example warnings would game the `@stability` contract — the warnings on
those are correct.

This reporter parses `garnet-stdlib/src/registry.rs` and verifies the wave was
**principled and scoped**: every `core::*` primitive is Stable, and `std::*`
still carries Experimental entries (proof it was not a blanket flip).

## Honest scope (do not soften)
Promotion reflects a real stability judgement, not warning-suppression. Examples
that use `std::*` experimental utilities (e.g. `novel_04`–`novel_06`) still emit
correct stability warnings; only the core-only example (`novel_07`) goes clean.
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

PRIM_RE = re.compile(
    r'p\(\s*"([^"]+)",\s*"([^"]+)",\s*\d+,[^,]+,\s*Layer::\w+,\s*Stability::(\w+)'
)

PROMOTED_FAMILIES = ["core::iter", "core::result", "core::option", "core::cmp", "core::math"]


@dataclass
class PromotionStatus:
    schema: str
    core_total: int
    core_stable: int
    core_experimental: list[str]
    std_experimental_count: int
    promoted_families: list[str] = field(default_factory=lambda: list(PROMOTED_FAMILIES))
    ok: bool = False


def read_status() -> PromotionStatus:
    src = REGISTRY.read_text(encoding="utf-8") if REGISTRY.is_file() else ""
    core_total = core_stable = std_exp = 0
    core_exp: list[str] = []
    for ns, name, stab in PRIM_RE.findall(src):
        if ns.startswith("core::"):
            core_total += 1
            if stab == "Stable":
                core_stable += 1
            elif stab == "Experimental":
                core_exp.append(f"{ns}::{name}")
        elif ns.startswith("std::") and stab == "Experimental":
            std_exp += 1
    # Principled + scoped: all core::* Stable, AND std::* still has Experimental.
    ok = bool(src) and core_total > 0 and not core_exp and std_exp > 0
    return PromotionStatus(
        schema="garnet.stdlib_promotion_status/v1",
        core_total=core_total,
        core_stable=core_stable,
        core_experimental=core_exp,
        std_experimental_count=std_exp,
        ok=ok,
    )


def render_markdown(r: PromotionStatus) -> str:
    lines = [
        "# Garnet stdlib promotion-wave status (S76)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- promoted families (core::*): {', '.join(r.promoted_families)}",
        f"- core::* primitives Stable: {r.core_stable}/{r.core_total}",
        f"- core::* still Experimental (must be 0): {r.core_experimental or 'none'}",
        f"- std::* still Experimental (kept, must be > 0): {r.std_experimental_count}",
        "",
        "The wave promotes the foundational core layer (frozen semantics, no host "
        "authority) and KEEPS std::* experimental (host authority / evolving APIs). "
        "Honest scope: a real stability judgement, not warning-suppression — std::*"
        "-using examples still warn correctly; only core-only examples go clean.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless every core::* primitive is Stable AND std::* "
        "still carries Experimental entries (proof the wave was scoped, not blanket).",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    if args.format == "md":
        print(render_markdown(r))
    else:
        print(json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(
            "stdlib-promotion gate FAILED: "
            f"core_total={r.core_total} core_stable={r.core_stable} "
            f"core_experimental={r.core_experimental} std_experimental={r.std_experimental_count}",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
