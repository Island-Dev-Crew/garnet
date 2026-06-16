# W-REBUILD — Final Workstream Report (2026-06-16)

**Status: the Foundation rebuild band (RB-0 … RB-7) is COMPLETE and merged to
`main` (`ed75c59`).** This is the closeout report the plan schedules after RB-7.
It records what landed, the honest deviations carried in each slice, what is
deferred, and the Jon-owned next steps — so a reviewer or future agent can
reconstruct the whole workstream from one document.

> **Doctrine that governed every slice:** *rebuild where Garnet's semantics are
> the product; integrate where the world already audited the hard part.*
> *No authority without evidence; acceptance is a decision made on evidence the
> author cannot fake.* Every slice shipped one PR, full ladder + adversarial
> review + a sealed manifest-verified Desktop bundle + CI green + the established
> auth-switched merge flow. No gate a PR merged under was modified by that PR;
> the release tag stayed Jon's.

---

## §1 · The slice ledger (RB-0 … RB-7)

| Slice | PR → commit | Outcome (one line) |
|-------|-------------|--------------------|
| **RB-0a** | [#384](https://github.com/Island-Dev-Crew/garnet/pull/384) → `3ccfd38` | `xtask truth` — machine-truth generator + marker stamping + `--check` drift guard (planted-mismatch proven). |
| **RB-0b** | [#385](https://github.com/Island-Dev-Crew/garnet/pull/385) → `8482294` | README replacement — verified front door; six script walkthroughs relocated to `docs/internals/`. |
| **RB-0c** | [#386](https://github.com/Island-Dev-Crew/garnet/pull/386) → `c4b9e28` | Version-narrative fixes — stale public-surface strings to zero. |
| **RB-0d** | [#387](https://github.com/Island-Dev-Crew/garnet/pull/387) → `f03d414` | Site stats from `truth.json`; 0.8.1 VSIX **prepared**; `@caps` blog **draft** (posting = Jon). |
| **RB-1** | [#388](https://github.com/Island-Dev-Crew/garnet/pull/388) → `0ba991e` | Caps lattice → `CapSet(u16)` bitset (OR propagation, XOR diff delta), differential-proven before the old impl was deleted; `diff-caps --machine`. |
| **RB-2** | [#389](https://github.com/Island-Dev-Crew/garnet/pull/389) → `f9196c7` | Crash-surface sweep — `deny(unwrap/expect)` on user-facing crates; `i64::MIN/-1` abort → identical cross-backend diagnostic; corpus smoke + fuzz. |
| **RB-2 follow-up** | [#390](https://github.com/Island-Dev-Crew/garnet/pull/390) → `a6bddac` | interp `%= 0` → "division by zero" — closes the pre-existing interp/VM divergence with a parity test. |
| **RB-3** (keystone) | [#393](https://github.com/Island-Dev-Crew/garnet/pull/393) → `3349ae3` | Registry-derived dispatch — PrimMeta `Binding`/`Guard` columns + `#[garnet_primitive]` adapter table; 82 hand-written registrations deleted; differential-proven to fn-pointer identity. |
| **Band stop report** | [#394](https://github.com/Island-Dev-Crew/garnet/pull/394) → `0eecb65` | RB-1..RB-3 ledger + nine decisions queued for Jon. |
| **RB-4a** | [#397](https://github.com/Island-Dev-Crew/garnet/pull/397) → `d43e9d2` | Rowan unification — legacy #221 CST oracle deleted (precondition verified); token parity re-anchored on the lexer; all-corpus parse gate restored. |
| **RB-4b.1** | [#400](https://github.com/Island-Dev-Crew/garnet/pull/400) → `2cb832d` | Substrate fidelity — `cst_to_ast` span-exact + error-verdict-equal with `parse_source` (transparent-wrapper see-through). |
| **J-queue rulings** | [#401](https://github.com/Island-Dev-Crew/garnet/pull/401) → `8a5029c` | Nine Jon rulings landed as spec RESOLVED blocks + `rfcs/0002-integer-overflow-policy.md`. |
| **RB-4b.2** (re-scoped) | [#402](https://github.com/Island-Dev-Crew/garnet/pull/402) → `cf65e51` | `SyntaxError` carries a token-range span + the LSP single-parse **finding** (single-parse deferred on a measured diagnostic-cascade blocker). |
| **RB-4b.3** | [#403](https://github.com/Island-Dev-Crew/garnet/pull/403) → `012021a` | Per-pass caps re-check on VM lowering (Directive-7 GHC-Core pattern) + planted-laundering trap; review found+fixed a HIGH shadow false-positive pre-merge. |
| **RB-4b.4** | [#404](https://github.com/Island-Dev-Crew/garnet/pull/404) → `a3bbc25` | Editions note (parked in the parser AGENTS.md) + spec reconciliation (Directive-7 → RESOLVED-PARTIAL; typed-views deferred). |
| **RB-5 STOP+REPORT** | [#405](https://github.com/Island-Dev-Crew/garnet/pull/405) → `04be497` | Measured tree-walk baseline + the indexed-frame blocker analysis; the `(depth,slot)` rewrite escalated to Jon (no code). |
| **RB-6 memo** | [#406](https://github.com/Island-Dev-Crew/garnet/pull/406) → `9d15590` | Backend/IR decision memo (DRAFT) + the RB-5 sequencing decision; wasm spike — interp compiles to `wasm32-wasip1` today. |
| **RB-7** | [#407](https://github.com/Island-Dev-Crew/garnet/pull/407) → `ed75c59` | The REPL joy slice — reedline (CLI-only, wasm portability preserved); `?doc`/`:caps`; pretty values; 19 unit tests. |

---

## §2 · Invariants the workstream established (and how each is proven)

- **Capability lattice = a `u16` bitset** (RB-1). Propagation is OR, the diff-caps
  delta is XOR; a registry-drift trap test fails closed. The old set impl was
  differential-proven equivalent before deletion.
- **The registry row IS the dispatch declaration** (RB-3). One
  `#[garnet_primitive]` declaration emits `PrimMeta` + binding + arity/caps
  guards; a registry/adapter mismatch is a red test.
- **One CST substrate** (RB-4a/4b.1). The rowan `garnet-cst` tree is canonical;
  `cst_to_ast` is span-exact with `parse_source`. The AST remains a parallel
  projection (typed-views deferred, awaiting an adopter).
- **The per-pass caps re-check MECHANISM exists** (RB-4b.3). On the AST→bytecode
  lowering, no native function's bytecode may require more host authority than
  the checker's transitive verdict grants — a **static cross-IR containment check
  with a deterministic planted-laundering trap.** Honestly scoped: NOT runtime
  enforcement, NOT a backend, NOT a typed-caps core IR, NOT multi-pass (those
  remain the Directive-7 aspiration).
- **Capabilities are edition-invariant** (RB-4b.4). Editions gate the lexical
  surface only; the AST/checker/interp/caps manifest are edition-invariant by
  construction.
- **The interpreter is wasm/WASI-portable today** (RB-6 spike).
  `wasm32-wasip1` compiles with no source change; `wasm32-unknown-unknown` is one
  `getrandom/js` flag away. **RB-7 preserved this** by keeping reedline a
  `garnet-cli` dependency only.
- **`:caps` is a DECLARED/available surface, never an enforced budget** (RB-7).
  `@caps` is enforced per-function at entry (S90); a bare prompt call holds no
  capability frame. The header says so.

---

## §3 · Deferred — Jon-owned next steps (NOT autonomous)

These are recorded, not abandoned. Each gets its **own** prompt from Jon.

1. **RB-5 implementation** against **Option C** (Jon, 2026-06-16). The string
   interner + `(depth,slot)` resolution land in the **`garnet-vm` bytecode
   resolved-IR** (where node identity exists by construction), with the tree-walk
   interpreter as the **zero-drift oracle** (build-both-compare). The RB-5
   STOP+REPORT baseline (`eval_fib_15` 394.97 µs · `eval_array_1000` 262.56 µs ·
   `eval_expr` 1.475 µs, M5 Pro) is the before-number. Accept-when: zero semantic
   change; measured Nx on those benches on that machine, nothing broader. **No AST
   name-representation change unless RB-6 re-opens that path.**
2. **W-PLAY playground spike** — the `getrandom/js` unblock for
   `wasm32-unknown-unknown`, a **separate spike after this closeout** (Jon kept it
   out of RB-7). The RB-6 memo's §5 synergy ledger is its brief.
3. **RB-8** (OPTIONAL, Jon-gated) — de-suffix the `*-v0.3` crates + flatten the
   root. Mechanical but wide; Jon's call on timing.
4. **NUC cross-OS REPL verification** — the RB-7 interactive TTY behaviour on
   Windows (`RB7_NUC_HANDOFF.md`). Until it lands, the REPL is **Mac-proven
   only** — not cross-OS-complete.
5. **Recorded language-semantics decisions** still open: add/sub/mul overflow
   policy (RFC-0002, its own slice), the eval/repl/test panic-firewall slice, and
   `trybuild` before Core Ring work (J-queue J5/J8/J9).

---

## §4 · Honest boundaries (what this workstream does NOT claim)

- **Not production / not 1.0.** Garnet remains a research-grade prototype (v0.x).
- **No backend was built.** RB-6 is a memo + a feasibility *compile*; no `.wasm`
  was executed, no codegen path exists. The IR/backend decision is Jon's.
- **The full Directive-7 vision is not delivered** — only its first concrete
  instance (the single-pass static caps re-check). A typed-caps core IR re-checked
  after every pass remains aspirational.
- **RB-5's performance win is not yet realized** — only the baseline is measured;
  the optimization is sequenced with RB-6 per Jon's Option-C ruling.
- **No release, tag, or public launch** was performed; the tag stays Jon's.

---

## §5 · Evidence index

Every slice's sealed bundle is under `/Users/IDC2.5/Desktop/dogfood/` (manifest-
verified): `garnet-rb1-caps-bitset-*` · `garnet-rb2-crash-surface-*` ·
`garnet-rb2-mod-zero-parity-*` · `garnet-rb3-registry-dispatch-*` ·
`garnet-rb4a-rowan-unification-*` · `garnet-rb4b1-substrate-fidelity-*` ·
`garnet-rb4b2-syntaxerror-spans-*` · `garnet-rb4b3-vm-caps-recheck-*` ·
`garnet-rb4b4-editions-note-spec-reconciliation-*` ·
`garnet-rb5-env-rebuild-stop-report-*` ·
`garnet-rb6-backend-ir-decision-memo-*` · `garnet-rb7-repl-joy-slice-*`. Spec +
per-slice acceptance criteria and all RESOLVED rulings live in
`W_REBUILD_SPEC.md`; the RB-5/RB-6 design decisions in their respective reports.

**Workstream verification at closeout:** `main` @ `ed75c59`; workspace tests
**2032/0**; clippy `-D warnings` clean; `cargo doc` clean; `cargo deny` ok;
agent-contracts ok; `xtask truth --check` ok; enforcement-parity + the trust-
kernel anti-rot gates (red-team / evidence-integrity / ultrapunch / domain-proof
/ academic-evidence) + release-readiness all PASS; mit-readiness no-regression.
