# Lane 2B blocked checkpoint

- Implementer: Codex GPT-5.6 Sol
- Independent reviewer: Claude Code Fable 5, MacBook Air
- Authenticated carrier / ceremony seat: Jon
- Branch: `mission/l2b-sealed-shelf-mcp`
- Upstream base: `cede73c03c5d535306ed179b5882e99e4d17b050`
- Launch: HOLD
- Band: 3 while U-17 remains open
- Pull-request refs fetched: none

## What is green

- Reviewer Verdict 01: APPROVE; raw-byte framing core carries unchanged.
- Verdict 02 F1 cure: prelude and flagship seal inputs are LF-canonical; two
  clean builds resealed byte-identically; fresh LF and default-Windows clones
  pass sealed 1/1, native stdio 2/2, and all negative traps 6/6.
- Deterministic Shelf reporter: accepted, 5/5, findings empty; byte-identical
  output in isolated LF and default-Windows checkouts.
- Unchanged WV-6 acceptance gate: accepted, 5/5 checks, 5 artifacts.
- `garnet-cli`: 460/460; fmt and strict clippy pass.
- Trust-kernel companion gate: `ok: true`, `problems: []`.
- Lane 0, MSRV, and frozen-backlog boot floors: PASS.

## Blocking conditions

1. The locked full Python battery differs from exact upstream main by one
   failure: base 928/17F/8E/3S versus lane 928/18F/8E/3S. The sole new failure
   is the protected regression that asserts both WV-6 and WV-7 are pending.
   WV-6 is now reporter-accepted while WV-7 remains pending. Verdict 02 did not
   authorize changing `scripts/test_garnet_wv_acceptance_status.py`; no test or
   gate was weakened to manufacture parity.
2. The frozen WV-6 manifest currently names branch candidate `e2820ce...`.
   Squash merge will discard that branch commit from main. The reviewer must
   prescribe a landed-first-parent rebinding ceremony or authorize a minimal
   squash-durable reporter/test change that never fetches `refs/pull/*`.
3. Final independent approval of the F1 cure, reporter content, proofs, and
   W_TRUST bindings remains pending in Review Request 03.

## Exact resume

1. Fetch the fork branch tip only; never fetch `refs/pull/*`.
2. Read `ops/lane2b/review/03-verdict.md` in full and verify its exact reviewed
   head/tree before acting.
3. If authorized, RED-record then make only the exact non-weakening protected
   fixture and squash-durability changes named by the verdict.
4. Re-run the full Python battery against exact built main and require zero lane
   delta; re-run all Shelf, WV-6, trust, Rust, fmt, and clippy gates.
5. Commit Review Request 04 for final approval. Jon alone opens/merges the PR.

Evidence: `ops/lane2b/evidence/13-p4-wv6-and-battery-stop.txt`.
