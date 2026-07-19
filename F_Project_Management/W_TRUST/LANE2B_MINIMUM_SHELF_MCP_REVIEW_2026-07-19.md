# Lane 2B Minimum Shelf + MCP rolling trust review — 2026-07-19

This is the path/digest-bound W_TRUST companion for
`mission/l2b-sealed-shelf-mcp`.

- Implementer: Codex GPT-5.6 Sol
- Independent reviewer: Claude Code Fable 5 (MacBook Air)
- Authenticated carrier / ceremony seat: Jon
- Integration base: `cede73c03c5d535306ed179b5882e99e4d17b050`
- Review Verdict 01: APPROVE
- Review Verdict 02: APPROVE-WITH-BLOCKERS
- Review Verdict 03: APPROVE-WITH-BLOCKERS; F1 truth-surface pairing authorized
- F1 precondition: cured and cross-checkout proven at
  `a6f0da2b81a9b181dafb83e15a17f8f313406e49`
- Launch: HOLD; Band 3 ceiling while U-17 remains open

This companion does not reopen S114 or authorize FIRE, merge, tagging,
publishing, signing, hosted registry infrastructure, network transport, or any
Ring surface beyond the one Tier-1 local tool. Every merge remains Jon-only.

## Binding 1 — protected CLI dispatch

Verdict 02 decision 2 independently reviewed and authorized exactly this path:

| Field | Bound value |
|---|---|
| Path | `garnet-cli/src/bin/garnet.rs` |
| Reviewed head | `c333db5f83114f6ad0525ba68e97602de95a8503` |
| Reviewed tree | `6dab95d30bebb4cd115faf942aa71b488d9e1a81` |
| Git blob | `27835ca37a8ebe20ec67820148ee9b9679d014a2` |
| Blob SHA-256 | `a0d049cd8a5bebba45a365e117d87418ad6e73b19d6cf0dd9f783596bc851a08` |
| Blob bytes | `13358` |
| Reviewer | Claude Code Fable 5 (independent reviewer, MacBook Air) |
| Carrier | Jon |

The reviewed change is the one dispatch line
`"mcp-serve" => cmd::mcp_serve::run(&args[1..]),`. The current branch resolves
that path to the same Git blob and SHA-256. Any future change to that path
voids this binding and requires re-review.

## Binding 2 — deterministic Shelf reporter

Verdict 02 decision 3 authorized new reporter logic at exactly this path after
F1 was cured:

| Field | Bound value |
|---|---|
| Path | `scripts/smoke_garnet_minimum_shelf.py` |
| Implementation head | `178edfcead4763b8e3a57f074aa58a9e2f3c3cd1` |
| Implementation tree | `a219e2f72ad483d2dd4eb9ebe98089da4282d9cb` |
| Git blob | `83a5354d680d69016a8a83443e2d28c829439a46` |
| Blob SHA-256 | `1eb19526d315f92c14e2380c49b5df465bd33a5aab93faaaf9ea4eff4aac6af2` |
| Blob bytes | `21529` |
| Authorization source | `ops/lane2b/review/02-verdict.md`, decision 3 |
| Final content verdict | Pending Review Request 03; never implied here |

The reporter's ordinary gate is read-only and deterministic. Its bounded
`--emit-wv6` producer refuses overwrite and writes only the frozen WV-6
destination from an already-accepted status plus committed byte-fenced inputs.
It consumes only committed, bounded artifacts; performs no network access;
fetches no Git refs; records no wall clock, hostname, ambient dependency, or
Jon-only action; validates the raw CRLF Content-Length transcript; pins the one
package and canonical prelude; and preserves the exact UNSIGNED limitation.

The final extended blob above ran from exact head `89f1894` in isolated LF and
default-Windows checkouts. Both emitted the same 1,849-byte verdict, SHA-256
`91d855f7413a4c3702da4189fad5f5040fa57d861187b764060dc3c422770c8e`,
with exit 0 and zero stderr. Evidence is committed at
`ops/lane2b/evidence/12-reporter-cross-checkout.txt`.

This binding records the reviewer-authorized implementation boundary. It is
not a self-issued final code approval: Request 03 must independently review the
exact reporter blob above. Any reporter-byte change before that verdict voids
this binding and requires a new digest plus disclosure.

## Binding 3 — WV-6 acceptance truth-surface pairing

Verdict 03 independently verified the F1 cure and authorized updating exactly
the stale protected repository-state test. The cure strengthens the state
expectations: WV-6 must be accepted with all five checks and five artifacts,
while WV-7 must remain pending with zero passed checks and a nonzero gate.

| Field | Bound value |
|---|---|
| Path | `scripts/test_garnet_wv_acceptance_status.py` |
| Implementation head | `115b1cdb315cf90ceb414c37e20effa186391e25` |
| Implementation tree | `b24c5d9c54deab2692924df026ccef8eb56d513f` |
| Git blob | `d10c665f1f4f09fbe97a990e30bb3dfbd007b570` |
| Blob SHA-256 | `b3929a2af9b6bb0365641c5313e227e55989db444215c648176bc2b272a14421` |
| Blob bytes | `7661` |
| Authorization source | `ops/lane2b/review/03-verdict.md`, F1 |
| Final content verdict | Pending Review Request 04; never implied here |

No malformed-evidence, missing-check, hash-mismatch, candidate-existence, or
fail-closed contract trap was removed or relaxed. The paired test asserts exact
state, gate exit code, passed/required counts, artifact count, and findings for
both WV-6 and WV-7. Any later byte change voids this binding.

## F1 cure and provenance

The F1 RED is preserved at
`ops/lane2b/evidence/09-f1-lf-checkout-red.txt`. The cure is preserved at
`ops/lane2b/evidence/10-f1-canonical-reseal-green.txt`:

- `.gitattributes` pins the byte-hashed prelude and flagship JSON outputs to LF;
- the compiled prelude contains zero CR bytes on both checkout conventions;
- two clean builds emitted byte-identical seals;
- the committed seal matches both rebuilds;
- sealed positive 1/1, native stdio 2/2, and negative traps 6/6 passed in both
  exact-cure checkout modes;
- Cargo.lock did not change;
- the predicate remains explicitly UNSIGNED and does not claim signer identity.

## Review boundary and remaining ceremony

Review Verdict 01 approves the framing core. Review Verdict 02 approves the
sealed/MCP implementation subject to F1 and authorizes the CLI and reporter
bindings. Review Verdict 03 independently verifies the durable F1 cure and the
first two bindings, then authorizes the exact protected truth-surface pairing
bound above. The focused test and full-battery parity are green locally; Review
Request 04 owns final review of Binding 3 and the still-required squash-durable
WV-6 landed-main contract.

No bare `Trust-Kernel-Review:` trailer is used. This companion, the immutable
verdicts, exact Git objects, and final independent review are the evidence.
