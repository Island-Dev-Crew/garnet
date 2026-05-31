# S34 Plan — structured diagnostics (machine + human)

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S34.
Branch: `codex/s34-diagnostics`.

## Goal
Diagnostics with BOTH a human-readable and a machine-parseable form, an
authoritative exit code, and the structured type that a future MCP/LSP server
serves.

## Design (bounded, honest)
- **`garnet-cli/src/diagnostics.rs`** (new): the reusable structured form.
  - `Severity { Error, Warning, Info }` (+ stable lowercase wire names).
  - `Diagnostic { severity, code: &'static str, message: String, span: Option<(usize, usize)> }`.
  - `from_parse_error(&ParseError) -> Diagnostic` — per-variant stable `code`
    (`parse.unexpected_char`, …) + the variant span (every `ParseError` carries one).
  - `from_check_report(&CheckReport) -> Vec<Diagnostic>` — per-`CheckError`-variant
    stable `code` (`check.safe_mode_violation`, `check.caps_coverage`, …) and
    severity (the two non-fatal variants → Warning/Info, the rest → Error).
    Check spans are `None` today (the `CheckError` variants are message-only) — an
    explicitly recorded honest partial.
  - `to_json(&[Diagnostic]) -> String` — deterministic, hand-rolled JSON (no
    `serde`, matching `manifest.rs`'s determinism stance) with correct string
    escaping. Stable field order; array order = diagnostic order.
- **`garnet check [--format human|json]`** (default `human`): thread a `Format`
  into `cmd::check::run`. `human` = today's miette/Display output (unchanged).
  `json` = emit `{ "diagnostics": [...], "summary": {...} }` to stdout and
  suppress the human chatter.
- **Authoritative exit code** (documented in `diagnostics.rs` + `print_help`):
  `0` = no fatal diagnostics, `1` = ≥1 fatal diagnostic / parse / IO error,
  `2` = usage. `garnet check` already follows this; S34 makes it the *named*
  contract and keeps `--format json` consistent with it.

## Crates touched (writable)
- `garnet-cli`: new `diagnostics.rs`, `cmd/check.rs` (format flag + json branch),
  dispatcher (`--format` parse), `print_help`, new integration test.
- `garnet-check-v0.3`, `garnet-parser-v0.3` — **read-only** (consume
  `CheckReport`/`CheckError`/`ParseError`).

## Load-bearing dogfood (per contract)
- `garnet check --format json <clean>` → `[]` diagnostics, exit 0, valid JSON.
- `garnet check --format json <file-with-error>` → a diagnostic object with
  `severity:"error"` + a stable `code`, exit 1; output parses as JSON.
- `garnet check <file>` (no flag) → unchanged human output.
- JSON is byte-deterministic for the same source (proven by a unit test).

## End-state / gates
- No new readiness lane (the contract lane table does not mandate one for S34).
- `cargo fmt`/`clippy -D warnings`/`test --workspace` green; `cargo doc -D warnings`;
  `cargo deny`; `--check-no-regression` exit 0 (unchanged); conformance; python suites.
- CHANGELOG `[Unreleased]` + contract S34 state. Dogfood bundle → PR (Navigata1)
  → CLI-merge (IslandDevCrew) → `s34` advance rides with the S35 PR.

## Honest scope / out of scope
- Machine form is added to `garnet check` only in S34; `parse`/`verify` JSON and
  the actual MCP transport are follow-ups (S34 ships the structured TYPE + the
  one consumer).
- Check diagnostics have no spans yet (the `CheckError` variants are message-only);
  parse diagnostics do. Adding spans to the checker is a separate slice.
