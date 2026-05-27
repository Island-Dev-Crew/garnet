# win-opus — S28 Plan: `core::iter` completion (zip / collect / chain)

**Slot:** win-opus · **Slice:** S28 (Jon-directed; finishes the core:: registry→runtime arc) ·
**Branch:** `agent-win-opus/s28-iter-completion` (off `origin/main` `ce1d4b3`, S27 merged)
**Baseline:** readiness 85.1% / 39 lanes, `--check-no-regression` exit 0.

## What's left

`core::iter` has 9 registered combinators; S21 dispatched 6 (map/filter/fold/take/drop/
enumerate). The last three — `zip`, `collect`, `chain` — are registered but not runnable.
Dispatching them makes **all 9** `core::iter` prims executable and closes the `core::`
registry→runtime gap (with S26 result + S27 option).

## Scope

`garnet-interp-v0.3/stdlib_bridge.rs` only — additive bridges + tests. These are
Value-level (not higher-order):
- `core::iter::zip(a, b)` — array of 2-element pairs, stopping at the shorter sequence.
- `core::iter::chain(a, b)` — concatenate two arrays.
- `core::iter::collect(x)` — materialize a sequence: a `Range` expands to its integers
  (exclusive `1..n` / inclusive `1..=n`), an Array passes through. Other values are a
  type error (no other lazy sequence exists in managed mode).
- `core_iter_completion` readiness lane; baseline surgically extended.

## Test proportion (~60/40)
"Code" = 3 bridges. "Test" = 5 bridge unit tests (zip-shorter, chain-concat, collect
range+array, non-sequence error, bound) + a source-level proof composing collect+chain+
zip with the S21 `fold` (`[36,5,3,2,2]`).

## Honest scope
- `collect` materializes a `Range` or passes an Array through; there is no lazy iterator
  protocol to collect (eager `map`/`filter` already return arrays).

## Dogfood block
```bash
cargo build -p garnet-cli
cargo test -p garnet-interp --test core_iter_completion_dispatch --no-fail-fast
cargo test -p garnet-interp stdlib_bridge --no-fail-fast
cargo fmt --all -- --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
```
