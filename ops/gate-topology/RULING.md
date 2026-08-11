MERGE-AUTHORITY RULING — F2 gate amendment + provenance exclusion
Jon Isaac, Island Development Crew · 2026-08-11 · issued in-chat, recorded here

ACCEPTED: one linear governance mini-lane, mission/gate-topology, off
efd4f6bae8b3afaba74594e57944b2548142aeae, curing both standing findings at
their shared root:
 (a) v2 rolling-gate walk amendment — when the post-review walk meets a merge
     commit, accept a parent outside the reviewed lineage IF AND ONLY IF the
     trust-kernel byte-set and digest at that merge equal those at
     reviewed_head — the equality independently verified by Claude Fable 5 at
     35ddc22 (wv6 Verdict 01, F2). Inequality remains a hard red. No other
     semantics change.
 (b) Add ops/wv6-reaccept/** to the provenance exclusions, by the same
     mechanism ops/lane1/** was added for Phase 0 — curing the disclosed
     U-35 record-class drift (1606→1607→1608) that holds the WV-6 verifier
     and Minimum Shelf reporter at PARTIAL.
Ceremony class: trust-kernel — RED fixture reproducing the topology findings
at 8ae41b6 before any cure; GREEN after; mutation case stays red; own
canonical W_TRUST review record; independent review by Claude Fable 5; then
merged into mission/wv6-reaccept as the third merge, after which the tip pair
is expected to collapse to fd96e6d910180f5e33999fbd693ea211e336389a13535930
d89b2a870ff54727 / 1606 and all gates become reachable-green.
Implementer: Codex on Hughs-MacBook-Pro. Jon-only actions unchanged:
approvals, merges, tags, token minting. F2's finding provenance: Claude
Fable 5, INDEPENDENTLY FOUND, wv6 Verdict 01.

## Confirmed lane claim

LANE CLAIM: mission/gate-topology · Hughs-MacBook-Pro · Codex ·
2026-08-11T17:16:12Z. Jon confirmed the claim in chat. This is an advisory
coordination record, not a mechanical lock.

The merge authority later corrected the RED target to the record-only tip
`162b96adb0a91c5fdc8c189dc2fcdd22ce996cab`; its sole parent is
`8ae41b6f9660ae0f098d2137f14a1a89397fcfe5`, its author is Claude Fable 5,
and its two-path delta is confined to `ops/wv6-reaccept/**`.
