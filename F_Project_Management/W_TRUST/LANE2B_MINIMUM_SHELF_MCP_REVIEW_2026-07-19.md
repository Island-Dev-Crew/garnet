# Lane 2B Minimum Shelf + MCP rolling trust review — 2026-07-19

This is the path/digest-bound W_TRUST companion for
`mission/l2b-sealed-shelf-mcp`.

- Implementer: Codex GPT-5.6 Sol
- Independent reviewer: Claude Code Fable 5 (MacBook Air)
- Authenticated carrier / ceremony seat: Jon
- Integration base: `cede73c03c5d535306ed179b5882e99e4d17b050`
- Review Verdict 01: APPROVE
- Review Verdict 02: APPROVE-WITH-BLOCKERS
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
| Implementation head | `1dce85d49cba4b3347f74d5ed459c839c35641a6` |
| Implementation tree | `eac23f1bf068c614f9edf1c299223900d3b271d9` |
| Git blob | `787ccbd4342ff196d7ba7870f2ebe53010cc025d` |
| Blob SHA-256 | `be74800c2a99e7cf23f49a8f504508a1ac6b8b1a1aea5fb58ce3136951699163` |
| Blob bytes | `17903` |
| Authorization source | `ops/lane2b/review/02-verdict.md`, decision 3 |
| Final content verdict | Pending Review Request 03; never implied here |

The reporter is read-only and deterministic. It consumes only committed,
bounded artifacts; performs no network access; fetches no Git refs; records no
wall clock, hostname, ambient dependency, or Jon-only action; validates the
raw CRLF Content-Length transcript; pins the one package and canonical prelude;
and preserves the exact UNSIGNED predicate limitation.

Two isolated exact-head checkouts — one `core.autocrlf=false`, one default
Windows `core.autocrlf=true` — each emitted the same 1,849-byte stdout verdict,
SHA-256 `f0714edd7f5f7ce8b3c33420c58fce0d78f0816f41acafc750d8ce60f2677f8e`,
with exit 0 and zero stderr. Both held zero `refs/pull/*` refs.

This binding records the reviewer-authorized implementation boundary. It is
not a self-issued final code approval: Request 03 must independently review the
exact reporter blob above. Any reporter-byte change before that verdict voids
this binding and requires a new digest plus disclosure.

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
sealed/MCP implementation subject to F1 and authorizes the two bindings above.
F1 is now green. The deterministic reporter has not yet received its final
content verdict; Review Request 03 owns that step.

No bare `Trust-Kernel-Review:` trailer is used. This companion, the immutable
verdicts, exact Git objects, and final independent review are the evidence.
