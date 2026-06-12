# Studio macOS Feature Judge Audit — 2026-06-12

**Method:** 19-agent judged evaluation of the macOS Studio app after the PR #391
parity port: 16 per-feature judges (the nine ported rows + seven existing
surfaces, including the locator/runner pattern judged as a structural feature),
2 gap finders (missing-features lens; realignment/low-value lens), then 2
independent auditors over all judge outputs — one consolidating, one
adversarial (re-verifying every load-bearing claim against source and striking
judge errors). Lane: MacBook Air, Claude Code (Fable 5), ultracode. This file
is the audit of the judges' outputs — the durable record; raw verdicts live in
the session workflow transcript.

**Honest scope:** evaluations of a research-grade prototype's local workbench.
Nothing here upgrades any enforcement, production, or release claim.

---

## Verdict summary (per feature)

| Feature | Verdict | Benefit | Alignment |
|---|---|---|---|
| Row 1 version truth | enhance → **built** (stale stamps fixed) | medium | aligned |
| Row 2 splash/boot | keep (polish folded into future chrome slice) | medium | aligned |
| Row 3 simple/power modes | enhance → **built** (fallback, shortcut gating, write-through) | medium | aligned |
| Row 4 validated settings | enhance → partially built; range alignment rides with async slice | medium | needs-realignment |
| Row 5 process discipline | enhance → async refactor **queued** (top of follow-up) | high | aligned |
| Row 6 truth tiles | adjust → **built** (live decoder bug fixed) | high | needs-realignment → fixed |
| Row 7 hover help | keep | medium | aligned |
| Row 8 evidence reader | enhance → **built** (preview wired, sort fixed, roots-derived copy) | high | aligned |
| Row 9 chrome | adjust → **built** (light theme rendered; ⌘4/⌘5 gated) | medium | needs-realignment → fixed |
| Examples | keep (file-open gap **built** via ⌘O/drag-drop) | high | aligned |
| Converter | keep | high | aligned |
| Release reporters | enhance → 4 missing Mac reporters queued AFTER dedup | medium | aligned |
| Agentic matrix | keep (async rider queued) | high | aligned |
| Overview | keep (truth tiles on Overview **built** via rank-1) | medium | aligned |
| Locator/Runner pattern | **adjust** — 12 char-identical families, ~1,000 dup lines → queued #4 | n/a | needs-realignment |

## Built this PR (audited `build_now`, 6 of 8)

1. **Truth decoder → real truth.json shape** (live bug the judges caught in the
   parity build itself: nested `workspace_tests.passed` vs flat key — the tile
   rendered "—" forever). `securityTestCount` deleted: truth.json's omissions
   block deliberately refuses to stamp it; rendering it would reintroduce the
   drift the truth surface exists to kill (adversarial auditor catch).
2. **Stale `garnet 0.4.2` stamps** (3 sites; `--smoke-test` failed
   unconditionally against the 0.8.1 CLI) → interpolate `StudioVersion.release`.
3. **Evidence preview** — `newestEntries` by modification date (lexical-sort
   newest-drop fixed); UI-dead `readEvidenceText` now renders the newest
   bundle's primary artifact in-app; status-bar root copy derived from
   `StudioEvidenceReader().roots`.
4. **Mode coherence** — simple-mode fallback selection, ⌘4/⌘5 + WorkflowGrid
   gating, mode write-through to settings.json.
5. **Light theme actually renders** — semantic/adaptive fills replace hardcoded
   near-blacks (a persisted user-facing option was claim-broken under a
   string-presence-only gate).
6. **Native file open** — ⌘O + drag-drop onto the editor (512 KiB cap, language
   from extension); user-initiated authority, NOT routed through the evidence
   reader (no widening).

## Queued follow-ups (audited, in order)

1. **Async run path** (both auditors' top defect): every spawn currently blocks
   the main actor (up to the matrix timeout); move `runBridged` call sites into
   background tasks with an `isRunning` flag, surface `timedOut`/`duration`
   structurally; the Windows-aligned timeout ranges (command 30…14400 default
   900; matrix 60…21600 default 5400) ride in the same slice — raising ceilings
   before the async fix would lengthen the freeze.
2. **Locator/Runner dedup**: 12 character-identical `*ScriptLocation` +
   `*ScriptLocator` families (exact 75-line stride) + 11 byte-identical runner
   bodies + 7 evidence-dir builders → one generic each (~1,000 lines removed,
   ~32% of the file); delete 11 write-only `@Published` paths and the dead
   `StudioBootSequence.Status`; drop the redundant double-`env` spawn.
3. **Four missing Mac reporters** (Notarization Status, Mac Domain Proofs,
   Proof/Benchmark Status, Benchmark No-Run): real parity gap, deliberately
   sequenced AFTER the dedup so each is ~10 lines instead of ~80 of boilerplate.
4. **Per-command evidence sealing** (StudioEvidenceWriter porting Windows
   `write_command_evidence`): depends on the async seam; makes the truncation
   marker's "when one exists" non-vacuous.
5. **Jon-gated, human-merge-only:** contract-test tightening (regex
   no-second-stamp guard, adaptive-color and shortcut-gating assertions).
   Gate edits must never ride with the source they police (integrity rule 1).

## Struck by the adversarial audit (examples)

- "Add a securityTestCount tile" — contradicts truth.json's deliberate
  omissions block.
- Deleting the `target/mac-studio-domain-proofs` evidence root to make the
  status-bar copy true — wrong direction; the root anticipates the Mac Domain
  Proofs reporter. Copy now derives from code instead.
- Release judge's 5-family dedup — undercounted a 12-family problem.
- All same-PR gate-file edits smuggled inside judge proposals — reserved for
  human merge.
- The standalone timeout-range slice — harm claim partly speculative; coupled
  to the async fix instead.

## Boundaries held

No provider/network path, no evidence-reader widening, no hand-written release
stats, no frozen-crate edits, no gate/CI changes in the shipped set; Co-typist
and any shell-contract amendment remain Jon-gated and undesigned.
