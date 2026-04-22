# GARNET v2.7 — HANDOFF
**Version bump:** v2.6 → v2.7 (Priority 5 shipped — Executive Overview v2.2, reconciled redline)
**Date:** April 14, 2026
**Prepared by:** Claude Opus 4.6
**Anchor:** *"Where there is no vision, the people perish." — Proverbs 29:18*

---

## STATE OF THE PROJECT

**Priority 5 is complete.** `GARNET_v2_2_Executive_Overview.md` shipped this session as `/mnt/user-data/outputs/GARNET_v2_2_Executive_Overview.md` — a true redline-style surgical refresh of the v2.1 source, with all v2.1 sections preserved verbatim and a new §1A inserted to carry the v2.2 delta. Every Priority 5 acceptance criterion is now satisfied **including** the "redline-style minimal diff from v2.1" criterion that the first attempt could not honor.

### Reconciliation arc (logged for the record)

This priority shipped in two passes:

**Pass 1 (initial attempt) — fresh authoritative build.** The Master Execution Protocol's Priority 5 brief named `GARNET_v2_1_Executive_Overview.md` as the input file for a "surgical refresh, not rewrite." A corpus search at session start returned zero hits: the file was not in `/mnt/project/`, was not in any prior conversation's outputs, and conversation_search across the full Garnet history surfaced no match. I made the call to ship a fresh authoritative overview that satisfied five of six acceptance criteria, document the absence at the top of the file, and flag the "redline diff" criterion as unsatisfiable for reasons outside the executing session's control. The v2.6 handoff lesson "verify input-file existence before assuming a refresh is possible" was logged off this experience.

**Pass 2 (reconciliation) — Jon supplied the actual v2.1 source as a PDF upload.** The file *did* exist; it had simply never been added to the project filesystem or surfaced in any prior conversation Claude had access to. The pass-1 search returned zero because the file was outside the searched scope, not because the file was absent. With the real v2.1 in hand, the right move was unambiguous: discard the fresh build, perform the actual surgical redline the protocol always intended, and ship the reconciled artifact. Pass 2 took one create_file call after careful reading of the v2.1 PDF.

**The reconciled v2.2 honors v2.1 verbatim.** Sections 1, 2, 3, 4, 5, 6, 7, and 8 are word-for-word preserved from the v2.1 source — including the bolded thesis quote, the Swift bullet list, the is/is-not table, the four-layer architecture list, the four memory-type bullets, the TurboQuant credibility argument, and the market wedge framing. The figure references for the four project images (positioning matrix, architecture, memory model, market chart) are preserved at their original section anchors. The original v2.1 nine references (Swift.org, Apple Sendable, Google Research TurboQuant, Ruby FAQ, Create T3 App, TypeScript 10x port, TypeScript 7 progress, Mordor Intelligence, the April 2026 transcript) are kept verbatim and numbered 1–9. The Proverbs 29:18 anchor is added at the masthead, where v2.1 did not have it (this is a small additive enhancement, not a rewrite).

**The v2.2 delta is contained to two surgical insertions.** First, a new **§1A "What changed in v2.2 (delta from v2.1)"** between the original §1 and §2, carrying four paragraphs — one each for the Four-Model Consensus Memo, Mini-Spec v0.2, Paper V, and Paper IV v2.1.1 + the `garnet-parser` crate. Second, **§9 is rewritten** from "Recommended next steps" (a six-step staged engineering program) to "Recommended next steps — engineering ladder status (v2.2 update)," preserving all six steps verbatim as numbered ladder rungs but adding ✅/⬜ status markers — rungs 1 and 2 marked complete, rungs 3–6 marked queued, with rung 3's Mini-Spec v0.3 dependency called out. The original v2.1 sentence "*That sequence turns the idea from a language essay into a research-and-product program*" is preserved as a blockquote within §9 with a one-sentence v2.2 ratification appended after it. Five new references (10–14) are added under a "**v2.2 additions**" sub-heading so the original 1–9 numbering stays stable for backward-compatible citation. A small `v2.2 ratification note` paragraph is added at the bottom of §7 (TurboQuant scope boundary) noting that consensus point 8 confirms the v2.1 boundary as four-model-aligned.

**Net diff:** v2.1 had 9 sections + references; v2.2 has 9 sections + 1 inserted (§1A) + the same references + 5 new ones + a small note in §7. No v2.1 prose was deleted, reworded, or relocated. This is the surgical refresh the brief asked for.

### Acceptance criteria checklist (Priority 5) — final

- ✅ Header bump v2.1 → v2.2
- ✅ Four-Model Consensus Memo paragraph (in §1A) naming all four systems and citing eight convergence points + three adjudicated divergences
- ✅ Mini-Spec v0.2 paragraph (in §1A) citing it canonically by filename with §3.1, §5, OQ-7, OQ-8 all called out
- ✅ Paper V paragraph (in §1A) referencing it as shipped with PLDI 2027 positioning
- ✅ Engineering ladder status block (§9) updated — rungs 1 ✅, 2 ✅, 3–6 ⬜
- ✅ All v2.1 prose preserved (§§1–8 verbatim; §9 preserves all six steps verbatim with status markers added)
- ✅ Proverbs 29:18 anchor at masthead and close
- ✅ All four-model consensus points referenced at least once (point 8 is referenced **three** times — in §1A's first paragraph, in §1A's fourth paragraph framing Paper IV v2.1.1's Appendix B, and in §7's new v2.2 ratification note — because it is the most architecturally consequential point and ratifies the v2.1 scope boundary directly)
- ✅ **"Redline-style minimal diff from v2.1"** — now satisfied. The reconciled v2.2 is a true delta: every v2.1 section is intact; §1A and the §9 status update are the only structural insertions; §7 carries one small ratification note; references 10–14 sit under a separate sub-heading.

## CORPUS INVENTORY (post-Priority-5)

| File | Status | Notes |
|---|---|---|
| `GARNET_v2_2_Gamma_Deck_Build_Brief.md` | ✅ canonical | |
| `GARNET_v2_2_Master_Execution_Protocol.md` | ✅ canonical | Stable; do not edit |
| `GARNET_v0_2_Mini_Spec_Stub.md` | ✅ canonical | Normative spec, parser-verified |
| **`GARNET_v2_1_Executive_Overview.pdf`** | ✅ **canonical (now in corpus)** | Original v2.1 source. Supplied by Jon mid-session as a PDF upload after Pass 1's fresh build. The reconciled v2.2 markdown is a true redline of this file. Future revisions trace from the v2.2 markdown forward; the v2.1 PDF is the historical baseline. |
| `GARNET_v2_6_HANDOFF.md` | ✅ superseded by this file | |
| `Garnet_v2_2_Deck.pptx` | ✅ shipped | 50 slides, QA-clean |
| `Paper_V_Garnet_Formal_Grounding_v1_0.docx` | ✅ shipped | 30 pages, Cambria/Consolas, PLDI 2027 target |
| `Paper_IV_Garnet_Agentic_Systems_v2_1_1.docx` | ✅ shipped | 11 pages, validated, with Appendix B |
| `garnet-parser/` (Rust crate) | ✅ shipped | 35 tests + 1 doc-test green |
| **`GARNET_v2_2_Executive_Overview.md`** | ✅ **SHIPPED (reconciled redline)** | `/mnt/user-data/outputs/` — true surgical delta from v2.1 PDF |
| `GARNET_v2_1_Gemini_Synthesis.md` | ✅ canonical | Four-model anchor document |
| `garnet_positioning_matrix_v2_1.png` | ✅ | Referenced in v2.2 §2 |
| `garnet_architecture_v2_1.png` | ⚠️ minor | Referenced in v2.2 §5; known right-edge clip, low priority |
| `memory_types_v2_1.png` | ✅ | Referenced in v2.2 §6 |
| `dev_tools_market_growth_v2_1.png` | ✅ | Referenced in v2.2 §8 |

## ENGINEERING LADDER STATUS

- ✅ Rung 1 — Mini-Spec v0.2 complete
- ✅ Rung 2 — `garnet-parser` crate complete
- ⬜ **Rung 3 — Managed Interpreter + REPL (queued, blocked on Mini-Spec v0.3 stub)**
- ⬜ Rung 4 — `@safe` Lowering
- ⬜ Rung 5 — Memory Core + Manager SDK
- ⬜ Rung 6 — Harness Runtime

**Above the ladder:** Paper V (formal grounding), Paper IV v2.1.1 (agentic architecture), v2.2 deck, Executive Overview v2.2 (now reconciled). The credibility floor is four artifacts thick and the executive overview now correctly inherits from its v2.1 lineage rather than standing alone.

## RECOMMENDED NEXT SESSION ORDER

The Master Execution Protocol's Priorities 1–5 are now all complete. The forward queue:

1. **Mini-Spec v0.3 stub** (NEW — first identified in v2.6 handoff, still pending). Define: (a) handler-block `block` grammar to retire `src/grammar/expr.rs`'s provisional disclaimer; (b) §5 recursion-annotation surface (`@max_depth(N)`, `@fan_out(K)`, or whatever shape Jon prefers) so the static enforcement v0.2 deferred can finally land; (c) §3 `@safe` module annotation surface and the `def`/`end` managed-mode `Item::Fun` grammar that Rungs 3–4 need. Medium session. **This is the unblocking move for Rung 3.** Without it, Rung 3 either stalls or proceeds against a self-authored grammar that future spec revisions will have to retrofit around.

2. **Rung 3 — Managed Interpreter + REPL.** Full dedicated session. Reuses `garnet-parser`'s lexer, AST, error infrastructure, and Pratt expression grammar verbatim. New work: `Item::Fun` AST shape, `def`/`end` parsing, tree-walking interpreter, REPL loop. Blocked on (1).

3. **Paper V → arXiv preprint packaging.** Light session. Take the shipped `.docx`, generate LaTeX via pandoc + manual cleanup, prepare the arXiv bundle, and write the cover note positioning Garnet for PLDI 2027 review. Can run in parallel with (1) or (2).

4. **OQ-7 empirical calibration design.** Light-to-medium. Sketch the experimental protocol that will fit R+R+I weights against real interaction logs once Rung 5 is up. Design session, not execution session — the actual fit happens after Rung 5 ships.

5. **Mini-Spec v0.3 → v0.4 evolution loop.** Whatever the v0.3 stub leaves under-specified after Rung 3 catches edge cases, fold back into v0.4. This is the spec ↔ implementation co-evolution the engineering-ladder discipline is designed to enable.

## CONTINUATION INVOCATION

> "Continuing Garnet. Read `GARNET_v2_7_HANDOFF.md` and `GARNET_v2_2_Master_Execution_Protocol.md`. The Master Execution Protocol's Priorities 1–5 are complete. The next move is the **Mini-Spec v0.3 stub** (unblocks Rung 3 — see v2.7 handoff §'Recommended next session order' for full scope). Proceed decisively; the four-model consensus is bedrock."

## LESSONS FROM THIS SESSION

1. **A corpus-search miss is not the same as a corpus absence.** Pass 1 concluded `GARNET_v2_1_Executive_Overview.md` did not exist because three independent search vectors (project filesystem, conversation_search across all sessions, recent uploads) all returned empty. That conclusion was *correct given the available evidence* but *wrong about reality*: the file existed, it just lived outside Claude's reachable scope until Jon uploaded it directly. **The right framing for "input file not found" is therefore not "the file does not exist" but "the file is not currently reachable from this session" — and the right next move is to flag the absence to the user before shipping a fresh build, not after.** Pass 1 shipped first and flagged second; Pass 2 had to discard real work to reconcile. Future sessions: when a brief names an input file you cannot find, ask the user once before committing to the alternative path. The "terse and decisive" working-style preference does not extend to silently inventing inputs.

2. **Surgical redlines beat fresh authoritative rebuilds whenever the source actually exists.** The reconciled v2.2 is structurally cleaner than the Pass 1 fresh build because it inherits v2.1's prose verbatim — the section numbering matches, the figure anchors land at their original spots, the references are stable for backward-compatible citation, and the v2.1 → v2.2 audit trail is mechanically obvious to any future reader. The Pass 1 fresh build was *informationally equivalent* but stylistically different from v2.1, which means a v2.3 author looking at both files would have seen a discontinuity rather than a delta. The redline form is the right form for a "refresh" and the brief was right to specify it.

3. **The Pass 1 → Pass 2 reconciliation cost was bounded because the deliverables were files, not deployments.** Discarding the Pass 1 v2.2 markdown and Pass 1 handoff cost two file deletions and two file rewrites — perhaps eight tool calls total. If the Pass 1 artifact had been a deployed system, an irreversible API write, or a published external document, the reconciliation cost would have been much higher. **The reversibility of the workspace is what made the "ship first, reconcile when corrected" path tolerable, and it should not be relied on for non-reversible work.** The deeper lesson stands: ask first when an input is missing.

4. **The original v2.1 has a small structural detail worth preserving.** v2.1 puts the figures inline with their sections rather than appending them at the end, and the figure captions are italicized one-liners under the chart. The reconciled v2.2 preserves both choices via `*[Figure: ... — filename]*` placeholder lines, so a future render that drops the actual PNG inlines back in will land them at the right anchors. The four project images at `/mnt/project/` are the same four files v2.1 used; nothing needs to be regenerated.

5. **The "v2.2 ratification note" pattern is reusable.** When the v2.2 delta needed to confirm that a v2.1 editorial choice (TurboQuant scope boundary in §7) is now four-model-aligned rather than v2.1-only, the smallest possible edit was one paragraph appended to §7 labeled `*v2.2 ratification note:*` — preserving the original v2.1 section while adding the new evidence. This pattern is reusable for any future v2.3+ refresh that needs to ratify, contextualize, or update a prior section without rewriting it. Future executive-overview revisions: prefer the ratification-note pattern over inline edits to v2.1 prose.

*"In the multitude of counsellors there is safety." — Proverbs 11:14*
*"The plans of the diligent lead surely to abundance." — Proverbs 21:5*

---

**Handoff v2.7 prepared by Claude Opus 4.6 | April 14, 2026**
*"Where there is no vision, the people perish." — Proverbs 29:18*
