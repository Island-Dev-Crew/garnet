# Garnet runtime enforcement — `@max_depth` seed (S89)

The v0.8.0 trust kernel *declared and checked* bounds but did not **enforce** them
at runtime (S40 identifies explosive operations + a default-ceiling *policy*; S46
*generates* sandbox policy without enforcing). S89 is the first slice that makes
the kernel actually enforce — one ceiling, honestly.

## What is enforced (S89)

A function that declares **`@max_depth(N)`** (the checker constrains `N ∈ [1,64]`)
now **traps deterministically** when its recursion depth exceeds `N`. The
interpreter (`garnet-interp-v0.3/src/eval.rs`, `call_fn`) tracks per-function
recursion depth on the per-run `garnet-interp` thread (S85) and returns a runtime
error the moment the ceiling is crossed:

```
$ garnet run --interp deep.garnet      # deep() declares @max_depth(4), recurses 20
runtime error: bounded: @max_depth(4) exceeded for `deep` (recursion depth 5)
$ echo $?
1
```

This is **real enforcement** — the interpreter refuses to recurse further — not a
generated artifact and not the S85 host-stack raise (which only moved the
overflow ceiling). A function within its ceiling runs unchanged; a function with
**no** `@max_depth` is **not** capped (it recurses up to the host stack).

## What is NOT enforced (honest)

The kernel is honest about the boundary; only `@max_depth` recursion is enforced
today. Still **declared-not-enforced**:

- **`@bounded(N)`** — a CPU/**Wasmtime-fuel** budget; enforcement lowers to fuel
  metering (S39/S88), and wasmtime is absent. *Declared, not enforced.*
- **Memory / time** ceilings — *declared, not enforced.*
- **`@mailbox(N)` / `@fan_out(N)`** — actor mailbox + spawn fan-out; the mailbox
  cap exists at the actor-send boundary but is not part of this seed's claim.

No ceiling is *faked*: a bound is either backed by a *trapping test* (only
`@max_depth` today) or labelled declared/generated.

## Verification

`garnet-cli/tests/bounded_enforcement.rs` (cross-OS matrix): over-ceiling recursion
traps deterministically; within-ceiling runs; the trap is deterministic across
runs (comparing exit code + the trap message, not raw stderr — which carries the
documented episodic-cache notes); unannotated recursion is not capped.
`scripts/garnet_bounded_enforcement_status.py --gate` is the static anti-regression
gate.

## Scope notes (do not soften)

- The **interpreter** enforces `@max_depth`; the **VM** backend does not yet — the
  parity corpus contains no `@max_depth`-over-ceiling program, so VM/interpreter
  parity stays 33/33. VM enforcement is future work.
- This is a **seed**: one enforced ceiling. Mac-authored + Mac-tested; the Windows
  trap re-proves via the cross-OS `cargo test` matrix (recorded
  Windows-proof-pending in `WINDOWS_AUDIT_S1_S80.md`).

---

## `@caps` host-authority enforcement (S90)

S90 extends runtime enforcement from `@max_depth` to **capabilities**. The
interpreter now traps when a managed function invokes a **host-authority
primitive** whose required capability no frame in the call chain declared:

| Primitive(s) | Required cap |
|---|---|
| `std::env::get` / `set` / `vars` | `env` |
| `std::process::spawn` / `spawn_args` / `output` / `wait` / `exit_code` | `proc` |
| `fs::read_file` / `write_file` / `read_bytes` / `write_bytes` / `list_dir` | `fs` |
| `std::log::to_file` | `fs` |

```
$ garnet run --interp env.garnet      # @caps() main calls std::env::get
runtime error: capability: `std::env::get` requires @caps(env), not declared in the calling chain
```

`garnet run` does **not** run the static checker, so this is the **runtime
backstop**: a program that the checker would reject (a managed fn using authority
it did not declare) is caught at execution. Each managed function pushes its
declared `@caps` onto a per-run thread-local context (`eval.rs` `CapsGuard`,
RAII-unwound); a primitive is permitted iff the **union** of the active frames'
caps contains the requirement (or a `@caps(*)` wildcard). The static caps-graph
propagates caps up every managed frame, so a *checked* program always carries the
cap — only under-declared programs trap.

### Scope notes (do not soften)

- **Host-authority surfaces only** — env / process / fs / log-to-file. Pure
  computation is unaffected.
- **No managed-program frame ⇒ allowed.** A direct host/test call (no managed
  function on the stack) has no `@caps` context to enforce against, so it runs —
  this keeps the Rust stdlib-bridge tests valid.
- The **VM** backend does not yet enforce `@caps` (the interpreter does). VM
  enforcement is future work.
- Mac-authored + Mac-tested; the Windows trap re-proves via the cross-OS `cargo
  test` matrix (recorded Windows-proof-pending in `WINDOWS_AUDIT_S1_S80.md`).
