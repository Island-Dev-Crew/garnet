# WV-6 integration re-acceptance — Review Verdict 02: B1/F3/F5 cure confirmation

request: `ops/wv6-reaccept/review/02-request.md` (addendum)
reviewer: Claude Code on **Claude Fable 5** (`claude-fable-5`, Anthropic),
  `Pulses-MacBook-Air.local` (Darwin 25.6.0, arm64 Apple M5; fanless — no
  timing claims; U-52 honored: no worktrees)
implementer_identity_as_found: OpenAI Codex, GPT-5-based, on the NUC —
  cross-family separation holds
reviewed_tip: `8ae41b6f9660ae0f098d2137f14a1a89397fcfe5` — the unique
  one-commit successor of this seat's Verdict 01 commit `23bc563`
swept_at: `2026-08-11T14:52:04Z` claim; Tuesday morning America/Chicago —
  Sabbath fence not armed; push not held
verdict: **APPROVE at `8ae41b6` — B1, F3, and F5 are discharged; F2 stands
  for Jon's governance ruling; the frozen candidate is untouched.**

## Confirmations, each recomputed by this seat

1. **Ancestry closes exactly.** `35ddc22` (Verdict 01's reviewed tip) →
   `23bc563` (this seat's verdict) → `8ae41b6` (one Codex cure commit).
   The delta is exactly six paths under the record/proof allowlist —
   the registrations edit, the request-02 addendum, and the two transcript
   relocations (four paths uncollapsed, two `R100` entries with rename
   detection) — zero product paths.
2. **B1 discharged.** Both transcripts moved `R100` **byte-identical** —
   blob OIDs unchanged across the move (`1dd43c9e…`, `2cb5bfc3…`), contents
   rehashing to exactly `e48a97d8…` and `064dd519…` — into
   `proofs/windows/launch-verification/wv6-transcripts/`, inside the U-25
   evidence fence. `garnet_text_byte_policy_status.py --gate` now exits 0
   at the tip.
3. **The sibling-path deviation is ruled CORRECT.** Verdict 01 named the
   acceptance-bundle directory "or a sibling transcripts/ directory there";
   the implementer chose the sibling because the bundle is manifest-closed
   — and the verifier's own code proves the reasoning:
   `garnet_wv_acceptance_status.py` enumerates the evidence root with
   `rglob("*")`, so any file added inside the bundle would enter the
   accepted manifest's exact-set check and force re-emission of an
   already-accepted producer record. Under this seat's no-rewrite doctrine
   an acceptance record does not move; the sibling placement preserves the
   bundle byte-for-byte — verified OID-identical across the cure — while
   landing the bytes in the fence. Correct application, explicitly ruled.
4. **The drift table is accurate, including its account of this seat.**
   Recomputed: reviewed tip `35ddc22` = `fd96e6d9…/1606` (the accepted
   baseline); this seat's verdict commit `23bc563` = `d7a08be5…/1607` —
   the drift began with this reviewer's own digest-included record, exactly
   as the addendum discloses; cure tip `8ae41b6` = `49e5686c…/1608`,
   correctly not embedded in its own request (no-self-SHA). The WV-6
   verifier at the tip is `partial`, 5/5 checks, with the **sole** finding
   being that digest mismatch; the Minimum Shelf gate likewise `partial`
   on only the same drift — recorded, not silenced. At the frozen head
   `410ff11` the verifier remains **accepted 5/5** and the frozen pair
   `fd96e6d910180f5e33999fbd693ea211e336389a13535930d89b2a870ff54727 / 1606`
   is byte-undisturbed. This is lawful U-35 head-versus-tip record drift.
5. **F3 discharged.** The registrations index now carries the full 64-hex
   base-transcript hash (the 61-hex truncation is gone — zero matches) and
   corrected encoding labels (UTF-16 LE rows now labeled as such).
6. **F5 discharged.** U-52 is REGISTERED as an active procedural constraint
   (worktree admissibility) and U-53 is PROPOSED — DEFERRED (gate
   diagnostics swallow underlying messages), both discoverable in the
   registrations record and journal.

## Standing

- **F2 stands** for Jon's governance ruling: the v2 rolling gate's
  post-review walk structurally cannot green this merge topology; no record
  on this branch can cure it, and this seat does not fix gates.
- F4 (non-self-attesting final transcript) and F6 (cosmetics) remain
  recorded, non-blocking, routed as Verdict 01 disposed them.

## Consequence

**APPROVE at exact tip `8ae41b6f9660ae0f098d2137f14a1a89397fcfe5`.** The
WV-6 re-acceptance ceremony stands verified end-to-end at frozen candidate
`410ff1182cdcefcec9fe046d1346205d8522ec9d` / `fd96e6d9…/1606`, with the
record drift above it disclosed and classified. Approval authorizes only
Jon's merge decision — with F2's CI limitation before him — and no tag,
release, or launch promotion. This seat fixed nothing.

## Reviewer stdout summary

Confirmation Verdict 02 (Claude Fable 5, Anthropic; implementer Codex on
the NUC) APPROVES at `8ae41b6`: the cure is the unique successor of this
seat's verdict commit; both transcripts relocated R100 byte-identical into
the evidence fence with the text-byte gate green at the tip; the
sibling-path deviation is ruled correct — the verifier's `rglob` bundle
enumeration proves the suggested in-bundle placement would have rewritten
an accepted record; the drift table recomputes exactly, including the
honest attribution that this reviewer's own verdict commit began the
record drift; WV-6 remains accepted 5/5 at the frozen head with the tip
partial on record drift alone; F3's hash and encoding corrections and
F5's U-52/U-53 registrations verified. B1/F3/F5 discharged; F2 stands for
Jon's governance ruling.
