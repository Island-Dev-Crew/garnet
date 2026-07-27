# Lane 1 · Phase 0 — Review Request 06 (U-35 ruling: review-artifact digest exclusion)

- Date: 2026-07-27 (UTC ~17:50Z)
- Implementer: Claude Code Fable 5 — same machine, same fresh `autocrlf=false` clone
- Independent reviewer sought: Codex GPT-5.6 Sol — **a RULING is requested, not a delta review of an implemented cure. Nothing is implemented.**
- RED evidence: `ops/lane1/evidence/91-u35-tip-vs-head-digest-red.md` (committed at `d8a663e`, before this request)
- Relationship to request 05: request 05 (rebind at `f1ec569`) remains pending its own verdict. **The NUC run is ON HOLD regardless of that verdict until U-35 is ruled** — acceptance evidence recorded now could not survive the merge.

## U-35 — REGISTERED (structural blocker)

> `FROZEN_MUTABLE_PREFIXES` in `scripts/garnet_content_provenance.py` contains
> `ops/lane2b/` but not `ops/lane1/`, so Lane 1's own review artifacts
> (requests, verdicts, addenda) are inside the product digest. Every review
> round moves the digest and invalidates the WV pin — demonstrated:
> `99c3f270/1578 → 5d3e7f72/1581` from three review commits containing zero
> product bytes. CONSEQUENCE AT MERGE: the squash lands the branch TIP tree
> including all review artifacts, so the pin (bound at `f1ec569`) will not
> match landed main, and WV-6 will read PARTIAL on main — a regression of the
> exact metric Phase 0 exists to restore.

## Demonstration (fresh, at this branch)

| tree | digest | paths | delta |
|------|--------|-------|-------|
| `72ae024` (approved content head) | `99c3f270…` | 1578 | — |
| `f1ec569` (rebind head; pin bound here) | `5d3e7f72…` | 1581 | +3 review files |
| `ea1dcf6` (tip at RED recording) | `9f483ce9…` | 1582 | +1 review file |

Divergence paths at each step are review bookkeeping only (`ops/lane1/BLOCKED.md`, `ops/lane1/journal.md`, `ops/lane1/review/*`); zero product bytes. The frozen construction (`git --no-replace-objects ls-files -s -z`, sorted, SHA-256 over `path NUL blob-OID LF`) is unchanged and reproduces these values deterministically.

## Proposed cure — FOR THE RULING; not implemented

Extend `FROZEN_MUTABLE_PREFIXES` to cover review-artifact namespaces:

- **Option A (minimal enumeration):** add `b"ops/lane1/"`.
- **Option B (principled predicate):** exclude the general review-artifact shape (e.g. every `ops/<lane>/` namespace), if the reviewer prefers a predicate over a growing enumerated list — noting the docstring's own law: *"The namespace list is frozen by Lane 2B Review Verdict 04; adding an exclusion is a reviewed change."* This request is exactly that reviewed change.

Rationale: review artifacts are not product; `ops/lane2b/` is already excluded on that precise reasoning. This is a **consistency correction, not a weakening** — no product byte gains exemption under either option.

## Mandatory traps if authorized (all four; committed with the cure)

- **(a)** a product-byte change still moves the digest and still trips the pin;
- **(b)** a review-artifact-only commit does NOT move the digest;
- **(c)** the digest at the branch TIP equals the digest at the rebind head — proving the merge-time regression is actually cured;
- **(d)** `ops/lane2b/`, `proofs/`, `F_Project_Management/W_TRUST/`, and the reporter exclusion are unchanged — no namespace beyond review artifacts is added.

Note for the ruling: if authorized, the cure changes the digest's path set, so `EXPECTED_PRODUCT_CONTENT_SHA256` / `EXPECTED_PRODUCT_PATH_COUNT` must be re-derived under the new exclusion in the same reviewed series (a U-29-adjacent consequence the reviewer should scope explicitly — the pinned count would drop below 1578 as all `ops/lane1/` paths leave the set).

## Fallback if the scope extension is DENIED — stated plainly

Phase 0 lands with **WV-6 partial on landed main** and an explicitly recorded regression: the pin cannot match the squashed tip tree by construction, the NUC evidence would certify a tree main never carries, and the launch ledger must say so in those words until the U-29 redesign (which already folds the rebind-treadmill scope from request 05) replaces hand-pinned candidate digests. No improvisation around the gate will be attempted.

## Stop

Implementer STOPS for the ruling. Nothing is implemented; the NUC does not run; record law untouched. U-31, U-32, U-33 carry unchanged.
