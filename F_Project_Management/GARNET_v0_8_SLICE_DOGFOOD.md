# GARNET v0.8 SLICE DOGFOOD CONTRACTS

Date: 2026-05-30
Purpose: Single source of truth for every v0.8 PR (S31–S80). Read by Claude
Code, Codex Desktop, Greptile/PR-Agent, the `dogfood-readiness` skill, and Jon.
Update this file in the same commit as the work it tracks.

The v0.7 successor of `F_Project_Management/GARNET_v0_7_SLICE_DOGFOOD.md`
(S15–S19 closed; the repo carried through S30). This file governs PRs titled
`S31:`–`S80:` and any later v0.8 slice added under § Slice Contracts.

**Map vs. contracts.** The strategic spine — why each slice exists, the
seam-by-seam Opus×Codex reconciliation, and the load-bearing bets — lives in
`F_Project_Management/SLICE_PLAN_RECONCILED_OPUS_X_CODEX.md`. **That document is
the map; this file is the per-slice acceptance contract.** Where they ever
disagree, the map states intent and this file states the merge bar.

---

## How v0.8 is built (differs from v0.7's fixed 4-slot split)

v0.7 used a fixed four-agent slot table. v0.8 is a **long-running goal-mode run**
governed by the upgraded `dogfood-readiness` skill (`Navigata1/dogfood-readiness`):

- **One slice per PR**, title `S<N>: <short>`, branched fresh from `origin/main`.
- **Merge confidence is fused and gated.** Each PR earns a 1–5 band from the
  falsification ledger, fused (`min`) with an external reviewer (Greptile, and —
  once S37 lands — the `diff-caps` capability signal). **Merge only at fused
  5/5, or a recorded human deferral.**
- **The goal ledger** (`.dogfood/goal.json`) tracks objective completion across
  the whole S31→S80 run — "slice k of N merged," not a hand-typed percentage.
  Slice readiness and overall objective completion are **kept as separate
  numbers** and never blended.
- **Resolution decreases past S60.** S31–S60 are planned in detail; S61–S80 are
  bets, re-sliced as reality arrives. Detailed contracts are authored as each
  band approaches.

---

## Version-tag bands (no 1.0 in the S31–S80 window)

| Band | Slices | Tag |
|---|---|---|
| v0.8 foundation | S31–S40 | (in flight) |
| v0.8 hardening | S41–S50 | v0.8 beta gate @ S50 |
| v0.8 adoption / release | S51–S60 | **v0.8.0 @ S60** |
| v0.8.1 (resolution ↓) | S61–S70 | **v0.8.1 @ S70** |
| v0.8.2 runway (resolution ↓) | S71–S80 | **v0.8.2 readiness decision @ S80** |

**1.0 is held past S80** until the bet stages validate. Both source plans agree.

---

## Slice State Machine

  not-started → planned → in-progress → review-ready → dogfood-passing → merged

| Transition | Required artifact |
|---|---|
| not-started → planned | Plan file at `.agent/plans/S<N>-plan.md` referencing this contract + the map by section |
| planned → in-progress | Draft PR open, title `S<N>: <short>`; goal ledger created/known |
| in-progress → review-ready | CI green · PR body uses the dogfood-readiness headings · dogfood block run locally with output attached |
| review-ready → dogfood-passing | Falsification audit run; fused merge confidence computed; grep loop driven toward 5/5 |
| dogfood-passing → merged | Fused **5/5** (or recorded human deferral) · squash-merged · CHANGELOG `[Unreleased]` updated · readiness baseline regenerated if a lane was added · goal ledger advanced |

Backward moves are allowed and require a one-line "regression note" in the PR body.

---

## Common Verification Primitives

Every slice's CI run executes these on top of its own block (inherited from v0.5–v0.7):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
cargo deny check
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
python3 scripts/garnet_conformance_matrix_check.py
python3 -m unittest discover scripts/ -p 'test_*.py'
```

---

## Cross-Slice Gates (every PR)

| Gate | Where enforced | Failure mode |
|---|---|---|
| `@caps` declared on new authority | `garnet check` in CI | Hard fail |
| Determinism preserved (committed-truth identical on every machine) | S9 cross-machine matrix + S31 reporter split | Hard fail |
| Determinism job never spawned with `--llm` | S19 CI guard | Hard fail |
| No new ambient `unsafe` | `cargo clippy` + audit | Hard fail |
| Honest voice in docs | Jon review | Block until corrected |
| Dogfood-readiness headings present | `.github/workflows/dogfood-readiness.yml` greps PR body | Hard fail |
| Fused merge confidence = 5/5 (or recorded human deferral) | `dogfood-readiness` skill grep loop | Block merge |
| `cargo deny check` clean | CI | Hard fail |
| Node-24 action minimums | `scripts/test_github_actions_node24_readiness.py` | Hard fail |
| MIT-readiness baseline regenerated if a lane was added | S0 `--check-no-regression` | Hard fail |
| Capability/authority change reviewed (diff-caps once S37 lands) | S37 `diff-caps` → fused band | Block 5/5 |

---

## Slice Contracts — S31–S40 (v0.8 foundation, detailed)

### S31 — v0.8 release truth + slice ledger + readiness contract

**PR count:** 2 (PR-1 foundation/doctrine; PR-2 deterministic-reporter fix).

**Goal:** Stand up the v0.8 map, this contracts file, and the **readiness
contract** the whole run is graded against — and **adopt** the upgraded
`dogfood-readiness` toolkit (fusion + grep-loop-to-5/5 + goal-mode ledger)
rather than rebuilding readiness machinery.

**PR-1 (this PR) — foundation & doctrine (docs + gate reconciliation):**
- `F_Project_Management/SLICE_PLAN_RECONCILED_OPUS_X_CODEX.md` — the merged spine (map).
- `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` — this file.
- **Adopt the toolkit:** reconcile `scripts/check_dogfood_pr_body.py` so the new
  skill's PR-body shape (`### Evidence bundle`, `### Merge confidence`, `### Goal
  progress`) passes the existing garnet CI gate *and* the legacy `### Desktop
  dogfood bundle` heading still validates (accept either as the evidence
  section). Update `scripts/test_check_dogfood_pr_body.py` to lock both.
- `.dogfood/goal.json` — the persisted goal ledger for the S31→S80 objective
  (committed so every session/machine reads the same "slice k of N").
- `CHANGELOG.md` `[Unreleased]` header corrected to current release truth
  (was `v0.6.0 in flight`; the repo is past that — relabel to `v0.8.0 in flight`).

**PR-2 — deterministic-reporter fix (substantive, separate PR):**
- The readiness reporter's machine-dependence (observed 85.2 vs 80.6 across
  machines) is a **bug that gates the cross-machine hand-off**, not just
  doctrine. Split **committed-truth** (the % that must be byte-identical on every
  machine) from **local-evidence** (machine-specific observations reported
  separately). Add the `reporter_determinism` lane + baseline regeneration here.

**Dogfood block (PR-1):**

```bash
# Toolkit is green and usable:
( cd /tmp/dogfood-readiness && python3 -m unittest discover -s tests )   # 32 ok
# The garnet PR-body gate accepts the new skill's headings:
python3 scripts/check_dogfood_pr_body.py --base origin/main --head HEAD --body-file <pr-body>
python3 -m unittest scripts.test_check_dogfood_pr_body
# Goal ledger initialized and queryable:
PYTHONPATH=/tmp/dogfood-readiness python3 -m dogfood_readiness --goal-action status --goal-file .dogfood/goal.json
```

**Honest partial labels available:**
- "S31 PR-1 is doctrine + ledger + gate-reconciliation only; no runtime behavior changes and no new readiness lane — the reporter-determinism fix and its lane land in S31-PR2."
- "The goal ledger tracks the *planned* spine; slices past S60 are bets and may be re-sliced (resolution decreases by design)."
- "v0.8 version-tag mapping (v0.8.0@S60 …) is a plan, not a shipped tag."

**State:** in-progress (this PR).

---

### S32 — edition / compatibility model (two-layer)

**Goal:** Install the compatibility evolution mechanism *before* users exist to
break — the earliest load-bearing slice (both source plans independently placed
compatibility first).

**[GRAFT] Two layers:** editions (parse-time, opt-in syntax/semantics shifts)
**+** GODEBUG-style runtime settings (semantic-time). Enforce a
**skin-deep / one-canonical-IR** invariant so **capability semantics are
edition-invariant** — an edition may change surface syntax but never what
authority a program holds.

**Dogfood block (target):** a program pinned to edition N and one to edition N+1
produce the **same capability manifest**; a GODEBUG-style toggle changes a
runtime default without changing the manifest.

**Honest partial labels available:** "Editions cover parse-time evolution;
runtime-settings layer may land staged." "Edition migration tooling is minimal in v0.8."

**State:** dogfood-passing (S32 PR). Two-layer mechanism shipped:
`garnet_parser::Edition` (`v1.0` default + a registered `v2.0` that exists only to
prove the mechanism), `[project].edition` resolution with legacy
`[package]`/`garnet-0.3` alias warning and unknown-edition hard error, the
lexer-only `async` edition-gated reserved word, the one-canonical-IR
AST/manifest-invariance invariant, and the `GARNET_DEBUG` GODEBUG layer
(`diagnostics` toggle; unknown keys warn). 22 unit/integration tests; proven
end-to-end through `garnet check` / `run --interp`. Lane `edition_compatibility`
added; baseline regenerated. Flips to `merged` on squash; the ledger
`s32 → merged(5)` advance rides with the S33 PR (keeps main honest). Honest
scope: mechanism + invariant only — no per-edition migration catalog, `--vm` on
the default edition, no manifest `[runtime]` table, no Mini-Spec edit (§16.3
already specifies the canonical form).

---

### S33 — one-command `garnet verify`

**Goal:** A single acceptance gate a user/agent runs locally and CI runs
remotely. **[GRAFT]** Build it to accept a **pluggable capability signal**
(diff-caps wires in at S37) and to **fuse** the internal readiness band with an
external reviewer (Greptile) via `min`; drive the grep loop to 5/5.

**Dogfood block (target):** `garnet verify` exits non-zero on a planted
regression; exits zero on a clean tree; emits the fused merge-confidence band.

**Honest partial labels available:** "Until S37, the capability signal slot is a
stub — the fused band uses the external reviewer + internal ledger only."

**State:** dogfood-passing (S33 PR). `garnet verify <path>` acceptance gate added
(`garnet-cli/src/verify_gate.rs` band+`min`-fusion logic + `cmd/verify_gate.rs`
CLI; routed by positional-arg count vs the 2-arg manifest verify): edition-aware
parse + safe-mode check over a file/dir, fused merge-confidence band (internal
`min` external `min` capability-signal), exit 0 on a clean tree / non-zero on a
planted regression. 6 unit + 4 integration tests + end-to-end CLI smoke
(clean→0/band5, planted→1, `--external-band` caps via `min`, dir walk, 2-arg
manifest verify preserved). Lane `garnet_verify_gate` added; baseline regenerated
(88.3%→88.6%). Capability-signal slot is a STUB until S37 `diff-caps`. Flips to
`merged` on squash; the `s33 → merged(5)` ledger advance rides with the S34 PR.

---

### S34 — structured diagnostics (machine + human)

**Goal:** Diagnostics with both a human-readable and a machine-parseable form,
an authoritative exit code, and the foundation for serving over MCP.

**State:** dogfood-passing (S34 PR). `garnet_cli::diagnostics` (Severity +
Diagnostic { severity, code, message, span }) + `garnet check --format
human|json`. json = deterministic hand-rolled JSON (a `diagnostics` array with
stable per-variant codes + a `summary`), no serde. Authoritative exit code
documented (0 clean / 1 fatal-or-parse-or-IO / 2 usage). 5 unit + 4 integration
tests (binary via `CARGO_BIN_EXE`). Honest scope: machine form on `garnet check`
only (parse/verify JSON + the MCP transport are follow-ups); check diagnostics
are span-less today (the `CheckError` variants are message-only), parse ones
carry spans. No new readiness lane (not mandated for S34 by the contract lane
table). Flips to `merged` on squash; the `s34 → merged(5)` advance rides with
the S35 PR.

---

### S35 — source annotations (`@caps` syntax)

**Goal:** The annotation syntax itself — the surface from which the capability
manifest is later derived. (Decomposition correction from the reconciliation:
annotations → manifest → diff, as three slices, not one assumed-existing syntax.)

**State:** dogfood-passing (S35 PR). NOTE: the `@caps` syntax already existed
(v3.4 CapCaps — `Annotation::Caps`, `Capability`, parsing, propagation); per the
goal's actual words this slice adds **the surface the manifest is derived from**:
`garnet_check::CapabilitySurface` + `capability_surface(module)` (aggregate /
per_function / has_wildcard; canonical `Capability::as_str()` normalization,
sorted + deduped). Consolidates + fixes `garnet trust-report` (was using
Debug-rendered caps, mislabeling net_internal / Other / wildcard). 7 unit tests;
trust-report tests stay green. No new readiness lane (not mandated). Honest scope:
surface artifact + bug-fix, not new syntax; top-level declared caps only; S36
builds the manifest from this surface. Flips to `merged` on squash; the
`s35 → merged(5)` advance rides with the S36 PR.

---

### S36 — capability manifest (derived from annotations)

**Goal:** Derive a per-program/per-package capability manifest from S35's
annotations. The manifest is the artifact `diff-caps` (S37) and `seal` (S38)
both consume.

**State:** dogfood-passing (S36 PR). `garnet_cli::cap_manifest::CapabilityManifest`
(schema `garnet-capability-manifest-v1` + the S35 `CapabilitySurface`) + the
`garnet caps <path>` command: deterministic JSON (`{schema, aggregate, functions,
wildcard}`) for a file (per-program) or a dir (per-package via `merge_surfaces`).
Distinct from the build `Manifest` (no caps). 4 unit + 3 integration tests; CLI
smoke verified. The artifact S37 `diff-caps` compares + S38 `seal` embeds. Honest
scope: declared surface only (not an undeclared-authority proof — S46; no
`[caps]`-budget enforcement). No new readiness lane (mandated at S37/S38). Flips
to `merged` on squash; the `s36 → merged(5)` advance rides with the S37 PR.

---

### S37 — `diff-caps` (capability-surface diff as acceptance gate)

**Goal:** The headline novelty: diff the capability surface between two revisions
and gate on authority changes. **[GRAFT]** feeds the **same fused 1–5 band** as
Greptile in S33's `garnet verify` (`min` governs the gate).

**Honest partial labels available:** "diff-caps reads the declared capability
surface; it does not prove the absence of undeclared authority (that is the
sandbox-policy job, S46)."

**State:** dogfood-passing (S37 PR). `garnet_check::diff_caps` (→ `CapsDiff` +
`authority_expanded()`: new aggregate cap or introduced `@caps(*)`). `garnet
diff-caps <old> <new>` exits non-zero iff authority expanded. **Completes the S33
graft:** `garnet verify --caps-baseline <old>` feeds the diff into the fused band
via `capability_band` (5 / 2; `min` governs), replacing the stubbed signal.
Shared `surface_for_path` consolidated across caps/diff-caps/verify. 6 unit + 4
integration tests; smoke verified (expansion→exit1/band2, reduction→exit0/band5,
verify --caps-baseline caps fused at 2/5). Lane `capability_diff_caps` added;
baseline regenerated (88.6%→88.8%). Honest scope: declared surface only (not an
undeclared-authority proof — S46); verify flags via band, diff-caps is the hard
gate. Flips to `merged` on squash; the `s37 → merged(5)` advance rides with S38.

---

### S38 — `seal` (signed reproducible bundle)

**Goal:** A signed, reproducible build bundle. **[GRAFT] wrap-don't-rebuild:**
express the attestation as an **in-toto predicate**, verify via **cosign**, emit
a **CycloneDX/SPDX SBOM**; the capability manifest is the no-SBOM-equivalent
extension. Do not rebuild signing or metering primitives.

**Honest partial labels available:** "seal wraps in-toto/Sigstore/cosign; Garnet
does not implement its own signing." 

**State:** dogfood-passing (S38 PR). `garnet seal <file>` emits a deterministic
in-toto Statement v1 (subject = BLAKE3 AST digest; predicate embeds the build
manifest + the S36 capability manifest = native SBOM-equivalent). `garnet_cli::seal`
builds the predicate; `cosign` signs it (detected, never required). 3 unit + 2
integration tests; output validated as JSON. Lane `seal_attestation` added;
baseline regenerated (88.8%→89.1%). Honest scope (contract anchor): seal wraps
in-toto/Sigstore/cosign — Garnet does not sign supply-chain itself. cosign /
syft / cyclonedx are ABSENT in this environment → predicate emitted UNSIGNED (the
wrapper prints the `cosign attest` command, does not auto-sign), capability
manifest is the SBOM-equivalent. Per-file seal (per-package is a follow-up).
Flips to `merged` on squash; the `s38 → merged(5)` advance rides with the S39 PR.

---

### S39 — `@bounded`

**Goal:** Bounded-resource annotation (CPU/mem/mailbox). **[GRAFT]
wrap-don't-rebuild:** lower to **Wasmtime fuel** metering (revisit only if WASM
overhead exceeds ~2–3×).

**State:** dogfood-passing (S39 PR). `@bounded(N)` (CPU/fuel budget of N
Wasmtime-fuel units) threaded through all 5 `Annotation` sites (AST, grammar, the
rowan CST converter, checker validation, doc-span). `garnet_check::bounded_functions`
extracts declared budgets; `garnet bounds <file>` reports them. Checker rejects
`@bounded(0)`; negative literals are parse errors (consistent with @mailbox /
@max_depth). 6 unit + 2 integration tests; CLI smoke verified. No new readiness
lane (not mandated for S39). Honest scope (wrap-don't-rebuild): budget declared +
reported; ENFORCEMENT lowers to Wasmtime fuel — wasmtime/wasm-tools ABSENT here,
so declared not runtime-enforced (no fuel meter faked). Mem bounds + unified
resource syntax are follow-ups. Flips to `merged` on squash; the
`s39 → merged(5)` advance rides with the S40 PR.

---

### S40 — explosive-operation / default-ceiling analysis

**Goal:** Static identification of unbounded/explosive operations and enforced
default ceilings, closing the foundation band.

**State:** dogfood-passing (S40 PR) — **closes Phase A (S31–S40)**.
`garnet_check::explosive_ops` is a compiler-exhaustive AST visitor flagging
unconditional `loop` (every loop — static termination is undecidable) and `spawn`
(fan-out), reporting per function whether each is governed by a declared bound
(`@bounded` / `@fan_out`) or the default-ceiling policy (`DEFAULT_LOOP_CEILING` /
`DEFAULT_SPAWN_FANOUT`). `garnet ceilings <file>` reports it. 5 unit + 2
integration tests; CLI smoke verified. No new readiness lane (not mandated).
Honest scope: static identification + default-ceiling POLICY; runtime ENFORCEMENT
lowers to the S39 `@bounded` / Wasmtime-fuel path — deferred (wasmtime absent),
no ceiling faked. Explosive set = loop + spawn (recursion is already governed by
`@max_depth` + the caps call graph; unbounded collection growth is a follow-up).
Flips to `merged` on squash; the `s40 → merged(5)` advance (completing the phaseA
ledger) rides with the S41 PR (first hardening-band slice).

---

## Slice Contracts — S41+ (v0.8 hardening; detailed as each is approached)

### S41 — async/concurrency contract

**Goal:** Codify Garnet's concurrency model as a canonical contract. The model is
**actors** (not async/await — `async` is reserved for a future edition, S32),
already built in `garnet-actor-runtime` (OS-thread + bounded mpsc mailbox;
`@mailbox` override; Result-returning `ask`; hot reload) and Mini-Spec §9. This
slice documents what is built + adds a checkable surface — it introduces no new
semantics.

**Dogfood block:** `C_Language_Specification/GARNET_CONCURRENCY_CONTRACT.md`
codifies actors / bounded mailboxes / ask-vs-tell / spawn-`@fan_out` /
`@bounded` / `@nonsendable` / hot reload, with an explicit deferred-scope
section. `garnet_check::concurrency_surface` classifies each actor's protocols
(ask if it returns a value, else tell) + handler count; `garnet concurrency
<file>` reports it.

**State:** dogfood-passing (S41 PR). 2 unit + 2 integration tests; CLI smoke
verified. No new readiness lane (not mandated). Honest scope: documents the
EXISTING model; no async/await; `@nonsendable` cross-boundary enforcement and
`@bounded` fuel enforcement are deferred (declared/reported, not enforced — no
faking); structured concurrency / cancellation beyond actor lifecycle + the
Result-`ask` is future work. Flips to `merged` on squash; the `s41 → merged(5)`
advance rides with the S42 PR.

---

### S42 — typed Result / error policy

**Goal:** Codify the typed-`Result`-first error policy and enforce a piece of it.
`core::result` combinators (S26) and `try`/`rescue`/`ensure`/`raise` (Mini-Spec
§7) already exist. The reconciliation flagged Ronacher's insight — *"agents
over-catch exceptions"* — so the enforceable rule is the **over-catch** guard.

**Dogfood block:** `C_Language_Specification/GARNET_ERROR_POLICY.md` codifies the
two error channels (typed `Result` preferred; exceptions for the exceptional) and
the over-catch anti-pattern. `garnet_check::overcatch_sites` flags catch-all
`rescue` clauses (no exception type); `garnet check` emits a **non-fatal advisory**
`check.over_catch` (human + JSON) — never changes the exit code (excluded from
`CheckReport::ok`). A typed rescue (`rescue e: T`) is not flagged.

**State:** dogfood-passing (S42 PR). 3 unit + 3 integration tests; CLI + JSON
smoke verified. No new readiness lane (not mandated). Honest scope: the over-catch
check is **advisory only** (no exit-code change, no auto-rewrite, no ban); no
typed-exception hierarchy or checked-exceptions are introduced. Flips to `merged`
on squash; the `s42 → merged(5)` advance rides with the S43 PR.

---

### S43 — docs-as-tests

**Goal:** Make documented examples executable so docs cannot rot — the "evidence
not courtesy" discipline the reconciliation flagged (Codex S43). `garnet doc`
(v0.4.2) already extracts `///` blocks; S43 turns the ` ```garnet ` fences inside
them into runnable, value-asserted tests.

**Dogfood block:** `garnet doctest <file>` — `garnet_cli::doctest::garnet_fences`
(pure fence extractor) lifts ` ```garnet ` blocks from each item's reused
`extract_doc_comments_before` doc block; the runner `cmd::doctest` loads the
file's definitions once (`Interpreter::load_source`) and evaluates each fence
(`eval_expr_src`) so an example can call the function it documents. Pass = runs
without error; an optional `# => value` marker asserts the displayed tail value.
Human + `--format json`; exit 1 iff any fence fails. Demonstrator
`examples/documented_math.garnet` (3 passing examples) is itself dogfooded and
CI-gated by the `garnet-cli` integration test `advertised_demonstrator_passes`
(runs the built binary on the demonstrator and asserts 3 passed).

**State:** dogfood-passing (S43 PR). 6 unit + 3 runner-unit + 5 integration
tests; CLI human + JSON smoke verified. No new readiness lane (not mandated).
Honest scope: examples run on the interpreter (not the VM backend); fences see
only the file's own definitions + stdlib (no cross-file imports, matching
`garnet doc`); a doc-rot guard, not a replacement for the test suite. Flips to
`merged` on squash; the `s43 → merged(5)` advance rides with the S44 PR.

---

### S44 — LSP safe-mode / cross-package precision

**Goal:** Make the LSP's safe-mode/capability diagnostics precise and consistent
with `garnet check`. The reconciliation (§145) flagged the LSP as a semantic
service on the compiler frontend (distinct from tree-sitter).

**Problem:** `garnet-lsp::check_diagnostics` mapped every `CheckError` except
`BoundaryNote` to a red `ERROR`, and set no machine code. So the S42 over-catch
advisory and stability-advice showed as errors in the editor, and the editor's
codes diverged from `garnet check --format json`.

**Dogfood block:** the checker owns the single source of truth —
`garnet_check::Severity` + `CheckError::severity()` / `CheckError::code()`
(both compiler-exhaustive). `garnet-cli/diagnostics.rs` (S34) delegates to them
(via `From<garnet_check::Severity>`); the LSP maps `severity()` →
`DiagnosticSeverity` (Error/Warning/Information) and sets
`Diagnostic.code = code()`. Over-catch + stability-advice now surface as
`INFORMATION`; safe-mode/caps/annotation/stability-error as `ERROR`; boundary
notes as `WARNING` — editor and CLI in lockstep.

**State:** dogfood-passing (S44 PR). New: garnet-check `severity_and_code_are_canonical`
+ `error_severity_agrees_with_fatal_set`; LSP `over_catch_advisory_surfaces_as_information_with_code`
+ `safe_mode_violation_surfaces_as_error_with_code`. S34 CLI diagnostics output
unchanged. Full ladder green. No new readiness lane (not mandated).
**Honest scope:** this delivers the *safe-mode precision* half. *Cross-package*
precision requires a module/package resolver (S45) — the LSP has none today, so
that half is **deferred to ride with S45**; no cross-file resolution is claimed.
Flips to `merged` on squash; the `s44 → merged(5)` advance rides with the S45 PR.

---

### S45 — package resolver / slopsquatting guard

**Goal:** Harden the resolver against slopsquatting — the live threat
(reconciliation §146, §202-208: hallucinated package names attackers
pre-register). When a requested name is unknown but closely resembles a known
one, warn before anything is trusted.

**Dogfood block:** pure `garnet_registry_stub::slopguard` — Damerau–Levenshtein
(optimal string alignment) distance + separator-confusable detection
(`foo-bar` vs `foo_bar`), returning deterministically ordered near-misses (a
length-relative threshold suppresses unrelated short-name noise).
`RegistryIndex::known_names()` supplies the corpus. `garnet add --registry`
runs the guard **only** when `resolve` reports an unknown *name* (a missing
version is not a near-miss), enriching the error with *"did you mean `…`? …
a slopsquatting risk; verify the source before adding."* Exit code unchanged
(the add already fails on `NotFound`).

**State:** dogfood-passing (S45 PR). 6 guard unit tests
(`osa_handles_transposition_and_edits`, `flags_a_single_typo`,
`flags_separator_confusable_first`, `exact_match_is_not_a_near_miss`,
`distant_and_tiny_names_are_not_flagged`, `ordering_is_deterministic_best_first`)
+ 2 CLI integration tests (near-miss warns; version-miss stays quiet). Full
ladder green. No new readiness lane (not mandated).
**Honest scope:** the registry is a filesystem stub, so "known names" are the
local index — **not** a global ecosystem feed; the guard is a prompt-to-verify
heuristic, **not** a security guarantee. Flips to `merged` on squash; the
`s45 → merged(5)` advance rides with the S46 PR.

---

### S46 — caps-to-sandbox policy (WASI / seccomp / egress)

**Goal:** Make `@caps` actionable — turn the declared capability surface into a
concrete, reviewable sandbox configuration (reconciliation §147: declared @caps
→ enforceable WASI/seccomp/egress policy).

**Dogfood block:** `garnet sandbox <file>` derives the capability surface (S35)
and emits three artifacts via the pure `garnet_cli::sandbox` mapper:
a **seccomp** profile (default `SCMP_ACT_ERRNO`; a baseline syscall set +
cap-gated groups for fs/net/time/proc), a **WASI** capability set
(preopens/sockets/clocks/env, stdio always), and an **egress** rule
(deny-all / loopback-only / allow). Deterministic JSON (`schema
garnet.sandbox/v1`) + human summary; `ffi`/`proc`/`*`/unknown caps emit explicit
warnings. Mapping table: `C_Language_Specification/GARNET_SANDBOX_POLICY.md`.

**State:** dogfood-passing (S46 PR). 8 unit tests (per-cap mapping + determinism
+ warnings) + 4 CLI integration tests. Full ladder green.
**Honest scope (do not soften):** **policy generation, not enforcement.** Every
artifact is marked `"enforced": false`; nothing runs under `wasmtime`, applies
seccomp to a live process, or installs a firewall. Runtime enforcement needs
`wasmtime` (WASI) or a Linux seccomp host — out of scope here and absent from the
build environment. The seccomp shape mirrors the OCI default-deny structure but
is not kernel-validated; the egress allowlist is a structural placeholder.
**No new readiness lane** — generation-only must not be mistaken for an
enforcement-readiness claim. Flips to `merged` on squash; the
`s46 → merged(5)` advance rides with the S47 PR.

---

### S47 — Windows / Linux / macOS build proof (+ Windows-propriety audit)

**Goal:** Answer the reconciliation's Windows-propriety question (§148-149) —
*does every attribute behave cross-platform, or is it just packaging?* — from the
CI matrix, and gate against silent cross-OS regression.

**Dogfood block:** `scripts/garnet_build_proof.py` classifies each target OS on
two axes: **behaves** (in the `cargo test --workspace` matrix of `ci.yml`:
`os: [ubuntu-latest, windows-latest, macos-latest]`) and **distributes**
(`linux-packages.yml`: deb/rpm + `macos-cli-tarballs`). `--format md|json`;
`--gate` exits non-zero if any of the three loses the *behaves* proof — wired into
the `agent-contracts` CI job alongside `test_garnet_build_proof.py` (7 unit
tests). `F_Project_Management/GARNET_BUILD_PROOF.md` carries the status table +
the per-surface propriety audit (pure logic CI-gated; determinism CI-gated; S46
seccomp policy a documented Linux-shaped gap; Windows CLI distribution deferred).

**State:** dogfood-passing (S47 PR). All three OSes *behave* (CI runs the full
suite on each); Linux + macOS *distribute*, Windows CLI packaging is an honest
reported gap. Full ladder green.
**Honest scope (do not soften):** **CI-attested, not locally re-run.** This is a
single-OS checkout; the gate verifies the CI matrix *covers* Windows/Linux/macOS
— it does not itself execute Windows or Linux. "Build proof" = compiles +
`cargo test --workspace` passes per OS in CI; it is not a claim of exhaustive
feature parity (the audit lists known platform-sensitive surfaces). No new
readiness lane (not mandated). Flips to `merged` on squash; the
`s47 → merged(5)` advance rides with the S48 PR.

---

### S48 — 12-domain / 7-novel proof matrix

**Goal:** Rigor evidence for a skeptical reviewer (reconciliation §44, §150): a
falsifiable inventory of the 12 demonstration domains and the 7 novel Paper VI
contributions.

**Dogfood block:** `scripts/garnet_proof_matrix.py` reuses the `CORE_12_CASES`
single source of truth (12 domains) and lists the 7 contributions **by title**,
anchoring each to in-repo evidence whose existence is checked. `--format md|json`;
`--gate` (wired into the `agent-contracts` CI job) fails if a domain example or a
contribution anchor disappears. Doc: `F_Project_Management/GARNET_PROOF_MATRIX.md`.

**State:** dogfood-passing (S48 PR). 12/12 domains present; all 7 contributions
exercised by existing evidence. 7 unit tests. Full ladder green.
**Honest scope (do not soften — Paper VI anchors):** an evidence **inventory**,
not empirical proof — no measurement, mechanized-proof, or external-study claim.
It does **not** re-adjudicate per-contribution verdicts; two contribution-
numbering schemes exist across docs, so contributions are listed by title and
Paper VI's aggregate scorecard ("4 supported, 2 partial (downgraded honestly),
0 refuted, 1 pending-infra") is quoted **verbatim**. No new readiness lane (not
mandated). Flips to `merged` on squash; the `s48 → merged(5)` advance rides with
the S49 PR.

---

### S49 — AI-PR-review-collapse wedge demo

**Goal:** The launch narrative (reconciliation §97/§151, GRAFT): make
machine-checkable capability review the answer to human PR review collapsing
under AI-generated volume — runnable, not a slogan.

**Dogfood block:** `examples/wedge_pr_review/{before,after}.garnet` simulate an
AI-suggested PR that silently widens `@caps(fs)` → `@caps(fs, net)`. Both versions
`garnet check` clean (the escalation is an authority change, invisible to the
checker), yet `garnet diff-caps` (S37) flags `caps GAINED: net` / `AUTHORITY
EXPANDED` (exit 1) and `garnet sandbox` (S46) shows egress `deny-all` → `allow`.
`garnet-cli/tests/pr_review_wedge.rs` (3 tests) is the cross-OS CI-gated proof via
the `cargo test --workspace` matrix; `scripts/smoke_garnet_pr_review_wedge.py`
generates the narrative report (`--format md|json`, 4 tests). Doc:
`F_Project_Management/GARNET_PR_REVIEW_WEDGE.md`.

**State:** dogfood-passing (S49 PR). The wedge fires as designed; full ladder
green. No new readiness lane (not mandated).
**Honest scope (do not soften):** the "human PR review collapses under AI volume"
claim is the **motivating thesis**, **not** a measurement made here — no
human-review-time numbers are fabricated. This is a **narrative composition** of
existing gates (S37/S42/S45/S46), not a new enforcement mechanism and not a
guarantee against all AI-PR risks (a PR that keeps its capability surface
unchanged is out of this gate's reach). Flips to `merged` on squash; the
`s49 → merged(5)` advance rides with the S50 PR.

---

### S50 — v0.8 beta gate (closes the S41–S50 hardening band)

**Goal:** The hardening-band milestone — a band-completion checkpoint that the
v0.8 hardening work (S41–S49) is done and its gates hold. **Not** a release.

**Dogfood block:** `scripts/garnet_v0_8_beta_gate.py` verifies the nine slices
S41–S49 are `merged` at confidence 5 in `.dogfood/goal.json` and re-runs the
band's anti-rot sub-gates (build-proof S47, proof-matrix S48); the gate is OPEN
only when both hold. It inventories what the band shipped and what is explicitly
deferred for beta, and surfaces the verbatim honesty anchors. `--format md|json`;
`--gate` (wired into the `agent-contracts` CI job) exits non-zero unless OPEN.
Doc: `F_Project_Management/GARNET_v0_8_BETA_GATE.md`.

**State:** dogfood-passing (S50 PR). Beta gate **OPEN** (band complete; sub-gates
pass). 8 unit tests. Full ladder green. No new readiness lane (not mandated).
**Honest scope (do not soften):** the gate does **not** cut a tag and does **not**
claim production readiness — Garnet remains a *research-grade prototype (v0.x.x),
not production-complete*; cutting `v0.8.0-beta` (or any tag) is a **release-truth
decision for Jon**. The v0.8.0 tag remains planned later in the roadmap. Flips to
`merged` on squash; the `s50 → merged(5)` advance rides with the S51 PR.

---

## Slice Contracts — S51+ (v0.8 adoption / release; detailed as approached)

### S51 — signed release lanes

**Goal:** Make Garnet's signing posture explicit (it is three lanes, not one) and
gate the lane Garnet actually owns (reconciliation §154: signed release lanes).

**Dogfood block:** **`garnet seal --out <path>`** writes the in-toto predicate to
a file (was print-only) so it feeds `cosign attest --predicate <path>`; the
cosign hint names the path. `scripts/garnet_signed_release_lanes.py` inventories
the three lanes — (1) program-manifest signing (`garnet build --sign`,
**active**, CI round-trip to `signature valid`), (2) release `SHA256SUMS`
signature (**deferred**, GPG/minisign TODO), (3) supply-chain attestation
(`garnet seal` → cosign, **partial**) — and `--gate` (CI) protects lane 1.

**State:** dogfood-passing (S51 PR). 3 Rust seal tests (incl. `--out`) + 6
reporter unit tests. Full ladder green. No new readiness lane (not mandated).
**Honest scope (do not soften):** Garnet does **not** sign its own supply chain
and does **not** bundle cosign/GPG/minisign; lanes 2–3 are deferred/partial **by
design** (external tools absent in this environment) and reported truthfully —
only lane 1 (owned end-to-end) is gated. Flips to `merged` on squash; the
`s51 → merged(5)` advance rides with the S52 PR.

---

### S52 — one-line install / readme check

**Goal:** The curl|sh ethos (Kelley): the one-line install must just work and its
docs must stay accurate (reconciliation §155).

**Dogfood block:** `scripts/garnet_install_readme_check.py` extracts the one-line
`curl … install.sh | sh` command from `README.md` and from the
`installer/sh.garnet-lang.org/install.sh` header, normalizes (strips the comment
marker, collapses whitespace), and asserts they are identical and that the
canonical install URL appears in both. `--gate` (CI) fails on drift; `--format
md|json`.

**State:** dogfood-passing (S52 PR). README ↔ installer install command is
byte-identical; gate passes. 6 unit tests. Full ladder green. No new readiness
lane (not mandated).
**Honest scope (do not soften):** a **doc-consistency** check, not a live network
install test (the installer pulls GitHub Releases). `install.sh` is separately
shellcheck-gated (the `shellcheck-installer` job); this slice does not duplicate
that and claims nothing about a real install succeeding. Flips to `merged` on
squash; the `s52 → merged(5)` advance rides with the S53 PR.

---

### S53 — tree-sitter grammar

**Goal:** Editor adoption infrastructure (highlighting/folding) via a tree-sitter
*syntax* grammar — distinct from the LSP *semantic* service (reconciliation
§33-37, §155: tree-sitter belongs in the adoption band, not the hardening band).

**Dogfood block:** `tree-sitter-garnet/grammar.js` defines the core Garnet syntax
(functions + `@`-annotations, struct/enum/impl, actors + `memory` kinds, control
flow, `match`, `try/rescue/ensure`, expressions incl. `|>`, `#`/`///` comments).
`scripts/garnet_tree_sitter_check.py` loads it with Node (a `grammar()` shim, so
rule thunks never run), asserts the grammar name + every expected core rule, and
`--gate` (CI) fails on a dropped rule. `tree-sitter-garnet/README.md` documents
`tree-sitter generate`/`test` for users with the CLI.

**State:** dogfood-passing (S53 PR). grammar loads (name `garnet`, 50 rules, no
missing core rules); 5 unit tests. Full ladder green. No new readiness lane.
**Honest scope (do not soften):** a **CORE** grammar (headline constructs, not
exhaustive), **structurally validated, not compiled** — `tree-sitter generate` +
corpus tests need the tree-sitter CLI, which is **absent** in this environment;
the hand-written `garnet-parser` remains the canonical grammar / source of truth.
Flips to `merged` on squash; the `s53 → merged(5)` advance rides with the S54 PR.

---

### S54 — VS Code / OpenVSX / Marketplace path

**Goal:** Make the VS Code extension marketplace-ready and document the publish
path (reconciliation §156), while honestly deferring the credentialed publish.

**Dogfood block:** added `keywords` to `editors/vscode/package.json`;
`scripts/garnet_vscode_publish_readiness.py` asserts every marketplace-required
field (`name`/`version`/`publisher`/`engines.vscode`/`repository`/`license`) +
recommended field + the README/LICENSE files are present, and `--gate` (CI) fails
on a regression. It reports the path: build VSIX (`vscode-extension.yml`) → GitHub
release asset on tag → OpenVSX (`ovsx publish`) / Marketplace (`vsce publish`).

**State:** dogfood-passing (S54 PR). Extension is marketplace-ready (no missing
required fields/files); 5 unit tests. Full ladder green. No new readiness lane.
**Honest scope (do not soften):** the actual OpenVSX/Marketplace **publish** needs
`OVSX_TOKEN`/`VSCE_PAT` credentials — **credential/account territory, deferred to
a human** (like the v0.8.0 tag). This slice makes the extension publish-*ready*
and documents the path; it publishes nothing and bundles no credentials. (The
extension `version` stays `0.7.0` pending the v0.8 release-versioning decision —
not bumped autonomously.) Flips to `merged` on squash; the `s54 → merged(5)`
advance rides with the S55 PR.

---

### S55 — WASM hello-world

**Goal:** Open the in-browser story (adoption driver + S56 playground enabler)
honestly: ship the hello-world + the wasm path, naming what blocks the build
rather than faking one (reconciliation §77, §156).

**Dogfood block:** `examples/hello.garnet` (canonical hello-world; checks clean,
runs). `scripts/garnet_wasm_readiness.py` inventories the path — the interpreter
compiled to wasm is the in-browser model (Garnet has **no** wasm backend) — and
names the blockers: no `wasm32` target, `wasm-pack`/`wasmtime` absent, and
`garnet-interp`'s `miette` `fancy` feature. `F_Project_Management/GARNET_WASM_TARGET.md`
documents the build path. `--gate` guards only the owned bits (example + doc).

**State:** dogfood-passing (S55 PR). Owned bits ready; 5 unit tests. Full ladder
green. No new readiness lane.
**Honest scope (do not soften):** **no wasm artifact is built and no browser run
is claimed.** The `wasm32` target, `wasm-pack`, and `wasmtime` are **absent** in
this environment, and the interpreter needs a portability fix (feature-gate
`miette` `fancy` off) first — so the wasm build is **deferred**; the absent
toolchain is an honest deferral, not a gated failure. Flips to `merged` on
squash; the `s55 → merged(5)` advance rides with the S56 PR.

---

### S56 — playground MVP

**Goal:** A browsable playground (adoption driver) that respects the existing
"no fake editor" stance — a static gallery now, live execution when wasm lands
(reconciliation §77, §156).

**Dogfood block:** `docs/playground.html` becomes a **static example gallery**
(pick a program → its source + recorded `garnet run` output) loaded from
`docs/playground/examples.json` (generated by `scripts/garnet_playground_build.py`).
`scripts/garnet_playground_readiness.py` validates the gallery is well-formed and
the page keeps its honesty markers (static / not-a-fake-editor / WebAssembly);
`--gate` fails on a malformed gallery or a lost honesty marker.

**State:** dogfood-passing (S56 PR). Gallery has 3 examples (hello,
documented_math, mvp_05_web_app), well-formed; honesty markers preserved. 4 unit
tests. Full ladder green. No new readiness lane.
**Honest scope (do not soften):** the playground is a **static gallery, not a
live editor** — outputs are recorded by `garnet run`, **not** computed in the
browser; live in-browser execution needs the WASM build (S55, deferred). It
deliberately does **not** ship a fake editor (the gate enforces the page says so).
Flips to `merged` on squash; the `s56 → merged(5)` advance rides with the S57 PR.

---

### S57 — idiomatic open corpus

**Goal:** Show *what good Garnet looks like* (Lattner; reconciliation §157) — an
open corpus of idiomatic programs that dogfood the hardening-band policies.

**Dogfood block:** `examples/idiomatic/typed_errors.garnet` (typed `rescue e: T`,
S42 policy — never a catch-all) and `examples/idiomatic/state_machine.garnet`
(exhaustive `match` over a finite enum, named `@caps`). `scripts/garnet_idiomatic_corpus.py`
holds them to a high bar: each must `garnet check` to **0 diagnostics** (fully
clean — not even a non-fatal advisory) and `garnet run` to its recorded output.
`examples/idiomatic/README.md` names the idioms.

**State:** dogfood-passing (S57 PR). 2/2 idiomatic programs clean + running; 5
tests (pure-logic + skip-unless-built live). Full ladder green. No new readiness
lane.
**Honest scope (do not soften):** a **style/discipline** corpus, not a
performance or coverage claim. "Idiomatic" means clean checker output + the
hardening-band idioms, proven deterministically. Flips to `merged` on squash;
the `s57 → merged(5)` advance rides with the S58 PR.

---

### S58 — benchmark campaign

**Goal:** A consolidated view of the Criterion benchmark campaign + a gate
against bench-rot (reconciliation §157), complementing the existing compile-only
evidence (`garnet_benchmark_no_run.py`).

**Dogfood block:** `scripts/garnet_benchmark_campaign.py` inventories all 6
harnesses (parser `parse`, CST `parse_cst_vs_ast`, interp `eval`, VM
`parse_compile_execute`, memory `vector` + `eviction`) — crate, what each
measures, the per-bench run command — and `--gate` (CI) fails if a declared bench
file or its Cargo `[[bench]]` entry disappears.

**State:** dogfood-passing (S58 PR). All 6 benches present + declared; 5 unit
tests. Full ladder green. No new readiness lane.
**Honest scope (do not soften):** inventories + verifies the harnesses **exist**;
it does **not** run them and reports **no measurements** — Criterion numbers are
environment-specific, recorded by an explicit campaign run, **not** fabricated
here (the no-measurement stance of `garnet_proof_benchmark_status.py`). Flips to
`merged` on squash; the `s58 → merged(5)` advance rides with the S59 PR.

---

### S59 — fuzz campaign

**Goal:** Strengthen + inventory the parser fuzz campaign + gate against
harness-rot (reconciliation §157).

**Dogfood block:** 5 new corpus seeds (`seed_hello`, `seed_typed_errors`,
`seed_state_machine`, `seed_documented_math`, `seed_safe_io_layer`) extend the
parser fuzz corpus to cover the S42–S57 grammar (typed rescue, `@caps`, enum +
exhaustive `match`, doctest fences, cross-boundary `Result`).
`scripts/garnet_fuzz_campaign.py` inventories the campaign (target `parse_input`,
crate `garnet-parser-v0.3`, nightly `cargo fuzz run` ≥ 1h, seed count) and
`--gate`s that the target file + Cargo `[[bin]]` + `fuzz-nightly.yml` wiring +
non-empty seed corpus all remain.

**State:** dogfood-passing (S59 PR). Harness wired + 13 seeds; 5 unit tests. Full
ladder green. No new readiness lane.
**Honest scope (do not soften):** verifies the harness **exists and is wired**;
it does **not** run the fuzzer and makes **no** bug-found (or bug-free) claim —
crashes surface in the nightly `cargo fuzz run` job; `cargo-fuzz` is absent in
this environment so the harness is verified structurally, not built here. Flips
to `merged` on squash; the `s59 → merged(5)` advance rides with the S60 PR.

---

### S60 — v0.8.0 tag (release-readiness gate + escalation)

**Goal:** Bring the whole v0.8 train to a tag decision (reconciliation §158:
"v0.8.0 tag"). The automatable, honest part — the **release-readiness gate** —
ships here; the **tag cut itself is escalated to Jon**.

**Dogfood block:** `scripts/garnet_v0_8_0_release_readiness.py` aggregates both
bands (hardening S41–S50, adoption S51–S59) + all 11 anti-rot sub-gates into a
single READY/NOT-READY verdict, with the in/deferred inventory + verbatim honesty
anchors. `--gate` (CI) fails unless both bands are merged and every sub-gate
passes. Doc: `F_Project_Management/GARNET_v0_8_0_RELEASE.md`.

**State:** dogfood-passing (S60 PR). Verdict **READY TO TAG (pending Jon)**:
hardening 10/10, adoption 9/9, sub-gates 11/11. 5 unit tests. Full ladder green.
**CRITICAL honest scope (do not soften):** this slice **does NOT cut a tag**.
Only `v0.4.2`/`v0.5.0` are tagged; cutting `v0.8.0` is a **release-truth/strategy
decision for Jon** — irreversible, reserved by the honesty anchors — and is
**escalated**, not made autonomously. The gate's "READY TO TAG" is evidence-backed
advice, not the act of tagging. **Decision (2026-05-31): Jon chose to defer the
`v0.8.0` tag and continue to S61+** — so the readiness gate is the slice
deliverable (`s60 → merged(5)`) and the tag stays uncut, cuttable on Jon's
authorization anytime. The `s60` advance rides with the S61 PR.

---

### S61 — FFI authority model

**Goal:** Govern the sharpest capability edge — foreign-function calls — as an
explicit, declared authority (reconciliation §163-164: the native-interop
boundary; "wrap Python/Mojo/CUDA with @caps/@bounded/seal").

**Dogfood block:** the FFI authority model — a function wrapping a native call
must declare `@caps(ffi)` (`Capability::Ffi`), which flows through the trust
kernel: surface (S35) → manifest (S36) → `diff-caps GAINED ffi` (S37) → seal
(S38) → `garnet sandbox` escape-hatch warning (S46). `examples/ffi/{no_native,
native_boundary}.garnet` + `garnet-cli/tests/ffi_authority.rs` (3 cross-OS tests:
both check clean, sandbox flags ffi, diff-caps flags gaining it). Spec:
`C_Language_Specification/GARNET_FFI_AUTHORITY.md`.

**State:** dogfood-passing (S61 PR). 3 Rust tests; full ladder green. No new
readiness lane.
**Honest scope (do not soften):** Garnet has **no FFI runtime** — the
interpreter does not execute `extern "C"` calls and this slice adds none. S61
ships the *authority model* (declaration → surface → diff → seal → sandbox-flag),
**not** native-call execution; its value is transparency + review, not
containment (the sandbox cannot constrain FFI). Rust/C ABI execution (S62/S63)
and WASI interop (S64) build on this and stay honest-partial where the native
toolchain / wasm runtime is absent. Flips to `merged` on squash; the
`s61 → merged(5)` advance rides with the S62 PR.

---

### S62 — Rust FFI proof

**Goal:** Prove a Garnet↔Rust `extern "C"` binding is a first-class, attested
authority under the S61 model (reconciliation §163-164).

**Dogfood block:** `examples/ffi/rust_extern.garnet` (a `@caps(ffi)` Rust-wrapper)
checks clean + runs; `garnet seal` emits an in-toto predicate
(`predicateType …/seal/v1`) whose embedded capability manifest **attests `ffi`**
— so a Rust-FFI binding is diffable (S37), reviewable (S49), signable (S51).
Spec: `C_Language_Specification/GARNET_RUST_FFI.md`; cross-OS proof
`garnet-cli/tests/rust_ffi_proof.rs` (2 tests).

**State:** dogfood-passing (S62 PR). 2 Rust tests; full ladder green. No new
readiness lane.
**Honest scope (do not soften):** proves the **authority + attestation** half;
Garnet has **no FFI runtime** — the value↔C-ABI marshalling layer and a linked
Rust `cdylib` are **deferred**, not added here. Flips to `merged` on squash; the
`s62 → merged(5)` advance rides with the S63 PR.

---

## Slice Bands — S41–S80 (forward; detailed contracts authored as each band approaches)

> Resolution intentionally decreases. S41–S60 are planned; S61–S80 are bets.
> Each slice still earns its own contract block + fused 5/5 before merge.

**S41–S50 · v0.8 hardening**
- **S41** async/concurrency contract · **S42** typed Result / error policy ·
  **S43** docs-as-tests · **S44** LSP safe-mode / cross-package precision
  (distinct from tree-sitter) · **S45** package resolver / slopsquatting guard ·
  **S46** caps-to-sandbox policy (WASI/seccomp/egress) · **S47** Win/Linux/macOS
  build proof **[GRAFT: + Windows-propriety audit lane]** · **S48** 12-domain /
  7-novel proof matrix · **S49** **[GRAFT] AI-PR-review-collapse wedge demo**
  (the launch narrative; measured review-time collapse) · **S50** v0.8 beta gate.

**S51–S60 · v0.8 adoption / release**
- **S51** signed release lanes · **S52** one-line install / readme check ·
  **S53** tree-sitter · **S54** VS Code / OpenVSX / Marketplace · **S55** WASM
  hello-world · **S56** playground MVP · **S57** idiomatic open corpus ·
  **S58** benchmark campaign · **S59** fuzz campaign · **S60 — v0.8.0 tag.**

> ## ⚠ RESOLUTION DECREASES BELOW THIS LINE (bets; both plans agree)

**S61–S70 · v0.8.1** — native-interop boundary + provenance
- **S61** FFI authority model · **S62** Rust FFI proof · **S63** C ABI proof ·
  **S64** WASI interop · **S65** AI-authorship provenance · **S66** model/prompt/
  tool attestation in `seal` · **S67** MCP/tool capability declarations ·
  **S68** capability transparency log stub **[GRAFT: seed cross-language
  capability-manifest standard]** · **S69** LLM suggest v0.2 / Paper VI prep ·
  **S70 — v0.8.1 tag.**

**S71–S80 · v0.8.2 runway (not rushed)**
- **S71** Paper VI Exp 3 actual run · **S72** self-hosted parser seed ·
  **S73** VM/interpreter parity campaign · **S74** safe-subset spec **[GRAFT:
  optional linear/effect-typed safe mode]** · **S75** formal-verification
  feasibility · **S76** stdlib promotion wave · **S77** external package pilots ·
  **S78** governance / RFC process **[GRAFT: RFC + edition process over BDFL;
  donate capability-manifest standard to OWASP/LF]** · **S79** website / deck
  reframing · **S80** v0.8.2 readiness decision.

**1.0** — held past S80 until the bet stages validate.

---

## PR Body Template (hybrid: satisfies the new skill AND garnet CI)

```markdown
## Summary
- What changed. Why it is narrow. What is deliberately not claimed.

## Dogfood Readiness

### Current truth
- [x] Base/head refs, dirty state, changed files recorded.

### Local verification
- [x] <command + observed result>

### Remote verification
- [x] <CI matrix / checks>

### Merge confidence
- [ ] Internal band (1–5; 5 == score ≥ 95) recorded.
- [ ] External reviewer score + source (e.g. Greptile 4/5) or marked not-run.
- [ ] Fused merge confidence **5/5** via grep loop, or recorded human deferral.

### Goal progress
- [ ] Overall completion is `goal-tracked` from `.dogfood/goal.json` (slice k of N).

### Desktop dogfood bundle / Evidence bundle
- [x] Evidence artifacts attached or copied to a durable project folder.

### Deferred / out of scope
- This PR does not claim production readiness.
```

> Note: the `### Desktop dogfood bundle / Evidence bundle` heading carries both
> labels so the legacy garnet CI gate and the new skill template both pass —
> this is the S31 gate-reconciliation in practice.

---

## Integration with Existing Scripts (readiness lanes)

New lanes land **with the slice that earns them**, not before:

| Slice | Lane added |
|---|---|
| S31-PR2 | `reporter_determinism` (committed-truth split) |
| S32 | `edition_compatibility` |
| S33 | `garnet_verify_gate` |
| S37 | `capability_diff_caps` |
| S38 | `seal_attestation` |
| (later bands) | authored as each slice approaches |

S31-PR1 (this PR) adds **no** lane and regenerates **no** baseline — it is
doctrine + ledger + gate reconciliation only.

---

## Honesty Anchors (carry forward from v0.5–v0.7, plus v0.8 additions)

Carried forward (stay verbatim — brand equity):
- "research-grade prototype (v0.x.x) — not production-complete"
- "tracked-slice ledger is complete, but that is not full MIT/productization completion"
- Paper VI scorecard: "4 supported, 2 partial (downgraded honestly), 0 refuted, 1 pending-infra"
- "human/aesthetic acceptance remains open"
- "registry stub serves a static index; no central registry, no auth, no publish flow"

New for v0.8:
- "compatibility is two-layer; capability semantics are edition-invariant by construction"
- "seal/@bounded WRAP in-toto/cosign/Wasmtime — Garnet does not reimplement signing or metering"
- "diff-caps reads the declared capability surface; sandbox enforcement of undeclared authority is S46"
- "merge confidence is the FUSED (min) internal+external band; a confident reviewer alone cannot satisfy 5/5"
- "slices past S60 are bets; the goal ledger tracks the planned spine, not a guarantee"
- "no 1.0 claim is implied anywhere in the S31–S80 window"

If a slice would let one of these soften, the slice's PR body says so explicitly and Jon decides.
