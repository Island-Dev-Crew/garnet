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
- `garnet repl` (RB-7) hosts the `reedline` line editor and the REPL ergonomics
  (`?doc`, `:caps`, completion, multiline). **`reedline` stays a `garnet-cli`
  dependency ONLY** — never add a terminal line editor to `garnet-interp`, which
  must keep compiling to `wasm32-wasip1` (RB-6). The command dispatch is a pure,
  unit-tested core with a non-TTY plain fallback; keep it that way so behaviour
  is testable without a terminal. `:caps` reports a **declared/available**
  authority surface and must stay labeled NOT an enforced budget — `@caps` is
  enforced per-function at entry (S90), and a bare prompt call holds no
  capability frame; never let `:caps` imply a live runtime grant.
- `garnet run` must load user source under the selected program entry's
  `@caps` frame before evaluating top-level `let`/`const` initializers. A
  load-time host call in an initializer is still authority-bearing runtime
  behavior and must not run outside the entry capability gate on either backend.
- Dependency preload (`--interp`) and the `garnet test` `src/main.garnet`
  helper preload are FAIL-CLOSED on authority (S114 acceptance, cond. #5): an
  authority trap while loading a vendored dep aborts `garnet run` with a
  non-zero exit (only benign parse/read/missing-vendor errors stay
  warn-and-continue), and any helper-preload failure fails that test file's
  tests rather than reporting a green "N passed; 0 failed". Setup failure must
  never produce a success exit. The authority trap is identified by the stable
  `capability:` message prefix from `garnet-interp`'s `require_capability`.
- `garnet agent-loop` is a four-stage gate: `check` -> `diff-caps` -> `run` ->
  `seal`. A proposal that fails `garnet check` is rejected before runtime or
  sealing, and the seal-out path must remain absent on that rejection path.
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
- Panic containment: `garnet eval`, `garnet repl`, `garnet test`, and
  `garnet doctest` invoke the interpreter on the main thread behind the
  unwinding panic firewall (`src/panic_firewall.rs`) — an interpreter panic
  becomes a controlled diagnostic exit, never a raw process abort or a
  killed REPL session. The `run` lane instead uses spawn-and-join on a
  large-stack thread. Stack overflow and other aborting faults are outside
  `catch_unwind` and require structural guards (e.g. the cyclic-value
  render guard); do not claim the firewall contains them. A new
  interpreter-invoking lane must route through the firewall or a
  structural guard (`tests/panic_firewall_lanes.rs` proves the lanes).
- `garnet_cli::mcp_schema` is the released MCP `2025-11-25` initialize-schema
  boundary. It validates known client capability and implementation metadata
  shapes while allowing only top-level vendor capability extensions. It is not
  a session, transport, tool router, or host.
- `garnet_cli::mcp` is the pure lifecycle/session boundary. It enforces
  initialize-first state, mandatory ping, initialized readiness, bounded exact
  request IDs, and explicit respond/no-response/close actions. Empty advertised
  capabilities and method-not-found responses are honesty fences: do not route
  tools, stdio, interpreter execution, or authority through this module.
- `garnet_cli::minimum_shelf` freezes Core Ring Tier 1 to exactly one
  Garnet-owned tool, `garnet.core.double`, with the exact input object
  `{ "value": <i64> }`. The implementation invokes Garnet in-process through
  the strict interpreter and panic firewall. Do not widen this module into a
  hosted registry, arbitrary source runner, external MCP host, or Tier 2/3
  surface; package sealing and raw-byte transport remain separate boundaries.
- New agent-documentation tooling should start as opt-in or checking behavior before becoming a language requirement.

## Required Checks

```sh
cargo test -p garnet-cli
cargo test -p garnet-cli --test mcp_initialize_schema
cargo test -p garnet-cli --test mcp_protocol_adversarial
cargo run -p garnet-cli -- --help
```

For template changes, create each template and run `garnet test` inside it when possible.

## Child Contracts

- `/garnet-cli/templates/AGENTS.md` owns scaffolded project-template expectations.
