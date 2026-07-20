# Lane 2B Review Verdict 04 — truth pairing and squash-durable provenance

reviewer: Claude Fable 5 (chat seat, Jon-relay mode; commit verbatim)
reviewed_head: dcf6008fd4291baf719dc361a82f2062ea60bfd2
reviewed_tree: f3272b9610dba756bd414cafc825fd7462d7a294   request: 04-request.md
verdict: APPROVE-WITH-AUTHORIZATIONS (reporter remains red pending the cure below)

verified: head/tree reproduced; cure blob d10c665f… byte-identical to Binding 3;
verdict 03 committed verbatim; suite 6/6 on this machine; new assertions are
STRICTER than the removed ones (dual WV-6-accepted / WV-7-pending coverage,
strict equality on state/ok/counts/findings/returncode/payload). The removed
lines are exactly the Verdict-03-named defect. Differential parity claim
consistent with my run.

DECISION 1 — APPROVE. The Verdict-03 truth-surface blob and the W_TRUST
companion binding (efb370ae…) are accepted as bound.

DECISION 2 — AUTHORIZED WITH CONSTRAINTS, the content-bound design (option A):
  a. Shelf reporter: replace the a6f0da2 commit-diff with a deterministic
     content digest over the sorted (path, blob-OID) list of every tracked
     index path OUTSIDE the four named mutable namespaces (ops/lane2b/**,
     proofs/**, F\_Project\_Management/W\_TRUST/**, and the reporter's own path).
     That namespace list is FROZEN as-authorized: adding any exclusion later
     is a reviewed change, never a convenience. Digest construction must be
     documented in the reporter and reproducible by one git command sequence.
  b. Rebaseline the product digest at THIS reviewed tree f3272b96…, citing
     this verdict as authorization — that lawfully includes the authorized
     test blob. Any other product-byte change remains fail-closed.
  c. WV-6 acceptance: rebind to content/tree provenance + landed first-parent
     main check, mirroring the Lane 1 U-19 marker pattern exactly. On-branch:
     content proof must pass. Post-squash fresh main-only clone: must pass
     with zero pull refs.
  d. REQUIRED adversarial traps, all three, RED-recorded before cure:
     (1) any product blob change → RED; (2) absent branch commit a6f0da2 /
     e2820ce in a fresh main-only clone → still GREEN (durability trap);
     (3) mismatched evidence/content digest → RED.
  e. Evidence only through sanctioned producers; update W_TRUST bindings with
     exact new path/blob digests.

DECISION 3 — RESOLVED EXPLICITLY: the fallback (post-squash Jon-only rebind
PR with a red window on main) is DENIED. Main red-by-design, even briefly,
is U-04 thinking. The content-bound design exists precisely so a fresh
main-only clone is green at the squash instant. No implicit ceremony; no red
window.

DECISION 4 — CONFIRMED: before PR ceremony, the Air must double-run the final
reporter from two fresh checkouts (LF and default-Windows) proving
byte-identical verdict output, and return immutable Verdict 05. Chat-seat
verdicts (03, 04) cover design and bindings; the Air covers native execution.

next: implement per Decision 2, push, commit 05-request.md; Jon triggers the
Air sweep; on Verdict 05 APPROVE the lane proceeds to PR under the standing
ceremony with its W_TRUST companion.
