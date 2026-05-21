# Garnet Bytecode v0.2

Status: S14 deepening of the S2 scaffold. Still NOT a production VM contract
and NOT a stable cross-version external ABI.

This document records the v0.6 bytecode surface introduced for S14. It
supersedes `GARNET_BYTECODE_v0_1.md` (which stays for archival reference). The
two load-bearing changes over v0.1 are the **explicit call-frame execution
model** and the **`GARNVM02` self-describing header**.

## Current Truth

- The tree-walk interpreter remains the semantic reference.
- The bytecode VM covers the same managed-mode MVP subset as v0.1, plus it now
  executes native function calls on an **explicit, heap-allocated call-frame
  stack** instead of recursing in the host (Rust) language.
- Unsupported forms still fall back to the tree-walk interpreter at function
  boundaries; the fallback set is unchanged from v0.1.
- The serializer is deterministic. It is not a stable external binary ABI.

## What changed from v0.1 (S14)

### Explicit call-frame stack

In v0.1, `VmEngine::execute` was a single recursive Rust function: each Garnet
function call recursed in the host language, so deep Garnet recursion overflowed
the **Rust** stack (verified: `countdown(100000)` aborted with a stack
overflow on `--vm`).

In v0.2, execution runs over an explicit `Vec<Frame>`:

- `Frame { function_idx, locals, stack, ip }` — one activation record per
  in-flight native call. Frames hold the function *index* (not a borrow) so the
  shared `&BytecodeProgram` stays decoupled from the engine's `&mut self`.
- `Instruction::Call` to a **native** callee advances the caller's `ip` past the
  call, then pushes a new `Frame`. No host recursion.
- `Instruction::Return` pops the current frame and pushes the return value onto
  the caller's operand stack (or returns it when the frame stack is empty).
- Builtins (`println` / `print` / `len`) and **fallback** callees execute
  inline. The fallback path still delegates to the tree-walk interpreter, which
  is its own recursion domain and is not flattened here.

Result: deep Garnet recursion runs on the heap. `countdown(200000)` and
mutual recursion to depth 500 run to completion on `--vm` (covered by
`garnet-vm/tests/function_call.rs`). Tail-call optimization is NOT performed —
each call still costs one heap frame; TCO is deferred to a later slice.

### `GARNVM02` header + explicit arity

The byte stream now begins with the ASCII magic `GARNVM02` (was `GARNVM01`).
Each function record carries an explicit `arity` field (a `u32`) immediately
before its parameter vector. The deserializer cross-checks
`arity == params.len()` and rejects a mismatch. This makes the schema more
self-describing and lets a reader validate a function header without trusting
the parameter-vector length alone.

`GARNVM01` artifacts are intentionally NOT readable by the v0.2 deserializer.
v0.1 explicitly disclaimed any cross-version ABI promise and no on-disk v0.1
artifacts are produced or consumed anywhere in the repository, so the break is
safe.

## Native Opcode Families

Unchanged from v0.1 — 15 families: `Const`, `LoadGlobal`, `LoadLocal`,
`StoreLocal`, `Pop`, `Binary`, `Unary`, `Jump`, `JumpIfFalse`, `MakeArray`,
`IterInit`, `IterNext`, `Call`, `CallMethod`, `Return`.

## Deterministic Serialization (v0.2 layout)

The byte stream begins with the ASCII magic `GARNVM02`, followed by:

1. constant table length (`u32`) and constants;
2. function table length (`u32`) and, per function:
   - name (length-prefixed UTF-8 string);
   - **arity (`u32`, S14 — must equal the params vector length)**;
   - params vector (`u32` length + length-prefixed strings);
   - locals vector (`u32` length + length-prefixed strings);
   - native marker (`u8`);
   - optional fallback reason (`u8` present-marker + optional string);
   - instructions (`u32` length + encoded instructions).

Integers are little-endian. Floating-point constants are written as raw
IEEE-754 bits. Strings are UTF-8 length-prefixed byte arrays. This is enough
for reproducible local artifacts; it is not yet a cross-version ABI promise.

## Observability: `--dump-lowering`

`garnet run --vm --dump-lowering <file>` prints, before execution:

```text
lowering: <N> native / <M> fallback functions (<K> native instructions)
lowered: <P>%
  fallback <fn>: <reason>   (one line per fallback function)
```

`<P>` is `native / (native + fallback)` as an integer percent. A fully native
program reports `lowered: 100%`. The flag applies to `--vm` only; passing it
with `--interp` prints a one-line note and is otherwise ignored.

## Fallback Boundary

Unchanged from v0.1. The S14 compiler still marks a function as fallback when
it sees unsupported forms, including but not limited to:

- structs, enums, protocols, actors, memory stores, modules, and imports;
- pattern matching;
- `try` / `rescue` / `ensure`;
- closures and block-yield forms;
- non-MVP method calls;
- field/index assignment;
- dynamic impl behavior;
- the `and` / `or` short-circuit operators (Ruby-style operand-returning
  semantics; native lowering needs value-preserving conditional-jump + `Dup`
  opcodes, deferred to a later fallback-reduction slice).

A native bytecode function may call a fallback function; the fallback function
executes in the tree-walk interpreter. `--dump-lowering` reports the fallback
count and per-function reasons rather than hiding them.

## Non-Claims

- No production native compiler proof is claimed.
- No stable cross-version bytecode ABI is claimed (`GARNVM02` is a tightened,
  self-describing schema, not a frozen format).
- No tail-call optimization (deep recursion costs one heap frame per call).
- No OS-thread actor bridge.
- No full safe-mode lowering.
- The `--vm` path does NOT pre-load vendored dependencies (the S12 resolver is
  `--interp` only); harmonizing the two run paths is deferred.
- No benchmark measurements are embedded in the status reporter; benchmark
  output is attached as PR evidence when used.
