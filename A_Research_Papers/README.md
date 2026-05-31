# Garnet Research Papers Index

This folder is semantic memory: it contains the research argument behind
Garnet, not day-to-day implementation truth. For current executable status,
start with `../CURRENT_STATE.md`, the Mini-Spec, the conformance matrix, and the
repo-native status reporters.

## Reading Order

1. `GARNET-The-Reconciliation-of-Rust-and-Ruby.md`
   - Public thesis paper. Best first read for the original Rust/Ruby synthesis.
2. `Paper_III_Garnet_Synthesis_v2_1.md`
   - Expanded synthesis, market framing, and positioning.
3. `Paper_VI_Garnet_Novel_Frontiers.md`
   - Novelty claims: LLM-native syntax, progressive type disclosure,
     compiler-as-agent, kind-aware memory, bidirectional error bridging,
     hot-reload boundaries, and deterministic builds.
4. `Paper_VI_Empirical_Validation_Protocol.md`
   - Pre-registered measurement plan for the novelty claims.
5. `Paper_VI_v4_0_Revisions.md`
   - Honesty pass on measured and partial results.
6. `Paper_VII_Implementation_Ladder_and_Tooling.md`
   - Engineering ladder and tooling discipline.
7. `Paper_IV_Addendum_v1_0.md` and `Paper_V_Addendum_v1_0.md`
   - Addenda to the agentic-systems and formal-grounding threads.

## Source Formats

Markdown files are preferred for agent and GitHub review. PDF and DOCX files
are presentation/archive companions. If the same topic exists in multiple
formats, treat the Markdown file as the easiest review surface unless a handoff
or current state document says otherwise.

## Naming Cleanup Policy

The current filenames preserve historical provenance but are hard to scan. Do
not rename these files one by one. Instead, use a dedicated documentation
normalization phase that:

- creates a mapping table from old path to proposed canonical path,
- updates inbound links in README, website pages, handoffs, and specs,
- preserves redirects or compatibility notes for historical references,
- runs `rg` before and after to prove no dead links were introduced,
- records the rename in the current-vs-historical ledger.

Recommended canonical shape for a future pass:

```text
A_Research_Papers/
  01-reconciliation-rust-ruby.md
  02-synthesis-positioning.md
  03-agentic-systems.md
  04-formal-grounding.md
  05-novel-frontiers.md
  06-empirical-validation.md
  07-implementation-ladder-tooling.md
  addenda/
  archive-formats/
```

Until that phase lands, this index is the quick-understanding surface.
