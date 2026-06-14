# W-REBUILD RB-band Stop Report — RB-1 → RB-3 complete

**Date:** 2026-06-12 · **Lane:** MacBook Pro (Claude Fable 5, ultracode) · **Stop point:** per the workstream order — "STOP+REPORT: after RB-3 (dispatch rebuilt, drift class dead)". RB-4a does not start until Jon green-lights.

## What landed (all merged to main, all CI-green, all bundled)

| Slice | PR → commit | The one-line truth |
|---|---|---|
| RB-1 caps bitset | [#388](https://github.com/Island-Dev-Crew/garnet/pull/388) → `0ba991e` | `CapSet(u16)`, OR-propagation, XOR diff delta — differential-proptest-proven against the old set impl before same-PR deletion; `diff-caps --machine` JSON verdict (Directive 15); human output golden-pinned. |
| RB-2 crash surface | [#389](https://github.com/Island-Dev-Crew/garnet/pull/389) → `f9196c7` | deny(unwrap/expect) live on cli(lib+bin)/interp/stdlib (planted-unwrap-proven); `i64::MIN / -1` abort → identical diagnostic on BOTH backends (red→green); malformed-corpus smoke; 18.18M fuzz execs / 601 s / zero crashes. Scoped claim only. |
| RB-2 follow-up | [#390](https://github.com/Island-Dev-Crew/garnet/pull/390) → `a6bddac` | interp `%= 0` now "division by zero" — cross-backend parity arm (pre-existing divergence found by RB-2's review, fixed as its own slice). |
| RB-3 keystone | [#393](https://github.com/Island-Dev-Crew/garnet/pull/393) | Registry row = THE dispatch declaration (`Binding`/`Guard` columns); interpreter `install()` derived from `all_prims()` × `#[garnet_primitive]` adapter table; **82 hand-written registrations deleted**; differential proven to fn-pointer identity before deletion; Guard column behaviorally bound (every bridged prim sweep-tested for trap/no-trap); all textual gates kept green with zero gate-script logic changes. |

**The drift class is dead:** the registry's "the interpreter populates from `all_prims()`" claim was false for the project's entire life — two hand-synced lists. It is now true, and every future mismatch (missing adapter, extra adapter, dual-key alias, cross-module dup, guard/adapter disagreement) is a deterministic compile error or red test, not silent drift. This macro is the Core Ring's binding factory (reassessment §6) — adding an audited binding is now a declarative act.

**Verification state at the band stop:** workspace 1999/0; clippy/fmt/doc/deny clean; contracts 22/22; all 11 trust/readiness gates PASS; `truth --check` ok (4 stamped surfaces); mit-readiness 92.8 unchanged. Every slice carried a sealed, manifest-verified Desktop dogfood bundle + a multi-agent adversarial review (22/27/24 agents; every finding verified or refuted; review probes caught one real macro defect pre-merge — the dual-key alias — now failed closed).

## Decisions RESOLVED by Jon (2026-06-12)

All nine landed in `W_REBUILD_SPEC.md` (inline `RESOLVED` blocks on the RB-1/RB-2/RB-3/RB-4b accept-when) + `rfcs/0002-integer-overflow-policy.md`:

1. **Clone criterion** → accept RB-5 as the vehicle; original "201→<40" amended honestly (measured 188 baseline, cap-set 7→0, total 185). RB-5 re-measures.
2. **Directive-15 bounds deltas** → amend the accept-when now (bounds not part of the declared-caps surface); a bounds-bearing attestation is a later human-approved manifest-standard/gate extension.
3. **FAIL-CLOSED comment form** → blessed as the second sanctioned allow-comment form (cite the documented contract).
4. **Runtime spans** → scheduled as a follow-up before public playground work; does not block RB-4b.
5. **add/sub/mul overflow** → checked-error by default + explicit wrapping ops; recorded in `rfcs/0002-integer-overflow-policy.md`; own slice.
6. **RB-3 LOC** → accept the architecture rationale; the win was killing dispatch drift, not LOC (measured +752).
7. **RB-3 mechanism** → ratified as the gate-compatible R1 realization: 78 adapter-registered + 2 deliberately Unbridged + 4 BRIDGE_ONLY.
8. **eval/repl/test panic firewall** → scheduled as a small follow-up slice.
9. **trybuild** → yes, add before Core Ring work begins.

Sequencing (Jon): RB-4b proceeds (4b.1 merged); after RB-4b → RB-5 → STOP+REPORT with measured numbers before the RB-6 memo (RB-6 memo-only, Jon-owned).

---

## Decisions needed from Jon (J-queue candidates)

1. **RB-1 clone criterion** — spec said "201 → <40". Measured: baseline was 188 (201 not reproducible); capability-SET clones (the R3 subject) went 7 → 0; total now 185. The remaining mass is match_coverage/borrow branch-state snapshots + String map keys — **RB-5's interner is the designed vehicle**. → Accept RB-5 as the vehicle, or amend the criterion?
2. **Directive-15 "bounds deltas"** — not part of the declared-caps surface diff-caps reads; the `--machine` JSON says so in its `scope` field. Closing it = a caps-surface/manifest extension (human-merge-only territory). → Amend the accept-when, or schedule the surface extension?
3. **RB-2 FAIL-CLOSED comment form** — `machine_key` CAN fail and aborting is its documented contract; calling it "INVARIANT: cannot fail" would be false. A second sanctioned allow-comment form now exists. → Bless it in the spec, or order the Result-propagating refactor?
4. **Runtime spans (RB-2)** — spec ordered "miette diagnostics with spans"; runtime aborts became span-less messages (parse layer already span-carrying). Threading spans through eval is a real slice. → Amend, or schedule?
5. **add/sub/mul integer overflow policy** — wraps in release, aborts in debug, today. A language-semantics decision (wrap vs checked-error), not a crash-surface item. → Needs a language ruling; RB-2 deliberately did not touch it.
6. **RB-3 LOC criterion** — spec expected ~−2000; measured **+752**. Adapter bodies could not die: the caps-enforcement gate greps their literal text, and Value-conversion logic is real code. What died is the registration list + the drift class. → Accept the architecture rationale, or order a follow-up consolidation?
7. **RB-3 mechanism deviations** — differential ran as table/fn-pointer identity (subsumes fixture-corpus execution); `all_prims()` remains the hand-written (gate-parsed) declaration table; the attribute carries key-only with metadata in the row; "all 80 via attribute" reconciles as 78 + 2 deliberately-Unbridged + 4 BRIDGE_ONLY. → Ratify as the gate-compatible realization of R1.
8. **eval/repl/test panic-firewall gap** (RB-2 named-deferred) — only `garnet run` has the interp thread firewall. → Schedule a small slice?
9. **trybuild** — macro compile-error paths are reasoned + helper-tested, not UI-tested; trybuild = one new external dev-dep. → Worth it?

## Queued next (not started, per stop discipline)
RB-4a (rowan unification), RB-4b (typed AST views + per-pass caps re-check), RB-5 (env rebuild/interner — carries decision 1), RB-6 (backend memo — escalation by design), RB-7 (REPL — `PrimMeta.doc` is ready for `?doc`). Parallel trust band (S141–S150) unaffected on other lanes.

*Evidence: bundles `garnet-rb1-caps-bitset-20260612T024936Z/`, `garnet-rb2-crash-surface-20260612T053932Z/`, `garnet-rb3-registry-dispatch-*/` on the Desktop, each manifest-verified; CHANGELOG entries carry every deviation in-repo.*
