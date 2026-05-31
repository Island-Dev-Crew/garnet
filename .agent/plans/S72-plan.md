# S72 — self-hosted parser seed

## Goal
Ship the self-hosting SEED: a Garnet program that parses a subset of Garnet's own
surface syntax. A maturity milestone on the v0.8 validation runway (S71–S80).

## What ships
- `examples/self_hosted_parser_seed.garnet` — parses `def name(params) { ... }`
  declarations from an embedded Garnet source string (name + arity + @caps managed
  flag), using only Stable, no-caps `str::` primitives (split/trim/replace/
  contains/starts_with) + indexing + for/if. Checks with 0 diagnostics; runs to:
    def main arity 0 caps yes
    def add arity 2 caps no
    def greet arity 1 caps no
    parsed defs: 3 managed: 1
- `scripts/garnet_self_hosted_parser_seed_status.py` (+ `--gate`) — static
  well-formedness + (when the binary is built) check-clean + run-matches.
- `scripts/test_garnet_self_hosted_parser_seed_status.py` — 5 unit tests.
- CI: canonical-examples job runs the binary-backed check+run proof;
  agent-contracts runs the static gate (`--no-run`) + tests.
- Spec `C_Language_Specification/GARNET_SELF_HOSTED_PARSER.md`; CHANGELOG;
  contract S72 block; this plan; ledger `s71 → merged`.

## Why this scope
The agent-contracts CI job is python-only (no compiler), so the reporter skips
the dynamic proof when the binary is absent and gates static well-formedness; the
real check+run proof runs in canonical-examples (which builds garnet). Used only
Stable str:: primitives to avoid experimental-primitive stability warnings (the
S76 cleanup target) so the seed checks cleanly.

## Verification
- `python3 scripts/test_garnet_self_hosted_parser_seed_status.py` → 5 OK.
- `garnet check`/`run` the seed → 0 diagnostics + expected output.
- Both gates rc 0 (dynamic w/ binary; static `--no-run`).
- Ladder: fmt/diff clean; `cargo test --workspace` 0 failed (no Rust changed).

## Honest scope (do not soften)
A SEED, NOT the production parser (garnet-parser-v0.3). No full AST/grammar
(nested braces, expressions, types, comments); neither replaces nor bootstraps the
Rust parser. Full self-hosting remains roadmap.
