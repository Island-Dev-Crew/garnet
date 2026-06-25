# AGENTS.md - Bytecode VM Contract

## Scope

Owns the Garnet bytecode VM scaffold, deterministic bytecode serialization, and VM/interpreter parity checks.

## Stable Contracts

- Checked integer division/remainder overflow (`i64::MIN / -1`, `% -1`) is a
  `VmError::Runtime("integer overflow: ...")` diagnostic with the SAME
  message as the interpreter (RB-2 cross-backend parity;
  `garnet-cli/tests/overflow_parity.rs`) — never a process abort.

- Per-pass caps re-check (RB-4b.3, Directive 7): `caps_recheck::recheck_caps`
  verifies the AST→bytecode lowering did not LAUNDER authority — no native
  function's bytecode may require more host capability (from its `Call`
  instructions × the stdlib registry) than the checker's per-function
  transitive verdict (`garnet_check::caps_graph::check_caps_coverage`)
  grants. It is a STATIC cross-IR caps-containment check (one-directional:
  lowered ⊆ declared) with a deterministic trap
  (`planted_laundering_call_is_trapped`), NOT runtime enforcement (S90/S92
  own that) and NOT a backend (RB-6 owns that). Call resolution mirrors
  `caps_graph::resolve_callee`: a bare `Call` naming a user function declared
  in the module shadows a same-named primitive (so a user fn `read_file`/`get`
  is not mis-read as the fs/env primitive). Fallback (non-native) functions
  are skipped — they run under interp S90 guards. The check is satisfied on
  every real program (`caps_recheck_corpus`); its value is catching a FUTURE
  lowering/optimization pass that widens authority. Seal-embedding of the
  verdict is RFC-gated (Jon), out of scope here.
- Bytecode VM claims must stay narrower than the tree-walk interpreter until each opcode family is dogfooded.
- Unsupported language forms must fall back explicitly and report fallback counts.
- VM fallback execution for a selected entry must load source under that entry's
  `@caps` frame, matching the interpreter path. Top-level `let`/`const`
  initializers may execute host calls during load, so the fallback loader cannot
  evaluate them outside the entry capability gate.
- `@max_depth(N)` accepts only `1..=64`. The VM runtime guard must reject
  invalid ceilings with the interpreter-equivalent diagnostic instead of
  treating oversized annotations as executable recursion budgets.
- Serializer and loader output must be deterministic across platforms.
- Benchmark evidence must separate harness presence from fresh measurement claims.

## Required Checks

```sh
cargo test -p garnet-vm
cargo build -p garnet-vm --release
cargo bench -p garnet-vm --bench parse_compile_execute
```
