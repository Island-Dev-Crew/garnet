# Garnet FFI authority model (S61)

Foreign-function calls (`extern "C"`, and by extension wrapping Python/Mojo/CUDA)
are the sharpest edge of a capability system: native code can do anything the OS
allows, so a sandbox cannot contain it. Garnet's answer is not to pretend it can,
but to make FFI an **explicit, declared, diff-gated, sealed** authority — the
opposite of an implicit escape hatch.

## The model

1. **No implicit FFI.** A function that performs (or transitively wraps) a native
   call must declare `@caps(ffi)`. There is no way to reach `extern "C"` without
   it appearing in the capability surface.
2. **FFI flows through the whole trust kernel.** `@caps(ffi)` is a first-class
   capability (`Capability::Ffi`), so it is:
   - surfaced by `garnet check` / the capability surface (S35),
   - recorded in the capability manifest (S36),
   - caught by `garnet diff-caps` when a PR *gains* it (S37) —
     `+ caps GAINED: ffi` → `AUTHORITY EXPANDED`,
   - embedded in the `garnet seal` in-toto predicate (S38),
   - and **flagged by `garnet sandbox` (S46)** as an escape hatch the
     seccomp/WASI policy cannot constrain.
3. **FFI is honestly uncontainable.** The sandbox policy says so verbatim:
   *"`ffi` capability: native calls cannot be constrained by seccomp or WASI —
   this policy flags but does not contain FFI."* The authority model's value is
   **transparency + review**, not containment: an `@caps(ffi)` gain is loud and
   diff-visible, so a reviewer (or the AI-PR-review-collapse wedge, S49) sees it.

## Demonstrated

`examples/ffi/native_boundary.garnet` declares `@caps(ffi)` on the function that
wraps a native call; `examples/ffi/no_native.garnet` is its capability-free
baseline. Proven by `garnet-cli/tests/ffi_authority.rs`:

- both `garnet check` clean (ffi is *declared*, so it is not a violation);
- `garnet sandbox native_boundary` emits the ffi escape-hatch warning;
- `garnet diff-caps no_native native_boundary` → `caps GAINED: ffi` /
  `AUTHORITY EXPANDED`, exit non-zero.

## Honest scope (do not soften)

Garnet has **no FFI runtime**: the tree-walking interpreter does not execute
`extern "C"` calls, and this slice does **not** add one. S61 ships the
*authority model* — how FFI is declared, surfaced, diffed, sealed, and flagged —
not native-call execution. Rust/C ABI execution proofs (S62/S63) and WASI interop
(S64) build on this model; they remain honest-partial where the native toolchain
or wasm runtime is absent.
