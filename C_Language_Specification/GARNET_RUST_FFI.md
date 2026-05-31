# Garnet ↔ Rust FFI (S62)

How Garnet binds a Rust `extern "C"` function, under the FFI authority model
(S61). The headline (Lattner T1): *"Mojo runs inside the GPU; Garnet runs above
it"* — Garnet's job at the native boundary is **authority and attestation**, not
re-implementing the native code.

## The binding design

A Garnet function that wraps a Rust symbol:

```garnet
@caps(ffi)
def rust_blake3(data) {
  # binds `extern "C" fn garnet_rust_blake3(ptr: *const u8, len: usize) -> ...`
  ...
}
```

- The Rust side exposes a `#[no_mangle] pub extern "C"` symbol with a C ABI
  (pointers + lengths, no Rust-specific layout across the boundary).
- The Garnet side declares `@caps(ffi)` — **mandatory**; without it the call is
  not reachable (S61). The marshalling layer (Garnet value ↔ C ABI) is the part
  that needs a runtime and is **deferred** (see honest scope).

## The proof: attestation, not execution

What S62 proves today is that a Rust-FFI binding is a **first-class, attested
authority**:

- `examples/ffi/rust_extern.garnet` (a `@caps(ffi)` Rust-wrapper) `garnet check`s
  clean and runs (with the body as a stand-in for the future binding).
- `garnet seal examples/ffi/rust_extern.garnet` emits an in-toto predicate
  (`predicateType: https://garnet-lang.org/attestation/seal/v1`) whose embedded
  capability manifest **attests `ffi`** in its aggregate — so a Rust-FFI binding
  can be diffed (S37), reviewed (S49), and signed (`cosign attest`, S51) like any
  other authority.

`garnet-cli/tests/rust_ffi_proof.rs` proves this cross-OS via the matrix.

## Honest scope (do not soften)

Garnet has **no FFI runtime**: the interpreter does not call the Rust symbol, and
this slice does **not** add the value↔C-ABI marshalling layer or link a real Rust
`cdylib`. S62 proves the **authority + attestation** half — a Rust-FFI binding is
declared, surfaced, diffed, and *sealed* — not native-call execution. The
execution half is deferred; the C ABI proof (S63) and WASI interop (S64) extend
this same authority/attestation spine.
