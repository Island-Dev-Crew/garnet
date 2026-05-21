# S14 — Bytecode VM v0.2 function-call lowering — Implementation Plan

Date: 2026-05-21
Contract: `F_Project_Management/GARNET_v0_6_SLICE_DOGFOOD.md` § S14
State: not-started → **planned** (this plan file commits the transition)
Reviewer: Jon (Island Development Crew)

> PR title: `S14: bytecode VM v0.2 — explicit call-frame stack + ABI v0.2`.

---

## 1. What the VM actually does today (empirically verified)

Reading `garnet-vm/src/vm.rs` + running probes on 2026-05-21:

- The compiler **already** lowers `Expr::Call` with an identifier callee into
  `Instruction::Call { name, argc }` (`compiler.rs:335`). Function calls are
  NOT a blanket fallback.
- The VM's `Call` handler (`vm.rs:259`) resolves builtins (`println`/`print`/
  `len`) then delegates to `call_function`, which **recursively calls
  `self.execute()`** for native callees (`vm.rs:141`). So native→native calls
  already work — `fact(10)` returns `3628800` on both `--vm` and `--interp`.
- **The real gap:** `execute()` recurses in the *host* (Rust) language. Each
  Garnet call adds a Rust stack frame. `countdown(100000)` via `--vm`
  **overflows the Rust stack and aborts** (verified). There is no explicit
  call-frame stack; the VM is not yet a "real" frame-based VM.

So the S2 PR evidence's "function calls fall back to tree-walk" was a
conservative description; the honest S14 deliverable is the **explicit
call-frame architecture**, not "make calls work" (they already do for shallow
depth).

## 2. Scope (in)

- **Explicit call-frame stack (core).** Rewrite `VmEngine::execute` into a
  frame-driven loop `run_frames(entry_idx, args)` over a heap-allocated
  `Vec<Frame>` where:
  - `Frame { function_idx: usize, locals: Vec<Slot>, stack: Vec<Slot>, ip: usize }`.
  - `Instruction::Call` to a **native** callee pushes a new `Frame` instead
    of recursing in Rust; the caller's `ip` is advanced past the call first.
  - `Instruction::Return` pops the frame and pushes the return value onto
    the caller's operand stack (or returns it if the frame stack is empty).
  - Builtins (`println`/`print`/`len`) and **fallback** callees execute
    inline (the fallback path still uses the tree-walk interpreter, which is
    its own recursion domain — out of scope to flatten).
  - Frames hold `function_idx` (not a borrowed `&BytecodeFunction`) so the
    `&'a BytecodeProgram` can be copied into a local and indexed without
    entangling `&mut self`.
  - Headline proof: `countdown(100000)` via `--vm` returns `0` instead of
    overflowing.
- **ABI v0.2.** Bump codec magic `GARNVM01` → `GARNVM02`. Add an explicit
  per-function `arity: u32` field (written after the name, before params) so
  a reader can validate arity without counting params. `deserialize_program`
  cross-checks `arity == params.len()` and errors on mismatch. The v0.1
  reader path is dropped (v0.1 artifacts were never a cross-version promise —
  documented in `GARNET_BYTECODE_v0_1.md` non-claims).
- **`--dump-lowering` flag** on `garnet run --vm`. Prints the compile
  summary including a `lowered: N%` line (native functions / total
  functions). Cheap surface over the existing `CompileSummary`.
- **Spec** `C_Language_Specification/GARNET_BYTECODE_v0_2.md` (new). v0.1 stays
  for archival reference; v0.2 documents the frame model, the `GARNVM02`
  header + arity field, and the short-circuit lowering. Non-claims carried
  forward (no cross-version ABI promise; closures/patterns/structs/try still
  fall back).
- **Test** `garnet-vm/tests/function_call.rs` (new):
  - deep recursion `countdown(100000)` runs without overflow (the headline);
  - mutual recursion (`is_even`/`is_odd`) matches the interpreter;
  - mixed-arity corpus (0/1/2/3-arg functions) matches the interpreter;
  - `&&` / `||` short-circuit lowers natively (0 fallback calls) and matches;
  - ABI v0.2 round-trip is byte-identical and `GARNVM02`-prefixed; an
    arity-mismatch byte stream is rejected.
- **Lane** `vm_function_call_lowering` in `scripts/garnet_mit_readiness_status.py`
  (verified when the new test + `GARNET_BYTECODE_v0_2.md` exist). Regenerate
  the baseline.
- **Bench** add a `function_call` group to
  `garnet-vm/benches/parse_compile_execute.rs` (or a new bench file) covering
  the recursive + mutual-recursion hot path.
- **CHANGELOG** S14 entry + **contract flip** S14 → in-progress.
- `.claude/plans/S14-plan.md` (this file).

## 3. Scope (out)

- **Closures, captured environments, dynamic-receiver method dispatch** still
  fall back. (S2 boundary unchanged.)
- **Pattern matching, try/rescue/ensure, struct/enum constructors** still
  fall back.
- **Tail-call optimization.** The frame stack is the prerequisite; TCO is a
  v0.7 slice.
- **Cross-version ABI promise.** v0.2 tightens the schema but is still not a
  stable external ABI.
- **VM-side dependency pre-load.** The S12 resolver is `--interp` only; this
  slice does NOT wire vendored deps into the `--vm` path. (Still deferred;
  the S12 lane already names it.)
- **Flattening the tree-walk fallback's own recursion.** Out of scope; the
  fallback interpreter is a separate engine.
- **`and` / `or` native lowering — DROPPED from S14** (was a candidate).
  Verified on 2026-05-21 that Garnet `and`/`or` are Ruby-style
  *operand-returning* (`true and 5` → `5`, `nil or 9` → `9`), not strict
  booleans. Correct native lowering needs a value-preserving conditional
  jump + a `Dup` opcode (the current `JumpIfFalse` pops its operand). That
  is its own opcode-design slice. `and`/`or` therefore still fall back in
  S14; documented in the lane's `deferred` and the PR body. The frame stack
  is the load-bearing deliverable and does not depend on it.
- **String interpolation, ranges, pipeline operator** native lowering — all
  follow-on fallback-reduction slices; not in S14.

## 4. Concrete tasks (ordered, TDD style)

1. Plan file (this) → commit.
2. **Red test.** `garnet-vm/tests/function_call.rs` with the deep-recursion
   case. Run; expect overflow/abort (the current failure mode) or a captured
   non-success. (Deep recursion aborts the process, so the test asserts the
   post-fix success; pre-fix it aborts — documented in the dogfood log as the
   red state, captured by running the probe binary directly rather than under
   the test harness which an abort would kill.)
3. **Frame stack.** Rewrite `execute` → `run_frames`. Keep `call_function`
   public API stable. Re-run `vm_scaffold.rs` (must stay green) +
   `function_call.rs` (deep recursion now passes).
4. **`&&`/`||` lowering** in `compiler.rs`. Extend `function_call.rs` with the
   short-circuit cases.
5. **ABI v0.2** in `codec.rs`. Extend `function_call.rs` with the round-trip +
   arity-mismatch cases. Update `vm_scaffold.rs::serializer_round_trips_*` if
   it asserts the magic (it doesn't today, but the round-trip must still hold).
6. **`--dump-lowering`** in `garnet-cli/src/cmd/run.rs`.
7. **Spec** `GARNET_BYTECODE_v0_2.md`.
8. **Lane + baseline + CHANGELOG + contract flip.**
9. **Bench.**
10. **Full ladder** (fmt, clippy, test --workspace, deny, readiness
    no-regression, conformance, scripts unittests).
11. **Desktop bundle** at
    `/Users/IDC2.5/Desktop/dogfood/garnet-s14-vm-frames-<UTCstamp>/`.
12. **Commit, push, PR, CI green, merge.**

## 5. Honest doubts and risks

- **Per-step instruction clone.** The frame loop clones the current
  `Instruction` each step to release the `&program` borrow before `&mut self`
  dispatch. `Instruction` is small + `Clone`; the bench will quantify the
  cost. If it regresses materially vs. the recursive version, switch to an
  index-based read that scopes the borrow tighter. (Acceptable trade for
  correctness + no host-stack-overflow.)
- **Fallback recursion unchanged.** A program that's all-fallback still
  recurses in the tree-walk interpreter and can overflow there. S14 only
  flattens the *native* call path; documented as an honest partial.
- **`&&`/`||` semantics.** Garnet truthiness drives short-circuit. The lowered
  form must match the interpreter exactly (including the returned value, not
  just the boolean) — the parity test guards this. If managed-mode `&&`
  returns the operand value (Ruby-style) rather than a strict bool, the
  lowering must preserve that. Verify against the interpreter first; if the
  semantics are subtle, keep `&&`/`||` as fallback and drop that sub-task
  (the frame stack is the load-bearing deliverable; `&&`/`||` is a bonus).
- **ABI break.** `GARNVM02` is intentionally not back-compatible with
  `GARNVM01`. Since v0.1 explicitly disclaimed a cross-version ABI and no
  on-disk v0.1 artifacts are shipped/consumed anywhere in the repo (verify
  with a grep for `GARNVM01` outside the crate), the break is safe.
- **`--dump-lowering` output contract.** The contract's dogfood greps
  `lowered: 100%`. The exact string must be emitted verbatim.

## 6. State-machine transitions

| Transition | When | Evidence |
|---|---|---|
| not-started → planned | this file | `.claude/plans/S14-plan.md` |
| planned → in-progress | draft PR opens | PR URL |
| in-progress → review-ready | CI green + dogfood bundle | PR check status |
| review-ready → dogfood-passing | Jon review | PR review |
| dogfood-passing → merged | squash-merge + lane verified | merge commit |

## 7. What I need from Jon

Nothing blocking. If `&&`/`||` lowering proves semantically subtle, I'll drop
it to a follow-on and ship the frame stack + ABI v0.2 alone — flagged in the
PR body if so.
