# Garnet WASM target (S55 / W-PLAY)

Running Garnet in the browser via WebAssembly is a major adoption driver and the
enabler for the live playground. This document separates the build lane that is
already proven from the browser integration that is still open.

## Execution model

Garnet has **no Wasm backend**: Garnet programs are not compiled to Wasm. The
browser path compiles the tree-walking **interpreter** to
`wasm32-unknown-unknown`, exposes a thin `wasm-bindgen` API, and evaluates
`*.garnet` source inside that module. The canonical first program remains
[`examples/hello.garnet`](../examples/hello.garnet).

## Current status (2026-07-14)

The Wasm build is no longer hypothetical. W-PLAY's build/execution sublane is
recorded by the clean-Windows WV-5 proof at
[`proofs/windows/launch-verification/wv5-wasm-lane-20260712-0915/wv5-wasm-lane-proof.json`](../proofs/windows/launch-verification/wv5-wasm-lane-20260712-0915/wv5-wasm-lane-proof.json):

- `garnet-wasm` exists and exposes real source-in/output-out execution through
  `run_source` / `run_source_json`;
- native Wasm-crate and interpreter output-capture tests passed;
- `cargo build -p garnet-wasm --target wasm32-unknown-unknown` passed;
- sequential `wasm-pack` web and Node package builds passed; and
- Node loaded the generated module and executed Garnet source successfully.

That proof also fixes the boundary: **Node execution is not browser-page
execution.** The live adapter, the check/diff-caps surface, committed or
reproducibly generated Pages assets, the under-30-second interaction, and the
Playwright browser proof remain W-PLAY work.

## Reproduction tools are not product blockers

A machine reproducing WV-5 needs Rust's `wasm32-unknown-unknown` target,
`wasm-pack`, and Node. Their absence on the machine running a status reporter is
a local setup observation, not evidence that Garnet lacks a Wasm build.
`wasmtime` is optional for an additional non-browser smoke; WV-5 used the actual
`wasm-bindgen` Node package. The current dependency graph may still contain
`miette`'s `fancy` feature, but the successful wasm32 and wasm-pack commands
prove that it is not a build blocker on the recorded lane.

## Remaining W-PLAY path

1. Add the Wasm-facing check and capability-surface/diff APIs required by the
   playground thesis.
2. Build the web package reproducibly and bind it from
   `docs/playground/live.js`.
3. Prove real output plus a visible authority expansion in a clean browser.
4. Record the under-30-second Playwright path and the built module hash.
5. Confirm the live GitHub Pages URL and then flip the launch ledger row.

## Honest scope (do not soften)

WV-5 proves an interpreter compiled to real Wasm and executed through Node. It
does not prove a live browser page, an OS sandbox, a Garnet-to-Wasm compiler, or
production readiness. The public "runs in your browser" claim remains blocked
until the W-PLAY Playwright evidence is committed and the readiness reporters
consume it.
