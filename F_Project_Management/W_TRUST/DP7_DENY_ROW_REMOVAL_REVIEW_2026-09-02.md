# DP7 deny-row removal — cross-family confirmation record (2026-09-02)

- PR: Island-Dev-Crew/garnet #540 · branch `mission/dp7-deny-row-removal`
- Base main: `080da696e22d14780283560f007856fc69f0c642` · candidate tip reviewed: `4d8dfb5f584836f5b0f7a358c0da95820de199c8`
- Implementing seat: Claude Fable 5.1 (records/reviewer lane, implementing here) · Reviewing seat: Codex (codex-cli 0.147.0 via the local wrapper, cross-family, read-only, detached worktree at the tip; L-15 satisfied)
- Path class: `deny.toml` is not a rolling-gate trust-kernel trigger (gate `ok: true`, `touched_paths []` at the tip); this markdown record is the review artifact (a `.review.json` on a zero-trigger head is rejected by the validator, U-66 class). The verdict below is the reviewing seat's output transported verbatim; the directing seat added nothing to it.
- Merge custody: the reviewing seat's finding 2 and integrity rule 1 agree — `deny.toml` configures the required `cargo-deny check` context, so this PR is human-merge-only. Jon merges.
- Verdict: **CONFIRM-WITH-FINDINGS**, zero blocking, bound to `4d8dfb5f…`; void if the content head moves (this record commit is a records-class head move the verdict anticipates; the content commit is unchanged).

## Reviewing seat output (verbatim)

```text
github/rulesets/required-context-producers.json-181-      "semantic_sha256": "208c12dea42613373c22f76d6ad72c7ac97a69cd443785f9bbe85d390eb193d3"
.github/rulesets/required-context-producers.json-182-    },
```

Command:
```text
sed -n '107,118p' F_Project_Management/LAUNCH/LAUNCH_READINESS.md
```

Verbatim result:
```text
## Jon-only actions

- push a git tag
- cut or release a version
- record the S114 acceptance decision
- change CI/release policy or any gate a PR merges under
- RB-6 backend decision
- RB-8 root-reorg cut
- fire the launch/marketing wave
```

Conclusion: yes. Root deny.toml configures the Security workflow’s required `cargo-deny check`, so it is a gate this PR merges under. The change is therefore human-merge-only despite not triggering the rolling trust-kernel path classifier.

VOID-ON-MOVE: this review binds only `4d8dfb5f584836f5b0f7a358c0da95820de199c8` and is void if HEAD moves.

VERDICT: CONFIRM-WITH-FINDINGS — bound to 4d8dfb5f584836f5b0f7a358c0da95820de199c8

1. Blocking: none. The sole-path diff, Cargo.lock identity, live unyanked state, advisory pass, rolling-gate projection, and retired-vocabulary check all confirm the claim.

2. Non-blocking: merge-custody restriction. deny.toml controls the required `cargo-deny check` context, while repository policy reserves changes to any gate a PR merges under for Jon’s human merge. This confirmation is not an approval and authorizes no autonomous merge.



---


```

## Lineage note (records-class, 2026-09-02)

- Rebased onto main `16454b55736ebf0e7778f0e1f308da9a896d8b97` (#546) under the strict up-to-date rule (U-74). Content commit `4d8dfb5f584836f5b0f7a358c0da95820de199c8` → `553ec8e17c4212961e7f87712252de05369e0fbb`; `git diff 4d8dfb5f584836f5b0f7a358c0da95820de199c8 553ec8e17c4212961e7f87712252de05369e0fbb -- deny.toml` is empty (the reviewed content is byte-identical), so the CONFIRM-WITH-FINDINGS verdict bound to the reviewed content stands; this note is the records-class head move that carries it. Merge remains Jon's (integrity rule 1).
