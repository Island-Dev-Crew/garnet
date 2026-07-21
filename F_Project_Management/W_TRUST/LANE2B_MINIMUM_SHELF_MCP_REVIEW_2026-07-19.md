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
- Review Verdict 04: APPROVE-WITH-AUTHORIZATIONS; Binding 3 accepted and the
  content-bound, squash-durable Shelf/WV repair authorized under exact constraints
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
| Implementation head | `b00d2bd61ba2738ba2a6f552fcbde5c1f6893bf2` |
| Implementation tree | `335852f77873400fde45bd4d9eca655e5ad77eab` |
| Git blob | `57b91324221a1ba6cf326b0b74607b3248e4693f` |
| Blob SHA-256 | `10255c50f0ad30762310d89c193d6bb20779c91cf57a94338f4c40f77c85fc16` |
| Blob bytes | `23438` |
| Authorization source | `ops/lane2b/review/04-verdict.md`, decision 2 |
| Final content verdict | Pending Air Review Request 05; never implied here |

The reporter's ordinary gate is read-only and deterministic. Its bounded
`--emit-wv6` producer refuses overwrite and writes only the frozen WV-6
destination from an already-accepted status plus committed byte-fenced inputs.
It consumes only committed, bounded artifacts; performs no network access;
fetches no Git refs; records no wall clock, hostname, ambient dependency, or
Jon-only action; validates the raw CRLF Content-Length transcript; pins the one
package and canonical prelude; and preserves the exact UNSIGNED limitation.

The content-bound blob above ran twice from exact implementation head
`6b77b63e2c4bada54cb8865e6640f8f70de76605` in isolated LF and
default-Windows checkouts. All four runs emitted the same 2,071-byte verdict,
SHA-256 `a9665b8356adc2757f0d7d09b69ce468368505792e6bbfc7fe710454ac6edfbe`,
with exit 0 and zero stderr. Evidence is committed at
`ops/lane2b/evidence/17-content-reporter-cross-checkout.txt`. The final Air
replay remains mandatory because this implementer-side result is not the
independent Verdict 05.

This binding records the Verdict-04-authorized implementation boundary. It is
not a self-issued final code approval: Request 05 must independently execute and
review the exact reporter blob above. Any reporter-byte change before that
verdict voids this binding and requires a new digest plus disclosure.

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
| Final content verdict | Accepted by Review Verdict 04, Decision 1 |

No malformed-evidence, missing-check, hash-mismatch, or fail-closed contract
trap was removed or relaxed. The paired test asserts exact state, gate exit
code, passed/required counts, artifact count, and findings for both WV-6 and
WV-7. Verdict 04 accepted this exact `d10c665f...` truth-pairing blob. It is
preserved as reviewed provenance and superseded for current-tree execution only
by the Decision-2 content-provenance binding below.

## Binding 4 — Verdict 04 content and squash durability repair

Verdict 04 Decision 2 froze the three mutable namespace prefixes and the
reporter-self exclusion, authorized content/tree plus first-parent-main
provenance, and required exact new W_TRUST path/blob bindings. The current
checkpoint binds:

| Path | Git blob | Blob SHA-256 | Bytes |
|---|---|---|---:|
| `scripts/garnet_content_provenance.py` | `dffa7f7887e9ddda9dcc8c2925291a531b1a6724` | `e29c8158e5590c3435a5632069b6555a2b6f4d1ce7fe3e488cdc1031fd87d8ba` | 6548 |
| `scripts/garnet_wv_acceptance_status.py` | `e69a61f25a136c5303c427e63607114a828003e2` | `574232a2dcb559fe7221e46f2bdb77164da52760d3472b9bab41b79c9dcb562d` | 19511 |
| `scripts/test_garnet_minimum_shelf_provenance.py` | `451c4fa7cf5d8beb776730d0beb2c9aec242fec4` | `aa7e90dfcb379e26e534352319977a48dc8dc41f76f758f017b041f3cfa9560a` | 4230 |
| `scripts/test_garnet_wv_acceptance_status.py` | `d607d654ce98be263fb9591f84f0bd1a8cab5a38` | `86859b83c67fe3fbaebed652d5f5fb35e8724e1e4e13ed58f0e99ba25525bc6f` | 9318 |

The canonical product digest is SHA-256
`810f256bcf9304999975120224419216422996ff3b804d1a9a8836d5bcc4c339`
over 1,529 sorted `(path, blob-OID)` index entries. The separately preserved
Verdict-04 reviewed-tree baseline is
`1e6692175ea8fe2dd5b04fad4a492dc8ce48767dd07d88fd11a0847ce96749d5`
over 1,527 paths at tree
`f3272b9610dba756bd414cafc825fd7462d7a294`. The two extra product paths are
the authorized shared content-provenance module and its adversarial test; no
unreviewed product exclusion was added.

The exclusion list is exactly and only `ops/lane2b/**`, `proofs/**`,
`F_Project_Management/W_TRUST/**`, and the reporter path itself. The shared
module documents the reproducible `git --no-replace-objects ls-files -s -z`
construction. Three RED-before-cure traps now prove product mutation RED,
missing discarded branch objects GREEN on first-parent main content, and
evidence/content mismatch RED. Final Air execution and content verdict remain
pending Review Request 05.

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
bindings. Review Verdict 03 independently verifies the durable F1 cure and
authorizes the exact protected truth-surface pairing. Review Verdict 04 accepts
that pairing and authorizes the bounded content/squash-durability repair above.
The final native Air run must still double-run the reporter from fresh LF and
default-Windows checkouts, reproduce the main-only squash result with no pull
refs, and issue immutable Verdict 05. Until that APPROVE exists, the lane is
blocked and no PR ceremony is authorized.

No bare `Trust-Kernel-Review:` trailer is used. This companion, the immutable
verdicts, exact Git objects, and final independent review are the evidence.
