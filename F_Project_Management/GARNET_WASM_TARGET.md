# Garnet WASM target (S55)

Running Garnet in the browser via WebAssembly is a major adoption driver and the
enabler for the playground (S56). This documents the path and its honest status.

## Execution model

Garnet has **no wasm backend** (it does not compile Garnet programs to wasm). The
in-browser path is to compile the **interpreter** (`garnet-interp`) to
`wasm32-unknown-unknown`, expose `Interpreter::load_source` / `eval`-style entry
points through a thin JS shim, and run `*.garnet` source in the browser — the
same tree-walking evaluator the CLI uses. The canonical program to run first is
[`examples/hello.garnet`](../examples/hello.garnet).

## Status (honest — not built here)

`scripts/garnet_wasm_readiness.py` reports this live. As of S55 the wasm build is
**deferred**, with concrete blockers:

| Blocker | What's needed |
|---|---|
| `wasm32` target not installed | `rustup target add wasm32-unknown-unknown` |
| `wasm-pack` absent | install wasm-pack (bundle the interp + JS bindings) |
| `wasmtime` absent | install wasmtime (for a non-browser wasm smoke) |
| `garnet-interp` pulls `miette` `fancy` | feature-gate the `fancy` (terminal/backtrace) feature **off** for the wasm build — it is not browser-portable |

The reporter's `--gate` guards only the **owned** bits (the hello-world example +
this doc); it does **not** fail on the absent toolchain — that is an honest
deferral, not a regression.

## The path (for an environment with the toolchain)

1. `rustup target add wasm32-unknown-unknown`.
2. Feature-gate `miette`'s `fancy` off for wasm (e.g. a `wasm` cargo feature that
   selects a minimal diagnostic renderer).
3. A small `garnet-wasm` crate: `wasm-bindgen` exports `run_source(src) -> String`
   wrapping `garnet_interp::Interpreter`.
4. `wasm-pack build` → an npm-consumable module the S56 playground imports.
5. Smoke: load the module, `run_source(read("hello.garnet"))` → "Hello from
   Garnet!".

## Honest scope (do not soften)

No wasm artifact is built and no browser run is claimed in this slice. Garnet has
no wasm backend; the interpreter-to-wasm build is deferred until the blockers
above are resolved. This slice ships the hello-world example, the readiness
reporter, and this path doc.
