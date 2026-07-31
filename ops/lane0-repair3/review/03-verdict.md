# Lane 0 Repair 3 — Review Verdict 03: U-47 correction + U-51 registration

request: `ops/lane0-repair3/review/03-u47-u51-addendum.md`
reviewer: Claude Code on **Claude Fable 5** (`claude-fable-5`, Anthropic),
  `Pulses-MacBook-Air.local` (Darwin 25.5.0, arm64 Apple M5; fanless — no
  timing claims)
implementer_identity_as_found: OpenAI Codex, GPT-5-based, on
  Hughs-MacBook-Pro — cross-family separation holds
reviewed_head: `c02b1679d549474249784ca5c4b8618cfdb39217`
  (tree `fe9ae5a6d5866f80b1b72a9530d35b33a9b9ee51` — both reproduced)
swept_at: `2026-07-31T17:51:53Z` (Friday 12:51 America/Chicago — Sabbath
  fence not armed; push not held)
verdict: **APPROVE at exact head `c02b167` — Verdict 02's sole blocker is
  cured; confirmation round, five checks, all PASS.**

## The five checks (each by this seat's own hands)

1. **Unique successor.** `git rev-list --count 3a5f628..c02b167` = 1;
   `c02b167^` = `3a5f628`. PASS.
2. **Exact delta.** `git diff --name-status 3a5f628 c02b167` is exactly
   `M ops/lane0-repair3/FINDINGS.md`, `M ops/lane0-repair3/journal.md`,
   `A ops/lane0-repair3/review/03-u47-u51-addendum.md`. Nothing else. PASS.
3. **Checker untouched.** `scripts/garnet_text_byte_policy_status.py` is
   byte-identical to the artifact audited at `608bae4` (blob OID
   `960336bc…` at both revisions). PASS.
4. **U-47 marking cured.** The row now reads: *"First recorded in Verdict 01
   (`d99b1ca`) at 05:09 CDT, then inherited and confirmed by this
   implementer seat's Rust/Cargo 1.95.0 battery run; registration and
   Lane 3 routing are this seat's contribution."* Exactly the cure
   Verdict 02 required. PASS.
5. **U-51 registered honestly.** PROPOSED — DEFERRED, with same-family
   provenance stated in the row ("found by the Codex audit seat,
   same-family"); covers the `.git/info/attributes` and
   `core.attributesFile` ambient channels plus both Verdict 02 residuals
   (the Git 2.40 floor and the committed diff-family boundary); routed to a
   successor slice without widening F1 or moving the reviewed checker; and
   the row's fork-head sweep extends to the 460 non-main heads, adopting
   Verdict 02's scope caveat. PASS — and this seat additionally confirmed
   the row's central claim by execution: in a fixture repo the cured
   checker names a committed CRLF blob on a clean state and goes falsely
   green when `.git/info/attributes` carries `<path> -diff`, even under
   the `GIT_ATTR_SOURCE` binding (INHERITED-AND-CONFIRMED; the discovery
   is the Codex audit seat's). The registered channel is real, so the
   successor slice inherits a true statement of work.

## Consequence

**APPROVE.** With this head, Verdict 01's B1 (disclosure) and F1/F2
(checker cures) and Verdict 02's B1 (U-47 marking) are all discharged. The
lane's remaining open work is exactly what the register names: the
freeze/rebind + NUC WV-6 re-acceptance successors, and the U-51 successor
slice for the ambient-attribute channels, Git version floor, and
committed-attribute boundary. Approval authorizes only Jon's merge
decision; no acceptance, tag, release, rebind, or NUC action is performed
or authorized here. This seat fixed nothing.

## Reviewer stdout summary

Confirmation Verdict 03 (Claude Fable 5, Anthropic; implementer Codex on
Hughs-MacBook-Pro) APPROVES at exact head `c02b167`: the cure is the unique
successor of `3a5f628`, the delta is exactly the two records plus the
two-line addendum, the audited checker is blob-identical, the U-47 row now
carries the first-recorded-in-Verdict-01 marking, and U-51 is honestly
registered with same-family provenance — its ambient-attribute false-green
channel reproduced by execution on this seat before approval.
