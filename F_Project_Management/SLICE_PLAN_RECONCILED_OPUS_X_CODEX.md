# Garnet Slice Plan — Reconciliation: Opus 4.8 Standard × Codex Reassessment
## Merged spine s31 → s80

> **⚠ Version-tag mapping superseded (2026-05-31).** This is a historical
> reconciliation artifact. Its version-tag granularity (§1 and §5: "v0.8.0@S60,
> v0.8.1@S70, v0.8.2 decision@S80") was **corrected** at the S70 checkpoint: the
> whole S30–S80 run is cut as **one `v0.8.0` tag at the end of S80**; S60 and S70
> are readiness checkpoints, not cuts; S81+ is the runway to v0.8.1. The authority
> is [`GARNET_v0_8_VERSION_MAP.md`](GARNET_v0_8_VERSION_MAP.md). The slice
> *structure* below is unchanged and still governs; only the tag labels moved.

**Method:** I produced an independent s31–s80 standard from the two research reports without seeing
Codex's plan (the diff-caps discipline applied to planning: two uncontaminated surfaces, then reason
about the delta). This document is the reconciliation.

**Verdict, stated plainly (MIT-grade bar, no ego):** Codex's plan is the **stronger execution spine**.
It caught real gaps mine missed and it carries repo-grounding I lack. My plan contributes **six
specific high-value grafts** Codex's doesn't carry. The correct output is **Codex's spine + the six
grafts** — not "pick one." Critically, the two load-bearing questions (editions early, 1.0 held)
are points of **independent agreement**, so there is no fundamental conflict to resolve — only a merge.

---

## 1. Where the two plans AGREE (the load-bearing questions)

These were the seams I said I'd defend hardest. Codex lands the same way independently:

| Question | Opus standard | Codex | Result |
|---|---|---|---|
| **Editions before pillar-deepening** | Compatibility spine at s37–s40, before trust-kernel hardening (s45+) | Edition/compatibility model at **S32** (2nd slice) | **Agree — Codex more aggressive.** Install evolution mechanism before users exist to break. Codex's earlier placement is *better*. |
| **Hold 1.0** | 1.0 held past s80 | Entire arc stays 0.8.x: v0.8.0@S60, v0.8.1@S70, v0.8.2 decision@S80 | **Agree emphatically.** No 1.0 in the 80-slice window. Codex's version-tag mapping is *better* aligned to the stated 8.1→8.2→…→1.0 model. |

Because the two hardest questions converge, this is a merge, not a contest.

---

## 2. Seam-by-seam diff (the seven seams I pre-registered)

**Seam 1 — Editions vs pillar-deepening.** RESOLVED in agreement; Codex stronger (S32 vs s37). Concede to Codex.

**Seam 2 — tree-sitter placement.** Opus: s41 (review-cheap tooling). Codex: **S53** (adoption/release band).
*Honest call: Codex is right, I was wrong.* tree-sitter is adoption infrastructure (editor highlighting,
Linguist) — it does **not** gate the trust kernel. diff-caps reads the compiler's capability surface,
not tree-sitter; the LSP can run on the compiler frontend. Codex correctly separates **LSP/semantic
service (earlier, hardening band)** from **tree-sitter (later, adoption band)** — a distinction I
blurred by putting them adjacent. Concede to Codex.

**Seam 3 — the 1.0 hold.** RESOLVED in agreement (see §1). Codex's version-tag granularity superior. Concede to Codex.

**Seam 4 — demo budget.** Opus: explicit **AI-PR-review-collapse wedge demo** as a first-class slice
(s53). Codex: proof matrices + benchmark/fuzz campaigns, **no dedicated narrative wedge demo**.
*This is where my plan is stronger.* A proof matrix is rigor evidence for a skeptical reviewer; the
wedge demo is a **narrative for an adopter deciding whether to try Garnet** — the artifact that makes
"evidence-native language for agent code" concrete and answers Zig's AI-PR ban. They are different
artifacts for different audiences. **GRAFT: add the wedge demo as an explicit slice in the adoption band.**

**Seam 5 — integrate-don't-rebuild fidelity.** Opus: explicit routing — @bounded→Wasmtime fuel (s49),
seal→in-toto/Sigstore predicate + CycloneDX/SPDX SBOM (s46–s47). Codex: slices named at a higher level
("seal", "@bounded") — integrate-vs-rebuild unspecified. Report 1 was emphatic that rebuilding metering
or signing is wasted novelty budget. **GRAFT: annotate Codex S38 (seal) and S39 (@bounded) with the
wrap-don't-rebuild discipline.**

**Seam 6 — readiness substrate.** Codex S31 includes a "readiness contract"; S33 is one-command
`garnet verify`. Neither references the existing `Navigata1/dogfood-readiness` toolkit (now upgraded
with merge-confidence fusion + grep-loop-to-5/5 + goal-mode ledger, 32 tests green). **GRAFT: point
S31's readiness contract and S33's verify gate at the upgraded toolkit — adopt, don't rebuild.**

**Seam 7 — diff-caps as internal reviewer.** Both plans put the gate (Codex S33 / Opus s33) *before*
diff-caps (Codex S37 / Opus s43), so the gate initially lacks the capability-diff signal. Resolution
(same for both): **build the gate to accept a pluggable capability signal; wire diff-caps in when it
lands.** diff-caps then feeds the *same* fused 1–5 band as Greptile (`min` governs the gate). GRAFT
the fusion wiring explicitly.

---

## 3. What Codex caught that I MISSED (the honest concession section)

These are genuine gaps in my plan. No defensiveness — Codex is right on all of them:

1. **Async/concurrency contract (Codex S41).** I omitted it entirely. Ryhl (Tokio) was one of the
   seven creators; @bounded includes mailbox size (actors); dual-mode implies a concurrency model.
   Real hole in mine.
2. **Docs-as-tests (Codex S43).** Executable docs = the "evidence not courtesy" discipline. I had a
   corpus slice but not this honesty mechanism.
3. **Web playground + WASM hello-world (Codex S55–S56).** Among the biggest adoption drivers other
   languages have (Rust/Go/TS playgrounds). I had none.
4. **Source-annotation decomposition (Codex S35→S36→S37):** annotations → manifest-from-annotations →
   diff-the-manifests. My plan sloppily assumed @caps syntax already existed. Codex's progression is
   cleaner and correct.
5. **VM/interpreter parity campaign (Codex S73).** If Garnet has two execution paths, parity is a real
   correctness concern. I didn't address backend parity at all.
6. **Self-hosted parser seed (Codex S72).** A maturity milestone I omitted.
7. **Fuzz campaign + benchmark campaign (Codex S58–S59).** I folded "validation" in abstractly; Codex
   is concrete.
8. **Stdlib promotion wave + external package pilots (Codex S76–S77).** Ecosystem maturation I underweighted.

**Plus: Codex has repo-grounding I lack.** Codex's plan was built with fast-mode repo access; mine
from the research reports. On anything tied to actual repo state (VM/interpreter parity, self-hosted
parser, the proof-matrix and Paper VI experiment structure), **Codex is the more reliable source.**

---

## 4. What the Opus standard GRAFTS onto Codex's spine (six contributions)

1. **Explicit AI-PR-review-collapse wedge demo** (seam 4) — the launch narrative, not a proof matrix.
2. **Integrate-don't-rebuild routing, named** (seam 5) — @bounded→Wasmtime fuel; seal→in-toto/Sigstore + SBOM.
3. **Two-layer compatibility** — editions (parse-time) **+** GODEBUG-style runtime settings (semantic-time),
   with the skin-deep/one-canonical-IR invariant so **capability semantics are edition-invariant**.
   Codex's single "compatibility model" slice may cover only the first layer.
4. **Deterministic-reporter fix as an explicit implementation slice** — not just "release-state
   correction" doctrine. The 85.2-vs-80.6 machine-dependence is a **bug that gates your three-machine
   hand-off** (Mac→Windows→Codex). Must be a fix, with committed-truth (% identical on every machine)
   split from local-evidence.
5. **diff-caps as Garnet's internal reviewer**, feeding the same fused merge-confidence band as Greptile
   (`min` governs) — now built into the dogfood-readiness toolkit.
6. **Adopt the upgraded dogfood-readiness toolkit** for the readiness contract and verify gate — don't rebuild.

*Minor (fold into existing Codex slices, not separate grafts):* optional linear/effect-typed safe mode
→ fold into Codex's **safe-subset spec (S74)**; capability-manifest-as-in-toto-predicate + donate-to-OWASP
→ fold into Codex's **capability transparency log stub (S68)** and **governance/RFC (S78)**.

---

## 5. THE MERGED SPINE (Codex structure + Opus grafts)

Codex's slice numbering and version-tag bands are the backbone. **[GRAFT]** marks an Opus addition or
annotation. Resolution decreases past S60 (bets, re-sliced as reality arrives) — both plans agree.

**S30 — live anchor.** Functional-core composition merged. No more demo-only expansion without release truth.

**S31–S40 · v0.8 foundation**
- **S31** — v0.8 PRD / release truth / compatibility doctrine / slice ledger / readiness contract.
  **[GRAFT]** Make the **deterministic-reporter fix** explicit here (committed-truth % identical on every
  machine, split from local-evidence) — it gates the three-machine hand-off. **[GRAFT]** Adopt the upgraded
  `dogfood-readiness` toolkit for the readiness contract (fusion + grep-loop + goal-mode ledger already built).
- **S32** — edition/compatibility model. **[GRAFT]** Make it **two-layer**: editions (parse-time) + GODEBUG-style
  runtime settings (semantic-time); enforce skin-deep/one-canonical-IR so **capability semantics are edition-invariant**.
- **S33** — one-command `garnet verify`. **[GRAFT]** Build the gate to accept a **pluggable capability signal**
  (diff-caps wires in at S37) and to **fuse** internal band + external reviewer (Greptile) via `min`; grep-loop to 5/5.
- **S34** — structured diagnostics (machine + human; authoritative exit code; service-over-MCP foundation).
- **S35** — source annotations (the @caps syntax itself).
- **S36** — capability manifest (derived from annotations).
- **S37** — `diff-caps` (capability-surface diff as acceptance gate; the headline novelty). **[GRAFT]** feeds the S33 fused band.
- **S38** — `seal` (signed reproducible bundle). **[GRAFT]** wrap-don't-rebuild: express as **in-toto predicate**,
  verify via cosign, emit **CycloneDX/SPDX SBOM**; the capability manifest is the no-SBOM-equivalent extension.
- **S39** — `@bounded`. **[GRAFT]** wrap-don't-rebuild: lower to **Wasmtime fuel** metering (revisit if WASM overhead >~2–3×).
- **S40** — explosive-operation / default-ceiling analysis.

**S41–S50 · v0.8 hardening**
- **S41** — async/concurrency contract. *(Codex caught this; I missed it.)*
- **S42** — typed Result / error policy. *(Ronacher: agents over-catch exceptions.)*
- **S43** — docs-as-tests. *(Codex caught this.)*
- **S44** — LSP safe-mode / cross-package precision (semantic service; **distinct from tree-sitter**).
- **S45** — package resolver / slopsquatting guard. *(Live threat — see §7.)*
- **S46** — caps-to-sandbox policy (declared @caps → enforceable WASI/seccomp/egress policy).
- **S47** — Windows / Linux / macOS build proof. **[GRAFT]** this is also the **Windows-propriety audit** lane
  (does every attribute behave correctly cross-platform, or is it just packaging?) — the second machine in your plan.
- **S48** — 12-domain / 7-novel proof matrix.
- **S49** — **[GRAFT] AI-PR-review-collapse wedge demo** (the launch narrative; measured review-time collapse). *(Opus contributes.)*
- **S50** — v0.8 beta gate.

**S51–S60 · v0.8 adoption / release**
- **S51** — signed release lanes. **S52** — one-line install / readme check (Kelley). **S53** — tree-sitter.
- **S54** — VS Code / OpenVSX / Marketplace path. **S55** — WASM hello-world. **S56** — playground MVP.
- **S57** — idiomatic open corpus (Lattner). **S58** — benchmark campaign. **S59** — fuzz campaign.
- **S60** — **v0.8.0 tag.**

> ## ⚠ RESOLUTION DECREASES BELOW THIS LINE (bets; both plans agree)

**S61–S70 · v0.8.1**
- **S61** — FFI authority model. **S62** — Rust FFI proof. **S63** — C ABI proof. **S64** — WASI interop.
  *(S61–S64 = the native-interop boundary: wrap Python/Mojo/CUDA with @caps/@bounded/seal — Lattner T1: "Mojo runs inside the GPU; Garnet runs above it.")*
- **S65** — AI-authorship provenance. **S66** — model/prompt/tool attestation in seal. **S67** — MCP/tool capability declarations.
  *(S65–S67 address the documented MCP "absence of capability attestation"; urgency confirmed §7.)*
- **S68** — capability transparency log stub. **[GRAFT]** seed the cross-language capability-manifest standard here.
- **S69** — LLM suggest v0.2 / Paper VI prep. **S70** — **v0.8.1 tag.**

**S71–S80 · v0.8.2 runway (not rushed)**
- **S71** — Paper VI Exp 3 actual run. **S72** — self-hosted parser seed. **S73** — VM/interpreter parity campaign.
- **S74** — safe-subset spec. **[GRAFT]** fold in the optional **linear/effect-typed safe mode** (Koka/Austral rigor).
- **S75** — formal-verification feasibility. **S76** — stdlib promotion wave. **S77** — external package pilots.
- **S78** — governance / RFC process. **[GRAFT]** RFC + edition process over BDFL; donate capability-manifest standard to OWASP/LF.
- **S79** — website / deck reframing. **S80** — v0.8.2 readiness decision.

**1.0** — held past s80 until the bet stages validate. Both plans agree.

---

## 6. The refined first move (S31)

Codex's recommendation — start now, S31 only, create the map — is sound. Two **[GRAFT]** refinements so
S31 is a *fix*, not only doctrine:

1. **Include the deterministic-reporter split** (committed-truth identical on every machine vs
   local-evidence reported separately). Your Mac→Windows→Codex hand-off depends on cross-machine reporter
   determinism; if S31 is pure doctrine the 85.2-vs-80.6 bug survives into the hand-off.
2. **Adopt the upgraded dogfood-readiness toolkit** for the readiness contract (fusion + grep-loop +
   goal-mode ledger are built and verified — `dogfood-readiness-fusion.patch`).

Also reconcile one **label**: your earlier "v0.8.1 = the five PRDs built end-to-end on Mac" does not map
1:1 to Codex's "v0.8.1 tag @S70." Codex distributes those five PRDs across S31–S70 (compatibility S32,
diagnostics/remarks S34, native-interop S61–S64, wedge demo grafted S49). Decide whether "v0.8.1" means
your five-PRD Mac build (a milestone spanning several Codex slices) or Codex's S70 tag, and label consistently.

---

## 7. Supply-chain corroboration (Codex live-check × Opus research)

Codex's live check (Cloud Security Alliance on Mini-Shai-Hulud/Sigstore; BleepingComputer on signed
malicious TanStack/Mistral npm packages; PipeLab on MCP tool poisoning) **independently corroborates**
Report 1 and Report 2, which surfaced the same incidents cold (validly-attested malicious npm worm; MCP
tool poisoning; slopsquatting at 19.7%). Two independent passes agreeing raises confidence the threat is
**live, not speculative**.

*Roadmap implication:* it strengthens the priority of diff-caps (S37), seal (S38), the slopsquatting guard
(S45), and tool-capability attestation (S67) — but **does not reorder** them, because each still depends on
its prerequisites (the guard needs seal; attestation needs the manifest). The discipline is: **do not let
these slip**, not move them ahead of their dependencies.

---

## Boundary note
This reconciliation grades plans, not running code. Codex's repo-grounded slices (VM/interpreter parity,
self-hosted parser, proof-matrix/Paper VI structure) reflect repo state I could not verify and are taken
as given. The merged spine is a map; each slice still earns its 5/5 through the dogfood-readiness gate
before merge. No 1.0 claim is implied.
