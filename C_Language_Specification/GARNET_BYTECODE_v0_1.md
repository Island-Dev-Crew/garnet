# Garnet Bytecode v0.1

Status: S2 scaffold, not a production VM contract.

This document records the v0.5 bytecode surface introduced for S2. It exists
to make the VM falsifiable and reproducible without claiming full-language
lowering.

## Current Truth

- The tree-walk interpreter remains the semantic reference.
- The bytecode VM covers a managed-mode MVP subset used by
  `examples/mvp_01_*.garnet` through `examples/mvp_05_*.garnet`.
- Unsupported forms fall back to the tree-walk interpreter at function
  boundaries.
- The serializer is deterministic, but it is not a stable external binary ABI.

## Native Opcode Families

S2 supports these native opcode families:

| Opcode family | Purpose |
| --- | --- |
| `Const` | Push deterministic constants. |
| `LoadGlobal` | Load a global value from the interpreter environment. |
| `LoadLocal` | Push a local slot value. |
| `StoreLocal` | Store a stack value into a local slot. |
| `Pop` | Discard a stack value. |
| `Binary` | Execute arithmetic and comparison operators for MVP values. |
| `Unary` | Execute numeric negation and boolean negation. |
| `Jump` | Unconditional branch. |
| `JumpIfFalse` | Truthiness branch using managed-mode truthiness. |
| `MakeArray` | Build an array value from stack elements. |
| `IterInit` | Materialize an iterable into a VM iterator. |
| `IterNext` | Advance a VM iterator for `for` loops. |
| `Call` | Call builtins, native bytecode functions, or fallback functions. |
| `CallMethod` | Call supported MVP methods such as `len`. |
| `Return` | Return from a bytecode function. |

The opcode family count is 15. Individual binary operators are modeled under
the `Binary` family rather than as separate opcodes.

## Deterministic Serialization

The byte stream begins with the ASCII magic `GARNVM01`, followed by:

1. constant table length and constants,
2. function table length and functions,
3. per-function parameter names, local names, native/fallback marker,
   optional fallback reason, and instructions.

Integers are little-endian. Floating-point constants are written as raw
IEEE-754 bits. Strings are UTF-8 length-prefixed byte arrays. This is enough
for reproducible local artifacts; it is not yet a cross-version ABI promise.

## Fallback Boundary

The S2 compiler marks a function as fallback when it sees unsupported forms,
including but not limited to:

- structs, enums, protocols, actors, memory stores, modules, and imports,
- pattern matching,
- try/rescue/ensure,
- closures and block-yield forms,
- non-MVP method calls,
- field/index assignment,
- dynamic impl behavior.

Fallback happens at function boundaries. A native bytecode function may call a
fallback function, and the fallback function executes in the tree-walk
interpreter. The PR evidence must report the fallback count instead of hiding
it.

## Non-Claims

- No production native compiler proof is claimed.
- No stable bytecode ABI is claimed.
- No OS-thread actor bridge is claimed.
- No full safe-mode lowering is claimed.
- No benchmark measurements are embedded in the status reporter; benchmark
  output must be attached as PR evidence when used.
