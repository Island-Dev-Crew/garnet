# S48 Plan — 12-domain / 7-novel proof matrix

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S48.
Map: reconciled plan §150 (+ §44 "a proof matrix is rigor evidence for a
skeptical reviewer").
Branch: `codex/s48-proof-matrix`. Base: `origin/main` @ `db67469` (S47).

## Research resolved
- 7 novel contributions (academic-strategy table): LLM-native syntax, Type
  spectrum, Compiler-as-agent, Kind-aware allocation, Error bridging, Hot-reload,
  Reproducible builds.
- 12 domains: the `CORE_12_CASES` in `scripts/smoke_garnet_studio_domain_matrix.py`
  (mvp_01..mvp_11 + mvp_11_mismatch) — reuse, do not re-declare.
- HAZARD: two contribution-numbering schemes exist (academic-strategy vs v0.5
  dogfood). ⇒ list contributions by TITLE, do NOT assign per-number verdicts;
  quote the aggregate Paper VI scorecard verbatim.

## Deliverables
- `scripts/garnet_proof_matrix.py`: import `CORE_12_CASES`; declare the 7
  contributions (title, §novelty, evidence anchors); existence-check every domain
  example + anchor. `--format md|json`; `--gate` exits 1 if anything is missing.
- `scripts/test_garnet_proof_matrix.py`: 7 unit tests (12 domains, 7
  contributions, verbatim scorecard, canonical titles, reuse, gate, md).
- Wire test + `--gate` into ci.yml agent-contracts.
- `F_Project_Management/GARNET_PROOF_MATRIX.md`: the two tables + honest scope.

## Dogfood
- `garnet_proof_matrix.py --format md` → 12/12 domains present, all 7
  contributions exercised; `--gate` exits 0. Would exit 1 if an example/anchor
  vanished.

## End-state / gates
- Full ladder green (zero Rust changed; workspace 0 failed). CHANGELOG + contract
  S48 block + matrix doc. Ledger: `s47 → merged(5)` advanced this branch; `s48`
  advance rides with S49.

## Honest scope (do not soften — Paper VI anchors)
- Evidence INVENTORY, not empirical proof; no measurement/mechanized-proof/
  external-study claims.
- No per-contribution re-adjudication; aggregate scorecard quoted verbatim
  ("4 supported, 2 partial (downgraded honestly), 0 refuted, 1 pending-infra").
- No new readiness lane.
