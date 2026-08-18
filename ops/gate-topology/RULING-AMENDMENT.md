# Ruling Amendment — Slice 3 Record-Class Tolerance

Received in chat from Jon's merge-authority seat on 2026-08-11. Recorded
verbatim before implementing the substituted mechanism.

> RULING AMENDMENT — slice 3 mechanism replaced; intent unchanged:
>
> WITHDRAWN: the ops/wv6-reaccept/** provenance exclusion. The digest
> definition does not change; fd96e6d9…/1606 at frozen head 410ff11 remains
> the pair everywhere, forever.
>
> SUBSTITUTED: extend the U-35 record-class tolerance. Locate Phase 0's actual
> tip-vs-head handling — the walk by which the WV verifier and shelf reporter
> tolerate post-acceptance drift confined to record-class paths (the shelf
> reporter's own PARTIAL text names this: "does not tolerate U-35 tip drift").
> Extend its record-class prefix set to cover ops/wv6-reaccept/** (and the
> W_TRUST record/verdict paths if absent). Acceptance semantics: GREEN iff
> (frozen head's pair == recorded pair) AND (diff frozen..tip ⊆ record-class).
>
> TESTS, red-first where new: at 162b96a — verifier green, shelf reporter
> deterministic check green, frozen pair reported unchanged at fd96e6d9…/1606 ·
> mutation case: one non-record byte in the drift → hard red · your slice-3 RED
> capture stands as the exhibit; supersede the trial-exclusion evidence with a
> note, never delete it.
>
> Slices 1–2 stand as committed. Then: W_TRUST record, request 01 for the Air,
> push, close-out.
