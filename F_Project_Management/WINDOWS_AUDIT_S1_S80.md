# Garnet Windows audit — S1–S80 (tracked burn-down ledger)

This is the **committed source of truth** for the Codex Windows audit of the
S1–S80 run (read-only audit against merged `main` @ `cc165e8`, where `v0.8.0` is
tagged by Jon Isaac, 2026-05-31). The machine ledgers it summarizes are committed
beside it: `.dogfood/windows-core-audit.json` (S1–S30) and
`.dogfood/windows-audit-goal.json` (S31–S80).

It exists so the **v0.8.1 runway burn-down (S81–S88)** has a tracked ledger: every
open `WIN-*` finding maps to the slice that closes (or honestly defers) it.
`scripts/garnet_windows_audit_status.py --gate` enforces that mapping.

## Verification highlights (from the audit)

- `git` fast-forward reached `cc165e8` on `origin/main`; `v0.8.0` tag points at HEAD.
- `cargo test --workspace --no-fail-fast`: green on Windows.
- `cargo clippy --workspace --all-targets -- -D warnings`: green on Windows.
- S72, S74, S75, S76, S77, S78, S79 direct gates pass on Windows.
- **S71 and S73 direct binary/provider-free Windows gates FAIL**; the **S80
  aggregate passes only because it uses `--no-run`** for binary-dependent gates.
- Studio/domain matrix 20/20 cases + 60/60 commands (previously verified); novel
  compositions 7/7 (previously verified).

## Open findings → owning burn-down slice (all 14)

| Finding | Severity | Audited slice | Owning slice | Summary |
|---|---|---|---|---|
| WIN-S33-001 | high | S33 | **S81** | `garnet verify <dir>` ignores uppercase `.GARNET` on Windows (case-sensitive collector). |
| WIN-S36-001 | high | S36 | **S81** | Capability manifests silently ignore uppercase `.GARNET`. |
| WIN-S37-001 | high | S37 | **S81** | `diff-caps` misses authority expansion from uppercase `.GARNET`. |
| WIN-S46-001 | high | S46 | **S81** | Sandbox-policy generation inherits the uppercase `.GARNET` discovery miss. |
| WIN-S38-001 | high | S38 | **S82** | Seal full predicate changes LF↔CRLF because `source_blake3` hashes raw bytes (no `*.garnet` eol pin). |
| WIN-S80-002 | medium | S80 | **S83** | Post-tag release truth split: `v0.8.0` tagged on HEAD, but S80 docs/ledger still read pre-tag/pending. |
| WIN-S71-001 | high | S71 | **S84** | Paper VI Exp 3 reporter passes Windows absolute paths to WSL bash (exit 127). |
| WIN-S73-001 | high | S73 | **S85** | VM/interpreter parity diverges on Windows: interpreter stack-overflows on `mvp_function_call_demo.garnet` (VM succeeds). |
| WIN-S80-001 | high | S80 | **S86** | S80 cut-readiness reports READY while direct Windows binary dogfood for S71/S73 fails (`--no-run`). |
| WIN-S6-001 | medium | S6 | **S87** | Memory-eviction reporter Markdown can fail on default Windows cp1252 stdout. |
| WIN-S31-001 | high | S31 | **S87** | MIT readiness reporter aborts when temp fixtures hit a denied Windows temp dir. |
| WIN-S31-002 | advisory | S31 | **S87** | Full readiness JSON/MD is machine-specific; cross-machine byte comparison needs a committed-only surface. |
| WIN-S38-002 | medium | S38 | **S88** | `cosign`/`syft`/`cyclonedx` absent on the Windows machine — signing/SBOM unverifiable there. |
| WIN-S39-001 | pending-infra | S39 | **S88** (+ **S89**) | Bounded execution is declared/report-only on Windows; no local Wasmtime fuel/epoch proof. S89 begins real in-engine enforcement. |

## Resolved since the prior audit

- **WIN-S70-001** — prior ledger-drift finding; resolved on current `main` (the S70
  version-map correction landed).

## Burn-down rules

- **S81–S88 fix the audit-proven trust + Windows/runtime gaps first**, ordered by
  blast radius (S81's one `.GARNET` fix clears four).
- A **`WINDOWS-PROVE`** slice is **never marked done from the Mac lane** — it is
  recorded here as `Mac-authored, Windows-proof-pending` with its named Windows
  proof command, for the Windows lane to verify and check off.
- Honest scope is preserved on every slice (no production/1.0 claim; `v0.8.1` is a
  research-grade-prototype runway).

## Windows-proof status (updated by the Windows lane)

| Slice | Windows proof command | Status |
|---|---|---|
| S81 | `garnet verify <dir: clean main.garnet + parse-broken BAD.GARNET>` → exit 1 | **Mac fix landed** (case-insensitive shared collector + unit test); Windows-proof-pending |
| S82 | fresh Windows checkout → `garnet seal <file>` `source_blake3` matches Mac | **Mac fix landed** (LF-normalized source hash + `.gitattributes` pin + LF/CRLF unit test); Windows-proof-pending |
| S84 | `python scripts/test_garnet_paper_vi_exp3_status.py` → 6/6 on Windows | pending |
| S85 | `garnet.exe run --interp examples/mvp_function_call_demo.garnet` → exit 0 `=> 7105`; parity 33/33 | **Mac fix landed** (interpreter on a 256 MiB thread + cross-OS integration tests); Windows-proof-pending |
| S89 | over-ceiling `@bounded` fixture traps deterministically on Windows | pending |
| S90 | undeclared-`@caps` fixture traps identically on Windows | pending |
