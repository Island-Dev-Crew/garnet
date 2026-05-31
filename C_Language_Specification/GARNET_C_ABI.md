# Garnet C ABI interop (S63)

The C ABI is the lingua franca of native interop: Rust's `extern "C"` (S62),
system libraries, CUDA/Mojo shims — all meet at it. S63 establishes the C ABI as
the canonical FFI contract and proves **compound native authority** through the
S61 model.

## The marshalling contract (design)

A C binding is a `@caps(ffi)` Garnet function over a `#[no_mangle]`/`extern "C"`
symbol. Across the boundary, only C-ABI-stable shapes pass:

| Garnet value | C type |
|---|---|
| `Int` | `int64_t` / `i64` |
| `Float` | `double` |
| `String` | `(const char* ptr, size_t len)` — no embedded-NUL assumption |
| `Bool` | `int32_t` (0/1) |
| `nil` / unit | `void` |
| (bytes) | `(const uint8_t* ptr, size_t len)` |

No Rust/Garnet-specific layout crosses the boundary; ownership is caller-retains
unless a paired free symbol is bound. (This is the *design*; the marshalling
layer is deferred — see honest scope.)

## Compound native authority (the proof)

A C binding that *does IO* needs **both** authorities, and the model surfaces
both. `examples/ffi/c_stat.garnet` binds a C `stat`-like symbol:

```garnet
@caps(ffi, fs)
def c_file_size(path) { ... }   # native (ffi) + touches the filesystem (fs)
```

`garnet-cli/tests/c_abi_proof.rs` proves cross-OS:

- `garnet check` clean (compound caps declared, `0 diagnostics`);
- `garnet sandbox` surfaces **both** — the `ffi` escape-hatch warning **and** the
  `fs` consequences (WASI preopens enabled, fs syscalls in the seccomp allow);
- `garnet seal` attests **both** (`aggregate: ["ffi","fs"]`).

So compound native authority is declared, diff-gated (S37), sandbox-surfaced
(S46), and sealed (S38) — no native call can smuggle in an undeclared authority.

## Honest scope (do not soften)

Garnet has **no FFI runtime**: the interpreter does not call the C symbol, the
value↔C-ABI marshalling layer is **not** implemented, and no `.so`/`.dylib` is
linked. S63 ships the C ABI *contract* + the **compound-authority** proof
(declaration → surface → diff → sandbox → seal), not native execution. WASI
interop (S64) extends the same spine.
