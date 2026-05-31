# The AI-PR-review-collapse wedge (S49)

The launch narrative for an adopter deciding whether to try Garnet — not a proof
matrix, a *story* with a runnable artifact behind it.

## The thesis

As AI writes more of the code, human PR review collapses under volume. Reviewers
skim large, plausible, AI-authored diffs and wave them through. The failures that
slip past are rarely logic bugs the checker would catch — they are **authority**
and **dependency** changes: a helper quietly gains network access, a dependency
name is a near-miss of a real one, an exception is swallowed wholesale.

Garnet's answer is to make those changes **machine-reviewable in one command**,
O(1) in the size of the surrounding diff:

- **`garnet diff-caps`** (S37) — flags any GAINED capability between two versions.
- **`garnet sandbox`** (S46) — shows the sandbox/egress consequence of a surface.
- **over-catch advisory** (S42), **slopsquatting guard** (S45) — the same posture
  for error-swallowing and dependency near-misses.

## The scenario (runnable)

`examples/wedge_pr_review/before.garnet` declares `@caps(fs)`. The AI-suggested PR
(`after.garnet`) adds plausible "telemetry" and silently widens the entry point
to `@caps(fs, net)` — a filesystem program that can now reach the network is an
exfiltration path.

```sh
garnet check   examples/wedge_pr_review/before.garnet   # 0 diagnostics
garnet check   examples/wedge_pr_review/after.garnet    # 0 diagnostics  ← invisible to the checker
garnet diff-caps examples/wedge_pr_review/before.garnet \
                 examples/wedge_pr_review/after.garnet   # + caps GAINED: net → AUTHORITY EXPANDED (exit 1)
garnet sandbox examples/wedge_pr_review/after.garnet     # egress: allow (was deny-all)
```

The escalation is **clean to the type/safe-mode checker on both versions** — it
is not a bug, it is an authority change — yet `diff-caps` catches it instantly.
That is the wedge: capability review does not depend on a human reading the whole
diff.

## Artifacts

- `garnet-cli/tests/pr_review_wedge.rs` — the CI-gated correctness proof (runs on
  every OS in the `cargo test --workspace` matrix).
- `scripts/smoke_garnet_pr_review_wedge.py` — the narrative report
  (`--format md|json`); `scripts/test_garnet_pr_review_wedge.py` exercises it.

## Honest scope (do not soften)

- The "human PR review collapses under AI volume" claim is the **motivating
  thesis**, **not a measurement made here**. No human-review-time numbers are
  fabricated. The artifacts measure only that the machine gates fire as designed
  on the scenario.
- This is a **narrative composition** of existing gates (S37/S42/S45/S46), not a
  new enforcement mechanism and not a guarantee against all AI-PR risks. A PR
  that stays within its declared authority, or that an attacker crafts to keep
  the capability surface unchanged, is out of this gate's reach.
