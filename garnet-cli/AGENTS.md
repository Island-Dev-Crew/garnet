# AGENTS.md — CLI Contract

## Scope

Owns the `garnet` binary, subcommand routing, template embedding, deterministic manifests, project scaffolding, formatting/docs commands, parse-mode routing, and user-facing command text.

## Stable Contracts

- CLI output must be truthful about release readiness and installer availability.
- Templates are embedded with `include_str!`; adding a template file requires adding it to `new_cmd.rs` or it will not ship.
- Public commands should fail clearly with actionable errors.
- `garnet check` is allowed to fail safe / `@bounded` programs with
  `check.bounded_loop` when loop bounds are not statically derivable. The
  message must preserve the static-only boundary and must not imply Wasmtime
  fuel or runtime loop enforcement.
- `garnet parse` defaults to AST mode. `garnet parse --mode cst <file>` routes
  to the canonical rowan `garnet-cst` parser and must report round-trip truth
  and recorded CST errors honestly.
- `garnet diff-caps` human text output and exit codes (0 = no expansion,
  1 = authority expanded, 2 = usage/parse error) are load-bearing for CI
  scripts and integration tests — byte-stable, never reworded casually.
  `--machine` (RB-1, Directive 15) is purely additive: a deterministic
  single-line JSON verdict (`garnet.diff-caps.machine/1`) with identical
  exit codes, scoped to the declared surface only (no bounds-delta claim;
  bound annotations are not part of the caps surface). On exit 2
  (usage/parse error) no JSON is emitted — stdout empty, error on stderr.
- Deterministic build/verify behavior must stay reproducible.
- Crash surface (RB-2): `src/lib.rs` AND `src/bin/garnet.rs` carry
  `#![deny(clippy::unwrap_used, clippy::expect_used)]` (tests exempt via
  `cfg_attr`). Sanctioned escapes: in-line `// INVARIANT:` allows, plus the
  ONE `// FAIL-CLOSED:` abort (`machine_key.rs` — cache integrity must not
  fail open; not an invariant, a documented contract). The
  malformed-corpus smoke (`tests/malformed_corpus_smoke.rs` +
  `tests/fixtures/malformed/`) asserts controlled 0/1/2 exits over check +
  both backends; keep it green and terminating (no unbounded recursion in
  the corpus — that is the S99 opt-in-ceiling boundary).
- New agent-documentation tooling should start as opt-in or checking behavior before becoming a language requirement.

## Required Checks

```sh
cargo test -p garnet-cli
cargo run -p garnet-cli -- --help
```

For template changes, create each template and run `garnet test` inside it when possible.

## Child Contracts

- `/garnet-cli/templates/AGENTS.md` owns scaffolded project-template expectations.
