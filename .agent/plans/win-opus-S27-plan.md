# win-opus — S27 Plan: `core::option` combinator dispatch

**Slot:** win-opus · **Slice:** S27 (Jon-directed; sibling of S26) ·
**Branch:** `agent-win-opus/s27-option-dispatch` (off `origin/main` `ef382fc`, S26 merged)
**Baseline:** readiness 84.7% / 38 lanes, `--check-no-regression` exit 0.

## What's left

The S17 registry tags 5 `core::option` primitives (some, none, map, and_then,
unwrap_or); the interpreter dispatches none. Same gap S26 closed for `core::result`.

## Scope

`garnet-interp-v0.3/stdlib_bridge.rs` only — additive bridges + tests. Option values
are `Value::Variant { path:["Option"], variant:"Some"/"None" }`, identical to the
prelude `some`/`none` builders. Bound qualified (avoids the bare-`map` collision).

- `core::option::some|none` — build the Option Variant.
- `core::option::map(o, f)` — Some(v)→Some(f(v)); None→None. Higher-order.
- `core::option::and_then(o, f)` — Some(v)→f(v) (callee returns Option); None→None.
- `core::option::unwrap_or(o, default)` — Some(v)→v; None→default.
- `core_option_dispatch` readiness lane; baseline surgically extended.

## Test proportion (~60/40)
"Code" = 5 bridges + Option helpers. "Test" = 6 bridge unit tests (each combinator +
None constructor + non-Option type error) + a source-level pipeline
(`core_option_dispatch.rs`) asserting `[10,7,6,8,5,99]`.

## Honest scope
- `and_then` trusts the callee to return an Option; no static shape check.
- Ergonomic method syntax (`option.map(..)`) is a later follow-on.

## Dogfood block
```bash
cargo build -p garnet-cli
cargo test -p garnet-interp --test core_option_dispatch --no-fail-fast
cargo test -p garnet-interp stdlib_bridge --no-fail-fast
cargo fmt --all -- --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
```
