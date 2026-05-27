# win-opus — S26 Plan: `core::result` combinator dispatch

**Slot:** win-opus · **Slice:** S26 (Jon-directed; continues the S21/S22 registry→runtime arc) ·
**Branch:** `agent-win-opus/s26-result-dispatch` (off `origin/main` `0f2b8f4`, S25 merged)
**Baseline:** readiness 84.3% / 37 lanes, `--check-no-regression` exit 0.

## What's left (the S26 source of truth)

The S17 registry tags 6 `core::result` primitives (ok, err, map, and_then, or_else,
unwrap_or) but the interpreter dispatches **none** of them — the same gap S21/S22 closed
for `core::math/cmp/iter` and the `std::` families. `core::result::map` in particular
last-segment-collides with the bare `map` (Map constructor), so it needs a qualified
bridge (the S21 qualified-first pattern).

## Scope

Edits `garnet-interp-v0.3` (`stdlib_bridge.rs`) only — additive bridges + tests. **No
parser/CST change**; `garnet-stdlib` registry already has the entries (the checker knows
them). Result values are `Value::Variant { path:["Result"], variant:"Ok"/"Err" }`,
identical to the prelude `ok`/`err` builders, so pattern-matching and `?` agree.

- `core::result::ok|err` — build the Result Variant (explicit, qualified).
- `core::result::map(r, f)` — Ok(v)→Ok(f(v)); Err passes through. Higher-order.
- `core::result::and_then(r, f)` — Ok(v)→f(v) (callee returns a Result); Err short-circuits.
- `core::result::or_else(r, f)` — Err(e)→f(e); Ok passes through.
- `core::result::unwrap_or(r, default)` — Ok(v)→v; Err→default.
- `core::result_dispatch` readiness lane; baseline surgically extended.

## Test proportion (~60/40)
"Code" = 6 bridges + Result helpers. "Test" = 6 bridge unit tests (each combinator +
non-Result type error, higher-order via a test callable) + a source-level railway
pipeline (`core_result_dispatch.rs`) asserting `[10,7,6,8,0,5,99]`.

## Honest scope
- `and_then`/`or_else` trust the callee to return a Result (Garnet is dynamically typed),
  like Rust's `?`; no static Result-shape check.
- Ergonomic method syntax (`result.map(..)`) is a later follow-on; S26 ships the
  qualified-function form.

## Dogfood block
```bash
cargo build -p garnet-cli
cargo test -p garnet-interp --test core_result_dispatch --no-fail-fast
cargo test -p garnet-interp stdlib_bridge --no-fail-fast
cargo fmt --all -- --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
```
