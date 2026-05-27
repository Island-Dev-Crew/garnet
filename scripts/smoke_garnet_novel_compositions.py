#!/usr/bin/env python3
"""Garnet novel-composition discovery harness (S20).

Proves the S20 `examples/novel_*.garnet` programs — each of which FUSES three or
four Paper-VI novel contributions into one program — `garnet check` clean and
`garnet run` with deterministic, byte-stable output. Complementary to (not a
duplicate of) `smoke_garnet_studio_domain_matrix.py`, which covers the existing
single-concern example corpus; this harness covers only the new compositions and
reports the composition matrix (which contributions each program fuses).

Evidence-first and deterministic: it runs the current Garnet CLI, asserts the
exact expected stdout line for each program, and makes no claim beyond that.
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

CHECK_CLEAN_MARKER = "0 diagnostics"


@dataclass(frozen=True)
class NovelCase:
    """One novel-composition program + the contributions it fuses + the exact
    `garnet run` stdout line that proves it executed correctly."""

    id: str
    file: str
    contributions: tuple[str, ...]
    run_expect: str
    # Substring the `garnet check` stdout must contain. Default is the
    # clean-checker marker; programs that call experimental primitives (S21
    # qualified dispatch) instead expect the non-fatal `@stability` advisory.
    check_expect: str = CHECK_CLEAN_MARKER


NOVEL_CASES: tuple[NovelCase, ...] = (
    NovelCase(
        "novel_01_capability_budgeted_memory_agent",
        "examples/novel_01_capability_budgeted_memory_agent.garnet",
        ("capability-budget", "memory-recall", "agent-pipeline"),
        "novel_01 capability_budgeted_memory_agent governance: 16",
    ),
    NovelCase(
        "novel_02_signed_provenance_pipeline",
        "examples/novel_02_signed_provenance_pipeline.garnet",
        ("blake3-signed-provenance", "agent-pipeline", "determinism"),
        "novel_02 signed_provenance verified: "
        "1f02c4147b45be3c9ea73f2cf73122d51cfc259034b845905f4e235bf2c325ce",
    ),
    NovelCase(
        "novel_03_release_gate_quorum",
        "examples/novel_03_release_gate_quorum.garnet",
        ("release-gate", "capability-budget", "blake3-signed-provenance", "memory-recall"),
        "novel_03 release_gate APPROVED quorum: 4",
    ),
    NovelCase(
        "novel_04_dispatched_stdlib_pipeline",
        "examples/novel_04_dispatched_stdlib_pipeline.garnet",
        ("core::iter-higher-order", "core::math", "core::cmp", "std::base64", "@stability-warnings"),
        "novel_04 tag: c2NvcmU9NQ==",
        # S21: calls qualified EXPERIMENTAL prims, so the checker emits non-fatal
        # `@stability` warnings (exit 0) rather than "0 diagnostics".
        check_expect="stability warning",
    ),
    NovelCase(
        "novel_05_s22_stdlib_memory_pipeline",
        "examples/novel_05_s22_stdlib_memory_pipeline.garnet",
        ("std::json", "std::regex", "std::uuid-v5", "std::log", "memory::mnemos-handles"),
        "novel_05 uuid: ee54a926-f375-5759-a5aa-67f7d8528cff",
        # S22 uses newly-dispatched experimental prims, so the checker emits
        # non-fatal `@stability` warnings rather than the clean marker.
        check_expect="stability warning",
    ),
    NovelCase(
        "novel_06_observability_provenance_pipeline",
        "examples/novel_06_observability_provenance_pipeline.garnet",
        ("std::log::to_file", "memory::episodic", "std::json", "blake3-signed-provenance"),
        "novel_06 provenance: "
        "791c7dcc2c4b11a669af74c23d74d6e0bdd5127f7bf1bc00fe490eec13822f96",
        # S25 capstone: composes S24's file sink + S22 memory/json + blake3.
        # Uses experimental prims, so the checker emits non-fatal `@stability`
        # warnings rather than the clean marker.
        check_expect="stability warning",
    ),
    NovelCase(
        "novel_07_functional_core_pipeline",
        "examples/novel_07_functional_core_pipeline.garnet",
        ("core::iter", "core::result", "core::option", "railway-pipeline"),
        "novel_07 final: 80",
        # S30 capstone: composes the full functional core (result/option/iter)
        # made runnable across S26-S28. Experimental prims → stability warning.
        check_expect="stability warning",
    ),
)


@dataclass(frozen=True)
class CaseResult:
    id: str
    contributions: list[str]
    check_ok: bool
    run_ok: bool

    @property
    def ok(self) -> bool:
        return self.check_ok and self.run_ok


def evaluate_case(
    case: NovelCase,
    check_code: int,
    check_stdout: str,
    run_code: int,
    run_stdout: str,
) -> CaseResult:
    """Pure verdict for a case given the two CLI invocations' results.

    `check` passes iff exit 0 AND the checker reports no diagnostics; `run`
    passes iff exit 0 AND the expected deterministic line is present. Separated
    from the subprocess driving so it is unit-testable with synthetic inputs."""
    check_ok = check_code == 0 and case.check_expect in check_stdout
    run_ok = run_code == 0 and case.run_expect in run_stdout
    return CaseResult(case.id, list(case.contributions), check_ok, run_ok)


# ── CLI resolution (mirrors smoke_garnet_studio_domain_matrix.py) ──────────
def resolve_garnet(explicit: str | None) -> list[str]:
    if explicit:
        return [explicit]
    env_value = os.environ.get("GARNET_CLI")
    if env_value:
        return [env_value]
    executable = "garnet.exe" if os.name == "nt" else "garnet"
    for profile in ("release", "debug"):
        candidate = ROOT / "target" / profile / executable
        if candidate.exists():
            return [str(candidate)]
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


def run_matrix(garnet: list[str]) -> list[CaseResult]:
    results: list[CaseResult] = []
    for case in NOVEL_CASES:
        ccode, cout = _run([*garnet, "check", case.file])
        rcode, rout = _run([*garnet, "run", case.file])
        results.append(evaluate_case(case, ccode, cout, rcode, rout))
    return results


def render_markdown(results: list[CaseResult]) -> str:
    passed = sum(1 for r in results if r.ok)
    lines = [
        "# Garnet novel-composition discovery matrix (S20-S22)",
        "",
        f"Programs: **{len(results)}** · passing: **{passed}/{len(results)}**",
        "",
        "| Program | Contributions fused | check | run |",
        "|---|---|:--:|:--:|",
    ]
    for r in results:
        lines.append(
            f"| `{r.id}` | {', '.join(r.contributions)} "
            f"| {'PASS' if r.check_ok else 'FAIL'} | {'PASS' if r.run_ok else 'FAIL'} |"
        )
    lines += ["", f"Gate: **{'PASS' if passed == len(results) else 'FAIL'}**"]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--garnet", help="path to the garnet CLI (default: auto-detect)")
    parser.add_argument("--format", choices=["markdown", "json"], default="markdown")
    args = parser.parse_args(argv)

    results = run_matrix(resolve_garnet(args.garnet))
    if args.format == "json":
        print(json.dumps([asdict(r) for r in results], indent=2))
    else:
        print(render_markdown(results))
    return 0 if all(r.ok for r in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
