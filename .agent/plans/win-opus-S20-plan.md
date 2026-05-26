# win-opus — S20 Plan: Novel-Composition Dogfood + Program-Execution Discovery

**Slot:** win-opus · **Slice:** S20 (post-v0.7 dogfood extension, Jon-directed)
**Branch:** `agent-win-opus/s20-program-execution` (off `origin/main` `3369bd3`)
**Baseline:** `cargo test --workspace` exit 0 · `cargo clippy -D warnings` exit 0 ·
readiness 81.3% / 31 lanes. S15–S19 + #232 (Windows/Linux domain-proof matrix) all merged.

## Directive (Jon)

Surface Windows/Linux polish + smoke tests + refactor iterations that improve
UX/functionality and a **foundational grounding that the programs execute**;
take the multi-domain tested programs and the Paper-VI novel usecases and test
their **build/compile/run** in **permutations and combinations** to surface
**novel discoveries** of program use / functionality / creation method — a story
for the agentic + standard coding industries.

## Key constraint — don't duplicate, stay in lane

`origin/main` already has `scripts/smoke_garnet_studio_domain_matrix.py` (#232,
win-codex): it runs the existing **20** examples through `garnet parse/check/run`.
So a generic execution matrix is already covered. My value-add is the **novel
composition layer**: NEW programs that compose multiple Paper-VI contributions in
ways the existing 20 don't, proving emergent agentic patterns end-to-end, plus a
focused harness + the "story."

**Ownership:** strictly additive. I write only NEW files in `examples/`,
`scripts/`, `C_Language_Specification/`, `.agent/plans/`, plus section-scoped
cross-cutting (CHANGELOG, CURRENT_STATE, dogfood, readiness lane, ledger). **No
edits** to any owned crate (garnet-lsp = win-codex; cst/parser = mac-opus;
suggest-llm/garnet-lang = mac-codex; interp/vm/memory/cli = unowned/read-only) or
to win-codex's domain matrix. Anything needing those → Handoff Request.

## Runnable managed-mode palette (learned from the proven corpus)

`def f(args) { … }` · `@caps(…)` · `let` / `let mut` / `+=` · `if/elsif/else` ·
`match x { "lit" => …, _ => … }` · arithmetic + comparison + booleans · strings ·
`println("…", val)` · `crypto::blake3(s)` (deterministic hex) · `raise "…"`.
The agentic/memory/trust concepts are **modeled deterministically** (the proven
pattern — no runtime I/O dependency, so output is reproducible + assertable).

## Deliverables

### 1. Novel-composition programs (`examples/novel_*.garnet`) — each composes ≥3 contributions
- **`novel_01_capability_budgeted_memory_agent.garnet`** — capability-budget
  (toolbelt_02) + memory-recall ranking (toolbelt_03) + researcher→synthesizer→
  reviewer pipeline (mvp_06). Emergent: *capability-aware, memory-backed agent
  decisioning gated by an authority budget.*
- **`novel_02_signed_provenance_pipeline.garnet`** — `crypto::blake3` signed
  fingerprint (mvp_11) + multi-stage pipeline + determinism. Each stage's output
  is content-addressed; the final provenance hash is verified against an embedded
  expected hash. Emergent: *content-addressed provenance across an agent pipeline
  (tamper-evident build lineage).*
- **`novel_03_release_gate_quorum.garnet`** — release-gate (toolbelt_04) +
  capability-budget + signed provenance + memory-of-prior-green, requiring a
  multi-signal quorum to "approve." Emergent: *multi-signal agentic release
  governance.*
Each: `garnet check` clean (0 unexpected diagnostics), `garnet run` deterministic
with asserted `println` output. Expected output captured by actually running them.

### 2. Discovery harness (`scripts/smoke_garnet_novel_compositions.py` + `test_*.py`)
Drives the built `garnet` CLI over the novel programs: `garnet check` (no
unexpected diagnostics) + `garnet run` (exact deterministic output match) +
a "composition matrix" report (which contributions each composes). JSON + markdown.
Cross-platform (Windows + POSIX `garnet`/`garnet.exe`). The unittest tests the
pure parse/aggregate logic with synthetic data (no cargo/CLI).

### 3. Story doc (`C_Language_Specification/GARNET_NOVEL_COMPOSITIONS.md`)
For each program: the contributions composed, the emergent capability, the
"novel discovery," and an honest scope note (modeled-deterministically; runtime
integration of actors/memory/Ed25519 is tracked separately). Calibrated-honesty
voice; ties to the agentic-coding story without overclaiming.

### 4. Readiness lane + Windows CLI UX note
- `novel_composition_dogfood` lane in `garnet_mit_readiness_status.py` (status
  driven by the harness gate), baseline regenerated (preserve `source`).
- Capture the Windows `garnet new/check/run` journey on these programs + any UX
  finding in the story doc / PR body (optionally a screenshot via computer-use).

### 5. Cross-cutting + PR
CHANGELOG `[Unreleased]` · CURRENT_STATE section · S20 dogfood contract block ·
ledger PR-OPEN/REVIEW/MERGED · PR body per template · grep-loop self-review to 5/5
· merge per Jon's call.

## Test proportion (~60/40)
"Code" = the 3 Garnet programs + the harness logic. "Test" = the deterministic
`garnet run` output assertions (real behavior, not string-presence) + the harness
unittest + `garnet check` clean assertions. Real behavioral verification.

## Dogfood block
```bash
python3 scripts/smoke_garnet_novel_compositions.py     # all novel programs check+run, output matches
python3 -m unittest scripts.test_garnet_novel_compositions
cargo fmt --all -- --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
python3 scripts/garnet_conformance_matrix_check.py
python3 -m unittest discover scripts/ -p 'test_*.py'
```

## Honest scope
- Compositions are **modeled deterministically** in managed mode (the proven
  runnable subset) — they demonstrate the *patterns* end-to-end via `garnet run`;
  full runtime integration (live actor mailboxes, Mnemos stores, Ed25519 signing)
  is tracked separately and not claimed here.
- The new S17 Layer-0/1 primitives aren't interpreter-dispatched yet, so these
  programs use the proven runnable subset + `crypto::blake3`, not the new prims.
