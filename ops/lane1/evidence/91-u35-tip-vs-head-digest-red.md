# U-35 RED — review artifacts move the product digest (tip vs pinned head)

- Recorded: 2026-07-27 (UTC ~17:45Z), at branch tip `ea1dcf634e7f2ba2c3673334670cfb8155a87f91`
- Implementer: Claude Code Fable 5 — fleet-fork identity
- Status: RED recorded BEFORE any cure; the cure (if authorized) is a reviewed change to `scripts/garnet_content_provenance.py` and is NOT implemented here.

## The structural defect

`FROZEN_MUTABLE_PREFIXES` in `scripts/garnet_content_provenance.py` is:

```
ops/lane2b/  ·  proofs/  ·  F_Project_Management/W_TRUST/   (+ the reporter path)
```

`ops/lane1/` is absent, so Lane 1's own review artifacts — requests, verdicts,
addenda, BLOCKED, journal, this very file — are **inside** the product digest.
Every review round therefore moves the digest and invalidates the WV pin.

## Demonstration (computed via `tracked_content_digest`, the frozen construction)

| tree | digest | paths |
|------|--------|-------|
| rebind head `f1ec569` (= pinned EXPECTED) | `5d3e7f727b56dbeb22d570c0b61a3b45b5e0b8df4e3b6305be896f9b5ed42b43` | 1581 |
| branch tip `ea1dcf6` | `9f483ce917e657a682965b0abbd208a91abfa6b0a75d53771ed56f46e6e008dc` | 1582 |

Divergence caused by exactly three paths, all review bookkeeping, zero product
bytes: `ops/lane1/BLOCKED.md`, `ops/lane1/journal.md`,
`ops/lane1/review/05-request.md`.

Prior demonstration of the same mechanism, same session: `99c3f270…/1578` (at
`72ae024`) → `5d3e7f72…/1581` (at `f1ec569`'s base) from three review commits
(`04-request`, U-33 addendum, `04-verdict`).

Note the self-reference: committing THIS evidence file and the 06-request moves
the digest again (1582 → 1584). Each documentation round worsens the condition
it documents — the treadmill in miniature.

## Consequence at merge (why this blocks the NUC)

Jon's squash lands the branch **TIP** tree — including every review artifact.
The WV pin is bound at the rebind head (`5d3e7f72…/1581`). Landed main's digest
will therefore NOT match the pin, and WV-6 will read **PARTIAL on main** — a
regression of the exact metric Phase 0 exists to restore. Running the NUC
acceptance before this is ruled would burn a native-platform run on evidence
that cannot survive the merge.
