# Landing Arc 6 register sweep — 2026-09-23 (status transitions after #551–#593)

Records lane, swept at `origin/main` = `0c6b901ead2578f424445ee6dcbe98e0353cea1c`
(`0c6b901e`, "Playground Phase 1: readable evidence, sharing, and offline browser
journeys (#593)").

This file appends status transitions and corrections. It rewrites no earlier row: the
2026-08-31 through 2026-09-04 sweeps keep their dated statuses as history, and a reader
takes the newest dated status for each id.

- Sweep seat: Claude Opus 5.5, records lane, macOS, worktree branched from `origin/main`.
- Sweep date: 2026-09-23.
- Family: every transition below is a same-family (Claude) reading of merged commits, PR
  metadata and committed review records. A transition records that a cure landed. It is
  not a re-review of the cure, and it adds no independence the landing record did not have.

## Collision sweep

- swept-at: 2026-09-23, after `git fetch --prune` on both remotes.
- source: 486 advertised refs — every `origin` and `fork` branch head except `fork/main`
  and the `HEAD` symrefs — resolving to 484 unique trees, plus every commit message
  reachable from those refs.
- pattern: `git grep -I -hoE 'U-[0-9]+([^0-9]|$)' <tree>`, then `grep -oE 'U-[0-9]+' | sort -u`,
  with the prose token `U-910` excluded.
- result **before**: `U-1`, `U-04` … `U-122` — **census 105**. `U-122` already occurs in
  two committed review records (see U-122 below) but has no register row. No id numbered
  above 122 occurs in any swept tree or commit message.
- result **after** this file: unchanged, **census 105**. This file writes the U-122 row and
  allocates no new id.

## Status transitions

Merge dates below are UTC, from the GitHub API.

| Id | Finding | Last dated status | Now | Landing |
|---|---|---|---|---|
| U-75 | Rolling gate stalls locally with no credential once a record and PR bind | open (`LANDING_ARC_REGISTER_SWEEP_2026-08-31.md:33`) | cured for the local traversal; residual open | `b2ac33a4` (#585, merged 2026-09-15): one batched `git cat-file` reader per repository. The CHANGELOG measures 5,162 git processes and about 60 s before, 38 and 0.62 s after. |
| U-83 | Four memory natives are caps-invisible | open (`LANDING_ARC_2_REGISTER_SWEEP_2026-09-01.md:42`, refined `LANDING_ARC_4_…:678`) | cured | `bec9410c` (#589, merged 2026-09-18): D-04 puts the four `memory::*` rows under the `mem` capability, entry-gated (ADR 0011). |
| U-91, U-92 | Laundering through function values; VM/interp trap-parity claim | open (`LANDING_ARC_4_…:38`, `:102`) | public-truth cure landed | `79ff6ffa` (#564, merged 2026-09-10), which re-created #562 and #553 (both closed unmerged). The limit itself stands: the checker builds no edge through a function value, as the scope document states. |
| U-93 | `diff-caps` skipped any `vendor/` directory | open until #555 merges (`LANDING_ARC_4_…:153`) | cured; residual open | `4ba75afb` (#555, merged 2026-09-11): only `.garnet/vendor` is skipped, `node_modules` is no longer skipped, and skips are disclosed by rule and count. |
| U-94, U-95 | `verify` compared 3 of 8 fields; signature covered a re-serialization | open (`LANDING_ARC_4_…:190`, `:216`) | cured | `b20869d0` (#554, merged 2026-09-04). |
| U-96, U-105 | Dogfood PR-body checker section and evidence rules; untimed Git subprocess | open (`LANDING_ARC_4_…:239`, `:425`) | cured | `452a0e21` (#551, merged 2026-09-05). |
| U-97 | WV acceptance reporter read evidence by path as translated text | open (`LANDING_ARC_4_…:270`) | cured | `64bb7bd3` (#552, merged 2026-09-11). |
| U-113 | The scope named two Declared primitives by identifiers that do not resolve | open (`LANDING_ARC_4_…:575`) | cured | `bec9410c` (#589): the bare `uuid::new_v4` and `uuid::new_v7` spellings leave `GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md`, which now names `std::uuid::new_v4` and `std::uuid::new_v7` (`:97`). Located with `git log -S` on the bare spelling over the scope document. |
| U-117 | Propagator missed primitives reached only through a cycle | closed in place by #589 (`LANDING_ARC_5_…`, U-117 row) | cured | `bec9410c` (#589). Listed here only for completeness; the arc-5 row was already updated in place. |
| U-118 | Seal subject digest was the shape-stable AST hash | "implemented in the T3 candidate; independent review and landing pending" (`LANDING_ARC_5_…:81`) | landed | `918c0aed` (#592, merged 2026-09-20). `seal/v2` binds LF-normalized source. |

### Residuals that stay open

- **U-75, credentialed path.** #585 cured the local object traversal. The transport
  timeout on the credentialed path, named in `LANDING_ARC_2_…:287-288` as "a second,
  separate cure", has not landed. #585 also leaves Windows pipe, kill and handle-release
  behaviour unverified; no Windows job runs the gate's tests.
- **U-93, generated trees.** `target`, `.git` and `.garnet-cache` are still skipped at any
  depth. The skips are disclosed by rule and count, not blocked. The `/2` provenance-by-version
  schema bump and adding the three files to the gate's trust set were left out of #555.

## Corrections

- **U-120 names the wrong landing PR.** `LANDING_ARC_5_…` says the reseal was "cured for
  0.8.2 in #550". #550 closed unmerged. The reseal landed in `040f8177` (#568, merged
  2026-09-11 UTC), which re-created #563, itself a re-creation of #550. The coupling U-120
  names still stands: the Minimum Shelf trust root is pinned to `CARGO_PKG_VERSION`, and
  #592 changed the seal format, so the #568 reseal recipe no longer applies as written.

## U-122 — a review record cannot be carried across a base update

- **Class:** governance property, documented so it is not worked around. It is the partner
  of U-121.
- **Family:** raised by Claude implementer seats in record lineage notes; same-family.
- **Statement:** a structured review record binds the base its PR was reviewed against. When
  `main` moves past that base, the record does not carry: the change is re-created on a fresh
  branch from the new `main` and recorded again. Two committed records cite this rule by id
  with no register row behind it:
  - `F_Project_Management/W_TRUST/RELEASE_0_8_2_BUMP_568.review.json` — "re-creates PR #563
    (itself re-creating #550) on a fresh branch from f0a4a675 after #551, #564, #566 and #567
    moved main past #563's record base d22a1c0 (U-122: a record cannot be carried across an
    update)".
  - `F_Project_Management/W_TRUST/U91_CAPABILITY_CLAIM_TRUTH_564.review.json` — "re-creates PR
    #562 (itself re-creating #553) on a fresh branch from 452a0e211f3d after #551 landed and
    pushed #562 BEHIND (U-122 …)".
- **Reproduce:** `git grep -n 'U-122' -- 'F_Project_Management/W_TRUST/*.review.json'`.
- **Status:** property, not a defect. As with U-121, the rolling-review contract should state
  it. U-121 itself is already stated there: `C_Language_Specification/GARNET_TRUST_KERNEL_ROLLING_REVIEW.md:155-157`
  says records "may not be modified, deleted, or type-changed".

## Process deviations recorded

- **#568, attempt-2 re-runs without the carrier.** CI run `34547010628` and
  Base-controlled trust run `34547009305` on `mission/version-bump-0.8.2-3` are attempt 2,
  triggered by `IslandDevCrew`. `AGENTS.md:381-390` makes the one-rerun exception
  ineligible for activation until its carrier exists and `r2_role_separation_v1` is
  executable and green. This records the deviation; #568's record is not edited.
- **Approve-then-re-run on #585, #589, #591, #592 and #593.** CI attempt 2 on each was
  triggered by `IDC-Trust-Review` (runs `35023225987`, `35380170456`, `35393141899`,
  `35478715760`, `35697335756`). The #585, #589, #591 and #593 records state the practice
  under their venue limits, and the #593 record says Jon accepted it on 2026-09-13. The
  two #592 records do not mention it. Whether `AGENTS.md` records it as a disclosed
  interim practice is the pending C7-46 ruling.
- **#575 has no rule-3 attribution line.** It was merged 2026-09-11T18:47:58Z by
  `IslandDevCrew`, and its merge commit, body, reviews and comments carry no attribution line.
  The merge actor stays unconfirmed until the maintainer states who merged it.

## T3 leftovers, cross-referenced

#592 shipped the T3 acceptance slice only. Its deferred scope maps to existing ids, so no
new id is allocated:

- runtime budgets (`@bounded`) → D-05;
- network listeners and per-host enforcement → D-03;
- converter output that parses → P1-19 (Launch Map item 27);
- a test for an edit that stays inside declared authority → carried in the CHANGELOG T3
  entry, which is now marked partial.

U-91's current status is the row above: the public-truth cure landed, and the function-value
limit is stated rather than cured.
