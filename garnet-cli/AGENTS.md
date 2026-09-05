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
- Dependency preload (`garnet run --interp`) is FAIL-CLOSED for the complete
  setup boundary (S114 acceptance, cond. #5; Lane 1 fail-soft repair): an
  unreadable or malformed dependency table, missing/non-directory/unwalkable
  declared vendor path, unreadable/non-regular vendored source, source
  parse/load failure, or authority trap aborts before user `main` and returns
  non-zero. None of these conditions may warn/skip and continue green. A bare
  file with no project manifest and a project with an empty dependency table
  remain valid controls. `Garnet.toml` is parsed as TOML 1.0, including quoted
  or spaced headers, dotted keys, inline top-level tables, and dependency
  subtables; each semantic dependency must contain exactly the path+vendor or
  registry+version+vendor string keys. Malformed or duplicate keys are RED.
  Each dependency named `<name>` is bound only to
  `.garnet/vendor/<name>`. Mismatched, dot, parent/traversing, absolute,
  symlinked, or otherwise escaping vendor paths are setup failures. Vendored
  source bytes are read from an identity-checked retained file handle and are
  never reopened after validation.
- `garnet test` discovery and helper setup are likewise FAIL-CLOSED. An existing
  project root must first resolve to a readable directory (an explicit missing
  path is not an empty project). An existing `tests` path that is not a readable
  directory, any per-entry enumeration or metadata failure, and any existing
  `src/main.garnet` that is unreadable or non-regular returns non-zero; helper
  parse/load failure fails the affected tests. Only an absent input under a
  validated project root (or the explicit `--no-main` opt-out) is omission, and
  setup failure must never become a green no-files or partial-context run.
  At most one positional project root is accepted; a second is a usage error.
  Discovered test and helper sources use the same identity-bound retained-handle
  read as vendored sources, so a path swap cannot change later-loaded bytes.
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
- The shared collector (`cmd::verify_gate::collect_targets`) must not hide
  DECLARED authority behind a directory name (crown C B-1). Skips are matched
  by NAME — a convention, not a verified ownership fact: `target`, `.git`,
  `.garnet-cache` at any depth, plus the ROOT-RELATIVE `<root>/.garnet/vendor`
  — the one path a dependency may bind to. An arbitrary `vendor/` or
  `node_modules/` at any depth is ordinary source and IS walked. A directory
  symlink met below the supplied root is not followed (a link loop must
  terminate) and is tallied under `symlinked-directory`; a linked `.garnet`
  FILE is read through the link; a link the walk cannot resolve (permission
  denied, a loop) is an error with no verdict, never a zero; the supplied root
  itself is resolved by the OS and walked. Every skip and every declined link
  is disclosed: `--machine` carries
  `skipped_path_count` + `skipped_paths` (rule names and counts only, never
  paths), additive within `garnet.diff-caps.machine/1`, and the human mode
  prints one `walk not total` line when the count is non-zero (byte-stable
  when it is zero). `skipped_path_count: 0` asserts that every directory the
  walk reached was read or tallied; ABSENCE of the field means a pre-cure
  binary and an UNKNOWN walk. `caps`, `verify` and `sandbox-policy` inherit the
  walk but do not yet surface the tally.
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
- `garnet_cli::mcp` is the transport-free lifecycle/session boundary. It enforces
  initialize-first state, mandatory ping, initialized readiness, bounded exact
  request IDs, and explicit respond/no-response/close actions. Its default
  session keeps empty capabilities and method-not-found responses; the explicit
  application callback lets a bounded host advertise and handle only its own
  methods after readiness. Do not put transport, interpreter execution, or
  authority logic in the lifecycle module.
- `garnet_cli::minimum_shelf` freezes Core Ring Tier 1 to exactly one
  Garnet-owned tool, `garnet.core.double`, with the exact input object
  `{ "value": <i64> }`. The implementation invokes Garnet in-process through
  the strict interpreter and panic firewall. Its MCP application host exposes
  only `tools/list` and `tools/call` after the released lifecycle is ready.
  Do not widen this module into a hosted registry, arbitrary source runner,
  network host, or Tier 2/3 surface; package sealing and raw-byte framing remain
  separate fail-closed boundaries.
- `garnet mcp-serve --package <dir>` is the only production constructor for
  the Minimum Shelf host. It accepts only the exact repo-bundled flagship:
  package manifest, source, and unsigned in-toto predicate bytes are BLAKE3
  pinned; source AST/build/capability bindings are re-derived; paths must be
  regular non-symlink files. A locally edited or freshly resealed lookalike is
  rejected before the host or interpreter tool is constructed. The unsigned
  predicate is content provenance, not an external identity signature.
- `garnet_cli::mcp_stdio` is the bounded byte framer for Minimum Shelf. It
  accepts exactly one canonical `Content-Length: N\r\n\r\n` header, caps header
  and body sizes, rejects text-mode LF framing, and emits framed JSON-RPC parse
  errors. Never route it through line or text readers. The real process entry
  must put Windows stdin/stdout in binary mode before serving.
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
