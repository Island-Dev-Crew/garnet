# Lane 2C Review Verdict 02 — B1 Evidence Cure and F2 Rename

request: `ops/lane2c/review/02-request.md`
reviewer: Claude Code on **Claude Fable 5** (`claude-fable-5`, Anthropic)
reviewer_machine: `Pulses-MacBook-Air.local`; Darwin 25.5.0; arm64 (Apple M5);
  fanless — functional and byte-level claims only, no timing claims
implementer_identity_as_found: OpenAI Codex, GPT-5-based model (exact
  submodel not exposed by its harness), on `NUCBOX_M2PRO_S`, Ubuntu WSL2
  x86_64 (`/dev/sdd` ext4)
branch: `mission/l2c-memory-teardown`
reviewed_head: `2576db3c76afac02433c1ddf14651685c9cbe568`
request_parent: `29cb1c700c47b48bd4f2902c94c794fd0c6a3cb7` (Verdict 01)
reviewed_product_head: `5cd113617acd35307bb028463833a8da2bbd6ad2` (unchanged)
reviewed_product_tree: `85faad1de5a2c47cb632bedea78dfb89d209001a` (unchanged)
swept_at: `2026-07-31T09:03:48Z` (Friday 04:03 America/Chicago — before
  Friday sunset; the Sabbath fence is not active)
scope: B1 and F2 only; Verdict 01's correctness analysis is NOT reopened
authorship: this verdict and the Part A correction commit preceding it are
  authored by this seat's own identity, `Claude Fable 5
  <noreply@anthropic.com>`, per the ceremony authorship ruling
verdict: **APPROVE — B1 is cleared by the six Memcheck captures; F2 is cured
  records-only; the product boundary is byte-unchanged. Approval authorizes
  only Jon's later merge decision.**

## Boundary verification

- The successor delta `29cb1c7..2576db3` is one commit, authored by the
  implementer's own identity (`OpenAI Codex <codex@openai.com>`), and touches
  `ops/lane2c/**` only — 18 paths: the six captures, the memcheck README,
  `replay_memcheck.sh`, verifier/manifest/measurement/record updates, the
  `DOCTRINE.md → PROPOSED-DOCTRINE.md` rename (R098), and this round's
  request.
- `git diff 5cd1136 2576db3 -- garnet-memory-v0.3` is **zero lines**. The
  product head and tree are byte-identical to what Verdict 01 reviewed, so
  Verdict 01's decision-equivalence proof and named tests stand without
  re-examination (request item 1: PASS).
- All six gates pass at the reviewed head (request item 2: PASS):
  `verify_evidence.py`, lane0 closeout, MSRV, frozen backlog, capability
  scope, evidence integrity — every one exit 0 on this seat.

## B1 — independent re-derivation

All six capture hashes were recomputed on this seat and match the request
table and `MANIFEST.sha256` exactly; a full `shasum -a 256 -c` over the
manifest returns zero non-OK lines. The leak tables below are re-derived
from the raw capture text by this reviewer, not copied from the record:

| Phase | Case | def. lost | ind. lost | poss. lost | still reachable | allocs | frees | outstanding |
|---|---|---|---|---|---|---:|---:|---:|
| Before | working-clear | 0 B/0 | 0 B/0 | 0 B/0 | 544 B/1 | 289,654 | 289,653 | **1** |
| Before | episodic-drop | 0 B/0 | 0 B/0 | 0 B/0 | 544 B/1 | 289,645 | 289,644 | **1** |
| Before | semantic-drop | 0 B/0 | 0 B/0 | 0 B/0 | 544 B/1 | 290,669 | 290,668 | **1** |
| After | working-clear | 0 B/0 | 0 B/0 | 0 B/0 | 544 B/1 | 1,066 | 1,065 | **1** |
| After | episodic-drop | 0 B/0 | 0 B/0 | 0 B/0 | 544 B/1 | 1,057 | 1,056 | **1** |
| After | semantic-drop | 0 B/0 | 0 B/0 | 0 B/0 | 544 B/1 | 2,081 | 2,080 | **1** |

Every error summary is `0 errors from 0 contexts`. The before-to-after delta
is zero bytes and zero blocks in every leak category for every case.

### The reason B1 clears, not just the result

The `total heap usage` lines are the substance. In all six captures the
alloc/free balance is **exactly one outstanding block**, and it is the same
block every time: a 544-byte pre-`main` runtime allocation (`malloc` reached
from `(below main)` via `libc_start_call_main.h:58`) — process bookkeeping,
not store data. Meanwhile the allocation count collapses:

```text
working-clear  289,654 → 1,066   (271.7x fewer allocations)
episodic-drop  289,645 → 1,057   (274.0x fewer)
semantic-drop  290,669 → 2,081   (139.7x fewer)
```

with frees tracking allocs exactly on both sides. **The fix removed
allocations, not deallocations.** The before-teardown's per-element
rooted-reachability scans allocated hundreds of thousands of transient
structures (live-set `BTreeSet`s, `reference_counts` maps, scan queues) —
each dutifully freed; the after-teardown's O(1) rejections allocate almost
nothing. Nothing the stores owned is retained in either tree: every
allocation except the shared runtime block is freed, before and after. That
is the freeing-behavior proof B1 demanded.

### The limit of what B1 establishes

Stated explicitly so neither leg is later cited for the other:

- **Memcheck at process exit cannot distinguish a collected cycle from
  backing storage freed at drop.** When the probe exits, Rust ownership
  frees the stores' backing memory regardless of whether the cycle fixture
  collected anything. A hypothetical collector that silently stopped
  collecting cycles would still produce these exact leak-free captures.
- **Leg 1 (this verdict, B1): freeing behavior.** The cheaper teardown
  introduces no leak and preserves alloc/free parity — established by the
  six captures above.
- **Leg 2 (Verdict 01, not reopened): cycle-collection correctness.** That
  previously collected cycles still collect rests entirely on Verdict 01's
  state-by-state decision-equivalence proof and the three named
  buffered-path tests
  (`root_buffer_release_collects_cycle_when_threshold_is_reached`,
  `allocator_edge_decrement_buffers_newly_unreachable_cycle`,
  `buffered_collection_scans_only_buffered_roots`), which ran green at the
  unchanged product head.

Both legs are now on record, separately grounded.

## Provenance and mechanics

- **Binaries reused, not rebuilt** (request item 4: PASS): the base and
  product binary SHA-256 values in this round's record
  (`4577447b…`, `0ca1e4e3…`) are byte-identical to the hashes recorded in
  the round-1 `measurement.json` at `e73f73d` — verified against the
  historical blob via `git show`, not against the current record alone. The
  captures self-identify their candidates: every before capture prints
  `candidate=efd4f6ba…` (base), every after capture `candidate=5cd1136…`
  (product), with `case`/`size=1024` matching their filenames (item 3:
  PASS).
- The quiet-state claim is correctly **none**: leak accounting is
  deterministic and the record says so rather than staging a ritual it does
  not need.
- Whitespace: `git diff --check` over the successor delta flags exactly the
  `==pid== ` separator lines of the six captures and nothing else — the
  single trailing space Valgrind emits. Preserving those bytes is required
  by the hash discipline; `verify_evidence.py` scopes its whitespace
  exclusion to exactly those six files. Disclosed, correct.
- `replay_memcheck.sh` and the memcheck section of `verifier-output.json`
  are present and the verifier parses the captures semantically
  (`parse_memcheck`), binding size 1024, per-phase binary provenance, and
  the expected path/pair set.

## F2 — cured, records-only (request item 5: PASS)

`ops/lane2c/DOCTRINE.md` became `ops/lane2c/PROPOSED-DOCTRINE.md` (rename,
98% content similarity; the diff is the rename plus status-line wording).
Current references use the proposed name (README); the former name survives
only where it is historical truth — Verdict 01, Request 01, and prior
journal entries — exactly the disposition the request describes. The binding
placement rule still lands via the governance surface (Repair #3), outside
this lane.

## F3 — correctly out of scope (request item 6: PASS)

The buffered self-cycle test was **not** implemented, and that was the right
call: it would have modified `garnet-memory-v0.3/tests/` or `src/`, moved
the product tree, invalidated the reused product binary, and reopened
Verdict 01's review of its own accord. F3 stands as a low-severity
recommendation for the next product-touching slice.

## Authorship record

- The Part A correction commit (`2b9a777…`, immediately preceding this
  verdict on the branch) records that Verdict 01's commit `29cb1c7` was
  authored under the fleet-fork identity while the actual seat was this one;
  history is not rewritten and `29cb1c7` remains byte-identical historical
  truth. That correction commit, and this verdict's commit, are authored
  `Claude Fable 5 <noreply@anthropic.com>` — the seat's own identity, per
  the ceremony authorship ruling.
- The cure commit `2576db3` is authored by the implementer's own identity.
  `IDC-Trust-Review` authors nothing anywhere on the branch (U-30 holds).
- The implementer did not author this verdict.

## Scope and not-verified

- The 18/18 Callgrind hash and count confirmation was not re-derived (out of
  scope by request); Verdict 01's correctness analysis was not reopened (the
  product tree is byte-unchanged, verified).
- This seat cannot execute the Linux x86_64 binaries or Valgrind on
  macOS/arm64; binary identity was verified by hash against the
  pre-existing round-1 provenance, and capture integrity by hash plus
  independent text re-derivation. No measurement ran on the Air.
- No implementation code, workflow, ruleset, or frozen-backlog surface was
  touched by this reviewer. The writes are: one journal correction entry,
  this verdict, and one journal heartbeat line — all `ops/lane2c/**`.

## Consequence

**B1 is CLEARED and F2 is CURED. Verdict 01's sole blocker is discharged:
APPROVE at reviewed head `2576db3c76afac02433c1ddf14651685c9cbe568`, product
boundary `5cd113617acd35307bb028463833a8da2bbd6ad2` /
`85faad1de5a2c47cb632bedea78dfb89d209001a`.** This approval authorizes only
Jon's later merge decision; it records no acceptance, merge, tag, release,
or launch promotion.

## Reviewer stdout summary

Cross-family Lane 2C Verdict 02 (Claude Fable 5, Anthropic, MacBook Air;
implementer Codex GPT-5-based on the NUC) APPROVES: all six Memcheck capture
hashes recompute exactly and their leak tables, re-derived from raw text,
show zero definitely/indirectly/possibly-lost bytes and an identical
544-byte pre-`main` runtime block as the sole outstanding allocation in
every capture — alloc/free balance is exactly one block in all six while
allocation counts drop 271.7x/274.0x/139.7x, proving the fix removed
allocations, not deallocations. Stated limit: process-exit Memcheck cannot
distinguish a collected cycle from drop-freed backing storage, so B1
establishes freeing behavior only; cycle-collection correctness rests
separately on Verdict 01's decision-equivalence proof and named tests at the
byte-unchanged product head. Binaries were reused (hashes match round-1
provenance), gates are green, the successor delta is `ops/lane2c/**` only,
F2's rename is records-only with historical names preserved, and F3 was
correctly left untouched. The Part A authorship correction landed before
this verdict; both are authored under this seat's own identity.
