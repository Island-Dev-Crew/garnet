# GARNET W-REBUILD — Foundation Rebuild Workstream
**Spec + paste-ready goal mode · prepared 2026-06-10 · slots into `GARNET_S129_S200_ECC_DOGFOOD_COMMAND_CENTER.md`**
**Doctrine:** *Rebuild where Garnet's semantics are the product. Integrate where the world already audited the hard part.*
**Principle:** *No authority without evidence. Acceptance is a decision made on evidence the author cannot fake.* Everything else — syntax, modes, stdlib — is negotiable; this is not. *(Reassessment 2026-06-11, Directive 6; amendments below marked "[2026-06-11 infusion]" source from its §5/§7.)*

---

## §0 · Where this slots (and what it does NOT claim)

W-REBUILD is one workstream, one lead lane (MacBook Pro · Claude Code Fable 5 · ultracode),
executed **after** the S131–S134 fleet consolidation merges. It claims two territories:

1. **The front-door band (S135–S140)** — Codex's gate map already reserves this for
   "README, repo front door, docs/site truth cleanup." W-REBUILD's RB-0 slices *are* that band,
   executed with the prepared artifacts (`README_PROPOSED.md`, `GARNET_TRUTH_DRIFT_PUNCHLIST.md`).
2. **A new Foundation workstream (RB-1…RB-7)** — registered in the runway between the
   front-door band and the playground band (S151–S165). It does not take S-numbers; the P0
   docs PR records it in the command center so every lane can see it.

**Runs in parallel, untouched:** the trust band (S141–S150 — independent S114 re-verification,
SLSA/Sigstore planning) proceeds on the MacBook Air / Windows NUC / independent-reviewer lanes.
Surfaces don't collide: trust-band work lives in CI plans, release evidence, and reviewer
packages; W-REBUILD lives in `garnet-check`, `garnet-interp`, `garnet-stdlib`, `garnet-parser`,
`garnet-cst`, `xtask`. Any CI/gate change the trust band wants is Jon-gated anyway.

**Explicitly NOT claimed by W-REBUILD** (stays with its band/lane): the docs/site learner IA
(first-PR #3), the playground prototype (#4), the independent S114 package (#5), Marketplace
publication, packaging, localization, launch anything.

**Why the rebuild precedes the playground band:** the playground wants a wasm32 interpreter —
build it once, on the post-rebuild interpreter; the caps bitset makes the playground's live
diff-caps demo a one-instruction diff; and a public playground running a binary with 71 unwraps
and 40 panics in the runtime is a first impression you don't get back.

---

## §1 · Pre-flight: the freeze (deploy gate)

W-REBUILD touches wide surfaces. Unmerged local work on any machine becomes unmergeable the
moment the bridge, checker, and parser internals move. Therefore, hard gate — the goal mode
**STOPS** unless all three hold:

1. Core fleet reports exist in `F_Project_Management/FLEET_REPORTS/` per `TEMPLATE.md`
   (macbook-pro × claude+codex, windows-nuc × claude+codex; air/surface as available).
2. The S131–S134 **source-of-truth consolidation PR is merged** — every local artifact has a
   verdict (commit / archive / ignore) and main is the single truth.
3. The lead machine's working tree is clean on current `origin/main`.

This is the answer to the noise-and-scatter concern: **consolidate → freeze → rebuild.** After
the freeze, anything not in a fleet report doesn't exist (corpus-search-miss ≠ absence — flag it,
never fabricate it), and everything that does exist has already been deprecated or integrated
*before* the foundations move under it.

**Kickoff prep (Jon, two minutes):** copy this file plus `README_PROPOSED.md` and
`GARNET_TRUTH_DRIFT_PUNCHLIST.md` into `F_Project_Management/W_REBUILD/` in the lead checkout,
then paste the §2 prompt. The session's P0 PR commits the pack so every lane reads the same spec.

---

## §2 · Paste-ready goal mode (Claude Code · Fable 5 · ultracode · MacBook Pro)

```text
ROLE: Claude Code (Fable 5, ultracode) on MacBook Pro — Garnet W-REBUILD lead:
front-door band (S135–S140) + Foundation rebuild workstream (RB-1..RB-7),
post-consolidation. Doctrine: rebuild where Garnet's semantics are the product;
integrate where the world already audited the hard part.

SOURCE OF TRUTH: /AGENTS.md + the closest child AGENTS.md for EVERY subsystem
touched; F_Project_Management/GARNET_S129_S200_ECC_DOGFOOD_COMMAND_CENTER.md;
F_Project_Management/W_REBUILD/ (W_REBUILD_SPEC.md = per-slice acceptance
criteria, README_PROPOSED.md, GARNET_TRUTH_DRIFT_PUNCHLIST.md);
F_Project_Management/FLEET_REPORTS/; live repo + GitHub truth.

RECON (every session start): git fetch origin main --tags --prune; record
status/HEAD/origin-main/open PRs/v0.8.1 release truth; run
scripts/garnet_readiness_status.py + garnet_mit_readiness_status.py (json).
Never assume another lane's result without checking repo/GitHub truth.

DEPLOY GATE (verify first; STOP+report if unmet): (1) core fleet reports exist
in FLEET_REPORTS/; (2) the S131–S134 source-of-truth consolidation PR is
MERGED; (3) clean working tree on current origin/main. If unmet: write or
refresh THIS machine's fleet report per TEMPLATE.md, then stop.

P0 (docs PR, no code): commit the W_REBUILD pack; register the workstream in
the command center runway (RB-0 = front-door band S135–S140; RB-1..RB-7 =
Foundation workstream before the playground band; trust band S141–S150 runs
parallel on other lanes). One PR, stop after merge.

SLICES (one per PR; full acceptance criteria in W_REBUILD_SPEC.md §3):
RB-0a xtask truth: truth.json generator + marker stamping + `--check` guard;
  prove the guard fails on a planted mismatch before landing.
RB-0b README replacement from README_PROPOSED.md — verify EVERY link, path,
  and number against repo truth first; relocate the six script walkthroughs to
  docs/internals/; <=180 lines; truth markers stamped.
RB-0c version-narrative fixes (punchlist A2): rg stale strings = 0 on public
  surfaces (24-prims / post-v0.5.0 / v0.5.0-research-grade / S1-LSP).
RB-0d site stats read truth.json; 0.8.1 VSIX re-pack PREPARED with checksums
  (release asset swap = Jon); @caps blog post DRAFTED (posting/dating = Jon).
RB-1 caps bitset: CapSet(u16), Copy; propagation = OR; diff-caps delta = XOR;
  differential property tests vs the old set impl BEFORE deleting it.
RB-2 crash-surface sweep: deny(clippy::unwrap_used,expect_used) on
  user-facing crates with allowlisted internal invariants; reachable aborts
  become miette diagnostics; malformed-input fuzz smoke, zero aborts.
RB-3 registry-derived dispatch: new garnet-prim-macros crate;
  #[garnet_primitive(...)] emits PrimMeta + registration + arity/caps guards
  from one declaration; stdlib_bridge hand-written arms deleted; differential
  tests across ALL primitives; truth primitive_count becomes macro-derived.
RB-4a rowan unification: migrate remaining consumers to garnet-cst; DELETE
  legacy garnet-parser src/cst.rs; byte-identical round-trip corpus green.
RB-4b AST as typed views over the green tree. (Error-recovery + incremental
  parsing: QUEUED post-workstream, feeds the playground band.)
RB-5 env rebuild: string interner + (depth,slot) resolution pass in
  garnet-check; Env frames replace string-keyed HashMap chain; criterion
  before/after committed as machine-local evidence; ZERO semantic change.
RB-6 backend decision MEMO only: post-RB-5 measured numbers + wasm32
  feasibility spike + fuel/WASI/playground synergy; escalate the
  one-lowering decision to Jon. NO backend code merged.
RB-7 REPL on reedline: completion + ?doc + :caps live authority budget from
  PrimMeta; recorded demo evidence; cross-OS proof handed to the NUC lane.

CONSTRAINTS: calibrated honesty above all; evidence, never claims; no
"enforced" without a deterministic trap; RB-1..RB-5 change ZERO language
semantics — workspace tests + the enforcement-parity gate stay green on every
slice, and ANY semantic delta = STOP+report, never patch around it; never push
a tag; never cut/re-cut a release; never touch CI, gates, diff-caps
thresholds, capability standards, or release policy without Jon; never install
ECC hooks; S114 stays labeled self-verified; crate de-suffixing and root
restructuring are OUT of scope unless Jon explicitly green-lights RB-8.

VERIFICATION LADDER (every slice): focused tests first -> cargo test
--workspace --no-fail-fast (0 failed) -> cargo clippy --workspace
--all-targets -- -D warnings -> cargo fmt --all -- --check ->
dogfood-readiness fused 5/5 -> python3 scripts/check-agent-contracts.py ->
PR to Navigata1 -> CI green -> merge IslandDevCrew -> switch back.

ORDER: P0 -> RB-0a..0d -> RB-1 -> RB-2 -> RB-3 -> RB-4a -> RB-4b -> RB-5 ->
RB-6 memo -> RB-7. STOP+REPORT: after the RB-0 band (front door truthful);
after RB-3 (dispatch rebuilt, drift class dead); after RB-5 (numbers in hand);
at RB-6 (the decision is Jon's); final workstream report with measured
results, machine-local evidence labeled as such.
```

---

## §3 · Per-slice spec & acceptance criteria

Common to all: one coherent slice per PR; the §2 verification ladder green; PR body records
exact commands + outputs + honest verdict; claim boundaries stated.

### RB-0a — `cargo xtask truth` (Punchlist Part C)
Extend the existing `xtask` crate with a `truth` command.
- Emits `docs/truth.json`: `version` (workspace Cargo.toml), `primitive_count` +
  `primitives_by_layer` (from `all_prims()`), `workspace_test_count` / `security_test_count`
  (from the test-list or CI summary source the repo already trusts), `tracked_slices`,
  `readiness_pct` (from the readiness reporters), `latest_tag`.
- Stamps values between `<!-- truth:KEY -->…<!-- /truth -->` markers in README.md and FAQ.md;
  `docs/index.html` "By the Numbers" reads `truth.json` (or is stamped, pick one and document it).
- `xtask truth --check` exits non-zero on any mismatch between machine truth and public surfaces.
**Accept when:** the guard demonstrably FAILS on a deliberately planted mismatch (include the
proof run in the PR body), then passes clean; agent-contract and docs checks green. *Wiring
`--check` into CI is a gate change → propose it in the PR body, Jon approves the CI hook.*
**Design note [2026-06-11 infusion, queued — NOT required for RB-0a acceptance]:** the
truth-guard family extends from numbers to *semantic claims* via caps-claims-as-doctests
(Directive 10): every `@caps`/`@bounded` claim in documentation becomes a compiled,
trap-tested example, so docs-vs-enforcement drift becomes a build failure. Mechanism queued
post-RB-0a; RB-0a ships the numeric guard only.

### RB-0b — README replacement
Land `README_PROPOSED.md` as `README.md` — **verify, then land**: every link and path resolves
in-repo (logo path, docs/release-signing.md, spec/matrix paths), every number passes
`xtask truth --check`, every claim matches CURRENT_STATE/status-page boundaries. Relocate the
six `python3 scripts/garnet_*` walkthroughs to `docs/internals/` (link from README's Learn-more
line if useful). Purge internal vocabulary (Objective/Continuation Pulse, bare S-numbers,
"manifested dogfood bundle") from README only — internal docs keep their language.
**Accept when:** ≤180 lines; zero dead links (`lychee` or a scripted check); truth markers
stamped; the calibrated-honesty table reads exactly as bounded as before or tighter.

### RB-0c — Version-narrative fixes (Punchlist A2)
README:193 post-v0.5.0 → v0.8.1 narrative; FAQ:55/57/49 v0.5.0-era claims re-anchored; S1-LSP →
S16 surface description. **Accept when:**
`rg -n "post-v0.5.0|24 stdlib|24 registry|24 bridged|v0.5.0 is research-grade|S1 LSP"` returns
zero hits on public surfaces (README, FAQ, docs/), and the *boundary language itself*
("production VM performance is not claimed") survives verbatim.

### RB-0d — Site stats, VSIX prep, blog draft
Site By-the-Numbers reads `truth.json` (A5). Build a 0.8.1-versioned VSIX from the release
toolchain + regenerated `SHA256SUMS(.asc)` and stage it (A3) — **the asset swap on the published
release is escalated to Jon, never autonomous**. Draft the @caps blog post (A4) in the blog's
existing voice; posting and dating are Jon's. **Accept when:** artifacts staged with checksums,
draft committed under a drafts path, escalation note filed, CURRENT_STATE VSIX row updated to
match whichever decision Jon makes.

### RB-1 — Caps lattice → bitset (verdict R3)
`CapSet(u16)` bitflags over the closed set (fs, net, net_internal, time, proc, ffi, env, star,
+ documented reserve bits); `Copy`; propagation = bitwise OR over the call graph; subset =
`a & !b == 0`; **diff-caps delta = XOR**. Keep the old set-based impl during the slice and run
differential property tests (proptest: random cap-sets and call-graphs → identical propagation,
coverage errors, and diffs); delete the old impl in the same PR once green.
**Accept when:** all existing caps/diff-caps/coverage tests pass unchanged; differential suite
green; `.clone()` count in `garnet-check/src` drops from 201 to <40; an honest perf note records
only what was measured; **[2026-06-11 infusion] diff-caps emits a structured machine verdict
(`--machine` JSON: verdict, gained/dropped caps, bounds deltas) as a day-one criterion alongside
the human one-glance artifact** (Directive 15 — the reviewer on the other side of the gate is
increasingly an agent; the XOR delta is the natural payload).

> **RESOLVED 2026-06-12 (Jon, J1 + J2).** (J1, clone criterion) The "201 → <40"
> figure was not reproducible: measured baseline at the slice commit was 188,
> the capability-SET clones (the R3 subject) went **7 → 0**, total now 185.
> **RB-5's interner is the accepted vehicle** for the remaining String-key /
> dataflow-snapshot clones; the original number is amended honestly and RB-5
> re-measures. RB-1 ACCEPTED ([#388](https://github.com/Island-Dev-Crew/garnet/pull/388)).
> (J2, Directive-15 bounds deltas) `--machine` ships verdict + gained/dropped
> caps + band; **bounds deltas are NOT part of the declared-caps surface
> diff-caps reads** (the JSON `scope` field says so). The accept-when is amended
> to drop bounds from the day-one payload; a bounds-bearing attestation is a
> later **human-approved caps-surface / manifest-standard extension** (RFC-gated),
> not an RB-1 criterion.

### RB-2 — Crash-surface sweep (verdict R6)
`#![deny(clippy::unwrap_used, clippy::expect_used)]` on `garnet-cli`, `garnet-interp`,
`garnet-stdlib` (user-facing paths), with a single documented allowlist pattern
(`#[allow] // INVARIANT: <why this cannot fail>`) for true internal invariants. Reachable aborts
become miette diagnostics with spans (miette is already wired). Run the existing parser fuzz
harness plus an interp smoke pass over malformed corpus inputs.
**Accept when:** deny lints active and clippy green; unwrap counts in user paths at/near zero
with every allowlisted invariant justified in-line; fuzz/malformed-input smoke (state minutes
run) produces zero aborts — scoped claim: "no abort on the corpus + N fuzz minutes," never
"never panics."

> **RESOLVED 2026-06-12 (Jon, J3 + J5 + J8).** (J3) A **second sanctioned
> allow-comment form is blessed** beyond `// INVARIANT: <why this cannot fail>`:
> `// FAIL-CLOSED: <why aborting is the contract>` for sites that genuinely
> CAN fail where aborting is the documented safety posture (e.g.
> `machine_key` — cache integrity must not fail open). It must cite the
> documented contract; calling such a site an "invariant" would be false.
> (J5, add/sub/mul integer overflow) Ruling: **checked-error by default**,
> with explicit wrapping operations where wrapping is wanted — a
> language-semantics decision recorded in `rfcs/0002-integer-overflow-policy.md`;
> implemented as its own slice, not folded into RB-2. (J8) The
> `eval`/`repl`/`test` lanes lack the `garnet run` interpreter panic firewall —
> **scheduled as a small follow-up slice.** RB-2 ACCEPTED ([#389](https://github.com/Island-Dev-Crew/garnet/pull/389)/[#390](https://github.com/Island-Dev-Crew/garnet/pull/390)).

### RB-3 — Registry-derived dispatch (verdict R1) · the keystone
New `garnet-prim-macros` crate. One attribute per native:
`#[garnet_primitive(module="std::json", name="parse", caps(fs), arity=1, layer=Std,
stability=Experimental, doc="…")]` — the macro emits the `PrimMeta` row, the `NativeFn`
registration, the arity/argument guards, and the caps requirement from one declaration, behind
**one fallible bridge signature** (no unwraps across the bridge — composes with RB-2).
`all_prims()` and the interpreter's `install()` become derived; `stdlib_bridge.rs` hand-written
arms are deleted down to a thin shim.
**Accept when:** all (80 at audit) primitives registered via attribute with zero behavior
change — differential test drives every primitive through old and new dispatch on a shared
fixture corpus before the old path is deleted; `truth.json` `primitive_count` now derives from
the macro registry; net LOC drops by roughly two thousand lines; prim doc strings survive into
PrimMeta (RB-7 depends on them).

> **RESOLVED 2026-06-12 (Jon, J6 + J7 + J9).** (J6, LOC) The "~−2000 lines"
> expectation is **amended — accept the architecture rationale.** Adapter
> bodies cannot be deleted (the caps-enforcement gate greps their literal
> `require_capability` text; the `Value`-conversion logic is real code), so
> measured net is **+752**. The real win was **killing the hand-synced-lists
> dispatch-drift class**, not LOC reduction; LOC is no longer an accept-when.
> (J7, mechanism) **Ratified as the gate-compatible R1 realization:** the
> registry row (not the attribute) carries metadata so the textual gates keep
> parsing; the differential ran as table/fn-pointer identity (subsumes
> fixture-corpus execution); `all_prims()` stays the hand-written declaration
> table; "all 80 via attribute" reconciles as **78 adapter-registered + 2
> deliberately Unbridged registry rows + 4 BRIDGE_ONLY natives.** (J9) **Add
> `trybuild` UI tests before Core Ring work begins** (the Ring extends the
> macro surface). RB-3 ACCEPTED ([#393](https://github.com/Island-Dev-Crew/garnet/pull/393)).
**Downstream consumer, named [2026-06-11 infusion]:** the **Core Ring** — the curated, sealed
binding set (reassessment §6: ~20–30 audited bindings, every function's authority declared and
verified) — consumes this slice's `#[garnet_primitive]` binding factory: registry-derived
dispatch makes adding an audited binding a declarative act instead of a bridge-file edit. The
Ring itself is a **post-RB-3 W-SHIP workstream** (Ring Tier 1 + the MCP/tool-server library are
a W-LAUNCH gate condition), not part of RB-3's acceptance.

### RB-4a / RB-4b — One syntax substrate (verdict R2)
**4a:** migrate the remaining LSP/formatter/consumer paths to `garnet-cst` (rowan); **delete**
`garnet-parser-v0.3/src/cst.rs` (the self-described "temporary legacy oracle").
**Accept when:** the legacy file is gone, LSP test suite green, byte-identical round-trip corpus
green, no consumer imports the legacy module.
**4b:** AST node types become typed views over the rowan green tree (rust-analyzer
architecture) rather than a parallel structure. **Accept when:** parser + interp + check suites
green with zero behavior change; spans/diagnostics quality preserved or improved.

> **RESOLVED 2026-06-12 (Jon, J4).** Threading miette spans through RUNTIME
> diagnostics (the RB-2 deferral — runtime aborts surface span-less) is
> **scheduled as a follow-up before public playground work** and does NOT
> block the current RB-4b sub-slices. Parse-layer diagnostics already carry
> spans.
**Additional RB-4 criterion [2026-06-11 infusion] — the GHC-Core pattern (Directive 7):** the
typed core IR **carries capabilities in its type system**, and the caps invariant is
**RE-CHECKED AFTER EVERY LOWERING PASS** — a pass that launders authority is caught at the pass
that introduced it, not as a downstream surprise; "the seal attests what the core proves" stays
mechanically true through the whole pipeline, and no future backend can silently widen
authority during codegen. (GHC re-typechecks Core after every optimizer pass; SPJ: "I know of
no other production compiler that has this property." Garnet's version checks the caps lattice.)

> **RESOLVED-PARTIAL by RB-4b.3 ([#403](https://github.com/Island-Dev-Crew/garnet/pull/403)
> → `012021a2`, 2026-06-14).** The criterion above states the *full* Directive-7
> vision — a typed core IR that **carries capabilities in its type system**,
> re-checked **after every lowering pass**. RB-4b.3 realizes the **first,
> concrete instance** of that vision and nothing more, stated honestly:
> `garnet_vm::caps_recheck` is a **static cross-IR caps-containment check**
> (lowered ⊆ declared) on the **one** lowering pass that exists today
> (AST→bytecode), with a **deterministic planted-laundering trap**. It is NOT a
> typed-caps core IR (caps are re-derived from the lowered bytecode's `Call`
> instructions × the stdlib registry, not encoded in a type), NOT "every pass"
> (there is one pass), NOT runtime enforcement (interp S90 / VM S92 own that),
> and NOT a backend (RB-6). Fallback (non-native) functions are skipped (they
> run under S90 guards); embedding the verdict into the seal predicate is
> RFC-gated. So "the seal attests what the core proves" is mechanically true
> **through this lowering**, and the trap guards against a **future** pass
> widening authority — but the typed-caps core IR and the multi-pass property
> remain the aspirational target a future typed-core / backend slice would
> complete. Do not read the criterion above as fully delivered.

**Design note [2026-06-11 infusion] — editions as the surface-collapse vehicle (Directive 9):**
the graduated-syntax collapse ships as a **new edition** with per-module opt-in and
mix-and-match interop — old-surface modules compile forever, no flag-day, never a Python-3
decade. Note the convergence: the repo's edition mechanism (S32) + GOVERNANCE.md already declare
editions RFC-gated with capability semantics edition-invariant; this note binds the collapse to
that existing vehicle rather than inventing a new one. Mechanism work lands with RB-4; the
design note costs nothing now.

> **RESOLVED by RB-4b.4 (2026-06-14).** The editions note is now **parked in
> `garnet-parser-v0.3/AGENTS.md` ("Editions (spec note)")**, following the
> `garnet-cst/AGENTS.md` precedent so `GARNET_v1_0_Mini_Spec.md` stays under the
> maintainer's hand. It records the landed mechanism (`Edition::{V1_0, Next}`,
> `async` reserved under `v2.0`, the one-canonical-IR invariant, `Garnet.toml`
> pinning) as FACT and the Directive-9 surface-collapse as design intent bound
> to the existing RFC-gated vehicle — explicitly *not yet built*. A parser
> Stable-Contract bullet now also locks "editions gate lexing only; caps
> edition-invariant; new edition = RFC-gated."
**Queued (not required for workstream completion):** error-recovery + incremental reparsing on
the unified substrate — schedule into the playground band where it pays first.

> **RB-4b decomposition + 4b.2 re-scope (Jon, 2026-06-12); landing status as of 2026-06-14.**
> RB-4b shipped as four sub-slices, all merged:
> · **4b.1 substrate-fidelity** ([#400](https://github.com/Island-Dev-Crew/garnet/pull/400)
> → `2cb832d`): `cst_to_ast` span-exact + error-verdict-equal.
> · **4b.2 `SyntaxError` spans + single-parse finding**
> ([#402](https://github.com/Island-Dev-Crew/garnet/pull/402) → `cf65e51`).
> · **4b.3 Directive-7 caps re-check on VM lowering** + planted
> authority-laundering trap ([#403](https://github.com/Island-Dev-Crew/garnet/pull/403)
> → `012021a2`); see the RESOLVED-PARTIAL note above for its honest scope.
> · **4b.4 editions note + spec reconciliation** (this block + the parser
> AGENTS.md editions note).
> **4b.2 was re-scoped** from "typed views + LSP single-parse" to `SyntaxError`
> spans + the single-parse finding: dropping `parse_source` from the LSP would
> DEGRADE diagnostics (`parse_cst`'s error recovery cascades — 8 errors for
> `@@@ def` vs one fail-fast), which the accept-when forbids, and the typed
> views have no adopter yet (extending them now = speculative). `SyntaxError`
> now carries a token-range span (foundation); **true single-parse is deferred
> until parser error-recovery is de-noised** (a follow-up, gated on
> diagnostic-quality parity).
>
> **The "4b" accept-when ("AST node types become typed views over the rowan
> green tree") is DEFERRED, not delivered.** RB-4b.1 made `cst_to_ast` a
> span-exact *projection* (the CST and AST remain parallel structures); the
> rust-analyzer-style typed-view collapse waits for a real adopter, per the
> 4b.2 finding. Do not read RB-4b as having unified AST and CST into one typed
> structure — it unified the *substrate* (one rowan CST, RB-4a) and proved the
> projection faithful, while the AST stays a parallel structure downstream
> consumers still use via `parse_source`.

### RB-5 — Environment rebuild (verdict R4)
String interner (e.g. `lasso`, or a small owned interner — integrate-grade either way) +
a resolution pass in `garnet-check` assigning `(depth, slot)` to every binding; interpreter
`Env` becomes indexed frames instead of the `Rc<RefCell<HashMap<String,Value>>>` parent chain.
**Accept when:** ZERO semantic change — full workspace + enforcement-parity gates green;
criterion before/after on the existing bench harness committed as machine-local evidence with
exact hardware noted; the honest sentence is "measured Nx on these benches on this machine,"
nothing broader.

> **STOP+REPORT landed + SEQUENCING DECISION (Jon, J·Option-C, 2026-06-14).**
> RB-5 stopped at the design gate — the `(depth,slot)` indexed-frame rewrite is
> not a safe single slice (no AST node identity; REPL incremental binding;
> five-chain `Env` + capture-by-reference closures; all reduce to a
> name-representation change in the shared AST or a resolved IR). Report:
> `RB5_ENV_REBUILD_STOP_REPORT_2026-06-14.md`
> ([#405](https://github.com/Island-Dev-Crew/garnet/pull/405) → `04be4974`),
> measured baseline `eval_fib_15` 394.97 µs / `eval_array_1000` 262.56 µs /
> `eval_expr` 1.475 µs (M5 Pro). **Jon's ruling: Option C — sequence RB-5 with
> RB-6, NOT a rejection.** RB-6 decides the durable IR/execution shape; THEN RB-5
> lands `(depth,slot)`+interner against that representation with
> build-both-compare and zero semantic drift. Guardrails: (1) preserve the
> just-stabilized parser/CST/AST substrate; (2) **no** standalone AST
> name-representation change **unless RB-6 explicitly keeps the AST as the
> execution substrate**; (3) RB-7 may run only if it does not collide with
> RB-6/RB-5 surfaces. The RB-5 baseline is RB-6's before-number.

### RB-6 — Backend decision memo (verdict R5) · Jon-gated, no code

> **MEMO DRAFTED + spike evidence prepared (2026-06-14); DECISION is Jon's.**
> `RB6_BACKEND_IR_DECISION_MEMO.md` carries: the §0 mandate (the Option-C
> sequencing decision above), the §2 tree-walk before-number, the **§3 wasm32
> feasibility spike** (`wasm32-wasip1` **compiles today** (cargo `Finished` + 16 MB rlib);
> `wasm32-unknown-unknown` has ONE blocker, `getrandom/js`; the host-authority
> surface is concentrated in the `@caps` primitives = the WASI import boundary),
> the §4 IR options A–D + the custom-VM-as-third-path parity cost, the §5 synergy
> ledger, the §6 Stroustrup doctrine, the §7 per-pass caps re-check HARD
> CONSTRAINT (RB-4b.3 already landed the mechanism — Option C inherits it), and
> the §8 integrate-lean recommendation (Option C now as the `(depth,slot)`+
> interner target; Wasmtime/WASI as the strategic back-half) with the measured
> ~2–3× reopen threshold. **§10 escalates the decision to Jon.** No backend
> merged; no `.wasm` executed; feasibility-compile only.
A decision document, not a slice: post-RB-5 tree-walk numbers; a wasm32 build feasibility spike
(does the interpreter compile to wasm32-unknown / wasm32-wasi today, and what blocks it);
the synergy ledger (one lowering buys @bounded→Wasmtime fuel + @caps→WASI sandbox + the
playground + embed-everywhere); the parity cost of keeping the custom VM as a third path; a
recommendation. **Accept when:** memo + spike evidence committed; decision escalated to Jon;
nothing merged into a backend.
**Memo template additions [2026-06-11 infusion]:** the memo additionally carries —
1. **The Stroustrup linker doctrine** (Directive 12) as the stated frame for
   integrate-vs-rebuild: *"we can have Dennis's mistakes, which we know, or my mistakes, which
   we don't know yet"* — Cranelift/Wasmtime is Garnet's "C."
2. **The per-pass caps re-check (RB-4 criterion) as a HARD CONSTRAINT on ANY backend
   candidate:** a backend that cannot re-verify the caps invariant per lowering pass is
   disqualified regardless of its performance numbers.
3. **The integrate-lean recommendation with its reopen threshold stated:** lean integrate
   (per the doctrine line and the research convergence in the Gap-6 appendix); revisit a
   custom lowering only if the integrated path measures **~2–3× overhead on representative
   workloads** — measured, machine-named, never assumed. The decision itself remains Jon's.

### RB-7 — The REPL joy slice (verdict R7)
Rebuild `repl.rs` (125 lines today) on `reedline`: history, multiline, tab-completion fed by
the macro-derived registry + live environment; `?std::json::parse` prints doc + caps + arity
from PrimMeta; `:caps` shows the session's live authority budget; pretty-printed values.
**Accept when:** recorded demo evidence (asciinema or equivalent) committed; doc page added;
Mac proof recorded, cross-OS proof handed to the NUC lane — never marked cross-OS-complete from
one machine.

### RB-8 — OPTIONAL, Jon-gated: de-suffix crates + root flatten
`garnet-parser-v0.3` → `garnet-parser` etc., research corpus to `research/` with history
preserved. **Deliberately last and gated:** ~229 scripts plus CI reference current paths; the
command center flags this as do-not-rush. If green-lit: full reference sweep, one PR, every
reporter re-run. If not green-lit: it waits, and nothing else in W-REBUILD depends on it.

---

## §4 · Stop points & escalation matrix

| Moment | Action |
|---|---|
| After RB-0 band | STOP + report: front door truthful, guard live |
| After RB-3 | STOP + report: dispatch rebuilt, drift class structurally dead |
| After RB-5 | STOP + report: measured numbers in hand |
| RB-6 | Jon decides the backend; lane prepares evidence only |
| VSIX asset swap, blog posting/dating, CI hook for `truth --check`, RB-8 | Jon, always |
| Any semantic delta in RB-1..RB-5 | STOP + report immediately — never patch around it |

---

## §5 · Transition guidance

**Before (the freeze):** fleet reports per `TEMPLATE.md` on every active machine (read-only,
parallel, ~30 min each) → MacBook Pro assembles the docs-only consolidation PR → every local
artifact gets its verdict (commit / archive / ignore). This is precisely the scatter-management
you flagged: the deprecate-or-integrate decision happens **once, before** the foundations move,
against a single visible truth — instead of repeatedly, afterward, against a moving one.

**During:** W-REBUILD runs on the lead lane. Parallel-safe for other lanes: trust-band evidence
packaging (Air), Windows/Linux/Tauri smoke + packaging plans (NUC), public-story audit (Air).
Not parallel-safe while RB-1..RB-5 are in flight: anything that edits `garnet-check`,
`garnet-interp`, `garnet-stdlib`, `garnet-parser`, `garnet-cst` — those surfaces are frozen to
the lead lane until the workstream report.

**After (the synthesis session, per Codex's suggestion — together):** with foundations solid,
synthesize the remaining goal modes as a set, each inheriting the W-REBUILD pattern (deploy
gate → slices → ladder → stop points): **W-PLAY** (playground band, S151–S165 — consumes the
RB-6 decision and the queued error-recovery work), **W-TRUST** (independent S114 + SLSA/Sigstore,
S141–S150 — independent lane, never self-graded), **W-SHIP** (distribution, S166–S178 — NUC-led),
**W-REACH/W-LAUNCH** (S179–S200 — drafts prepared, the public moment Jon-owned). The
synthesis input is three documents: the W-REBUILD final report, the consolidated fleet truth,
and the command center — nothing else should be needed, which is itself the test that the
foundation held.

---

**Kickoff invocation (Jon's pattern):**
> "Continuing Garnet. Read `F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md` and
> `F_Project_Management/GARNET_S129_S200_ECC_DOGFOOD_COMMAND_CENTER.md`. Verify the deploy
> gate, then execute W-REBUILD per the §2 goal mode. Proceed decisively."

*"Where there is no vision, the people perish." — Proverbs 29:18*
*One PR per slice. Evidence, never claims. The tag is Jon's. Roll Tide.*
