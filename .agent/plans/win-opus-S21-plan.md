# win-opus — S21 Plan: Interpreter dispatch for S17 Layer-0/1 primitives + Mnemos combination

**Slot:** win-opus · **Slice:** S21 (Jon-directed; closes the S17 deferred line) ·
**Branch:** `agent-win-opus/s21-interp-dispatch` (off `origin/main` `0970a4c`)
**Baseline:** `cargo test --workspace` + `cargo clippy -D warnings` exit 0.

## Directive (Jon)

"Since the S17 Layer-0/1 stdlib primitives were not interpreter-dispatched, build
towards that proper and more robust lib offering now that S17 is passed… Mnemos
utilization in the different combinations and smoke tests of the diverse system
programming build options… surface anything meaningful or unique."

## Ownership

Takes on `garnet-interp-v0.3` (eval.rs, stdlib_bridge.rs) under Jon's explicit
direction. The v0.7 slice-ownership lockdown was for the parallel S15–S19 build
(now all merged); `garnet-interp` is unowned, so no active-agent conflict. Also
edits my own `garnet-stdlib` (Mnemos combo test, dev-dep). New examples + scripts
+ docs + section-scoped cross-cutting. No edits to garnet-lsp/cst/parser/
suggest-llm/garnet-lang or win-codex's matrix.

## The foundation (done + verified live)

`eval_path` (eval.rs) now resolves the **fully-qualified name first** before the
last-segment fallback — backward-compatible (`Storage::read_block` → `read_block`
still works). This lets stdlib prims bind under their full path
(`core::math::sqrt`) without colliding with bare prelude builtins that share a
last segment (`map` = Map ctor, `ok`/`err` = Result builders).

## Dispatch set (qualified bridges in stdlib_bridge.rs)

| Module | Prims | Backing |
|---|---|---|
| `core::math` | abs, sqrt, pow, floor, ceil, round | dispatches `garnet_stdlib::math` |
| `core::cmp` | min, max, clamp, ordering | Value-level (`partial_compare`); stdlib::cmp is the Rust reference |
| `core::iter` | map, filter, fold, take, drop, enumerate | Value-level; map/filter/fold are **higher-order via `crate::eval::call_value`** |
| `std::base64` | encode, decode | dispatches `garnet_stdlib::base64` |

18 newly-runnable primitives. Verified live: `core::math::sqrt(16.0)→4`,
`core::iter::map([1,2,3,4], double)→[2,4,6,8]`, `core::iter::filter`/`fold`,
`std::base64::encode("hi")→"aGk="`. Rust trampoline tests (known-value,
higher-order via a test callable, base64 roundtrip, binding coverage).

## Runnable proof + Mnemos

- `examples/novel_04_dispatched_stdlib_pipeline.garnet` — calls the qualified
  prims end-to-end (`garnet run`), deterministic output; extend
  `scripts/smoke_garnet_novel_compositions.py` to include it.
- **Mnemos × stdlib combination smoke** — a Rust integration test composing the
  four Mnemos stores (working / episodic / semantic / procedural) with the
  dispatched stdlib (blake3 provenance → episodic memory → base64 recall),
  proving the memory core composes with the stdlib across system-programming
  build options. Lives in `garnet-stdlib` (which I own) with `garnet-memory` as
  a dev-dependency.

## Readiness + docs + PR

`interp_stdlib_dispatch` readiness lane + baseline; CHANGELOG / CURRENT_STATE /
S21 dogfood block; fork-PR (cached cred is `Navigata1`/fork-only) + grep-loop to
5/5; **merge handed to Jon** (org-write required).

## Novel discoveries to surface

- Qualified-first `eval_path` turns the stub module system into real qualified
  dispatch — the proper foundation for `std::`/`core::` and future packages.
- **First-class-function iterator combinators from managed Garnet**
  (`core::iter::map(arr, fn)`) — functional-style programming now runnable.
- Mnemos composes with content-addressed provenance (blake3) + base64 across the
  four memory kinds.

## Honest scope

- `std::json` (needs a serde_json dep + Value↔json converter), `std::regex`,
  `std::uuid`, `std::env`, `std::process`, `std::log` dispatch → **S22**.
- A full managed-mode `memory::` prim family (Garnet-callable Mnemos with a
  handle Value) → **S22**; S21 proves Mnemos × stdlib at the Rust/system level.
- `core::cmp`/`core::iter` are Value-level bridges (Garnet's dynamic `Value`
  can't be passed as the stdlib's monomorphic generic `T`); the `garnet_stdlib`
  generics remain the tested Rust reference.

## Dogfood block
```bash
cargo build -p garnet-cli --release
garnet run examples/novel_04_dispatched_stdlib_pipeline.garnet   # deterministic
python3 scripts/smoke_garnet_novel_compositions.py
cargo test -p garnet-interp -p garnet-stdlib --no-fail-fast
cargo fmt --all -- --check; cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
```
