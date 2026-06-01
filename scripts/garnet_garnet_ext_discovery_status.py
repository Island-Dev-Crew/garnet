#!/usr/bin/env python3
"""`.garnet`/`.GARNET` target-discovery status (S81).

The shared target collector (`garnet-cli/src/cmd/verify_gate.rs`, `collect_targets`
→ `walk`) matched the file extension case-sensitively, so Windows' case-insensitive
filesystem silently skipped uppercase `.GARNET` files — a *trust* hole spanning
`garnet verify` (WIN-S33-001), capability manifests (WIN-S36-001), `diff-caps`
(WIN-S37-001), and sandbox-policy generation (WIN-S46-001). S81 makes the one shared
collector case-insensitive (`eq_ignore_ascii_case`), which closes all four because
`garnet-cli/src/cap_manifest.rs::surface_for_path` reuses the same collector.

This is a static anti-regression gate: it asserts the collector is case-insensitive
and that the capability surfaces still route through the shared collector — so a
future edit cannot quietly re-introduce the case-sensitive skip.

## Honest scope (do not soften)
Mac-authored + Mac-unit-tested (macOS preserves filename case, so a `BAD.GARNET`
fixture reproduces the skip). The end-to-end Windows proof
(`garnet verify <dir with BAD.GARNET>` → exit 1 on a real Windows FS) is recorded
in `WINDOWS_AUDIT_S1_S80.md` as Windows-proof-pending for the Windows lane.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
COLLECTOR = ROOT / "garnet-cli" / "src" / "cmd" / "verify_gate.rs"
CAP_MANIFEST = ROOT / "garnet-cli" / "src" / "cap_manifest.rs"


@dataclass
class DiscoveryStatus:
    schema: str
    collector_case_insensitive: bool
    no_case_sensitive_compare: bool
    cap_manifest_reuses_collector: bool
    ok: bool = False


def _read(p: Path) -> str:
    return p.read_text(encoding="utf-8") if p.is_file() else ""


def read_status() -> DiscoveryStatus:
    collector = _read(COLLECTOR)
    cap = _read(CAP_MANIFEST)
    case_insensitive = 'eq_ignore_ascii_case("garnet")' in collector
    # The old case-sensitive compare must be gone from the collector.
    no_old = '|e| e == "garnet"' not in collector
    reuses = "collect_targets(path)" in cap
    ok = case_insensitive and no_old and reuses
    return DiscoveryStatus(
        schema="garnet.garnet_ext_discovery/v1",
        collector_case_insensitive=case_insensitive,
        no_case_sensitive_compare=no_old,
        cap_manifest_reuses_collector=reuses,
        ok=ok,
    )


def render_markdown(r: DiscoveryStatus) -> str:
    return "\n".join([
        "# Garnet `.GARNET` discovery status (S81)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- shared collector is case-insensitive (`eq_ignore_ascii_case`): "
        f"{'yes' if r.collector_case_insensitive else 'NO'}",
        f"- old case-sensitive `== \"garnet\"` compare removed: "
        f"{'yes' if r.no_case_sensitive_compare else 'NO'}",
        f"- `cap_manifest::surface_for_path` reuses the shared collector: "
        f"{'yes' if r.cap_manifest_reuses_collector else 'NO'}",
        "",
        "One shared-collector fix closes WIN-S33/S36/S37/S46. Mac-authored + "
        "unit-tested; the end-to-end Windows proof is recorded in "
        "WINDOWS_AUDIT_S1_S80.md as Windows-proof-pending.",
        "",
    ])


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless the shared collector is case-insensitive and the "
        "capability surfaces still route through it (anti-regression).",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    print(render_markdown(r) if args.format == "md" else json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(f"garnet-ext-discovery gate FAILED: {asdict(r)}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
