# S43 Plan — docs-as-tests (`garnet doctest`)

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S43 (detailed
block authored this slice).
Map: reconciled plan §75 — "Docs-as-tests (Codex S43). Executable docs = the
'evidence not courtesy' discipline."
Branch: `codex/s43-doctest`. Base: `origin/main` @ `6e9dd3e` (S42).

## Goal (turn documented examples into runnable, value-asserted tests)
`garnet doc` (v0.4.2) already recovers `///` doc blocks by a backward source
scan (the lexer drops comments). S43 lifts the ` ```garnet ` fences inside those
blocks and executes them, so a documented example is a checked claim, not decor.

## Deliverables
- `garnet-cli/src/doctest.rs` — pure `garnet_fences(doc_block) -> Vec<Fence>`
  (fence grammar + `# => value` marker). Unit-tested, no parser/interp.
- `garnet-cli/src/cmd/doctest.rs` — the runner: reuse `cmd::doc`'s
  `extract_doc_comments_before` + `item_span` (now `pub(crate)`), compute the
  absolute source line, `Interpreter::load_source(file)` once, then
  `eval_expr_src(fence)` per fence; `# => value` asserts the displayed tail.
  Human + `--format json`; exit 1 iff any fence fails.
- Dispatch arm `doctest` in `bin/garnet.rs` + help line in `lib.rs`.
- `examples/documented_math.garnet` — dogfooded demonstrator (3 examples).

## Dogfood
- `garnet doctest examples/documented_math.garnet` → 3 passed, exit 0; JSON
  `"ok":true`. Wrong `# =>` → exit 1 with `expected/got`. New agentic-matrix
  probe gates the demonstrator at 3 passed.

## End-state / gates
- No new readiness lane (not mandated). Full ladder green; CHANGELOG + contract
  S43 block. Ledger: `s42 → merged(5)` advanced in this branch (rode per S42
  PR body); `s43` advance rides with the S44 PR.

## Honest scope
- Interpreter only (not the VM backend).
- Fences see only the file's own defs + stdlib; no cross-file imports (matches
  `garnet doc`).
- A doc-rot guard, not a replacement for the test suite.
