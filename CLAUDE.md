# CLAUDE.md - Garnet Claude Code Operating Brief

Claude Code must treat this file as a bridge into Garnet's live project truth,
not as the primary source of truth. Git state, current docs, and fresh
verification output outrank every historical handoff. If this file disagrees
with `git` or a current reporter, trust `git`/the reporter and fix this file.

## First Read

Before editing, read:

1. `F_Project_Management/GARNET_POST_0_8_1_SYSTEM_HANDOFF.md` — the current
   post-cut operating brief and non-negotiable boundaries.
2. `F_Project_Management/GARNET_v0_8_1_PLAN.md` — the active runway/plan.
3. `F_Project_Management/AGENT_COORDINATION_LEDGER.md` — the live multi-agent
   ledger (most recent entries = current frontier).
4. `CURRENT_STATE.md` — reviewer/contributor orientation.
5. `CHANGELOG.md` — the canonical, same-PR release ledger.
6. Root `AGENTS.md` and the nearest subsystem `AGENTS.md` before edits.

## Current Repository Truth

Verify with `git` before relying on any line here.

- **Latest tagged release: `v0.8.1`** (annotated tag → commit `8107c01`,
  re-cut signed 2026-06-07). `v0.8.0` (`cc165e8`) and `v0.4.2`/`v0.5.0` precede it.
- **Binary status:** the `v0.8.1` Release ships **signed `garnet-0.8.1-*` CLI
  binaries** (`.deb`/`.rpm`/darwin tarballs) + a CycloneDX SBOM + a GPG-signed
  `SHA256SUMS.asc` (public key in `docs/garnet-release-signing.pub.asc`, fpr
  `04D5…6ED1`; verify per `docs/release-signing.md`). The S91–S120 trust-kernel
  work is in the published 0.8.1 binary. The older `v0.8.0` tag still carries the
  `garnet-0.5.0-*` build. Still research-grade, not production/1.0.
- **Remotes:** `origin` = `Island-Dev-Crew/garnet` (main); `fork` =
  `Navigata1/garnet` (PRs open from the fork → origin).
- Garnet is a **research-grade prototype (v0.x.x), not production / 1.0.**

## Boot Verification

Run this before any edit:

```sh
cd /Users/IDC2.5/Desktop/Garnet
git fetch --prune origin
git fetch --prune fork
git status --short --branch
git remote -v
git log --oneline --decorate --max-count=12 origin/main
gh pr list --repo Island-Dev-Crew/garnet --state open --json number,title,headRefName,isDraft,mergeStateStatus,url --limit 50
```

For any dogfood archive you depend on, verify its manifest before citing it.

## Work Selection

Do not hardcode the next slice from this file. Choose it from the live plan
(`GARNET_v0_8_1_PLAN.md`), the post-cut handoff, the coordination ledger, the
goal ledger (`.dogfood/goal.json`), and open PR state after the boot
verification above. The cut act and release tags are **Jon-owned** — never push
a tag autonomously.

## Mandatory Discipline

- Verify git state and manifests before edits.
- One slice per PR; narrow and evidence-backed. Branch from fresh `origin/main`.
- Red tests before behavior changes; run focused tests before full verification.
- **"Enforced" only means a deterministic trap proven by test.** Never call a
  generated policy "enforced"; never call the S114 red-team "independent."
- Preserve the named-deferred fences: `@bounded` (Wasmtime fuel), memory, time,
  `@mailbox`, and macOS/Windows OS-sandbox application remain
  declared-not-enforced; only `@caps` + `@max_depth` are enforced (both
  backends), with seccomp applied on **Linux only**.
- **Four integrity rules:** (1) a PR may not modify the gate it merges under
  (CI / dogfood skill / diff-caps thresholds / capability-manifest standard
  changes are **human-merge-only**); (2) a capability-surface widening must fail
  the gate and block merge; (3) every autonomous merge records agent / model /
  gate-version; (4) the release **tag stays Jon's**.
- Update docs / conformance / dogfood / CHANGELOG ledgers when readiness changes.
- Copy durable evidence to `/Users/IDC2.5/Desktop/dogfood/` and reseal manifests
  when a dogfood bundle is part of the deliverable; do not leave it only in
  `/tmp`.
- If anything fails, report the command, failure, and last known good state.

## Standard Verification Ladder

Focused (per slice):

```sh
cargo fmt --all -- --check
git diff --check
cargo test --workspace --no-fail-fast        # or the crates the slice touches
python3 scripts/garnet_v0_8_1_release_readiness.py --gate
```

Trust-kernel anti-rot gates (when the slice touches the trust spine or evidence):

```sh
python3 scripts/garnet_red_team_status.py --gate
python3 scripts/garnet_evidence_integrity_status.py --gate
python3 scripts/garnet_ultrapunch_dossier_status.py --gate
python3 scripts/garnet_domain_proof_artifacts_status.py --gate
python3 scripts/garnet_academic_evidence_status.py --gate
```

Full:

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
cargo audit
cargo deny --all-features check
```
