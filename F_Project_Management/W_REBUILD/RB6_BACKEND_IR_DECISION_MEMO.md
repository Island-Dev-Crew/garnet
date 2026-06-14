# RB-6 — Backend / IR Decision Memo (DRAFT for Jon · 2026-06-14)

**Status: memo + spike evidence prepared; the DECISION is Jon's.** Nothing is
merged into a backend. This is the decision document the W-REBUILD plan schedules
as RB-6 (verdict R5) — *Jon-gated, no code*. It exists to let Jon decide the
**durable IR / execution-representation shape**, because (per the RB-5 STOP+REPORT
and Jon's Option-C ruling) that decision gates how RB-5's `(depth,slot)`
resolution + interner are implemented.

---

## §0 · Mandate (the RB-5 sequencing decision)

**RB-5 sequencing decision — Option C (Jon, 2026-06-14).** RB-5 (string interner
+ `garnet-check` `(depth,slot)` resolution + indexed-frame `Env`) is **sequenced
with RB-6, not rejected**. The RB-5 STOP+REPORT
(`RB5_ENV_REBUILD_STOP_REPORT_2026-06-14.md`, merged in
[#405](https://github.com/Island-Dev-Crew/garnet/pull/405)) is the **before-number**
for this memo. Jon's guardrails, verbatim intent:

1. **Preserve the just-stabilized parser/CST/AST substrate.**
2. **RB-6 decides the durable IR/backend shape.**
3. **Then** implement RB-5 against the chosen representation, with
   build-both-compare and **zero semantic drift**.
4. **Do not** do the full indexed-frame rewrite now, and **do not** do an AST
   name-representation change as a standalone optimization **unless RB-6
   explicitly keeps the AST as the execution substrate.**
5. RB-7 may proceed only if it does not collide with RB-6/RB-5 surfaces.

This memo's job is therefore narrow: give Jon the evidence to pick the execution
representation, so RB-5 can land against it.

---

## §1 · The decision to make

Today the interpreter walks `garnet_parser::ast::Module` **directly** — there is
no resolved IR between the AST and execution. RB-5's `(depth,slot)` + interner
*is* the introduction of a resolved layer. So the question is singular:

> **What representation does the execution path walk — the AST (+ a resolution
> side-structure), a dedicated resolved IR, or a lowering to an existing
> backend's IR?**

Four shapes are on the table (§4). The choice determines where `(depth,slot)`
and interned names live, whether node identity exists by construction, and
whether the REPL's incremental-binding problem (RB-5 blocker 2) is solved once or
papered over.

---

## §2 · Post-RB-5 tree-walk numbers (the before-number)

Machine: **Apple M5 Pro · 18c · 48 GB · macOS 26.5 · rustc 1.95.0**, existing
harness `cargo bench -p garnet-interp --bench eval` (single run, this machine):

| bench | median |
|-------|--------|
| `eval_fib_15` | 394.97 µs |
| `eval_array_1000_map_reduce` | 262.56 µs |
| `eval_expr_arithmetic` | 1.475 µs |

These are the tree-walk baseline. No "after" exists — by design, the optimization
is the decision being made here. Clone audit: 81 `.clone()` in interp `src`, ~18
name-keyed (the interner-addressable mass); J1 188/7→0/185.

---

## §3 · wasm32 feasibility spike (the empirical core)

Run on this machine, 2026-06-14, rustc 1.95.0 (sealed raw output in the RB-6
bundle):

- **`cargo build -p garnet-interp --target wasm32-wasip1` → COMPILES TODAY
  (cargo printed `Finished … in 13.75s`, which it emits only on a successful
  build, and produced a 16 MB debug `.rlib`).** The whole tree-walk core —
  `garnet-interp`, `garnet-parser`, `garnet-stdlib`, `garnet-memory`,
  `garnet-prim-macros`, plus `miette`/`serde_json` — links for `wasm32-wasip1`.
  Build warnings only (in the transitive `garnet-memory` dep, e.g. an unused
  `new_lock_marker`), no errors, no source change. **The interpreter is
  WASI-ready now.** *(Tooling note: the spike script's exit-code capture logged
  blank through a subshell; success is established by the `Finished` line + the
  linked `.rlib`, both sealed in the bundle.)*
- **`cargo build -p garnet-interp --target wasm32-unknown-unknown` → ONE blocker:**
  `getrandom v0.2.17` (transitive via `rand 0.8` → `rand_chacha` → `rand_core`)
  refuses `wasm*-unknown-unknown` without its `"js"` feature. This is the
  standard, well-understood wasm-in-browser config fix (enable `getrandom/js`),
  not an architectural obstacle. So the pure-sandbox/browser target is **feasible
  with one dependency-feature change**, not a rewrite.
- **Host-authority surface map** (counting real `std::{fs,net,process,thread,time}`
  *usage*, excluding doc-comment mentions): `garnet-check` **none**;
  `garnet-parser` **none** (its lone match is a doc-comment example path, not a
  real touch); `garnet-interp` uses **`fs`/`process`/`time`** (3 std domains);
  `garnet-stdlib` is where it concentrates — **`fs`/`net`/`process`/`thread`/`time`**
  (5 std domains), exposed as the **7 `@caps` primitive modules**
  (`fs`/`net`/`process`/`time`/`log`/`uuid`/`ratelimit`). The host-bound surface
  is **concentrated in exactly the `@caps` primitives** — which is the point: that
  surface is what maps to **WASI imports** under a Wasmtime host. The capability
  boundary and the wasm import boundary are the same boundary.

**Honest scope of the spike:** this is a *feasibility compile* ("does it build,
what blocks it"), **not** a running wasm interpreter, a perf number, or a backend.
No `.wasm` was executed; no codegen path was built.

---

## §4 · Options for the execution representation

- **A — Keep the AST as the execution substrate; add a resolution side-structure.**
  RB-5's `(depth,slot)` + interned names attach via a `NodeId` added to the AST or
  a resolver-built side table the interpreter consults. *Smallest substrate
  disturbance that still unblocks RB-5, but it touches the shared AST (Jon's
  guardrail 4 applies) and does not address the REPL incremental-binding problem
  cleanly.*
- **B — A dedicated resolved IR the interpreter walks.** Lower AST → a small
  resolved IR that carries `(depth,slot)`, interned symbols, and node identity by
  construction. The interpreter walks the IR, not the AST. *Solves node identity
  and the REPL problem at the lowering boundary; a genuine new IR to own and keep
  in parity.*
- **C — Reuse the custom VM bytecode (`garnet-vm`) as the resolved IR.** `garnet-vm`
  already lowers AST → bytecode (`compile_source`) and already re-checks caps per
  lowering pass (`caps_recheck`, RB-4b.3). Make the bytecode the resolved
  representation `(depth,slot)`+interner target; the tree-walk interp stays as the
  reference oracle. *Reuses an asset that exists and already satisfies the per-pass
  caps constraint (§7); the VM is currently narrower than the interp and would need
  to reach parity.*
- **D — Lower to an external backend's IR (Cranelift / Wasmtime / wasm).** Per §3,
  WASI is one feature-flag away and the `@caps`↔WASI boundary aligns. `(depth,slot)`
  + interning become a front-half of a lowering whose back-half is an existing,
  battle-tested codegen + sandbox. *Highest leverage (see §5), but the largest
  integration surface and the one most in need of the parity-cost discipline (§8).*

**The custom-VM-as-a-third-path parity cost (required by the spec):** keeping
`garnet-vm` as a *separate* third execution path (alongside the tree-walk interp
and any external backend) means every language feature must be dogfooded **three**
times to parity. Option C *folds* the VM into the answer (it becomes the resolved
IR, not a third path); Option D plus a retained VM is the most expensive shape.

---

## §5 · The synergy ledger (one lowering, many payoffs)

A single AST→(resolved IR / wasm) lowering is load-bearing for more than speed:

| One lowering buys | Mechanism | Status today |
|---|---|---|
| `@bounded` → execution fuel | Wasmtime fuel metering on the lowered module | `@bounded` is declared-not-enforced (Wasmtime fuel is the named fence) |
| `@caps` → OS sandbox | WASI import allow-list = the `@caps` surface (§3) | `@caps` enforced in-process (S90/S92); WASI would make it an OS boundary |
| the playground | a `.wasm` interpreter runs in the browser | `wasm32-wasip1` compiles today (§3); browser target is one feature-flag away |
| embed-everywhere | one portable artifact | same |
| per-pass caps proof carried to codegen | the RB-4b.3 re-check at each lowering pass | mechanism landed (§7) |

The convergence the reassessment recorded (`F_Project_Management/RESEARCH/
GARNET_REASSESSMENT_2026-06-11.md`) is that these are **the same lowering**, not
five projects.

---

## §6 · The Stroustrup linker doctrine (Directive 12)

The stated frame for integrate-vs-rebuild: *"we can have Dennis's mistakes, which
we know, or my mistakes, which we don't know yet."* **Cranelift/Wasmtime is
Garnet's "C"** — a mature substrate whose known limitations beat an
un-de-risked custom codegen's unknown ones. This biases toward Options C/D
(reuse) over a from-scratch optimizing backend, **unless** a measured reason
appears (§8).

---

## §7 · HARD CONSTRAINT — the per-pass caps re-check (RB-4 criterion)

Any backend/IR candidate **must** be able to re-verify the capability invariant
**per lowering pass** — a candidate that cannot is **disqualified regardless of
its performance numbers.** This is not aspirational: RB-4b.3
([#403](https://github.com/Island-Dev-Crew/garnet/pull/403)) **landed the
mechanism** for the AST→bytecode pass (`garnet_vm::caps_recheck`, a static
cross-IR containment check + a deterministic planted-laundering trap). So:

- **Option C inherits a working per-pass caps re-check** — the VM path already has
  it. This is a real point in C's favor.
- **Options B / D must carry the same property** to each new lowering pass they
  introduce. The RB-4b.3 pattern is the template they must follow.

---

## §8 · Recommendation (integrate-lean) + reopen threshold

**Lean integrate.** The spike (§3) shows the interpreter is already wasm/WASI
portable and the `@caps`↔WASI boundary aligns; the doctrine (§6) and synergy (§5)
both point at reuse over a custom optimizing backend. Concretely, the recommended
**ordering** for Jon's decision:

1. **Pick the resolved-IR shape first** — recommendation: **Option C** (reuse the
   `garnet-vm` bytecode as the resolved IR) **as the near-term `(depth,slot)`+
   interner target**, because it already satisfies the §7 hard constraint and
   avoids both a shared-AST disturbance (guardrail 4) and a brand-new IR to own.
   The tree-walk interp remains the **reference oracle** for build-both-compare.
2. **Treat Option D (Wasmtime/WASI) as the strategic back-half** the §5 synergy
   argues for, sequenced after C proves the resolved-IR + parity discipline.
3. **Keep no permanent third path** (§4 parity cost): the VM should become *the*
   resolved IR (C), not a perpetual third execution mode.

**Reopen threshold (must be measured, never assumed):** revisit a custom
optimizing lowering only if the integrated path measures **~2–3× overhead on
representative workloads**, machine-named, against the §2 tree-walk baseline. The
decision itself remains Jon's.

---

## §9 · How RB-5 lands against the chosen IR

Once Jon picks the representation:

- **If C/B/D (a resolved IR):** `(depth,slot)` + interned symbols are emitted
  **by the lowering** into that IR, where node identity exists by construction —
  RB-5 blockers 1 (no AST node identity) and 2 (REPL incremental binding, solved
  at the lowering boundary) dissolve. The tree-walk interp stays as the
  zero-drift oracle; RB-5 is a build-both-compare against the §2 numbers.
- **If A (keep the AST):** only then is a names-only AST representation change in
  scope (Jon's guardrail 4), explicitly excluding `Value` map keys to stay clear
  of the deterministic-serialization gate.

Either way, RB-5's accept-when (zero semantic change; measured Nx on the §2
benches on this machine) is unchanged.

---

## §10 · Decision requested from Jon

1. **Resolved-IR shape:** A / B / C / D (recommendation: **C now, D as the
   strategic back-half**).
2. **wasm playground / browser target:** worth enabling `getrandom/js` to unblock
   `wasm32-unknown-unknown` as a near-term spike, yes/no?
3. **RB-5 implementation go-ahead** against the chosen IR (build-both-compare,
   zero drift), and whether RB-7 (REPL slice) may run in parallel (it does not
   collide with the resolved-IR surfaces if it only rebuilds `repl.rs` I/O).

---

## §11 · What I did NOT do (honesty)

- **No backend merged, no codegen built, no `.wasm` executed.** §3 is a
  feasibility *compile* only.
- **No IR/representation chosen** — that is Jon's decision (§10). This memo
  recommends; it does not decide.
- **No AST, `Value`, gate, or interpreter behavior changed.** The only artifacts
  are this memo + the recorded sequencing decision + the sealed spike evidence.
- The recommendation is **integrate-lean per the doctrine + the spike**, but the
  reopen threshold (§8) is explicitly measured-not-assumed, and no performance
  claim is made beyond the §2 tree-walk baseline.
