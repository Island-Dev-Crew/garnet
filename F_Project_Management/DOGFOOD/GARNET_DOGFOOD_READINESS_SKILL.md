# Garnet Dogfood Readiness Skill

Status: active process skill

Purpose: make every readiness slice reviewable, reproducible, and honest about
what remains deferred. This skill fuses the PR #71 dogfood evidence gate with
the useful No Mistakes pattern: work on a branch, verify locally, preserve
evidence, then let remote CI confirm the merge decision before promotion.

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

Include the markdown output in Desktop dogfood bundles and use the JSON output
when a future dashboard or project tracker needs machine-readable progress.

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
