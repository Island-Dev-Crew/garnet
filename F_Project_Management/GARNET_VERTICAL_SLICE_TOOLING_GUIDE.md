# Garnet Vertical Slice And Tooling Guide

Generated: 2026-05-09

This file is the lightweight companion to
`F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md`.
Use the implementation plan as the authoritative checkbox ledger. Use this
guide to keep every future readiness slice consistent in shape, evidence, and
publishing discipline.

## Current Verified Baseline

- Repository: `Island-Dev-Crew/garnet`
- Primary remote: `origin = https://github.com/Island-Dev-Crew/garnet.git`
- Working fork: `fork = https://github.com/Navigata1/garnet.git`
- Main baseline after the latest integration: `origin/main` at `5b940d8`
- Integrated PR: `#67`, "Phase 4BC: Fold static boolean relational guards"
- Merge commit: `5b940d8`, "Integrate verified Garnet readiness train through Phase 4BC"
- Open PRs after that merge: none
- Desktop evidence root: `/Users/idc2.0/Desktop/dogfood`

The merged train covers the cumulative readiness work from Phase 6N through
Phase 4BC. It includes Memory Core cache hardening, trait overlap/coherence
improvements, borrow-checker slices, and a long sequence of safe-mode match
coverage improvements through static boolean relational guard facts.

## Canonical Project Files

Read these first in a new session:

1. `F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md`
2. `CURRENT_STATE.md`
3. `F_Project_Management/GARNET_CURRENT_VS_HISTORICAL_LEDGER.md`
4. `F_Project_Management/ROADMAPS/GARNET_v0_5_LANGUAGE_COMPLETION_ROADMAP.md`
5. `F_Project_Management/DOGFOOD/GARNET_v0_5_DOGFOOD_READINESS_PHASE_LOG.md`
6. `C_Language_Specification/GARNET_v0_4_2_Conformance_Suite.md`
7. `F_Project_Management/v0_5_ROADMAP_INDEX.md`

Use older handoff files as history, not current truth. Prefer git metadata,
current docs, local tests, and remote checks over transcript memory.

## Slice Selection Rule

Every slice should be narrow enough to be reviewable and strong enough to
raise readiness evidence.

Good slice:

- starts from a documented deferred or partial row
- has a red test before implementation
- aligns with existing runtime semantics or explicitly updates the spec
- avoids broad "complete language" claims
- updates current-state, roadmap, conformance, and dogfood docs
- includes mutation or parent/head evidence when the behavior is subtle

Bad slice:

- claims production completeness without a production backend
- implements a broad evaluator or solver without bounded test cases
- treats unknown runtime behavior as false, safe, or covered
- expands security/trust boundaries without a threat-model note
- leaves Desktop dogfood artifacts only in `/tmp`

## Standard Tooling Flow

### 1. Provenance

Run these before starting or after a compaction:

```sh
git fetch --prune origin
git fetch --prune fork
git status --short --branch
git rev-parse --show-toplevel
git remote -v
git log --oneline --decorate --max-count=12 origin/main
gh pr list --repo Island-Dev-Crew/garnet --state open --json number,title,headRefName,isDraft,mergeStateStatus,url --limit 50
```

### 2. Branch

Branch from the refreshed `origin/main` unless a deliberately stacked PR train
is being built.

```sh
git switch -c codex/<phase-slug> origin/main
```

### 3. Red Tests

Add focused tests first. Typical surfaces:

- checker unit/integration tests under `garnet-check-v0.3/tests/`
- interpreter tests under `garnet-interp-v0.3/tests/`
- parser tests under `garnet-parser-v0.3/tests/`
- Memory Core tests under `garnet-memory-v0.3/tests/`
- top-level conformance tests under `garnet-cli/tests/conformance_skeleton.rs`
- phase gate assertions under `garnet-cli/tests/conformance_phase_gates.rs`

Record the failing command before implementation.

### 4. Implementation

Keep the patch small and local to the proven gap. Reuse existing helpers and
patterns. Do not add dependencies without explicit approval. When runtime
truth is narrower than the aspirational spec, keep the implementation narrow
and document the deferred boundary.

### 5. Documentation

For readiness slices, update whichever of these are affected:

- `CURRENT_STATE.md`
- `F_Project_Management/GARNET_CURRENT_VS_HISTORICAL_LEDGER.md`
- `F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md`
- `F_Project_Management/ROADMAPS/GARNET_v0_5_LANGUAGE_COMPLETION_ROADMAP.md`
- `C_Language_Specification/GARNET_v0_4_2_Conformance_Suite.md`
- `C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.csv`
- `F_Project_Management/DOGFOOD/GARNET_v0_5_DOGFOOD_READINESS_PHASE_LOG.md`
- `F_Project_Management/DOGFOOD/GARNET_DOGFOOD_PHASE_OWNERSHIP_REGISTER.md`

### 6. Local Verification Ladder

Use focused tests first, then broaden:

```sh
cargo fmt --all -- --check
git diff --check
cargo test -p <crate> --test <focused-test> <filter> -- --nocapture
cargo test -p garnet-cli --test conformance_skeleton <filter> -- --nocapture
cargo test -p garnet-cli --test conformance_phase_gates <filter> -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
cargo audit
cargo deny --all-features check
```

Known acceptable `cargo deny` state when exit code is zero: duplicate-version
warnings for `cpufeatures` and `unicode-width`; advisories, bans, licenses, and
sources must still be ok.

### 7. Dogfood Evidence

Each meaningful readiness slice should produce a Desktop checkpoint under:

```sh
/Users/idc2.0/Desktop/dogfood
```

Useful bundle shape:

- `dogfood-readiness-report.md`
- `dogfood-readiness-data.json`
- `dogfood-readiness-matrix.md`
- `dogfood-readiness-mutations.md`
- raw focused test logs
- remote check table after PR checks complete
- PR metadata JSON
- HTML deck and screenshot when generated
- `artifact-files.txt`
- `MANIFEST.sha256`
- `manifest-verify.log`

Update the root Desktop index and manifest after copying a bundle.

### 8. GitHub PR Flow

Use `gh` for normal PR publication and checks:

```sh
git push -u fork <branch>
gh pr create --repo Island-Dev-Crew/garnet --base main --head Navigata1:<branch> --draft --title "<title>" --body-file <body-file>
gh pr view <number> --repo Island-Dev-Crew/garnet --json statusCheckRollup,mergeStateStatus,isDraft,url
gh pr edit <number> --repo Island-Dev-Crew/garnet --body-file <body-file>
```

Readiness-sensitive PRs are guarded by `.github/workflows/dogfood-readiness.yml`.
If the diff touches language/spec/conformance, Memory Core, checker,
interpreter, parser, examples, workflow, or project-management evidence paths,
the PR body must keep the `## Dogfood Readiness` section from the template and
fill in current truth, local verification, remote verification, Desktop
dogfood bundle, and deferred/out-of-scope evidence. This adapts the useful
`no-mistakes` pipeline idea to Garnet without adding a push proxy: the evidence
travels with the PR and CI rejects readiness-sensitive PRs that omit it or make
an unqualified production ARC completion claim.

Use `F_Project_Management/DOGFOOD/GARNET_DOGFOOD_READINESS_SKILL.md` as the
model-agnostic readiness workflow. It fuses the PR evidence gate with local
gate discipline and adds `python3 scripts/garnet_readiness_status.py` for
checkbox-based slice completion reporting.

The CLI may not have org merge permissions. If `gh pr merge` returns a
permission error, use the org-authorized browser/Desktop session. The browser
path successfully merged PR `#67` when CLI merge was blocked.

## Security And Trust Surfaces

For dogfood readiness, include security review rows when a slice touches:

- cache keys, replay, or provenance
- path handling, symlink handling, permissions, or temp files
- persistence formats and malformed input
- concurrency or file locking
- command execution or external tools
- network, release, installer, or package paths
- safe-mode ownership, borrowing, drop discipline, or boundary freezing
- actor `Sendable` or cross-thread/cross-actor payloads

Security proof should be concrete: a regression test, a static gate, a CI
security job, a threat-model update, or a verified negative case. Avoid
security prose without a falsifiable check.

## Next Slice Candidates After Phase 4BC

Current concrete unchecked implementation row:

- Memory Core Step 9: promote fixture-backed roots toward production ARC
  allocator roots.

Treat that as a test-first, bounded slice. A good first move is not "production
ARC complete"; it is a falsifiable production-facing allocator interaction
such as finalization reporting, root-buffer/decrement behavior through the
allocator surface, or safe-mode exclusion evidence that is no longer only a
standalone fixture graph.

Still-deferred larger tracks:

- full CFG NLL, loop fixed points, drop elaboration, two-phase borrows
- imported-package trait coherence and specialization
- cross-file/package const imports and broad const evaluation
- recursive/open payload match reasoning and open-domain range exhaustiveness
- native backend, proof mechanization, and empirical PLDI-grade validation
- org release publication and network-backed installer smoke

Keep each one in its own narrow PR train with evidence.
