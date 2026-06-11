# AGENTS.md — xtask Contract

## Scope

Owns repository automation that is too project-specific for ordinary cargo commands.

## Stable Contracts

- Keep xtask commands deterministic and CI-friendly.
- Prefer explicit named tasks over hidden side effects.
- If an xtask becomes a user-facing product command, promote it into `garnet-cli` with tests.

## Commands

- `seven-run` — 7× `cargo test --workspace` consistency harness; non-zero on
  pass/fail-count divergence.
- `truth` (RB-0a) — regenerates `docs/truth.json` from machine-derivable
  sources only (workspace `Cargo.toml` version, `all_prims()` counts, the
  readiness reporters, `git tag`, a measured `cargo test --workspace` run)
  and stamps `<!-- truth:KEY -->VALUE<!-- /truth -->` markers in `README.md`
  and `FAQ.md`. `--skip-tests` carries the previous test measurement forward
  with its provenance.
- `truth --check` — non-zero exit on any mismatch between live machine truth,
  `docs/truth.json`, and the stamped surfaces (`--with-tests` re-measures the
  suite). Invariants: every truth.json field is derived, never hand-entered;
  unknown marker keys and unterminated markers are hard errors, never
  silently skipped; `security_test_count` is deliberately omitted (no trusted
  derivation exists — the omission is recorded inside `docs/truth.json`).
  Wiring `--check` into CI is a gate change → Jon-gated, propose only.

## Required Checks

```sh
cargo test -p xtask
```
