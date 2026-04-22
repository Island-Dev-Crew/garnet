# GARNET v2.2 — MASTER EXECUTION PROTOCOL
**Priorities 2–5 | Single-source briefs for fresh-session execution**
**Prepared:** April 12, 2026 | Claude Opus 4.6
**Anchor:** *"Where there is no vision, the people perish." — Proverbs 29:18*

---

## HOW TO USE THIS FILE
Each priority is a self-contained brief. In any fresh session within this project:

> "Continuing Garnet. Read `GARNET_v2_N_HANDOFF.md` and `GARNET_v2_2_Master_Execution_Protocol.md`. Execute Priority [2/3/4/5] per the brief. Proceed decisively."

At end of each session, the executing Claude **must** write an updated handoff file (`GARNET_v2_3_HANDOFF.md`, `v2_4`, etc.) marking the priority complete and advancing the pointer. The handoff file remains the single source of truth; this protocol file is stable.

## SESSION ROUTING TABLE

| Priority | Deliverable | Est. Session Load | Output File(s) | Handoff Bump |
|---|---|---|---|---|
| 1 | 50-slide Gamma deck | Full / dedicated | `gammaUrl` + deck brief execution | v2.2 → v2.3 |
| 2 | Paper V — Formal Grounding | Full / dedicated | `Paper_V_Garnet_Formal_Grounding_v1_0.docx` | v2.3 → v2.4 |
| 3 | Paper IV v2.1.1 finalization | Medium | `Paper_IV_Garnet_Agentic_Systems_v2_1_1.docx` | v2.4 → v2.5 |
| 4 | Rung 2 — `garnet-parser` crate | Full / dedicated | Rust crate in outputs | v2.5 → v2.6 |
| 5 | Executive Overview v2.2 refresh | Light | `GARNET_v2_2_Executive_Overview.md` | v2.6 → v2.7 |

---

# PRIORITY 2 — PAPER V: THE FORMAL GROUNDING OF GARNET

**Full title:** *The Formal Grounding of Garnet: Affine Type Theory, RustBelt, and the Mathematics of Mode Boundaries*
**Target venues:** PLDI, POPL, OOPSLA (plus arXiv preprint)
**Output:** `Paper_V_Garnet_Formal_Grounding_v1_0.docx` (binary docx via docx skill)
**Skill to invoke first:** `/mnt/skills/public/docx/SKILL.md`

## Purpose
Establish Garnet's academic credibility floor by grounding the dual-mode design in formal type theory and mechanized semantics. This is the paper that makes Garnet submittable to top-tier PL venues and answers the question Gemini surfaced: *what is the mathematical justification for the mode boundary?*

## Section outline
1. **Abstract** — dual-mode language, affine types for `@safe`, ARC for managed, formally grounded mode boundary, RustBelt-style soundness target
2. **Introduction** — the Rust/Ruby tension, prior formal work, contribution statement
3. **Background**
   - 3.1 Affine vs linear type systems
   - 3.2 RustBelt, Iris, and Coq mechanization for Rust
   - 3.3 ARC semantics (Swift precedent)
   - 3.4 Typed actors and session types
4. **Garnet core calculus** — λ-calculus with two sub-calculi (λ_managed, λ_safe) and a bridging judgment
5. **Mode boundary semantics** — the formal rules governing crossings; ownership transfer at the boundary; the `@safe` escape hatch
6. **Soundness theorem (sketch)** — progress + preservation for each sub-calculus; non-interference across the boundary
7. **RustBelt-style grounding** — how Iris separation logic applies to λ_safe; managed side uses a simpler semantic model
8. **Memory primitives as typed resources** — working/episodic/semantic/procedural memories as affine resources with decay (R+R+I formula from OQ-7)
9. **Typed actor protocols** — session-typed channels, compiler-enforced protocols, multi-agent consistency (OQ-8)
10. **Related work** — Rust, Swift, Pony, Encore, Koka, Granule, ATS
11. **Discussion & limitations** — what is proved, what is conjectured, engineering ladder dependency
12. **Conclusion** — Garnet as the first dual-mode language with a formal mode-boundary calculus
13. **References**

## Gemini contribution integration map
- RustBelt/Iris/Coq grounding → §3.2 + §7
- Affine type theory framing → §3.1 + §4
- R+R+I decay formula → §8
- RLM recursion guardrails as normative → §9 discussion
- PolarQuant/QJL mechanics → *not here* (belongs in Paper IV appendix, Priority 3)

## Execution notes
- Doctoral tone, dense math notation, JetBrains Mono for inline code
- Use actual inference rules (typeset) where possible
- Target length: 18–25 pages
- Cite: Jung et al. (RustBelt POPL'18), Swierstra (Iris), Walker (linear types tutorial), Swift evolution proposals for ARC/actors/Sendable
- End with explicit "submittable to PLDI 2027" positioning

---

# PRIORITY 3 — PAPER IV v2.1.1 FINALIZATION

**Output:** `Paper_IV_Garnet_Agentic_Systems_v2_1_1.docx` (binary docx)
**Inputs needed from project:**
- `Paper_IV_Garnet_Agentic_Systems_v2_1.docx` (existing v2.1)
- `GARNET_Paper_IV_v2_1_1_Micro_Update.md` (pending patch, already in project)
- `GARNET_v2_1_Gemini_Synthesis.md` (source of PolarQuant/QJL mechanics)

## Execution steps
1. Read docx skill, then read existing Paper IV v2.1 binary
2. Apply the micro-update patch verbatim (it was written to be drop-in)
3. Add new **Appendix B — PolarQuant & QJL Mathematical Mechanics** based on Gemini's synthesis:
   - PolarQuant: polar coordinate quantization, mathematical formulation, runtime-layer positioning
   - QJL: quantization with Johnson-Lindenstrauss projection, sketch of the guarantee, engineering implications
   - Explicit callout: *these are runtime signals, not language-core guarantees* (consensus point 8)
4. Regenerate as clean binary `.docx` — not markdown
5. Verify all four-model consensus points still reflected in the body
6. Update handoff v2.4 → v2.5

## Acceptance criteria
- Binary docx opens cleanly in Word
- Appendix B is mathematically rigorous, ~3–5 pages
- Micro-update patch is fully absorbed (no dangling TODOs)
- Consensus point 8 (TurboQuant = runtime) is explicit in the appendix framing

---

# PRIORITY 4 — RUNG 2: `garnet-parser` CRATE

**Output:** Rust crate at `/mnt/user-data/outputs/garnet-parser/`
**Target:** Tokenize + parse Mini-Spec v0.2 §2.1, §4.1, §5.1–5.3 grammars
**Skill to invoke first:** none required; pure Rust

## Crate layout
```
garnet-parser/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── lexer.rs          # tokenizer
│   ├── token.rs          # token enum
│   ├── ast.rs            # AST node types
│   ├── parser.rs         # recursive-descent parser
│   ├── error.rs          # ParseError + spans
│   └── grammar/
│       ├── mod.rs
│       ├── managed.rs    # §2.1 managed-mode surface
│       ├── safe.rs       # §4.1 @safe mode
│       └── actors.rs     # §5.1–5.3 typed actors
├── tests/
│   ├── lex_tests.rs
│   ├── parse_managed.rs
│   ├── parse_safe.rs
│   └── parse_actors.rs
└── examples/
    ├── hello.garnet
    ├── safe_hot_path.garnet
    └── actor_pingpong.garnet
```

## Cargo.toml essentials
- edition = "2021"
- deps: `logos` (lexer) or hand-rolled, `thiserror`, `miette` for diagnostics, `insta` for snapshot tests
- dev-deps: `pretty_assertions`

## Execution steps
1. Read `GARNET_v0_2_Mini_Spec_Stub.md` from project — extract §2.1, §4.1, §5.1–5.3 grammar verbatim
2. Scaffold Cargo project in outputs
3. Define `Token` enum covering all keywords from the spec (`@safe`, `actor`, `own`, `borrow`, `let`, `def`, `end`, `spawn`, `send`, etc.)
4. Implement lexer with span tracking
5. Implement AST node types mirroring spec productions
6. Implement recursive-descent parser, one module per grammar section
7. Write at least 12 test fixtures covering happy paths and 4 error cases per module
8. Write 3 example `.garnet` files demonstrating each mode
9. Ensure `cargo build && cargo test` passes cleanly in a sandbox before presenting
10. Update handoff v2.5 → v2.6 with rung 2 marked complete, rung 3 (managed interpreter + REPL) queued

## Acceptance criteria
- `cargo test` green
- All three grammar sections parse their example files
- Error messages use `miette` spans for good diagnostics
- README includes Proverbs 29:18 + Iron Canvas framing + rung 2 status

---

# PRIORITY 5 — EXECUTIVE OVERVIEW v2.2 REFRESH

**Output:** `GARNET_v2_2_Executive_Overview.md` (markdown, light refresh)
**Input:** `GARNET_v2_1_Executive_Overview.md` from project

## Execution steps
1. Read existing v2.1 executive overview
2. Apply these edits (surgical, not rewrite):
   - Header bump: v2.1 → v2.2
   - Add one paragraph in "Current State" citing the **Four-Model Consensus Memo** as the new credibility floor (Claude Opus 4.6, GPT-5.4 Pro, Grok 4.2, Gemini 3.1 Pro Deep Research — eight convergence points, three adjudicated divergences)
   - Add one paragraph citing **Mini-Spec v0.2** as canonical (§3.1 affine types + RustBelt, §5 RLM guardrails, OQ-7 decay, OQ-8 multi-agent consistency)
   - Add one paragraph referencing **Paper V** if already complete at time of refresh
   - Update the engineering ladder status block to reflect whichever rungs have landed
   - Preserve all existing language; this is a delta, not a rewrite
3. Update handoff v2.6 → v2.7

## Acceptance criteria
- Redline-style minimal diff from v2.1
- All four-model consensus points referenced at least once
- Mini-Spec v0.2 cited by filename
- Proverbs 29:18 anchor preserved

---

# HANDOFF UPDATE PROTOCOL (every priority MUST follow)

At the end of any priority execution, the executing session writes `GARNET_v2_{N+1}_HANDOFF.md` containing:

1. Version bump + date
2. "State of the project" updated to reflect the completed priority
3. Corpus inventory updated with new file(s)
4. Engineering ladder status updated (✅ / ⬜)
5. "Recommended next session order" advanced by one
6. Continuation invocation pointing at the new handoff + this protocol file
7. Proverbs 29:18 anchor preserved

This ensures the handoff file is always the single source of truth and Jon's invocation pattern never changes:

> "Continuing Garnet. Read `GARNET_v2_N_HANDOFF.md` and `GARNET_v2_2_Master_Execution_Protocol.md`. Execute Priority N."

---

# TONE & STYLE (applies to all priorities)
- Doctoral-grade rigor, vision-driven execution
- Iron Canvas aesthetic where visual: #0a0a0f OLED, garnet/rust/ruby palette, Instrument Serif / DM Sans / JetBrains Mono
- Scripture references welcome; match Jon's warmth (💎🫡👊🏾🙏🏾)
- Four-model convergence is credibility floor — reference naturally, not performatively
- Decisive execution on the bedrock; clarifying questions only on truly ambiguous asks

---

*"In the multitude of counsellors there is safety." — Proverbs 11:14*
*"The plans of the diligent lead surely to abundance." — Proverbs 21:5*
*"Where there is no vision, the people perish." — Proverbs 29:18*

**Master Execution Protocol prepared by Claude Opus 4.6 | April 12, 2026 | v2.2**
