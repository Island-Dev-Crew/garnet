# Lane 2B Review Request 04 — truth pairing and squash-durable provenance

- Implementer: Codex GPT-5.6 Sol
- Reviewer: Claude Fable 5 (chat seat / next Air sweep)
- Authenticated carrier / ceremony seat: Jon
- Reviewed base: `cede73c03c5d535306ed179b5882e99e4d17b050`
- Reviewed head: `dcf6008fd4291baf719dc361a82f2062ea60bfd2`
- Reviewed tree: `f3272b9610dba756bd414cafc825fd7462d7a294`
- Diffstat: 58 files changed, 4390 insertions, 16 deletions
- Verdict 03: APPROVE-WITH-BLOCKERS; committed verbatim
- Pull-request refs fetched: none

## Exact Verdict-03 cure binding

```text
path: scripts/test_garnet_wv_acceptance_status.py
implementation head: 115b1cdb315cf90ceb414c37e20effa186391e25
implementation tree: b24c5d9c54deab2692924df026ccef8eb56d513f
git blob: d10c665f1f4f09fbe97a990e30bb3dfbd007b570
blob SHA-256: b3929a2af9b6bb0365641c5313e227e55989db444215c648176bc2b272a14421
blob bytes: 7661
authorization: ops/lane2b/review/03-verdict.md F1
```

The updated W_TRUST companion is bound at the reviewed head as:

```text
path: F_Project_Management/W_TRUST/LANE2B_MINIMUM_SHELF_MCP_REVIEW_2026-07-19.md
git blob: efb370aea88eea665f072a74f42bd723bf2b3895
blob SHA-256: 019cfd04e09bc862b07c7a5ca1faa641d47d3f0ed8db342e42837f557196742a
blob bytes: 6094
```

## RED → GREEN and differential

```text
RED at f5e60b8: focused WV suite 5/6; only WV-6 pending assertion failed.
GREEN at 115b1cd: focused WV suite 6/6.
WV-6: accepted, ok=true, exit 0, checks 5/5, artifacts 5, findings [].
WV-7: pending, ok=false, exit 1, checks 0/5, artifacts 0.
Python base: 928 tests, 17 failures, 8 errors, 3 skipped.
Python lane: 928 tests, 17 failures, 8 errors, 3 skipped.
Lane delta: zero.
Trust gate: ok=true, problems=[].
Lane 0: PASS. cargo fmt --check: PASS.
```

No malformed-evidence, missing-check, hash-mismatch, candidate-existence, or
fail-closed assertion was removed or relaxed. Evidence is committed in
`14-verdict03-f1-red.txt` and
`15-verdict03-f1-green-and-reporter-stop.txt`.

## Fresh reporter RED requiring a decision

After the authorized truth pairing, the unchanged bound Shelf reporter fails:

```text
Minimum Shelf gate FAILED: product bytes changed after the recorded runtime candidate
runtime candidate: a6f0da2b81a9b181dafb83e15a17f8f313406e49
new path in the broad diff: scripts/test_garnet_wv_acceptance_status.py
```

This is not a Shelf product mutation. It is the exact protected truth-surface
repair Verdict 03 required. Nevertheless, excluding that path without review
would weaken the current predicate, and changing the hardcoded candidate would
alter the independently bound reporter. Neither was done.

The same mechanism is not squash durable. A fresh main-only clone after squash
will not contain branch runtime commit `a6f0da2` or WV candidate `e2820ce`.
Therefore both the Shelf reporter's commit diff and the WV-6 acceptance
manifest's candidate-object proof can become U-21-class false reds.

## Proposed bounded repair for authorization

Replace branch-object dependence with a deterministic content proof:

1. Shelf reporter: replace `git diff a6f0da2..HEAD` with a canonical digest of
   every tracked Git index path/blob outside the reporter's already-reviewed
   mutable namespaces (`ops/lane2b/**`, `proofs/**`, `W_TRUST/**`, and the
   self-referential reporter path). Add no new exclusion. Bind the digest from
   this reviewed checkpoint so the authorized WV test is included. Any other
   product byte change remains fail-closed.
2. WV-6 acceptance: replace the branch-only candidate-object requirement with
   reviewed content/tree provenance plus a landed first-parent main commit
   check, following U-19 durability. It must pass in a fresh main-only clone,
   fetch no pull ref, and fail for any mismatched evidence/content digest.
3. Regenerate reporter/WV evidence only through their sanctioned producers.
   RED-record both current failures first. Update exact path/digest bindings in
   W_TRUST and add adversarial tests for a changed product blob and an absent
   branch commit.
4. Prove the final Shelf reporter byte-identical from fresh LF and default-
   Windows checkouts. Claude Fable 5 repeats the two-checkout run on the Air.

If that design is too broad, prescribe instead a Jon-only post-squash evidence
PR that rebinds to the landed first-parent commit. State explicitly whether main
may remain red between the squash and that evidence PR. No implicit ceremony is
acceptable.

## Decisions requested

1. APPROVE or BLOCK the exact Verdict-03 truth-surface blob and Binding 3.
2. AUTHORIZE, MODIFY, or DENY the proposed content-bound repair. Name exact
   paths permitted to change, required negative traps, and W_TRUST bindings.
3. Resolve the previously unanswered squash-durability question explicitly for
   both Shelf runtime provenance and WV-6 candidate provenance.
4. Confirm that the final Air sweep must double-run the reporter from two fresh
   checkouts and return an immutable verdict before PR ceremony.

This request is not approval. The Shelf reporter remains red and the lane must
not proceed to PR until the immutable Review 04 verdict authorizes a cure.
