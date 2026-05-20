# Garnet Dogfood Readiness Skill

Status: active process skill

Purpose: make every readiness slice reviewable, reproducible, and honest about
what remains deferred. This skill fuses the PR #71 dogfood evidence gate with
the useful No Mistakes pattern: work on a branch, verify locally, preserve
evidence, then let remote CI confirm the merge decision before promotion.

As of 2026-05-20 this skill is on its v0.5.x pulse. There are now two
distinct readiness reporters and both matter — do not conflate them:

- `scripts/garnet_readiness_status.py` — implementation-plan slice tracker
  (`F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md`
  checkboxes). Reports `87/87` (`100.0%`) today.
- `scripts/garnet_mit_readiness_status.py` — MIT / productization lane
  reporter, the live "what's actually verified" gauge. Reports
  `71.9 % / 21 lanes / 12 verified` after the v0.5.1 sweep. **This is
  the load-bearing signal**, and the S0 `--check-no-regression` flag is
  enforced on every PR via `.github/workflows/dogfood-readiness.yml`.

The org `v0.5.0` release is tagged (`13a5805`) with Linux `.deb`/`.rpm`,
macOS aarch64/x86_64 CLI tarballs, unified `SHA256SUMS`, and
darwin-arm64/linux-x64 VSIX assets. `scripts/verify_org_release_smoke.sh`
passed against `Island-Dev-Crew/garnet` `v0.5.0` without source fallback;
the published darwin-arm64 VSIX produced an injected standalone VS Code
diagnostic.

## When To Use

Use this skill for any PR touching language readiness, Memory Core, parser,
checker, interpreter, conformance tests, workflows, project-management truth,
examples, or release evidence.

Skip it only for plainly unrelated text or repository housekeeping that does
not touch a readiness-sensitive path.

## Current-Truth Boot

Run this before selecting a slice:

```sh
git fetch --prune origin
git fetch --prune fork
git status --short --branch
gh pr list --repo Island-Dev-Crew/garnet --state open --json number,title,headRefName,isDraft,mergeStateStatus,url --limit 50
python3 scripts/garnet_readiness_status.py
```

Use the output to decide whether the next action is implementation, cleanup,
or process hardening. Historical handoffs are useful context, but live git
state and tracked docs win.

The dogfood gate is intentionally "no-mistakes"-style: it requires explicit local
verification, explicit remote checks, and no unqualified production ARC completion
claims in readiness-sensitive PRs before merge review.

## Slice Completion Signal

The canonical sliced implementation plan is:

```text
F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md
```

The script below computes the current checkbox-based completion status:

```sh
python3 scripts/garnet_readiness_status.py --format markdown
python3 scripts/garnet_readiness_status.py --format json
```

If you want a quick milestone completion view, reuse the markdown output and read
the `## Section Completion` block (added by this skill):

```sh
python3 scripts/garnet_readiness_status.py | sed -n '/## Section Completion/,$p'
```

Include the markdown output in Desktop dogfood bundles and use the JSON output
when a future dashboard or project tracker needs machine-readable progress.

### Live run snapshot

- `origin/main` at `e43d378` (post v0.5.0 tag + v0.5.1 sweep — S0 + S1–S10
  all in `merged` state; eleven v0.5 slice contracts closed).
- Open PRs in `Island-Dev-Crew/garnet` after the v0.5.1 sweep: none.
- All v0.5 slices merged through the documented fork → PR → IslandDevCrew
  squash-merge path; the slice discipline is captured in
  `F_Project_Management/GARNET_AGENT_HANDOFF_2026_05_20.md` and the v0.5.x
  PR set ([#185](https://github.com/Island-Dev-Crew/garnet/pull/185),
  [#186](https://github.com/Island-Dev-Crew/garnet/pull/186),
  [#187](https://github.com/Island-Dev-Crew/garnet/pull/187),
  [#188](https://github.com/Island-Dev-Crew/garnet/pull/188),
  [#189](https://github.com/Island-Dev-Crew/garnet/pull/189),
  [#191](https://github.com/Island-Dev-Crew/garnet/pull/191),
  [#193](https://github.com/Island-Dev-Crew/garnet/pull/193), tag
  authorization [#199](https://github.com/Island-Dev-Crew/garnet/pull/199)–[#202](https://github.com/Island-Dev-Crew/garnet/pull/202),
  and v0.5.1 follow-on [#208](https://github.com/Island-Dev-Crew/garnet/pull/208),
  [#209](https://github.com/Island-Dev-Crew/garnet/pull/209),
  [#211](https://github.com/Island-Dev-Crew/garnet/pull/211),
  [#213](https://github.com/Island-Dev-Crew/garnet/pull/213)).
- Completion: `87/87` tracked implementation-plan slices (`100.0%`)
  AND `71.9 % / 21 lanes / 12 verified` on the MIT-lane reporter — the
  load-bearing signal is the lane count balanced against `active-partial`
  / `planned` / `blocked` lanes, not the headline %.
- Release evidence: `Island-Dev-Crew/garnet` release `v0.5.0` exists at
  tag `13a5805` with Linux `.deb`/`.rpm` packages, macOS aarch64 and
  x86_64 CLI tarballs, unified `SHA256SUMS`, and darwin-arm64 + linux-x64
  VSIX assets.
- Installer evidence: `./scripts/verify_org_release_smoke.sh` passed
  against the org `v0.5.0` release without source fallback; the installer
  honestly fell back from the unavailable signed `.pkg` to the
  aarch64-apple-darwin tarball and verified its `SHA256SUMS` entry. Apple
  Developer ID notarization, signed `.pkg`, and Marketplace / OpenVSX
  publication remain credential- or infra-blocked, not technical.
- v0.6 forward-look: `F_Project_Management/GARNET_v0_6_SLICE_DOGFOOD.md`
  is the successor contract; slices S11 (scaffold) and S12–S16 (resolver,
  registry stub, VM v0.2 function-call lowering, CST, LSP v0.2) are
  scoped there.

## Evidence Gate

Readiness-sensitive PRs must satisfy `.github/workflows/dogfood-readiness.yml`.
The PR body must include:

- current-truth refresh evidence,
- local verification commands,
- remote verification status,
- Desktop dogfood bundle path,
- deferred and out-of-scope claims.

The gate intentionally rejects unqualified production ARC completion claims.
Allocator-wrapper evidence is valuable, but production allocator-integrated
ARC and runtime finalizer invocation stay deferred until source and tests prove
that exact path.
Do not shorten that into a production allocator-integrated ARC completion
claim.

## Local Gate Discipline

Use this order for substantial slices:

1. Branch from refreshed `origin/main`.
2. Write or update the narrow failing test first when behavior changes.
3. Implement the smallest behavior that makes the test pass.
4. Run focused verification.
5. Run the full verification ladder when code or workflows changed.
6. Generate a Desktop dogfood bundle with logs, PR metadata, and manifest.
7. Open a draft PR with the dogfood evidence section filled in.
8. Wait for remote checks before merge.

This is the Garnet-native version of a No Mistakes gate: local evidence first,
remote confirmation second, no silent quality bypass.

## Desktop Bundle Minimum

Every readiness bundle should include:

- `dogfood-readiness-report.md`
- `dogfood-readiness-data.json`
- `dogfood-readiness-matrix.md`
- `dogfood-readiness-mutations.md`
- `readiness-slice-status.md`
- command logs for focused and full verification
- `MANIFEST.sha256`

Generate `readiness-slice-status.md` with:

```sh
python3 scripts/garnet_readiness_status.py > readiness-slice-status.md
```

## Quality Bar

Green tests are necessary but not sufficient. A slice is reviewable only when
the code, docs, PR body, Desktop evidence, and deferred-boundary claims all
tell the same story.
