#!/usr/bin/env python3
"""VM / interpreter parity campaign (S73).

Garnet has two execution backends — the tree-walking interpreter
(`garnet run --interp`, the default) and the bytecode VM (`garnet run --vm`).
This campaign runs every program in `examples/*.garnet` through BOTH backends
and asserts they agree, so the two paths cannot silently diverge.

## Parity predicate (deterministic surface)
For each program, parity holds iff the two backends produce the **same stdout**
and the **same exit code**. We compare **stdout + exit code only** — the
deterministic surface — and deliberately ignore stderr, because:
  - the VM wraps runtime errors with a cosmetic `vm error:` prefix the
    interpreter does not (same underlying exception, different wrapper); and
  - the episodic cache (`.garnet-cache/episodes.log`) emits run-to-run
    nondeterministic strategy notes on stderr.
Program *output* (stdout) is deterministic across cache state, so it is the
sound channel for a semantic-parity comparison.

## Honest scope (do not soften)
This is **corpus-based** parity over the shipped examples, NOT a proof of total
semantic equivalence between the backends. Divergences (if any) are reported, not
hidden. The stderr wrapper-prefix difference is a known, cosmetic,
non-semantic difference and is documented, not "fixed away" here.
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXAMPLES = ROOT / "examples"


def _garnet_binary() -> Path | None:
    exe = "garnet.exe" if os.name == "nt" else "garnet"
    for profile in ("release", "debug"):
        cand = ROOT / "target" / profile / exe
        if cand.exists():
            return cand
    return None


def parity_verdict(stdout_i: str, rc_i: int, stdout_v: str, rc_v: int) -> bool:
    """Pure predicate: backends agree on stdout AND exit code."""
    return stdout_i == stdout_v and rc_i == rc_v


@dataclass
class ParityResult:
    schema: str
    corpus_size: int
    binary_available: bool
    parity_ok: int
    divergent: list[str] = field(default_factory=list)
    ok: bool = False


def _run(binary: Path, mode: str, program: Path) -> tuple[str, int]:
    proc = subprocess.run(
        [str(binary), "run", mode, str(program)],
        capture_output=True,
        text=True,
        timeout=120,
    )
    return proc.stdout, proc.returncode


def read_result(run_binary: bool = True) -> ParityResult:
    corpus = sorted(EXAMPLES.glob("*.garnet")) if EXAMPLES.is_dir() else []
    binary = _garnet_binary() if run_binary else None
    parity_ok = 0
    divergent: list[str] = []

    if binary is not None:
        for prog in corpus:
            so_i, rc_i = _run(binary, "--interp", prog)
            so_v, rc_v = _run(binary, "--vm", prog)
            if parity_verdict(so_i, rc_i, so_v, rc_v):
                parity_ok += 1
            else:
                divergent.append(f"{prog.name} (interp rc={rc_i} / vm rc={rc_v})")

    well_formed = len(corpus) > 0
    if binary is None:
        ok = well_formed  # static gate: corpus present
    else:
        ok = well_formed and not divergent and parity_ok == len(corpus)

    return ParityResult(
        schema="garnet.vm_interp_parity/v1",
        corpus_size=len(corpus),
        binary_available=binary is not None,
        parity_ok=parity_ok,
        divergent=divergent,
        ok=ok,
    )


def render_markdown(r: ParityResult) -> str:
    lines = [
        "# Garnet VM / interpreter parity campaign (S73)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- corpus: {r.corpus_size} `examples/*.garnet` programs",
        f"- garnet binary available: {'yes' if r.binary_available else 'no (dynamic run skipped)'}",
        f"- parity-ok (same stdout + exit code on both backends): "
        f"{r.parity_ok}/{r.corpus_size if r.binary_available else 'n/a'}",
        f"- divergences: {'none' if not r.divergent else r.divergent}",
        "",
        "Parity is compared on the deterministic surface (stdout + exit code); the "
        "VM's cosmetic `vm error:` stderr prefix and episodic-cache stderr notes are "
        "ignored by design. Honest scope: corpus-based parity over the shipped "
        "examples, not a proof of total backend equivalence.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero if any program diverges between backends (when the "
        "binary is present) or the corpus is empty. Skips the run if the binary is absent.",
    )
    parser.add_argument("--no-run", action="store_true", help="skip running the binary")
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_result(run_binary=not args.no_run)
    if args.format == "md":
        print(render_markdown(r))
    else:
        print(json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(
            "vm-interp-parity gate FAILED: "
            f"corpus={r.corpus_size} binary={r.binary_available} "
            f"parity_ok={r.parity_ok} divergent={r.divergent}",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
