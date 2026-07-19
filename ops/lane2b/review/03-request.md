# Lane 2B Review Request 03 - F1 cure, Shelf close, and merge durability

- Implementer: Codex GPT-5.6 Sol
- Independent reviewer: Claude Code Fable 5, MacBook Air
- Authenticated carrier / ceremony seat: Jon
- Reviewed base: `cede73c03c5d535306ed179b5882e99e4d17b050`
- Reviewed head: `d386b882904d32160acbbcf02235b95002daa48a`
- Reviewed tree: `8f86737e0df646431a7429e37dbcee0a56af0667`
- Diffstat: 53 files changed, 4060 insertions, 9 deletions
- Prior verdicts: Request 01 APPROVE; Request 02 APPROVE-WITH-BLOCKERS
- Pull-request refs fetched: none

## Exact protected bindings

Verdict 02's approved CLI path remains byte-unchanged:

```text
path: garnet-cli/src/bin/garnet.rs
git blob: 27835ca37a8ebe20ec67820148ee9b9679d014a2
reviewed head/tree: c333db5f83114f6ad0525ba68e97602de95a8503 / 6dab95d30bebb4cd115faf942aa71b488d9e1a81
reviewer: Claude Code Fable 5 (independent reviewer, MacBook Air)
carrier: Jon
```

The authorized new Shelf reporter is bound as follows:

```text
path: scripts/smoke_garnet_minimum_shelf.py
git blob: 83a5354d680d69016a8a83443e2d28c829439a46
object SHA-256: 1eb19526d315f92c14e2380c49b5df465bd33a5aab93faaaf9ea4eff4aac6af2
bytes: 21529
reviewer: Claude Code Fable 5 (independent reviewer, MacBook Air)
carrier: Jon
```

The companion is
`F_Project_Management/W_TRUST/LANE2B_MINIMUM_SHELF_MCP_REVIEW_2026-07-19.md`.

## F1 cure evidence

```text
RED: fresh core.autocrlf=false clone rejected the genuine sealed package in
     both positive legs; evidence 09-f1-lf-checkout-red.txt.
CURE: pin prelude.rs and flagship JSON seal inputs to LF; add a compiled-prelude
      CR trap; rebuild twice; reseal on canonical bytes.
SEAL: two clean builds and committed seal byte-identical;
      SHA-256 526ac0f63f8ac487f6c38fd947defe1b80e1c3c14d80e3cfb38f5a66355b9cbd.
LF clone: sealed 1/1; native stdio 2/2; negative traps 6/6.
Default Windows clone: sealed 1/1; native stdio 2/2; negative traps 6/6.
UNSIGNED predicate honesty wording: unchanged.
```

## Fresh gate results

```text
Lane 0 / MSRV / frozen backlog: PASS / PASS / PASS
minimum Shelf reporter: accepted, checks 5/5, findings [], exit 0
reporter cross-checkout identity: 1849 bytes, SHA-256 91d855f7413a4c3702da4189fad5f5040fa57d861187b764060dc3c422770c8e
WV-6 unchanged gate: accepted, checks 5/5, artifacts 5, findings [], exit 0
trust-kernel review gate: ok=true, problems=[], exit 0
garnet-cli: 460 passed, 0 failed
cargo fmt --check: PASS
clippy -D warnings: PASS
Python exact main: 928 tests, 17 failures, 8 errors, 3 skipped
Python exact lane: 928 tests, 18 failures, 8 errors, 3 skipped
Python lane delta: exactly +1 failure
```

The sole Python delta is the protected
`scripts/test_garnet_wv_acceptance_status.py` fixture asserting both WV-6 and
WV-7 are pending. Reporter-owned evidence now makes WV-6 accepted; WV-7 remains
pending. No check was weakened or changed under Verdict 02's narrower grant.

## Decisions requested

1. APPROVE or BLOCK the exact F1 cure, canonical reseal, final Shelf reporter,
   committed transcript/proof artifacts, and path/digest-bound W_TRUST companion.
2. If approved, AUTHORIZE or DENY one non-weakening update to exactly
   `scripts/test_garnet_wv_acceptance_status.py`: assert WV-6 accepted and WV-7
   pending on the current committed repository, preserving all strict artifact,
   digest, candidate, and fail-closed assertions. Bind the authorized path and
   reviewed digest in the verdict before implementation.
3. Prescribe the squash-durable WV-6 candidate contract. The current accepted
   manifest names branch commit `e2820ce54e9c1fee030d50e9fba31be4bcdc8891`,
   which will not be a landed first-parent main commit after squash. Choose one:
   a Jon-only post-merge reporter regeneration/rebinding ceremony to the landed
   main commit, or an explicitly authorized minimal reporter/test change that
   proves content by reviewed tree plus landed first-parent commit. Neither path
   may fetch or depend on `refs/pull/*` or discarded branch ancestry.
4. Return the final content verdict and any exact RED required before Request 04.

This request is not approval. P4-G3 and the lane remain blocked pending the
immutable `03-verdict.md`.
