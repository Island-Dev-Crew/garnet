# win-opus — S24 Plan: `std::log` file sink (`@caps(fs)`)

**Slot:** win-opus · **Slice:** S24 (Jon-directed; closes the S23/S22/S17 deferred line) ·
**Branch:** `agent-win-opus/s24-log-filesink` (off `origin/main` `cc69b60`, S23 merged)
**Baseline:** readiness 83.4% / 35 lanes, `--check-no-regression` exit 0.

## What S23/S22/S17 left (the S24 source of truth)

Three lanes name the same deferral: S22's lane and `garnet-stdlib/src/log.rs:3`
("Routing those lines to a file sink needs `@caps(fs)` and is deferred to v0.8"),
and the S17 lane's deferred list ("the `@caps(fs)` file-sink path of `std::log`").
Today `std::log::{info,warn,error,debug}` only *format* a `[LEVEL] message` string;
nothing writes it anywhere.

## Scope (closes the deferral)

Edits `garnet-stdlib` (log module + registry — win-opus-owned) + `garnet-interp-v0.3`
(bridge dispatch — Jon-directed). **No parser/CST change** (`to_file` is a plain
identifier).

### 1. `garnet-stdlib/src/log.rs`
- `to_file(path, level, message) -> Result<String, StdError>`: formats the same
  `[LEVEL] message` line, **appends** it + `\n` to `path` (create-if-missing, append
  mode), and returns the formatted line. IO failures → `StdError::Io`.
- Keep `info`/`warn`/`error`/`debug` (formatting) unchanged.
- Rust unit tests: append three lines to a unique temp file, read back, assert exact
  ordered contents + format; appends don't truncate; error on an unwritable path (a
  directory).

### 2. `garnet-stdlib/src/registry.rs`
- Register `std::log::to_file`: Layer 1, cap `fs` (it touches the filesystem),
  `@stability(experimental)`.

### 3. `garnet-interp-v0.3/src/stdlib_bridge.rs`
- `bridge_log_to_file` (arity 3: path, level, message) → returns the formatted line as
  `Value::Str`; register under `std::log::to_file`. Bridge unit test.

### 4. `garnet-interp-v0.3/tests/stdlib_s24_dispatch.rs`
- `@caps(fs)` Garnet `main` writes two lines via `std::log::to_file(path, ...)`, then
  reads them back with `read_file(path)` and asserts both lines are present in order —
  an end-to-end file-sink proof through the interpreter.

### 5. Readiness + cross-cutting + PR
- New lane `log_file_sink_runtime` (verified when artifacts present); baseline
  surgically extended.
- CHANGELOG, CURRENT_STATE, S24 contract block in `GARNET_v0_7_SLICE_DOGFOOD.md`, ledger
  (record S23 MERGED `cc69b60` + S24 STARTED).
- Fork PR → grep-loop to 5/5 → CI green → merge via the org-write Chrome work profile.

## Test proportion (~60/40)
"Code" = the `to_file` fn + the bridge trampoline + registry. "Test" = the Rust
append/read-back behavioral assertions + the source-level end-to-end write→read proof.

## Novel discovery
Managed Garnet can now **persist structured logs to disk** under an `@caps(fs)`
authority — closing the loop from S23's process-output capture to durable, capability-
checked observability.

## Honest scope
- File sink is line-append text (create-if-missing); no rotation, structured/JSON sinks,
  or async writers (those are later).
- Capability enforcement remains the checker's job (registry tags `to_file` `@caps(fs)`).

## Dogfood block
```bash
cargo build -p garnet-cli
cargo test -p garnet-stdlib log --no-fail-fast
cargo test -p garnet-interp --test stdlib_s24_dispatch --no-fail-fast
cargo test -p garnet-interp stdlib_bridge --no-fail-fast
cargo fmt --all -- --check ; cargo clippy --workspace --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
python3 scripts/smoke_garnet_novel_compositions.py
```
