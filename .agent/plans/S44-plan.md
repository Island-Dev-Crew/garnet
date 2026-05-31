# S44 Plan — LSP safe-mode / cross-package precision

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S44.
Map: reconciled plan §145 — "LSP safe-mode / cross-package precision (semantic
service; distinct from tree-sitter)."
Branch: `codex/s44-lsp-precision`. Base: `origin/main` @ `c063cc8` (S43).

## Problem (found read-only)
`garnet-lsp::check_diagnostics` mapped every `CheckError` except `BoundaryNote`
to `ERROR` and set no `code`. The S42 over-catch advisory + stability-advice
therefore showed as red errors in editors, and codes diverged from
`garnet check --format json` (S34). Severity/code knowledge was duplicated.

## Slice (one mechanism: canonical severity+code, two consumers)
- `garnet-check`: add `pub enum Severity { Error, Warning, Info }` and
  `impl CheckError { pub fn severity(&self) -> Severity; pub fn code(&self) -> &'static str }`
  — exhaustive matches, the single source of truth.
- `garnet-cli/diagnostics.rs` (S34): `from_check_error` delegates to
  `err.severity().into()` + `err.code()` (`From<garnet_check::Severity>` for the
  CLI `Severity`). Output unchanged → S34 tests stay green.
- `garnet-lsp/lib.rs`: `check_diagnostics` maps `err.severity()` →
  `DiagnosticSeverity` and sets `Diagnostic.code = err.code()`.

## Dogfood
- garnet-check: `severity_and_code_are_canonical` (table) +
  `error_severity_agrees_with_fatal_set` (Error ⇔ flips `ok`).
- garnet-lsp: over-catch → INFORMATION + `check.over_catch`; safe-mode → ERROR
  + `check.safe_mode_violation`.

## End-state / gates
- Full ladder green; CHANGELOG + contract S44 block. Ledger: `s43 → merged(5)`
  advanced in this branch; `s44` advance rides with the S45 PR.

## Honest scope
- Delivers the **safe-mode precision** half (CLI/LSP diagnostic parity).
- **Cross-package precision deferred to S45** (the package/module resolver) —
  the LSP has no module resolver today; no cross-file resolution is claimed.
- No new readiness lane (not mandated).
