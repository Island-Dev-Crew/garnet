#!/usr/bin/env python3
"""Report S6 memory-eviction benchmark status for Mnemos.

This reporter inventories `garnet-memory-v0.3/benches/eviction.rs` and the
four Mnemos memory kinds it covers (working / episodic / semantic /
procedural). It does NOT execute `cargo bench` — full Criterion runs take
minutes and are evidence the maintainer captures separately. The reporter
makes the harness's existence + per-kind coverage falsifiable so CI can
catch silent regressions.

Output: deterministic Markdown to stdout. Optionally JSON via --format json.
Exit code: always 0 (this is an evidence reporter, not a gate).
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

from garnet_reporter_io import configure_utf8_stdout  # noqa: E402

configure_utf8_stdout()

BENCH_FILE = ROOT / "garnet-memory-v0.3" / "benches" / "eviction.rs"
CARGO_TOML = ROOT / "garnet-memory-v0.3" / "Cargo.toml"

KINDS = ("working", "episodic", "semantic", "procedural")


@dataclass(frozen=True)
class KindCoverage:
    kind: str
    bench_function_present: bool
    naive_baseline_present: bool
    policy_path_present: bool


@dataclass(frozen=True)
class EvictionStatus:
    bench_file: str
    bench_file_present: bool
    cargo_entry_present: bool
    kinds: list[KindCoverage]
    coverage_complete: bool


def _bench_text() -> str:
    if not BENCH_FILE.exists():
        return ""
    return BENCH_FILE.read_text(encoding="utf-8")


def _cargo_has_eviction_entry() -> bool:
    if not CARGO_TOML.exists():
        return False
    text = CARGO_TOML.read_text(encoding="utf-8")
    # Look for a [[bench]] block whose name is "eviction" with harness = false.
    pattern = re.compile(
        r"\[\[bench\]\]\s*\nname\s*=\s*\"eviction\"\s*\nharness\s*=\s*false",
        flags=re.MULTILINE,
    )
    return bool(pattern.search(text))


def _check_kind(text: str, kind: str) -> KindCoverage:
    bench_fn = f"bench_{kind}"
    fn_present = re.search(rf"\bfn\s+{bench_fn}\s*\(", text) is not None
    # The bench uses bench_kind(c, MemoryKind::<Variant>, ...). Confirm each
    # kind is exercised by its capitalized variant name.
    variant = kind[:1].upper() + kind[1:]
    variant_present = re.search(rf"MemoryKind::{variant}\b", text) is not None
    return KindCoverage(
        kind=kind,
        bench_function_present=fn_present,
        naive_baseline_present=("evict_naive_fifo" in text),
        policy_path_present=variant_present and ("evict_policy_driven" in text),
    )


def read_status() -> EvictionStatus:
    text = _bench_text()
    kinds = [_check_kind(text, k) for k in KINDS]
    coverage_complete = bool(text) and all(
        k.bench_function_present and k.naive_baseline_present and k.policy_path_present
        for k in kinds
    )
    try:
        display = str(BENCH_FILE.relative_to(ROOT))
    except ValueError:
        display = str(BENCH_FILE)
    return EvictionStatus(
        bench_file=display,
        bench_file_present=BENCH_FILE.exists(),
        cargo_entry_present=_cargo_has_eviction_entry(),
        kinds=kinds,
        coverage_complete=coverage_complete,
    )


def render_markdown(status: EvictionStatus) -> str:
    lines = [
        "# Garnet Memory Eviction Benchmark Status (S6)",
        "",
        f"Bench file: `{status.bench_file}` (present: {status.bench_file_present})",
        f"Cargo `[[bench]] name = \"eviction\"` entry: {status.cargo_entry_present}",
        f"Per-kind coverage complete: **{status.coverage_complete}**",
        "",
        "| Kind | bench_fn | naive baseline | policy path |",
        "|---|---|---|---|",
    ]
    for k in status.kinds:
        lines.append(
            f"| {k.kind} | {'✅' if k.bench_function_present else '🟠'} "
            f"| {'✅' if k.naive_baseline_present else '🟠'} "
            f"| {'✅' if k.policy_path_present else '🟠'} |"
        )
    lines.extend(
        [
            "",
            "## Not claimed",
            "",
            "- A fresh Criterion measurement run is NOT embedded in this status; "
            "`cargo bench -p garnet-memory --bench eviction` produces those numbers "
            "and the maintainer attaches them as Desktop evidence.",
            "- End-to-end store throughput under eviction is exercised by "
            "`garnet-memory-v0.3/benches/vector.rs` and the store-specific test "
            "suites; this reporter covers the per-kind policy harness only.",
            "- Production allocator path remains tracked in "
            "`C_Language_Specification/MEMORY_CORE_ROADMAP.md` — S6 closes half of "
            "Paper VI Contribution 3's gap, not the full path.",
            "",
        ]
    )
    return "\n".join(lines)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--format",
        choices=("markdown", "json"),
        default="markdown",
        help="output format",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    status = read_status()
    if args.format == "json":
        print(json.dumps(asdict(status), indent=2))
    else:
        print(render_markdown(status), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
