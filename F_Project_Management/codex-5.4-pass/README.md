# Codex 5.4 Pass — Reconciliation Inbox

**Purpose:** Drop folder for whatever GPT Codex 5.4 (high-thinking) produces when the user runs a parallel pass over Garnet. When the user returns with Codex output, Claude reads everything in this directory + reconciles with the existing project.

**Anchor:** *"Two are better than one, because they have a good reward for their labour." — Ecclesiastes 4:9*

---

## What to drop in this folder

When you run Codex against the project, save its output into this directory:

```
codex-5.4-pass/
├── README.md                    ← this file (do not delete)
├── codex_output_<topic>.md      ← whatever Codex emits, one file per topic
├── codex_diffs/                 ← if Codex proposes file diffs, put them here
└── codex_questions.md           ← any questions Codex raises
```

Useful prompts to give Codex:

1. *"Read the entire `Garnet_Final/` corpus. Identify any contradictions between the seven research papers, the Mini-Spec v1.0, and the engineering artifacts. Report each contradiction with file paths + line numbers."*
2. *"Read `F_Project_Management/GARNET_v4_2_HANDOFF.md` and the Phase 6 handoffs. Identify any phase deliverable that's claimed complete but that the source/handoffs don't substantiate. Report each gap."*
3. *"Read `Paper_VI_Empirical_Validation_Protocol.md` and `GARNET_v4_0_PAPER_VI_EXECUTION.md`. For each of the 7 contributions, verify the execution result is consistent with the pre-registered pass/fail criterion. Flag anything that looks like a post-hoc threshold adjustment."*
4. *"Read `Mini-Spec v1.0` and identify any feature it specifies that's not implemented in the workspace crates (parser/interp/check/memory/actor-runtime/stdlib/cli/convert). Report each gap with where in the spec the feature appears."*
5. *"Read the README and the v4.2 final handoff and write a 1-page critical-review memo as if you were an MIT reviewer skeptical of the project. What's the strongest argument against accepting the submission as-is?"*

The fifth is the most valuable — adversarial reading is what catches gaps a friendly reading misses.

---

## What Claude does on reconciliation

When you ping me with the Codex output landed here:

1. **Read every file** Codex produced in this directory
2. **Categorize** each finding into:
   - **Merge** — additive, not contradicted by pre-registered claims, improves the corpus
   - **Investigate** — plausible but needs verification before merging
   - **Reject** — contradicts pre-registered Phase 1C / Paper VI commitments OR misunderstands the spec
3. **Diff against the existing project** — for each merge candidate, identify the smallest patch that lands the change
4. **Write `CODEX_RECONCILIATION_REPORT.md`** in `F_Project_Management/` documenting:
   - Total findings: N
   - Merged: M (with the patches landed)
   - Rejected: R (with the rationale per finding)
   - Open questions: Q (with what additional info is needed)
5. **Apply the merges** as discrete commits each citing the Codex-finding ID
6. **Update memory** + the v4.2 handoff if any architectural claim shifts

The discipline that holds: pre-registered Phase 1C thresholds are NOT renegotiable. If Codex suggests that a Paper VI partial-result should be re-classified as supported, that's REJECTED — the v4.0 honest scorecard stands. Codex output is treated as a second pair of eyes, not as authoritative.

---

## What NOT to drop in here

- The user's prompts to Codex (those go in `prompt-history.md` if you want to keep them)
- Speculative redesigns that aren't in scope for v4.2 (those go to `F_Project_Management/post-MIT-roadmap.md` if anywhere)
- Anything Codex hallucinated about the codebase (file paths or symbols that don't exist) — flag those for me to verify, but they're not load-bearing

---

*"In the multitude of counsellors there is safety." — Proverbs 11:14*

*Drop folder created 2026-04-17 in anticipation of the user's Codex 5.4 pass. When output lands, ping Claude in a fresh session referencing this README.*
