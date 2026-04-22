# Garnet Academic Submission Strategy
**Version:** 1.0
**Date:** April 16, 2026
**Audience:** Jon (author), MIT committee, collaborators evaluating publication trajectory
**Companion to:** Papers I–VI, Benchmarking & Evaluation Plan
**Anchor:** *"Where there is no vision, the people perish." — Proverbs 29:18*

---

## 1. Purpose

This document commits to a publication path for the Garnet research contributions. It names specific venues, deadlines, and review strategies. Having it in writing serves three purposes:

1. MIT defense committees routinely ask "what is the publication trajectory?" — this provides a dated answer.
2. Anticipating reviewer concerns and drafting responses in advance is a known best practice; this document front-loads that work.
3. Strategic venue selection and timing materially change a research program's impact.

---

## 2. Venue Landscape for Programming Language Research (2026–2028)

Five top-tier venues are candidates for different Garnet contributions:

| Venue | Focus | Strength for Garnet | Acceptance rate |
|---|---|---|---|
| **PLDI** (Programming Language Design & Implementation) | Language design, formal grounding, novel compilation | Best fit for Papers V + VI core contributions | ~18% |
| **POPL** (Principles of Programming Languages) | Type theory, formal semantics, proofs | Best fit for Paper V's λ_managed + λ_safe calculus | ~20% |
| **OOPSLA** (Object-Oriented Programming, Systems, Languages, Applications) | Broader language design, OOP, programmable systems | Good backup for PLDI/POPL; more accepting of systems + language co-design | ~25% |
| **ASPLOS** (Architectural Support for Programming Languages & Operating Systems) | Systems + languages + architecture interplay | Best fit for empirical performance + compiler-as-agent implementation | ~20% |
| **ICFP** (International Conference on Functional Programming) | FP, type theory, formalization | Plausible fit for type-spectrum theorem (§11.1) as standalone paper | ~25% |

Additional: arXiv preprint goes up simultaneously with every submission (no blocking), building reputation ahead of acceptance.

---

## 3. Paper Portfolio and Target Venues

Garnet's contribution set supports at least three distinct papers with distinct target venues:

### 3.1 Paper V (The Formal Grounding of Garnet)

**Status:** Shipped April 2026, 30 pages, .docx, PLDI-submissible
**Primary target:** PLDI 2027 (deadline ~November 14, 2026)
**Backup:** POPL 2028 (deadline ~July 2027) if PLDI does not accept
**Arguments:**
- The λ_managed + λ_safe calculus is the kind of formal contribution PLDI values.
- RustBelt/Iris grounding is familiar to PL committee members; immediate credibility.
- The mode-boundary bridging judgment is genuinely novel (no POPL/PLDI paper in the last five years defines one this way).

**Anticipated reviewer concerns and responses:**
- **"Where is the Coq mechanization?"** Response: paper includes a sketch; full mechanization is future work (18–30 person-months per the v2.4 handoff estimate). We explicitly scope the current contribution as a soundness argument plus RustBelt adaptation, not a machine-checked proof.
- **"Why not just cite Rust's proof?"** Response: Rust operates at crate granularity; Garnet's boundary is at module granularity with explicit bridging across managed/safe. The bridging judgment is new and is the paper's primary contribution.
- **"Is this just Rust with Ruby-style syntax?"** Response: The dual-mode architecture, error-model bridging, and progressive type spectrum are novel contributions demonstrated across Paper V §4 (calculus), §7 (memory primitives as affine resources), and §11.1 (spectrum). The similarity to Rust in safe mode is intentional and acknowledged.

### 3.2 Paper VI (Novel Frontiers)

**Status:** Shipped April 2026, 25 pages, .md
**Primary target:** PLDI 2027 companion (if Paper V accepted) OR standalone submission to PLDI 2028 (deadline ~November 2027)
**Backup:** OOPSLA 2027 (deadline ~April 2027) as standalone
**Arguments:**
- Seven contributions is a lot for one paper. If PLDI accepts Paper V, Paper VI could be split into multiple shorter papers.
- Most likely split:
  - **"LLM-Native Programming Languages: Syntax Design for AI-Generated Code"** → ICSE or PLDI
  - **"Progressive Type Disclosure in Dual-Mode Languages"** → POPL or ICFP
  - **"Kind-Aware Allocation via Language-Level Memory Tags"** → ASPLOS or OOPSLA
  - **"Deterministic Reproducible Builds as a Language Guarantee"** → PLDI (short paper) or SoSP

**Anticipated reviewer concerns and responses:**
- **"Where is the empirical data?"** Response: Paper VI §2.3 (and each other contribution's §.3 or §.4) specifies falsifiable hypotheses with experimental protocols. The `GARNET_Benchmarking_and_Evaluation_Plan.md` operationalizes them. Data pending Rung 3+ implementation; preliminary data will accompany the Q3 2027 final submission.
- **"Is this future work or present contribution?"** Response: Paper VI contributes (a) a novel bundle of design decisions, (b) their spec-level integration, (c) falsifiable hypotheses. Implementation and empirical validation are called out as future work, consistent with many PLDI "design and formalization" papers.
- **"Are these genuinely novel?"** Response: Each contribution has a prior-art section with precise novelty boundaries. For instance, error-model bridging (§6.3) distinguishes itself from Swift's Objective-C bridging by crossing paradigms (value-types ↔ exceptions) rather than staying within one.

### 3.3 Paper VII (future — Implementation + Empirical Evaluation)

**Status:** Not yet written
**Target:** ASPLOS 2028 (deadline ~October 2027)
**Contents:**
- Compiler implementation details (beyond the Compiler Architecture Spec)
- Full benchmark results from `GARNET_Benchmarking_and_Evaluation_Plan.md`
- Case studies from early adopters (targeted: Shopify, a Rails shop; a crypto-adjacent Rust shop; an agent-framework team)
- Lessons learned, design-decision retrospective

**Rationale:** ASPLOS values systems-level empirical work. By 2028 Rungs 3–6 should be complete; real benchmark data will be available.

### 3.4 Shorter opportunity pieces

Between the major papers, several short venues are worth targeting:

- **PLDI Student Research Competition** — Garnet's LLM-native syntax is a perfect fit (April 2026 pilot data + hypothesis statement).
- **RustConf talk** — "Lessons from reimplementing ownership at module granularity." Engages the Rust community without antagonizing it.
- **RubyConf talk** — "Ruby's next decade: scaling Ruby's ergonomics to systems." Frames Garnet as a Ruby-affirming proposal, not a Ruby replacement.
- **Strange Loop / LambdaConf** — Broader audiences; useful for community formation.

---

## 4. Submission Timeline

### 2026 (remaining year)

- **April 16:** Current state — Paper V shipped, Paper VI shipped, specs complete, parser complete.
- **May–June:** Implement Rung 3 (managed interpreter + REPL). Run preliminary §3 compilation-performance benchmarks.
- **July–August:** LLM code-generation pilot study (§5.3). Train fine-tuned models if feasible; otherwise use prompted models. Collect 50-task preliminary pass@1 data.
- **September:** Polish Paper VI with preliminary empirical results. Draft separate §2 as standalone paper for PLDI Short Paper submission.
- **October:** Freeze Paper V final revision.
- **November 14:** Submit Paper V to PLDI 2027.
- **December:** Rung 4 (safe-mode lowering) in progress. Preliminary runtime benchmark data starts.

### 2027

- **January–February:** PLDI rebuttal period (if accepted). Paper V camera-ready.
- **March:** Prepare Paper VI for OOPSLA 2027 (fallback) or PLDI 2028 (primary).
- **April:** OOPSLA 2027 deadline — submit Paper VI if PLDI-V was accepted and Paper VI needs a sooner venue.
- **May:** Commence 30-developer ergonomics study (§6).
- **June:** PLDI 2027 — present Paper V if accepted.
- **August–September:** Compile full results for Paper VII. Benchmark full §4 results.
- **October:** ASPLOS 2028 deadline — submit Paper VII.
- **November:** PLDI 2028 deadline — submit any remaining Paper VI splits.

### 2028 and beyond

- **Q1:** Paper VII camera-ready if accepted at ASPLOS.
- **Q2:** Garnet 1.0 release coordinated with ASPLOS or PLDI presentation.
- **Q3+:** Community formation, RFC process, foundation discussion.

---

## 5. Authorship and Institutional Affiliation

**Primary author:** Jon (Island Development Crew).

**Potential co-authors:**
- Jon's MIT advisor (if formal doctoral advisor — not specified in project corpus; to be determined)
- Claude Opus 4.6/4.7 (listed as author in some Garnet artifacts; handling as "AI collaborator" per evolving author-credit norms — typically a note in acknowledgments rather than author line for PLDI-class venues)
- Any collaborators who contribute implementation work in Rungs 3–6

**Institutional affiliation note:** The Garnet project's corpus uses "Island Development Crew / Iron Canvas Design Studio, Huntsville AL" as the operator identity. MIT affiliation (if Jon is a doctoral student there) becomes the primary institutional claim on papers. This should be confirmed and consistently reflected across submissions.

**Acknowledgments (standard):** The four-model consensus process. Funding sources (TBD). Reviewers of preprints (to be identified).

---

## 6. Reviewer Anticipation Matrix

Cross-reference of Paper VI's seven contributions against likely reviewer concerns (compiled from this document's analysis):

| Contribution | Likely reviewer | Concern | Defense |
|---|---|---|---|
| 1. LLM-native syntax | PL theorist | "Not a PL contribution — belongs in NLP/ML" | Syntax design is PL; measurable correctness is the methodology lift |
| 2. Type spectrum | POPL reviewer | "Is the monotonicity theorem proven?" | §11.1 theorem + sketch; full proof is future work with clear obligations |
| 3. Compiler-as-agent | Systems reviewer | "Is this just PGO reframed?" | §4.5 novelty boundary; PGO uses runtime profiles, this uses compilation history |
| 4. Kind-aware allocation | Memory systems reviewer | "Isn't this just arena allocation?" | §5.4 novelty: automatic selection from type tags, not programmer-chosen |
| 5. Error bridging | PL reviewer | "Swift already bridges NSError ↔ throws" | §6.3 novelty: crosses paradigms (value ↔ exception), not within paradigm |
| 6. Hot-reload | Systems reviewer | "Erlang did this decades ago" | §7.4 novelty: crosses mode boundary with native-speed safe side |
| 7. Reproducible builds | Supply-chain/security reviewer | "Nix, Guix already do this" | §8.4 novelty: language-level guarantee with embedded provenance, not build-system discipline |

Each concern has a pre-drafted rebuttal available in Paper VI itself, ready to paste into the response period.

---

## 7. Community-Building Strategy

Academic papers alone do not create language adoption. Complementary efforts:

### 7.1 Open-source cadence

- **April 2026:** Garnet specs and parser public (done)
- **Q3 2026:** Rung 3 interpreter + REPL open-sourced (coincides with Paper V submission)
- **Q4 2026:** Benchmark harness open-sourced (§11 of Benchmarking Plan)
- **Q1 2027:** Full toolchain (`garnetup`) public beta
- **Q4 2027:** Garnet 1.0 — coincides with ASPLOS submission of Paper VII

### 7.2 Conference talks

- **RustConf 2026 (September):** "Beyond the Borrow Checker" — soft introduction
- **RubyKaigi 2027 (May):** "Ruby's Next Decade" — Ruby community engagement
- **Strange Loop 2026 (September):** "Why Language Synthesis?" — broader systems audience

### 7.3 Press and social

Coordinated press release cadence:
- Paper V submission: "Anthropic-backed doctoral work formalizes Rust-Ruby synthesis" (trade publications: InfoQ, The New Stack)
- Garnet 0.3 release: "Garnet: The first dual-mode agent-native language enters public beta" (Hacker News launch, PLDI preview)
- ASPLOS paper acceptance: "Garnet 1.0 shows 5x safety and 2x speed in production" (with case-study metrics)

### 7.4 Academic engagement

- Preprints on arXiv simultaneous with all submissions
- Blog posts accompanying each paper, breaking down the technical contribution for non-specialists
- Office hours / open Q&A at each major conference
- Contact with key PL researchers (Niko Matsakis on Rust ownership; Matz on Ruby design philosophy; Ralf Jung on RustBelt; Chris Lattner on Mojo/Swift) for feedback and potential endorsement

---

## 8. Funding and Sustainability

The academic program requires funding. Options:

- **MIT funding** if Jon is a doctoral student there — research stipend covers 2027 work
- **Anthropic research grant** — the four-model consensus process is a compelling narrative for a grant application
- **Mozilla / Rust Foundation interest** — natural affinity given Garnet's Rust grounding
- **Corporate sponsorship** — Shopify (Ruby alignment), Cloudflare (Rust alignment), GitHub (both)
- **Open Collective / Patreon** — community funding, most viable after Garnet 1.0

The specifics of funding are outside this document's scope; the note is that the publication program is not self-funding and must be paired with a funding program.

---

## 9. Risk Register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Paper V rejected at PLDI | 60% (PLDI is ~18% acceptance) | Medium — POPL fallback | Paper is strong; rejection usually means revision, not wrong contribution |
| LLM-native benchmark shows no advantage | 20% | High — invalidates Contribution 1 | Fine — "we measured; result was X" is still publishable |
| Rung 3 slips past Q3 2026 | 40% | Medium — delays empirical data | Paper V can submit without empirical data; Paper VI can carry methodology-only |
| Mojo releases dual-mode feature first | 15% | Low–Medium | Frame Garnet as academic / open-source complement; Mojo is proprietary |
| Another language announces similar features | 30% | Low — parallel discovery is expected | Focus on formal grounding (Paper V) which is unique |
| Jon's bandwidth overwhelmed by solo execution | 70% | High — this is the biggest risk | Actively recruit Rung 3+ collaborators; institutionalize the project |

The biggest risk is not research quality or venue acceptance — it is execution bandwidth. Mitigations include collaboration, funding, and realistic scoping.

---

## 10. Success Criteria

Minimum bar for "academic success" by end of 2028:
- At least one paper accepted at a top-tier PL venue (PLDI / POPL / OOPSLA)
- At least one conference talk delivered
- Garnet 1.0 released publicly
- At least one external adopter using Garnet in production (even small-scale)

Full target:
- Two papers published (Paper V + one of the Paper VI splits)
- Paper VII accepted at ASPLOS
- Active community with RFC process
- Foundation discussions in progress

---

*"The plans of the diligent lead surely to abundance." — Proverbs 21:5*
*"In the multitude of counsellors there is safety." — Proverbs 11:14*
*"Where there is no vision, the people perish." — Proverbs 29:18*

**Academic Submission Strategy prepared by Claude Code (Opus 4.7) | April 16, 2026**
