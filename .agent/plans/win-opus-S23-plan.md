# win-opus — S23 Plan: `std::process` structured argv + output capture

**Slot:** win-opus · **Slice:** S23 (Jon-directed; closes an S22 deferred line) ·
**Branch:** `agent-win-opus/s23-process-runtime` (off `origin/main` `7f27f91`)
**Baseline:** `cargo test --workspace` + `cargo clippy -D warnings` exit 0; readiness
82.9% / 34 lanes, `--check-no-regression` exit 0.

## Directive (Jon)

Continue the post-v0.7 runtime-completion cadence: read PR#237/S22, confirm nothing
is left, then drive S23 to full completion (and onward to S24/S25). Each slice closes
the previous slice's documented deferred surface with real behavioral tests.

## What S22 left (the S23 source of truth)

S22's own readiness lane + `garnet-stdlib/src/process.rs:5` document the deferral:

> `std::process::spawn` still uses the v0.7 whitespace-delimited command-line
> contract from `garnet_stdlib`; richer argv handling is v0.8+.

Today `std::process::spawn(cmdline)` splits a single string on ASCII whitespace
(`process.rs:31`) — so an argument containing spaces is silently re-split, and there
is **no stdout/stderr capture** at all (`wait` runs `wait_with_output()` then discards
the captured output, keeping only the exit code). For an agentic/system-programming
language the single most useful process primitive — "run a command, get its output" —
is missing.

## Scope (closes the deferral)

Extend `garnet-stdlib` (win-opus-owned since S17) + `garnet-interp-v0.3` (unowned;
taken under Jon's direction since S21). **No parser/CST change** — the new prim names
(`spawn_args`, `output`) are plain identifiers, so they parse as path segments via the
existing fallback (mac-opus's parser/cst untouched).

### 1. `garnet-stdlib/src/process.rs`
- `spawn_args(program: &str, args: &[String]) -> Result<Proc, StdError>` — explicit
  argv vector; the program and each arg are passed literally (no shell splitting), so
  arguments with spaces survive. Empty `program` → `InvalidInput`.
- `output(program: &str, args: &[String]) -> Result<Output, StdError>` — run to
  completion via `Command::output()`, capturing `stdout`/`stderr` (lossy-UTF-8) and the
  exit code. `Output { code: Option<i32>, stdout: String, stderr: String }` with
  `code()/stdout()/stderr()` accessors.
- Keep `spawn`/`wait`/`exit_code` byte-for-byte (backward compatible).
- Rust unit tests: stdout capture + exit code (echo marker → contains marker, code 0;
  nonzero-exit command → that code); spaced-argument integrity (a single arg containing
  a space round-trips as ONE argv element); empty-program error.

### 2. `garnet-stdlib/src/registry.rs`
- Register `std::process::spawn_args` and `std::process::output`: Layer 1, cap `proc`,
  `@stability(experimental)` (keeps the layer gate ≥95% `@stability` + the checker's
  authoritative required-caps set correct).

### 3. `garnet-interp-v0.3/src/stdlib_bridge.rs`
- `bridge_process_spawn_args` (arity 2: `Str`, `Array<Str>`) → `Value::Process`.
- `bridge_process_output` (arity 2: `Str`, `Array<Str>`) → `Value::Map { "code", "stdout",
  "stderr" }` (ergonomic via `.get`; no new `Value` variant needed).
- Register both under their fully-qualified names; bridge unit tests.

### 4. `garnet-interp-v0.3/tests/stdlib_s23_dispatch.rs`
- Source-level proof: a `@caps(proc)` `main` calling `std::process::output(...)` asserts
  the captured stdout contains the marker and `code == 0`; `spawn_args` + `wait` +
  `exit_code` round-trips; spaced argument preserved.

### 5. Readiness + cross-cutting + PR
- New lane `process_runtime_completion` (status driven by the S23 dispatch tests);
  baseline surgically extended (per-lane floors + `source` preserved).
- CHANGELOG `[Unreleased]`, CURRENT_STATE section, S23 contract block in
  `GARNET_v0_7_SLICE_DOGFOOD.md`, ledger STARTED/PR-OPEN/REVIEW.
- Fork PR (cached cred is `Navigata1`/fork-only) → grep-loop to 5/5 → CI green → **merge
  via the org-write Chrome work profile** (Jon authorized browser-control merge for this
  goal).

## Test proportion (~60/40)
"Code" = the two stdlib fns + Output type + two bridge trampolines + registry. "Test" =
the Rust behavioral unit tests (capture/exit/argv integrity) + the source-level dispatch
tests (real `garnet run` semantics, asserted values) — behavior, not string-presence.

## Novel discovery to surface
Managed Garnet can now **run a host command and consume its captured output** with
structured argv — the foundation for agentic tool-use (shelling out to `git`, linters,
formatters) from a capability-checked managed program.

## Honest scope
- Process stdout/stderr are host-dependent (line endings, locale); the deterministic
  proof asserts substring/exit-code, not byte-exact full output. A byte-exact composed
  program (newline-normalized) is the S25 capstone, not S23.
- Still synchronous managed-mode execution; no async/OS-thread or streaming-stdout claim.
- Capability enforcement remains the checker's job (static, registry-driven); the
  interpreter trusts the checker, unchanged by S23.

## Dogfood block
```bash
cargo build -p garnet-cli
cargo test -p garnet-stdlib process --no-fail-fast
cargo test -p garnet-interp --test stdlib_s23_dispatch --no-fail-fast
cargo test -p garnet-interp stdlib_bridge --no-fail-fast
cargo fmt --all -- --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
python3 scripts/smoke_garnet_novel_compositions.py
```
