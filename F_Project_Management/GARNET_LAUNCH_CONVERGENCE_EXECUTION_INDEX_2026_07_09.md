# Garnet Launch-Convergence Execution Index

**Design:** `F_Project_Management/GARNET_LAUNCH_CONVERGENCE_DESIGN_2026_07_09.md`
**Baseline:** `9c9ca9e3538e4dd226e9cea356cf7ecd1ba92297`
**Execution model:** one focused PR per slice; Garnet dogfood and full remote CI before merge
**Hard stop:** Jon alone chooses FIRE/HOLD, tags, releases, and public posting

## Ordered Workstreams

| Order | Workstream | Entry condition | Exit evidence | Detailed plan |
|---|---|---|---|---|
| 0 | Design and plan | Approved decomposition | Design + execution index merged | this PR |
| 1 | Truth Lock | Workstream 0 merged | tracked launch ledger; native Linux reporter truth; current machine snapshot; contracts aligned | `docs/superpowers/plans/2026-07-09-garnet-truth-lock.md` |
| 2 | W-PLAY | Truth Lock merged | `garnet-wasm`; browser check/run/diff-caps; clean-browser 30-second proof | written against post-Truth-Lock `main` |
| 3 | Minimum Shelf | Truth Lock merged; disjoint worktree from W-PLAY | filesystem registry launch set; MCP stdio library; reject-without-seal demo | written against post-Truth-Lock `main` |
| 4 | Front Door | Truth Lock claim ledger frozen; W-PLAY/shelf evidence available before final copy | `/why`; live playground route; synchronized status; browser/mobile proof | written after proving artifacts exist |
| 5 | Launch Lock | Workstreams 1-4 merged | manifest-verified launch packet and cross-OS/browser matrix | written after Workstream 4 |
| 6 | Post-launch State of the Union | Jon records launched commit/tag | self-contained HTML and prioritized remaining-work queue | generated only after launch |

## Concurrency Rules

- Workstream 1 is serial because it owns shared reporters and truth contracts.
- W-PLAY and Minimum Shelf may run in parallel after Truth Lock, using separate
  worktrees and disjoint files.
- Front Door may prepare layout in parallel, but final claims and links wait for
  W-PLAY and shelf artifacts.
- The readiness reviewer is read-only before publication and after remote CI.
- `garnet-memory-core-implementer` is reserved for an approved post-launch
  Memory Core slice unless a launch defect proves Memory Core is critical.
- No target-system verification lane authors shared-kernel fixes.

## Universal Slice Gate

Every mergeable implementation slice runs:

```sh
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
python3 scripts/check-agent-contracts.py
python3 scripts/test_check_agent_contracts.py
```

Add `cargo doc`, browser/Playwright, platform smoke, and reporter gates when the
touched surface requires them. Build a manifest-verified dogfood bundle,
validate the PR body, wait for the complete remote CI matrix, and merge through
the established account-switched flow. Never modify the gate a PR merges under
without Jon's explicit human-merge approval.

## Stop Conditions

- A live claim exceeds its deterministic trap or artifact.
- A capability widening is not rejected.
- A slice requires changing its own acceptance gate.
- A target-platform claim lacks target-platform execution.
- A tag, release, public post, account, signing, or marketplace action is next.

At a stop condition, preserve evidence and report. Do not paper over the gap.
