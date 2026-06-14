# RB-5 — Environment Rebuild · STOP+REPORT (2026-06-14)

**Status: STOPPED at the design gate — measured baseline captured; the
implementation is blocked on a substrate/IR decision that is Jon's to make.**

This is the STOP+REPORT the W-REBUILD plan schedules *after RB-5, with measured
numbers, before the RB-6 memo*. It reports what RB-5 asked for, the measured
current-state baseline, the architecture map, the three blockers that stop the
`(depth,slot)` indexed-frame rewrite from being a safe single slice, why they
all reduce to a question RB-6 is meant to answer, and a recommendation. No code
changed; main is at `a3bbc254` (RB-4b complete).

> **Why STOP rather than ship a rewrite.** RB-5's accept-when is **ZERO
> observable semantic change**. The interpreter's `Env` is the hot path of every
> evaluation; the rewrite the spec sketches (`string interner` + a `garnet-check`
> resolution pass assigning `(depth,slot)` + indexed frames) requires changing
> how names are represented in the **AST that RB-4a/4b just stabilized and that
> parser/check/interp/vm all share**. That is a substrate change, and the four
> integrity rules + the narrow-slice discipline make a substrate change a
> human-approved decision, not an autonomous big-bang. The honest move (the
> Jon-endorsed RB-4b.2 pattern) is to take RB-5 as far as is *safe* — measure
> the baseline — and surface the decision here.

---

## §1 · What RB-5 asked for (verbatim accept-when)

From `W_REBUILD_SPEC.md` §3 RB-5: a **string interner** + a **resolution pass in
`garnet-check` assigning `(depth, slot)` to every binding**; the interpreter
`Env` becomes **indexed frames** instead of the
`Rc<RefCell<HashMap<String,Value>>>` parent chain. **Accept when:** ZERO semantic
change — full workspace + enforcement-parity gates green; **criterion
before/after on the existing bench harness** committed as machine-local evidence
with exact hardware noted; the honest sentence is "measured Nx on these benches
on this machine," nothing broader. RB-5 is also the **J1 clone-criterion
vehicle** (measured baseline 188 clones; cap-set 7→0 after RB-1; 185 remaining).

---

## §2 · The measured baseline (machine-local evidence)

**Machine:** Apple M5 Pro · 18 cores · 48 GB · macOS 26.5 · rustc 1.95.0
(stable, aarch64-apple-darwin). Single-run, this machine only — no broader claim.

**Bench harness:** `garnet-interp-v0.3/benches/eval.rs`, run with
`cargo bench -p garnet-interp --bench eval` (Criterion 0.5, `harness = false`).
Three benches, all driving `Interpreter::call`/`eval_expr_src`:

| bench | what it exercises | baseline (median) |
|-------|-------------------|-------------------|
| `eval_fib_15` | deep recursion → per-call `MaxDepthGuard` name clone + parent-chain lookup | **394.97 µs** |
| `eval_array_1000_map_reduce` | 1000-element map/reduce → closure calls + lookups | **262.56 µs** |
| `eval_expr_arithmetic` | single arithmetic expression parse+eval | **1.475 µs** |

**Clone audit (the J1 mass RB-5's interner targets):** `garnet-interp-v0.3/src`
holds **81** `.clone()` sites; ~**18** are name-keyed `String` clones the interner
could remove. The hottest are the recursion guard —
`MaxDepthGuard::enter(f.def.name.clone())` (`eval.rs:743`) and
`m.entry(name.clone())` (`eval.rs:520`), **one `String` clone per call**, on the
`fib` hot path — plus `Value::Map`/`Struct` `BTreeMap<String,Value>` key clones
(`eval.rs:463`, `:926`). The J1 finding already recorded the rest of the mass as
*"checker branch-state snapshots + String map keys"* (RB_BAND_STOP_REPORT,
2026-06-12).

---

## §3 · Architecture map (current `Env`)

`Env` (`garnet-interp-v0.3/src/env.rs:16-24`) is **five** parallel
`RefCell<HashMap>` chains — `vars`, `protocols`, `impl_methods`,
`dynamic_impl_methods`, `active_block` — plus `parent: Option<Rc<Env>>`.

- **Lookup** (`get`, `env.rs:95`) walks the parent chain; **define** (`env.rs:84`)
  always binds locally (implicit shadowing); **set** (`env.rs:209`) walks the
  chain and mutates the *first* scope that has the name.
- **Closures** are `Value::Fn(Rc<FnValue>)` with `captured: Rc<Env>` set to
  `Rc::clone(env)` at creation (`eval.rs:132`) — **capture by reference, not
  snapshot**: a later `env.set` on a captured binding **is visible inside the
  closure** (interior mutability). A call does `Env::new_child(&f.captured)`
  (`eval.rs:768`).
- **Binding sites:** let/var/const (`stmt.rs:15`), params via `bind_params`
  (`value.rs:504`), per-iteration for-loop var (`stmt.rs:62`), match-arm patterns
  (`control.rs:36`), block scopes (`stmt.rs:127`). **Top-level fns are
  late-bound** — a fn may call another defined later (no forward-ref error).
- **The REPL** holds one long-lived global `Env` in `Interpreter` (`lib.rs:49`)
  and **accretes bindings incrementally**, line by line, across interactions.

---

## §4 · The three blockers (why the indexed-frame rewrite is not a safe single slice)

1. **The AST has no node identity.** Every node carries only a `Span`
   (byte-offset range, `token.rs`), which is **not** globally unique and is unsafe
   as a side-table key. To attach a `(depth,slot)` (or an interned name id) to
   each *reference site* so the interpreter can use it without re-resolving, you
   need either a real `NodeId` **added to the AST** (a parser/substrate change) or
   a **separate resolved IR** the interpreter walks instead of the AST. Both are
   substrate-level.

2. **The REPL fights static whole-program resolution.** `(depth,slot)` assumes the
   scope structure is known at resolve-time. The REPL's long-lived global `Env`
   gains bindings one line at a time, so global slot assignment would have to
   re-run and grow incrementally and stay consistent with already-captured
   closures. This is the same *class* of measured blocker that re-scoped RB-4b.2
   (a whole-program assumption the incremental front-end violates).

3. **The `Env` is five chains, and closure capture is by reference.** Indexed
   frames naturally replace only `vars`; `protocols`/`impl_methods`/
   `dynamic_impl_methods`/`active_block` stay map-chains, so the rewrite is partial
   by construction. And the capture-by-reference mutation-visibility (§3) must be
   reproduced exactly by whatever frame objects replace the `Rc<Env>` — an
   off-by-one in depth assignment silently mutates the wrong binding, a class of
   bug a green workspace can miss (closure + REPL state edge cases).

**Shared root cause:** every interner win *and* the `(depth,slot)` win require the
resolved id to be **available at each reference site without re-hashing the
name** — i.e. a name-representation change in the AST or a resolved IR. There is
no win that does not route through that change.

---

## §5 · Why this is entangled with RB-6

RB-6 is the **backend/IR decision memo** — explicitly *"what IR does the
execution path use."* The interpreter today walks `garnet_parser::ast::Module`
**directly**. A `(depth,slot)` resolution pass + interner is precisely the
introduction of a **resolved IR layer between the AST and execution**. So the
"where do resolution results live" question §4 raises **is** the RB-6 question.
Deciding it inside RB-5, autonomously, would pre-empt the RB-6 memo and bake a
substrate change before its trade-offs (custom resolved IR vs. lowering to an
existing backend's IR, per the Stroustrup-linker doctrine the memo carries) are
weighed. The VM (`garnet-vm`) already has its *own* compile path and is the
natural home for `(depth,slot)` lowering — another reason to sequence this with
the backend decision rather than bolt indices onto the tree-walk `Env` first.

---

## §6 · Options (for Jon)

- **A — Full indexed-frame rewrite now.** Maximum payoff, maximum risk; requires
  a `NodeId`/resolved-IR substrate change right after stabilizing the substrate,
  under a zero-semantic-change bar with the REPL/closure edge cases. *Not
  recommended autonomously.*
- **B — Interner-only, via an AST name-representation change** (names →
  `Rc<str>`/`Symbol` in the AST + `FnDef`). Removes the hot name-clone mass
  (measurable on `fib`), defers indexed frames. Still a shared-AST substrate
  change (parser/check/interp/vm) — smaller than A but not zero-risk, and it
  touches `Value::Map`/`Struct` key types if pushed to the full clone mass (which
  reaches the **deterministic-serialization gate**).
- **C — Sequence `(depth,slot)` + interner with the RB-6 IR decision
  (RECOMMENDED).** Let RB-6 decide whether the execution path keeps walking the
  AST, gets a resolved IR, or lowers to a backend; resolution + interning land in
  whichever IR that decision picks, where node identity exists by construction and
  the REPL/closure questions are answered once. RB-5's measured baseline (this
  report) becomes the RB-6 memo's before-number.
- **D — Narrow safe increment only** (e.g. key `MaxDepthGuard` by `Rc<FnValue>`
  identity to drop the per-call name clone). Tiny, measurable on `fib`, no AST
  change — but changes the guard's per-name→per-definition identity model (an
  unproven-unobservable edge for shadowed function names), so it is *not* a clean
  zero-change win and the payoff is small.

---

## §7 · Recommendation

**Option C.** RB-5's deliverable for this STOP is the **measured baseline + this
analysis**; the `(depth,slot)` resolution pass and the interner should be
sequenced with the **RB-6 IR decision**, because they are the same decision wearing
two hats. This keeps the just-stabilized substrate stable, honors the
zero-semantic-change bar (by not forcing a risky rewrite), and gives the RB-6 memo
a real before-number to reason about. If Jon wants a standalone measurable win
sooner, **Option B** (interner via an AST name-representation change) is the next
smallest coherent slice, explicitly scoped to *names only* (not `Value` map keys,
to stay clear of the serialization gate).

**Decision requested from Jon:** A / B / C / D — and, if B, whether names-only
interning may touch the shared AST `FnDef`/`Ident` representation.

---

## §8 · What I did NOT do (and why) — honesty

- I did **not** write the indexed-frame `Env` or the resolution pass — they
  require a substrate/IR decision (§4–§5) that is Jon's, and forcing them would
  violate the narrow-slice + four-integrity-rules discipline.
- I did **not** ship a token interner with marginal payoff to "close" RB-5 — every
  real interner win routes through the same AST change (§4), so a no-AST interner
  is either negligible or semantically risky (Option D).
- I did **not** touch any gate, the AST, or `Value`. Main is unchanged at
  `a3bbc254`; the only artifact is this report.
- The baseline is **single-run, this machine**; no Nx claim is made because there
  is no "after" — by design, that is the decision being escalated.
