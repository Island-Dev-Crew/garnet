#!/usr/bin/env python3
"""Rolling S114 trust-kernel review gate (S114 acceptance, phase P4).

S114 (the enforced trust-kernel red-team) must not become a one-time ceremony.
This gate makes it a recurring control: when a change touches the **trust
kernel** — the checker, interpreter, VM, stdlib bridges/registry, wasm runner,
CLI authority flows, the capability reporters, or the public `/why` claims and
the enforcement scope table — the change must be **accompanied by a review
companion**: a scoped S114 acceptance update, a fresh review/verdict artifact,
or an explicit `Trust-Kernel-Review:` commit trailer naming who reviewed.

"No self-grading" stays an operational gate, not a slogan: the trailer names a
reviewer; a bare code change to the trust spine with no review companion fails
`--gate`.

This gate REPORTS by default. `--gate` exits 1 when the trust kernel is touched
without a review companion. Wiring it into CI is a workflow change (human-merge-
only per CLAUDE.md integrity rule 1); until then it is a local/manual control.

Diff source (mirrors scripts/check_dogfood_pr_body.py):
  --changed-file PATH ...   explicit changed paths (bypasses git; repeatable)
  --base REF --head REF     git diff REF...REF (three-dot) for the changed set
  (default)                 git diff against merge-base(HEAD, origin/main)
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

SCHEMA = "garnet.trust_kernel_review/v1"
ROOT = Path(__file__).resolve().parents[1]

# --- The machine-readable trust-kernel file set (the S114 trigger surface). ---
# Any changed path under these prefixes, or matching these exact files, is a
# trust-kernel change.
TRUST_KERNEL_PREFIXES = (
    "garnet-check-v0.3/src/",       # checker: capability_surface, caps_graph, capset
    "garnet-interp-v0.3/src/",      # interpreter: eval, stdlib_bridge, lib (latch/strict)
    "garnet-vm/src/",               # VM: scope parity + entry-frame install
    "garnet-stdlib/src/",           # capability registry (single source of truth)
    "garnet-wasm/src/",             # wasm runner authority surface
)
TRUST_KERNEL_FILES = (
    "garnet-cli/src/cmd/run.rs",            # run lane + dependency preload
    "garnet-cli/src/cmd/test.rs",           # test runner entry authority + helper preload
    "garnet-cli/src/cmd/eval.rs",           # eval lane
    "garnet-cli/src/cmd/doctest.rs",        # doctest lane
    "garnet-cli/src/bin/garnet.rs",         # the strict-no-frame latch site
    "scripts/garnet_launch_readiness_status.py",   # launch/S114 reporter
    "scripts/garnet_caps_enforcement_status.py",   # caps enforcement gate
    "scripts/garnet_capability_scope_status.py",   # claim-scope fixture
    "scripts/garnet_bounded_enforcement_status.py",  # @max_depth gate
    "scripts/garnet_red_team_status.py",           # red-team static gate
    "docs/why.html",                               # public enforced claims
    "C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md",  # scope fence
)

# --- What counts as a review companion accompanying a trust-kernel change. ---
REVIEW_COMPANION_PREFIXES = (
    "proofs/independent/s114/",             # a landed/updated independent bundle
    "F_Project_Management/W_TRUST/",        # a review/verdict/acceptance note
    "F_Project_Management/VALIDATION_REPORTS/",  # a lane review report
)
REVIEW_COMPANION_FILES = (
    "F_Project_Management/LAUNCH/S114_ACCEPTANCE.json",  # scoped acceptance re-affirmed
)
# A commit trailer that attests a review happened (names the reviewer).
REVIEW_TRAILER = "Trust-Kernel-Review:"


@dataclass
class TrustKernelReviewStatus:
    schema: str
    ok: bool
    trust_kernel_touched: bool
    touched_paths: list[str] = field(default_factory=list)
    review_companion_present: bool = False
    companion_paths: list[str] = field(default_factory=list)
    review_trailer_present: bool = False
    changed_count: int = 0
    problems: list[str] = field(default_factory=list)
    trust_kernel_prefixes: list[str] = field(default_factory=lambda: list(TRUST_KERNEL_PREFIXES))
    trust_kernel_files: list[str] = field(default_factory=lambda: list(TRUST_KERNEL_FILES))


def _norm(path: str) -> str:
    # Strip leading "./" as a prefix — NOT lstrip("./"), which strips '.' and
    # '/' as a character set and eats the dot off ".github/..."-style names
    # (harmless for this trigger set today, but the pattern was copied from the
    # dogfood gate where it silently skipped workflow files).
    p = path.replace("\\", "/")
    while p.startswith("./"):
        p = p[2:]
    return p


def is_trust_kernel(path: str) -> bool:
    p = _norm(path)
    return p in TRUST_KERNEL_FILES or p.startswith(TRUST_KERNEL_PREFIXES)


def is_review_companion(path: str) -> bool:
    p = _norm(path)
    return p in REVIEW_COMPANION_FILES or p.startswith(REVIEW_COMPANION_PREFIXES)


def _git(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(["git", *args], capture_output=True, text=True, cwd=ROOT)


def _changed_from_git(base: str | None, head: str) -> list[str]:
    if base is None:
        mb = _git("merge-base", head, "origin/main")
        if mb.returncode != 0:
            return []
        base = mb.stdout.strip()
    if not base:
        return []
    diff = _git("diff", "--name-only", f"{base}...{head}")
    if diff.returncode != 0:
        return []
    return [ln for ln in diff.stdout.splitlines() if ln.strip()]


def _trailer_present(base: str | None, head: str) -> bool:
    if base is None:
        mb = _git("merge-base", head, "origin/main")
        base = mb.stdout.strip() if mb.returncode == 0 else ""
    if not base:
        return False
    log = _git("log", f"{base}..{head}", "--format=%B")
    return REVIEW_TRAILER.lower() in log.stdout.lower()


def read_status(
    changed: list[str] | None = None,
    base: str | None = None,
    head: str = "HEAD",
    trailer: bool | None = None,
) -> TrustKernelReviewStatus:
    if changed is None:
        changed = _changed_from_git(base, head)
    touched = sorted({_norm(p) for p in changed if is_trust_kernel(p)})
    companions = sorted({_norm(p) for p in changed if is_review_companion(p)})
    trailer_present = trailer if trailer is not None else _trailer_present(base, head)
    companion_present = bool(companions) or trailer_present

    problems: list[str] = []
    if touched and not companion_present:
        problems.append(
            f"{len(touched)} trust-kernel path(s) changed with no review companion "
            f"(a scoped S114 acceptance update, a proofs/independent/s114 or W_TRUST or "
            f"VALIDATION_REPORTS artifact, or a '{REVIEW_TRAILER} <reviewer>' commit trailer): "
            + ", ".join(touched[:8])
            + (" ..." if len(touched) > 8 else "")
        )

    return TrustKernelReviewStatus(
        schema=SCHEMA,
        ok=not problems,
        trust_kernel_touched=bool(touched),
        touched_paths=touched,
        review_companion_present=companion_present,
        companion_paths=companions,
        review_trailer_present=trailer_present,
        changed_count=len(changed),
        problems=problems,
    )


def render_markdown(s: TrustKernelReviewStatus) -> str:
    lines = [
        "# Garnet rolling S114 trust-kernel review status",
        "",
        f"_Schema {s.schema}._",
        "",
        f"- changed paths inspected: {s.changed_count}",
        f"- trust-kernel touched: **{s.trust_kernel_touched}**",
        f"- review companion present: **{s.review_companion_present}** "
        f"(files: {len(s.companion_paths)}, trailer: {s.review_trailer_present})",
        f"- overall: **{'ok' if s.ok else 'REVIEW REQUIRED'}**",
    ]
    for p in s.touched_paths:
        lines.append(f"  - trust-kernel: {p}")
    for p in s.problems:
        lines.append(f"  - PROBLEM: {p}")
    lines.append("")
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument("--base", default=None, help="base ref/SHA for git diff")
    parser.add_argument("--head", default="HEAD", help="head ref/SHA for git diff")
    parser.add_argument(
        "--changed-file",
        action="append",
        default=None,
        dest="changed_files",
        help="explicit changed path (repeatable); bypasses git diff",
    )
    parser.add_argument(
        "--assume-trailer",
        action="store_true",
        help="treat a review trailer as present (for testing / manual attestation)",
    )
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit 1 when the trust kernel is touched without a review companion",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    trailer = True if args.assume_trailer else (False if args.changed_files is not None else None)
    s = read_status(
        changed=args.changed_files,
        base=args.base,
        head=args.head,
        trailer=trailer,
    )
    print(render_markdown(s) if args.format == "md" else json.dumps(asdict(s), indent=2))
    if args.gate and not s.ok:
        print("trust-kernel review gate: REVIEW REQUIRED", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
