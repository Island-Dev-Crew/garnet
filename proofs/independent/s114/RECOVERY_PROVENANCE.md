# S114 independent-evidence recovery — provenance

This directory makes the S114 independent-review evidence **durable and
gate-enforced**. Before this landing, the strongest evidence existed only as
**unreachable Git objects** in `C:\garnet` (and on a deleted branch / the
Navigata1 fork), subject to `git gc` pruning. Mission phase P1 (condition #2 of
the S114 acceptance) restores it into tracked history where
`scripts/garnet_evidence_integrity_status.py --gate` verifies its manifest
hashes on every run.

**This is a durability landing, not a re-verification.** Each bundle records
the reviewer's captures at the reviewer's stated base commit; those bases
predate later `main` history. Nothing here claims verification of current HEAD.

## What was recovered

### 1. Codex independent verdict (`proofs/independent/s114/codex-verdict-20260625/`)
- **Origin commit:** `61cfbae` — *"docs: add S114 Codex independent verdict"* (dangling in `C:\garnet`; protective ref `s114/recovered-codex-verdict`).
- **Reviewer:** Codex (OpenAI) — the cross-lineage **independent** verifier.
- **Review base (audit pin):** `a7f946dc405612e43580e4d983e40a049dab04b8` (from the bundle's `environment.json`).
- **Origin branch:** `codex/s114-independent-verdict-20260625` (deleted from origin; never PR'd).
- **Verdict document:** landed byte-identical at
  `F_Project_Management/W_TRUST/S114_CODEX_INDEPENDENT_VERDICT_2026_06_25.md`
  (the dossier Step-9 prescribed home). It is **verbatim** as sealed; its
  internal references to
  `F_Project_Management/W_TRUST/S114_CODEX_INDEPENDENT_VERDICT_2026_06_25_proofs/...`
  point to the **pre-relocation** path — that proof bundle is now this
  directory (`proofs/independent/s114/codex-verdict-20260625/`). It was moved
  under `proofs/` so the repo's integrity gate sweeps it.
- **Proof files:** all **784** files are **byte-identical** to `61cfbae`
  (verified: each file's SHA-256 matches the origin manifest).
- **Manifest:** `MANIFEST.sha256` here is **re-expressed bundle-relative** — the
  common path prefix
  `F_Project_Management/W_TRUST/S114_CODEX_INDEPENDENT_VERDICT_2026_06_25_proofs/`
  was stripped from each entry so the gate (which resolves entries relative to
  the manifest's own directory) can verify it. **The 784 hashes are unchanged.**
  The origin manifest (`61cfbae`) had repo-root-relative paths and its own
  SHA-256 was `ba07c1e8f86db719ed8721be9e109c72d07adee6a6c68abf9557c51562ea0a6a`.
  Reproduce the re-expression: strip that prefix from the origin manifest lines;
  the hash column is identical.

### 2. Windows lane-2 S114 review (`proofs/validation/s114-review/windows-20260628-lane2/`)
- **Origin commit:** `6153726` — *"validation: record Windows S114 review proof"* (Jon Isaac, 2026-06-28; dangling in `C:\garnet`; protective ref `s114/recovered-lane2-review`).
- **Reviewed commit:** `2e2fe843e87be0c8fc9a4745a5bb138fba597d23` (the commit's sole parent — lineage is structural).
- **Validation branch:** `validation/2026-06-25-codex-s114-review` (present in the nested Desktop repo and on the Navigata1 fork).
- Landed **byte-identical at its original path** (its `MANIFEST.sha256` is
  already bundle-relative and self-verifies). Report at
  `F_Project_Management/VALIDATION_REPORTS/2026-06-28_windows_codex_lane2_s114_review.md`.
- **Lane-2 verdict summary (from the report):** S114 re-verification *mostly
  HELD* on Windows — 5/5 HELD on the HIGH probes; the vendored dependency
  preload lane was PARTIAL (trap surfaced, no secret leak, but the process
  exited 0). That fail-soft is the target of mission phase P3.

## S114 review lineage (for audit)

| Role | Who | Base / commit |
|------|-----|---------------|
| Original red-team | Claude fleet | PR #365 |
| **Independent re-verifier** | **Codex (OpenAI)** | base `a7f946d`; verdict `61cfbae` |
| Fixes | — | `4994867` (#420), `47a7ba7` (#421) |
| Final review (≠ independent) | Opus (Claude) | — |
| Windows lane-2 review | Jon | reviewed `2e2fe84`; bundle `6153726` |
| Relabel (pending acceptance) | Jon | PR #438 |
| Scoped acceptance | Jon | `F_Project_Management/LAUNCH/S114_ACCEPTANCE.json` (2026-07-12) |

## Verify

```
python scripts/garnet_evidence_integrity_status.py --gate
```

Both bundles are swept by `proofs/**/MANIFEST.sha256` and hash-verify against
the committed bytes; `.gitattributes proofs/** -text` keeps them free of EOL
normalization on every platform.
