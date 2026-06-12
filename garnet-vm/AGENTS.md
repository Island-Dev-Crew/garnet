# AGENTS.md - Bytecode VM Contract

## Scope

Owns the Garnet bytecode VM scaffold, deterministic bytecode serialization, and VM/interpreter parity checks.

## Stable Contracts

- Checked integer division/remainder overflow (`i64::MIN / -1`, `% -1`) is a
  `VmError::Runtime("integer overflow: ...")` diagnostic with the SAME
  message as the interpreter (RB-2 cross-backend parity;
  `garnet-cli/tests/overflow_parity.rs`) — never a process abort.

- Bytecode VM claims must stay narrower than the tree-walk interpreter until each opcode family is dogfooded.
- Unsupported language forms must fall back explicitly and report fallback counts.
- Serializer and loader output must be deterministic across platforms.
- Benchmark evidence must separate harness presence from fresh measurement claims.

## Required Checks

```sh
cargo test -p garnet-vm
cargo build -p garnet-vm --release
cargo bench -p garnet-vm --bench parse_compile_execute
```
