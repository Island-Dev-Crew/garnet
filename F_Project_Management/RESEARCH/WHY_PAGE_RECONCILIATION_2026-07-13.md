# /why page reconciliation — 2026-07-13

**Status:** corpus-of-record. This note reconciles the July 8–11 cowork audit
stream (Reports 01/01R, STATE v2–v4, the "62-day CRA crisis" retraction, and
the 2026-07-13 re-verification report Jon supplied) against what actually
exists in this repository, so no corpus-first agent inherits the corrected
errors. Verified findings below were re-checked against primary sources on
2026-07-13 from this repo's lane.

## The headline determination

**The audited /why content upgrade never merged into this repository.**
No ref, branch, or commit in `Island-Dev-Crew/garnet` history contains the
upgraded threat card (Shai-Hulud/Megalodon), the Veracode citation, the
insurance card (Klaimee/Testudo/Armilla/Munich Re HSB/ISO), the
language-leaders (Ronacher) card, the FDA/PCCP quote, or the
"honest account" → "bounded/calibrated" vocabulary swap. The cowork stream's
"PR 479" is **not** this repo's PR #479 (which is the S114 CI gate-wiring PR,
merged by Jon 2026-07-13) — the numbering collision is coincidental.

The live `docs/why.html` is the **pre-upgrade** state fenced by the S114
mission: published #459/#460, upgraded once by #471 (the two bounded
`enforced:` claims), byte-frozen through mission phase P2, and pinned by the
claim fixture (`scripts/garnet_capability_scope_status.py`, CI-wired in #479).

Consequences for the re-verification report's checklist:

| Checklist item | In-repo status |
|---|---|
| 1. Ronacher card Path 1 vs Path 2 | **No card shipped.** No live misattribution. Corrections recorded below for the eventual upgrade PR. |
| 2. eBuilder alongside Veracode | Page cites **eBuilder only** — the original state. Neither the retracted swap nor the additive fix ever landed. Add Veracode *additively* when the upgrade ships. |
| 3. FDA quote de-quote/re-source | **No FDA quote on the page.** The in-repo research corpus was already clean: `GARNET_REASSESSMENT_2026-06-11.md` sources list attributes the envelope-membership phrasing to "McDermott+ and Ballard Spahr analyses". |
| 4. "successor" → "days later" | **Megalodon appears nowhere in-repo.** Wording rule recorded below. |
| 5. Vocabulary swaps landed? | **Not landed** — `honest account` remains in the meta description (line 8) and byline (line 212), because the upgrade never shipped. Truth-Lock does not forbid it; swap rides the upgrade PR if Jon wants it. |
| 6. Shelf-scope re-ratification | **Zero in-repo commits in the July 8–11 window** touched shelf scope (W_REBUILD/, spec, or shelf-related paths). The shelf gate remains `manual-deferred` in the launch reporter. Nothing was committed under the invented crisis; nothing to re-ratify in-repo. |

## Corpus corrections (verified 2026-07-13, primary sources)

Recorded so the eventual upgrade PR starts from verified copy:

1. **Ronacher is two posts, never one.** Verified directly against
   lucumr.pocoo.org on 2026-07-13:
   - *"Agentic Coding Recommendations"*, **June 12, 2025**
     (`/2025/6/12/agentic-coding/`) contains, verbatim including the typo:
     > "Hiding permission checks in another file or some config file will
     > amost guarantee you that the AI will forget to add permission checks
     > in when adding new routes."
   - *"A Language For Agents"*, **February 9, 2026**
     (`/2026/2/9/a-language-for-agents/`) contains the effect-marker sketch,
     verbatim: `fn issue(sub: UserId, scopes: []Scope) -> Token needs { time, rng }`
     — and does **not** contain the permission-checks sentence.
   - Rule: cite both posts (the problem named June 2025; the fix sketched
     Feb 2026 — a stronger two-point card), quote verbatim with links,
     bracket any typo cleanup as `a[l]most`. Never fuse the two posts.
2. **FDA/PCCP envelope phrasing** ("…when it only implements modifications
   contemplated by the relevant PCCP") is **McDermott's summary** of the FDA
   guidance, not confirmed verbatim regulator text. De-quote into indirect
   speech, attribute to McDermott's analysis, or quote the FDA PDF directly.
3. **Megalodon lineage:** CSA/The Hacker News lean TeamPCP, but OX Security
   states there is **no threat-intel or code-analysis evidence** connecting
   Megalodon to the Shai-Hulud crew. Write "days later, the Megalodon
   campaign pushed…", never "its successor".
4. **Figures (per the 2026-07-13 re-verification):** Shai-Hulud expanded to
   ~172 packages across 403 malicious versions within 48 hours (npm + PyPI),
   with stolen OIDC tokens minted through Sigstore into valid SLSA Build L3
   attestations; Megalodon: 5,718 commits across 5,561 repositories in a
   six-hour window on May 18, 2026 — CISA-corroborated (alert of May 28,
   2026). Any "figures not independently audited" caveat now *understates*;
   cite the CISA alert.
5. **Veracode Spring 2026:** only 55% of generation tasks produce secure code
   (45% introduce a known flaw) while syntax correctness exceeds 95% — cite
   **additively alongside** the existing eBuilder 2026 review, per 01R.

## Known blemish on the live page (pre-existing, queued)

The threat card's source line (docs/why.html:287) ends with the editorial
instruction "— link primary sources in the published page", which shipped
as-is in #459/#460. Queue the actual primary-source links with the upgrade
PR rather than patching the line in isolation.

## Rules for the eventual /why upgrade PR

- Build every card from primary sources; no copy inherited from the cowork
  stream without re-verification (this episode's core lesson — a "quote
  verified" line that was never verified).
- The claim fixture holds the boundary automatically: exactly **two**
  `enforced:` claims, no forbidden universal-enforcement phrasing — a
  content upgrade must not add a third or soften the fences paragraph.
- `docs/why.html` is in the trust-kernel rolling-review trigger set (#478):
  the PR needs a review companion (this note's family) or a
  `Trust-Kernel-Review:` trailer.
- `/why` is not service-worker precached — corrections ship immediately.

## Audit trail

- Cowork stream: Reports 01 → 01R (retraction of eBuilder swap + card cut),
  STATE v2→v4 (false negatives self-corrected), CRA-crisis retraction (v4
  restored trim-the-shelf as an option, not a mandate).
- Jon's 2026-07-13 re-verification report (pasted into the mission session):
  verified Shai-Hulud/Megalodon/Veracode/Zero/insurance-card facts; raised
  Flags 1–3; this note re-verified Flag 1 against both Ronacher posts
  directly and mapped all six checklist items to in-repo reality.
- S114 mission context: `ops/mission/state.json` (phases P0–P5),
  `F_Project_Management/LAUNCH/S114_ACCEPTANCE.json` condition #3 (preserve
  /why wording; never expand claims because of S114).
