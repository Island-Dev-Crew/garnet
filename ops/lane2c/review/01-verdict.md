# Lane 2C Review Verdict 01 — Memory Teardown Integrity

request: `ops/lane2c/review/01-request.md`
reviewer: Claude Code on **Claude Fable 5** (`claude-fable-5`, Anthropic)
reviewer_machine: `Pulses-MacBook-Air.local`; Darwin 25.5.0; arm64 (Apple M5);
  fanless — functional and byte-level claims only, no timing claims
review_family: Anthropic Claude — cross-family from the implementer
implementer_identity_as_found: OpenAI Codex, GPT-5-based model (exact submodel
  not exposed by its harness), on `NUCBOX_M2PRO_S` (Windows 11 Pro 10.0.26200;
  measurement guest Ubuntu WSL2 x86_64) — recorded consistently in the
  request, `W_TRUST.md`, `measurement.json`, and the lane journal
branch: `mission/l2c-memory-teardown` (fetched from the fork by explicit
  refspec; zero `refs/pull/*` in the clone)
reviewed_base: `efd4f6bae8b3afaba74594e57944b2548142aeae` (exact origin/main)
reviewed_product_head: `5cd113617acd35307bb028463833a8da2bbd6ad2`
reviewed_product_tree: `85faad1de5a2c47cb632bedea78dfb89d209001a`
branch_tip: `e73f73d7345b2934f8a563270fc788d3ccfe4ddc` (`ops/lane2c/**` records only)
swept_at: `2026-07-30T16:37:58Z`
curve_evidence: independently confirmed before this round (18/18 profile
  hashes and instruction counts); NOT re-derived here by instruction
verdict: **BLOCKED — one blocker (B1, missing memcheck leak evidence). The
  correctness questions all resolve favorably with proofs and named tests;
  the blocker is evidence-only and requires no product-tree change.**

## Boot and mechanical verification

- `core.autocrlf` global: `false` before cloning. Fresh space-free,
  non-sync-managed clone; `origin` = `Island-Dev-Crew/garnet`, `fork` =
  `Navigata1/garnet`; the lane branch fetched only by
  `+refs/heads/mission/l2c-memory-teardown:refs/review/l2c`. Zero
  `refs/pull/*` after all fetches (request item 1: PASS).
- Product head/tree reproduce exactly; the product diff is exactly the three
  claimed paths, 195 insertions / 5 deletions: `cycle.rs` +95/−5, the crate
  example +100, `AGENTS.md` +5 (item 2: PASS).
- Lineage: three commits, linear, single-parent, merge-base exact
  `origin/main`. All three are authored and committed by
  `OpenAI Codex <codex@openai.com>`; `IDC-Trust-Review` is absent, so the
  hard U-30 rule holds. This is the same fleet-fork authorship deviation
  recorded as Lane 1 Verdict 11 finding F1; whatever ratification the
  ceremony seat applied there applies here identically (observation, not a
  new blocker — the identity disclosure is total).
- No manifest of any kind exists under `ops/**`; root `Cargo.lock` SHA-256 is
  exactly `01b8986b1cee0ef6a53ac439bd018b54fc1dca825a8f845a259ed8001e6715fa`
  (also byte-proof that zero dependencies were added); the MSRV gate reports
  exactly the 18-manifest set (item 3: PASS).

Commands executed on this seat, all green (item 4: PASS):

```text
python3 -I ops/lane2c/verify_evidence.py --gate                exit 0
python3 -I scripts/garnet_lane0_closeout_status.py --gate      exit 0
python3 -I scripts/garnet_msrv_status.py --gate                exit 0
python3 -I scripts/garnet_frozen_backlog_status.py --gate      exit 0
python3 -I scripts/garnet_capability_scope_status.py --gate    exit 0
python3 -I scripts/garnet_evidence_integrity_status.py --gate  exit 0
cargo +1.95.0 test --locked -p garnet-memory --test cycle      20 passed / 0 failed
cargo +1.95.0 test --locked -p garnet-memory                   all suites pass; stress 4 ignored as designed
cargo +1.95.0 test --locked -p garnet-cli cache                3 + 5 passed / 0 failed
cargo +1.95.0 clippy --locked -p garnet-memory --all-targets -- -D warnings   exit 0
cargo +1.95.0 doc --locked -p garnet-memory --no-deps          exit 0
cargo +1.95.0 fmt --all -- --check                             exit 0
git diff --check efd4f6b..5cd1136                              clean
cargo +1.95.0 run --locked -p garnet-memory --example lane2c_teardown_probe --release -- working-clear 256   runs plainly (item 7: PASS)
```

The `+1.95.0` pin is a seat requirement, not a repo defect: this Air's
default rustc is 1.94.1, below the workspace MSRV (the same seat artifact
documented in Lane 1 Verdict 11 Leg 6).

## Blocker candidate 1 — correctness of the incoming-edge accounting: NO BLOCK

`garnet-memory-v0.3/src/cycle.rs` was read in full at the product head, and
the exact `efd4f6b..5cd1136` diff was compared line by line.

### Q-A: Can a cycle that was previously collected now survive? **No.**

The old buffering predicate was
`!live.contains(&id) && counts.get(&id) > 0`, where
`counts(id) = roots(id) + (incoming active-ARC edges of id)` by construction
of `reference_counts()`. The new predicate short-circuits on
`incoming_arc_edges == 0` before the live scan. Case analysis over all
reachable states at decision time:

| state | old decision | new decision |
|---|---|---|
| `roots > 0`, any incoming | reject (id ∈ live) | reject (gate or live) |
| `roots == 0`, `incoming == 0` | reject (`counts == 0`) | reject (O(1) gate) |
| `roots == 0`, `incoming > 0` | `!live` | `!live` (gate passes) |

The decisions are **equivalent state-by-state**, not merely similar — the
gate rejects exactly the states the old `counts > 0` conjunct rejected,
because `counts ≥ incoming` and a rooted node is always in `live`. Collection
itself is untouched: `collect_buffered_cycles` still filters buffered
candidates with the full `reference_counts()` and live set at flush time, and
the trial-deletion pass (`mark_gray` / `scan_candidate` / `scan_black` /
`collect_white`) is byte-identical. Cycle membership implies at least one
incoming ARC edge, so a genuine cycle candidate can never hit the O(1) gate.

Tests that foreclose survival regressions, all passing on this seat:

- `root_buffer_release_collects_cycle_when_threshold_is_reached`
  (tests/cycle.rs:204) — releases the last root of a two-node cycle through
  the buffered path; the candidate has `roots == 0, incoming == 1`, passes
  the gate, and the whole cycle is collected with exact finalization order.
- `allocator_edge_decrement_buffers_newly_unreachable_cycle`
  (tests/cycle.rs:282) — the `remove_edge_to_buffer` decrement path; the
  detached cycle is collected.
- `buffered_collection_scans_only_buffered_roots` (tests/cycle.rs:229) —
  buffered flush collects the cycle while leaving unrelated garbage alone.
- `releasing_the_last_root_makes_cycle_collectable` (tests/cycle.rs:146) —
  the full-scan path, which does not use the counter at all.
- `isolated_root_release_never_enters_candidate_buffer` (src/cycle.rs unit
  test) — the O(1) rejection side.

One genuine gap: no test drives a **self**-cycle through the *buffered* path
(`trial_deletion_collects_self_cycles` uses the full scan). The composition
is covered by the equivalence proof (a self-edge increments the counter to 1,
so the gate passes), but the direct test is missing — recorded as finding F3,
LOW, non-blocking.

### Q-B: Is the `checked_sub().expect(...)` panic reachable? **No.**

The panic requires removing an ARC→ARC edge that exists in `edges` while the
target's counter is 0. Exhaustive enumeration of every mutation of
`CycleNode::edges` in the crate (the struct is private to `cycle.rs`; no
other module can touch it): `insert` occurs only in `add_edge`, paired with
`+1` exactly when the insert is new and both endpoints are ARC; `remove`
occurs only in `remove_edge_recorded`, paired with the checked `−1` under
the same predicate; `retain`/`clear` occur only inside
`collect_candidate_cycles`, which is immediately followed by
`rebuild_incoming_arc_edges()` recomputing every counter from the surviving
edge sets. `CycleAllocationMode` is fixed at node creation with no mutator,
so the ARC-edge predicate is stable between an edge's insert and its remove.
Collected nodes cannot gain or lose edges through the public API (every path
goes through `ensure_active`), and all error paths return before mutating.
Therefore counter == true incoming-edge count is an invariant, and the
`expect()` is an unreachable defensive assertion, not a reachable crash. If
it ever fired it would mean the invariant itself was broken by future code —
fail-stop is the correct behavior for a determinism fixture at that point,
because continuing would silently mis-collect. (Style note, non-blocking:
this reachability argument belongs in a comment on `remove_edge_recorded`.)

### Q-C: Is drop ordering semantically unchanged for observers? **Yes.**

`finalization_order` is produced by `collect_white`'s postorder over
`BTreeSet`-ordered children — byte-identical before and after the change.
The candidate sets feeding it are decision-equivalent (Q-A), and the only
new operation inside collection, `rebuild_incoming_arc_edges()`, runs after
edge cleanup and before report construction and mutates counters only —
no field of `CycleCollectReport` can differ. Asserted by
`finalization_order_follows_collect_white_postorder` (tests/cycle.rs:172)
and the order assertions inside the buffered-path tests.

Request items 5–6 close on the same evidence: duplicate add and missing
remove are guarded by the `BTreeSet` insert/remove booleans (unit test
`incoming_arc_edge_count_tracks_unique_add_and_remove`); safe-affine nodes
never count (`is_arc_edge` requires both endpoints managed;
`safe_mode_allocations_are_not_cycle_collection_candidates`,
`cycle_aware_allocator_edge_removal_keeps_safe_affine_node_excluded`);
collected-edge cleanup rebuilds wholesale; rooted cycles are retained
(`rooted_cross_kind_cycle_is_retained`); isolated store roots take the O(1)
path while real ARC-peer candidates keep rooted-reachability plus trial
deletion (`WorkingStore::clear`/`Drop` → `alloc.release_root` →
`release_root_to_buffer` is the exact production path of the store fixture).

## BLOCKER B1 — no leak evidence for the cheaper teardown

The lane contains **zero** memcheck output. Every valgrind reference in the
branch is Callgrind-only. Demonstrating command (from the branch tip):

```sh
grep -rEil 'memcheck|leak-check' ops/lane2c/   # no matches
```

A teardown made 442x–1749x cheaper (working-clear 1024:
804,658,305 Ir → 460,159 Ir) by skipping per-element graph scans must be
shown to still free what it freed. Callgrind Ir counts prove cost; they
prove nothing about allocation disposition. Required cure, on the
implementer's measurement guest (WSL2, where `valgrind-3.22.0` already
exists — this fanless macOS/arm64 seat cannot run valgrind at all):

```sh
# at base efd4f6b (before) and product head 5cd1136 (after), same three cases at 1024:
cargo build --locked -p garnet-memory --example lane2c_teardown_probe --release
valgrind --tool=memcheck --leak-check=full --error-exitcode=99 \
  target/release/examples/lane2c_teardown_probe working-clear 1024
valgrind --tool=memcheck --leak-check=full --error-exitcode=99 \
  target/release/examples/lane2c_teardown_probe episodic-drop 1024
valgrind --tool=memcheck --leak-check=full --error-exitcode=99 \
  target/release/examples/lane2c_teardown_probe semantic-drop 1024
```

The six outputs (three cases × before/after) must land under
`ops/lane2c/evidence/memcheck/{before,after}/` with entries in
`MANIFEST.sha256` and verification wired into `verify_evidence.py` — the
same hash discipline as the Callgrind profiles. The acceptance bar is: zero
definitely/indirectly-lost bytes in both trees, or an explicit accounted
explanation of any delta. Until then, U-08's cure is demonstrated for cost
and unproven for correctness of disposition. **This blocker requires no
change to the product tree; it is evidence-only.**

## QUESTION (not a blocker) — the 89.8s quiet window

The record claims all nine base profiles, all nine product profiles, and the
4/4 stress set ran inside `2026-07-29T08:34:09.807Z → 08:35:39.594Z`
(89.79s). This seat **cannot reproduce that wall time**: valgrind does not
exist on macOS/arm64, and request item 9's quiet-machine law excludes
measurement on the Air regardless. What the committed artifacts support:
the window endpoints are internally consistent (OneDrive stop stamps precede
the window, restore stamps follow it; the stress capture cites the same
window), but only the two endpoints are recorded — there are no per-run
timestamps, so the 18-runs-plus-stress packing inside the window rests on
the implementer's record alone. Per the mandated note: **Callgrind Ir counts
are load-independent, so a window shortfall would NOT invalidate the 18/18
reproduced instruction counts or the curve ratios; it would narrow only the
quiet-state claim** — the environmental assertion about what else ran during
measurement. The record already binds this correctly on its own:
`wall_clock_is_claim_evidence: false` and `wall_clock_claim: "none"`, so no
repository claim leans on the window. Recommendation (non-blocking): future
measurement records should include per-run UTC stamps so the packing claim
is checkable.

## Findings

### F1 — Codex commit authorship (OBSERVATION, carried from Lane 1 Verdict 11 F1)

All three branch commits are authored/committed by
`OpenAI Codex <codex@openai.com>` rather than the fleet-fork identity. U-30's
hard prohibition (no `IDC-Trust-Review` authorship) holds; disclosure is
complete. Same ceremony ratification as Lane 1 F1 applies.

### F2 — `DOCTRINE.md` naming (MEDIUM, non-blocking, rename recommended)

`ops/lane2c/DOCTRINE.md` is implementer-authored inside its own lane. Its
content is right and correctly self-scoped ("proposed register entry",
"does not modify the frozen backlog", U-46 "global registry disposition
remains outside this implementer lane" — request item 10: PASS). But the
filename reads as repository doctrine, which an implementer lane cannot
issue. Recommend renaming to `ops/lane2c/PROPOSED-DOCTRINE.md` (or
equivalent) in a records commit, with the binding placement rule landing in
the governance surface via Repair #3. Until the rename, readers must rely on
the file's own disclaimer line rather than its name.

### F3 — no buffered self-cycle test (LOW, non-blocking)

`trial_deletion_collects_self_cycles` exercises the full-scan path only. Add
a test that releases the last root of a self-edged node through
`release_root_to_buffer` and asserts collection, closing the one composition
the suite does not pin directly.

## Scope and not-verified

- The 18/18 profile hashes and instruction counts were independently
  confirmed before this round and were not re-derived here (per instruction);
  this seat did re-run the deterministic verifier over the committed bundle
  (`verify_evidence.py --gate`, exit 0).
- No performance measurement was run on this seat (quiet-machine law;
  fanless; no valgrind on macOS/arm64). No timing claim is made.
- Request item 11 holds: no wall-clock, production-ARC, review, merge, or
  launch claim appears in the artifacts (`claim_boundary` fields plus a
  clean keyword sweep of the lane records).
- This reviewer modified no implementation code and performed no PR,
  approval, merge, tag, release, or acceptance action. The only writes are
  this verdict and one journal heartbeat line, both `ops/lane2c/**` records.
- The implementer did not author this file, satisfying the request's
  verdict-authorship rule.

## Consequence

**BLOCKED on B1 only.** The graph-accounting change is correct by proof and
by test on this seat; the curve evidence stands; the record's claim
boundaries are honest. When the six memcheck captures land green under the
lane's hash discipline, nothing in this verdict requires re-opening the
correctness analysis — B1 is evidence-only, and the product head
`5cd113617acd35307bb028463833a8da2bbd6ad2` (tree `85faad1d…`) is the exact
boundary any subsequent approval should bind.

## Reviewer stdout summary

Cross-family Lane 2C Verdict 01 (Claude Fable 5, Anthropic, MacBook Air; the
implementer is Codex GPT-5-based on the NUC) answers the correctness
challenge in the fix's favor — the O(1) incoming-edge gate is
decision-equivalent to the old predicate state-by-state, previously
collected cycles cannot survive (foreclosed by the named buffered-path
tests), the `checked_sub().expect()` is unreachable through the public API
by pairing-invariant enumeration, and finalization order is byte-identically
produced — but returns **BLOCKED** on exactly one numbered blocker: the lane
contains zero memcheck evidence, so the 442x–1749x cheaper teardown is
proven for cost and unproven for freeing behavior; six
`valgrind --tool=memcheck --leak-check=full` captures (three cases at 1024,
before and after) must land under the lane's hash discipline on the WSL2
measurement guest. The 89.8s quiet-window claim cannot be reproduced on this
seat and rests on recorded endpoints only; per the mandated note, any
shortfall would narrow the quiet-state claim, not the load-independent Ir
counts, and the record already declines to use wall clock as claim evidence.
`DOCTRINE.md` should be renamed to state it is proposed (F2), and a buffered
self-cycle test is recommended (F3). All request gates, tests, clippy, doc,
fmt, and the plain example run are green on this seat at the exact product
head.
