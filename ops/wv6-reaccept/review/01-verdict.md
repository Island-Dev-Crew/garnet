# WV-6 integration re-acceptance — Review Verdict 01

request: `ops/wv6-reaccept/review/01-request.md`
reviewer: Claude Code on **Claude Fable 5** (`claude-fable-5`, Anthropic)
reviewer_machine: `Pulses-MacBook-Air.local`; Darwin 25.6.0 (updated since the
  Repair 3 rounds' 25.5.0); arm64 (Apple M5); fanless — functional and
  byte-level claims only; no timing claims; no native-Windows execution and
  no Wasm rebuild are possible on this seat, and neither was attempted
implementer_identity_as_found: OpenAI Codex, GPT-5-based agent (version not
  exposed by its harness), on `NUCBOX_M2PRO_S` (Windows 11 Pro 10.0.26200) —
  cross-family separation holds
branch: `mission/wv6-reaccept` (fork; explicit refspec; zero `refs/pull/*`)
reviewed_head: `35ddc22809d647ae6637e280db560efa3cc537ed`
  (tree `3bdd5dcfb21b4373a9bb44c72a772d92012f7c23` — both reproduced)
frozen_commit: `410ff1182cdcefcec9fe046d1346205d8522ec9d`
  (tree `57ce26ae1ab8d24609180486bc5fce6179f37957` — both reproduced)
frozen_pair: `fd96e6d910180f5e33999fbd693ea211e336389a13535930d89b2a870ff54727 / 1606`
boot: U-52 honored — fresh clone, `autocrlf false`, zero worktrees for the
  entire review; other revisions read via git plumbing only
swept_at: `2026-08-11T00:21:58Z` boot, Monday evening America/Chicago — the
  Sabbath fence never armed; the push was not held
subject: everything above the two merge commits; the constituent branches
  carry my Lane 2C Verdicts 01/02 and Repair 3 Verdicts 01/02/03 and were not
  re-reviewed
verdict: **BLOCKED on exactly one blocker (B1 — the reviewed head violates
  the repository's own U-25 text-byte policy; relocation-only cure on
  already-authorized surfaces). Every mandated recomputation reproduces; the
  WV-6 acceptance chain itself is sound and verified end-to-end.**

## Provenance discipline

Every item is marked INDEPENDENTLY FOUND or INHERITED-AND-CONFIRMED by where
this seat got it. Nothing was inherited from the chat seat; every number
below was recomputed here or by this seat's own verification legs.

## Recomputed and confirmed (INHERITED-AND-CONFIRMED throughout)

1. **Both merges reproduce conflict-free.** `git merge-tree --write-tree`
   over the named parents reproduces the committed merge trees byte-exactly:
   `efd4f6b + 1bc64c4 → 2196efcd…` (= `814c0bc^{tree}`) and
   `814c0bc + 9e0f7a4 → 897c808f…` (= `2bcc6dd^{tree}`). No hand resolution
   exists to hide.
2. **The pair chain, two agreeing methods at ten revisions** (repository
   provenance function AND an independent raw reconstruction):
   diagnostic `43d68dc3…/1604` at the merged head; `2cb25d0b…/1605` at the
   request commit through `f6d0239`; the two authorized producer movements
   (`9e118486…/1605` after the Wasm regeneration, `7b67b8e5…/1605` after the
   browser proof); frozen `fd96e6d9…/1606` at `410ff11` — held identically at
   the rebind `31e3515`, the acceptance `ec8ce8f`, and the tip `35ddc22`.
   Post-freeze confinement verified: every changed path above `410ff11` is
   the reporter self-path, `proofs/**`, or `F_Project_Management/W_TRUST/**`.
3. **The rebinds are pin-movement only.** Rebind-1 (`176004a`): exactly the
   four candidate constants plus the `.gitattributes` `EXPECTED_FILE_SHA256`
   entry, with four matching `PROOF.json` mirrors. Rebind-2 (`31e3515`):
   exactly the four candidates to the frozen values, four mirrors moving in
   step. The reporter has no logic change anywhere in `efd4f6b..35ddc22`;
   its exclusion predicate is imported from `garnet_content_provenance.py`
   (blob byte-identical across the range), never duplicated; all six
   `EXPECTED_FILE_SHA256` pins recompute exactly from head blobs; historical
   anchors `1e669217…/1527` byte-identical across the range. The superseded
   intermediate pair — recorded here in full per this seat's own
   Verdict 02 precedent —
   `2cb25d0b47f55c9dd987bf69fc8a97dade5a4767ef0beda0abc8616808dddc0c / 1605`
   is preserved un-erased in history (rebind-1's target, rebind-2's old
   side) and its supersession is narrated in U-56 Exhibit One; only its
   digits were history-only until this verdict.
4. **The acceptance chain closes, by this seat's own execution.**
   `WV_ACCEPTANCE.json` carries exactly the verifier's twelve-key contract,
   binds `reviewedHeadSha 410ff11… / reviewedTreeSha 57ce26ae… /
   productContentSha256 fd96e6d9…`, `state evidence_complete`, `platform
   windows`, `jonOnlyActionsPerformed []`, and the five contract checks all
   `passed`. `python3 -I scripts/garnet_wv_acceptance_status.py --wv WV-6
   --gate` exits 0: **state `accepted`, 5/5, findings []** — the state that
   has been `partial` since Lane 1 Phase 0. The shelf gate exits 0 with
   `ok:true, findings:[]` at the frozen pair. All five sibling evidence
   artifacts recompute to their manifest hashes; the bundle directory is
   byte-complete with no extra files. The acceptance record is
   producer-shaped field-for-field against `--emit-wv6`'s own code — no
   field exists that the emitter cannot produce.
5. **The Wasm regeneration is genuine and bound.** `provenance.json` binds
   source census `c36f0e45…` over exactly 176 sorted unique inputs including
   `garnet-memory-v0.3/examples/lane2c_teardown_probe.rs`; this seat
   re-executed the census from the producer's own `INPUT_ROOTS` logic in
   `scripts/build_playground_wasm.py` and reproduced **both** the binding
   176-input aggregate and the falsified manual 175-input `556ea1c6…` —
   confirming the U-56 story from the code, not the record. Committed
   artifact bytes hash exactly as recorded (`garnet_wasm.js` unchanged at
   `bf725099…/6581`; `garnet_wasm_bg.wasm` changed to `60887f72…/2217102`;
   `provenance.json` `156dccd2…/8486`); the js-unchanged/wasm-changed split
   is consistent with the manifest-verified dependency chain
   `garnet-wasm → garnet-interp → garnet-memory` compiling `cycle.rs`;
   `garnet_wasm_readiness.py --gate` passes at head with
   `browser_ready true`. A byte-level rebuild was not attempted: the
   recorded toolchain is Windows-native (win32-x64 esbuild), impossible
   here — bindings and bytes were verified instead, stated per this seat's
   machine-honesty limits.
6. **Battery discipline (U-54) verified by name, within this seat's
   limits.** This Air cannot execute native-Windows batteries; the
   transcripts' hashes and internal consistency were verified instead. The
   base transcript hashes to
   `aacddc271ed737db4d8c44b0f7b91e6788b0ca47a62c72efd4b88c747cf685ed` and
   its FAIL/ERROR set is EXACTLY the registered ten-name baseline (four
   errors, six failures, name-for-name). The final transcript hashes to
   `e48a97d8d62c46355db7346e764387e19f8c14a5cb59164e339e76b0dcbc5405` and
   its set is EXACTLY the same ten names — zero candidate-only names, totals
   lines consistent (1,130 → 1,141 tests = the eleven tests the merged
   branches added). The pre-cure transcript shows the ten plus the eight
   stale-pin failures, as the request's classification records.
7. **Registrations and custody.** Every hash-checkable claim in
   `WV6_REACCEPTANCE_REGISTRATIONS_2026-08-10.md` and the lane journal
   reproduces: all six preserved transcripts, the acceptance-bundle
   outputs, the Wasm triple, the browser proof and screenshot, the Studio
   lockfile, both custody exhibits, the U-56 aggregates, and the live tip
   pair. U-54/U-55/U-56 registration claims verified at the SHA; provenance
   markings in the record are consistent with the request's account. The
   pre-rebind versus post-rebind bundle hash differences are coherent
   two-commit custody, not drift.
8. **Trust-kernel delta and the canonical record (request check 5).** The
   branch's trust delta versus base is exactly the three Repair 3 policy
   scripts (the shelf reporter is not a trust-kernel path — prefix and file
   list checked). Exactly one canonical record is in the diff: the merged
   `LANE0_REPAIR3_SLICE5.review.json`, whose `touched_paths` equal this
   branch's trust set and whose content digest matches this branch's
   recomputed digest. Check 5 is therefore **satisfied by the merged
   record**, with the gate-design limitation recorded as F2 below.
9. **Lineage.** Nine single-parent commits above the merges, all authored
   and committed `OpenAI Codex <codex@openai.com>`; the merge commits have
   exactly the stated parents; `IDC-Trust-Review` appears only as prose in
   seat-description text, never as an author, committer, or trailer.

## BLOCKER B1 — the reviewed head violates the U-25 text-byte policy (INDEPENDENTLY FOUND; relocation-only cure)

```sh
python3 -I scripts/garnet_text_byte_policy_status.py --gate       # exit 1 at 35ddc22
# violations:
#   F_Project_Management/W_TRUST/WV6_REACCEPTANCE_FINAL_FULL_PYTHON_BATTERY_2026-08-10.txt
#   F_Project_Management/W_TRUST/WV6_REACCEPTANCE_NATIVE_WINDOWS_2026-08-09.txt
python3 -I scripts/garnet_text_byte_policy_status.py --ref 410ff11 --gate   # exit 0
```

The tip commit `35ddc22` itself committed two genuine CRLF-bearing Windows
transcripts (1,793 and 64 CRLF pairs; one with a UTF-8 BOM, one ANSI)
under `F_Project_Management/W_TRUST/` — outside the `proofs/**` and
`ops/**/evidence/**` fences that the repository's own U-25 policy (built and
reviewed in Repair 3, by this seat) reserves for captured bytes. The gate is
right and the placement is wrong: these are quintessential captured evidence
bytes whose hashes the registrations record pins, so they must keep their
bytes and live inside a fence. The violation is undisclosed in the packet,
and merging as-is turns the standing text-byte gate red on main — the exact
class this seat blocked in Repair 3 Verdict 01.

Cure (relocation-only, on surfaces already authorized post-freeze, no
product movement): move the two transcripts byte-identically under a fenced
evidence path — the natural home is
`proofs/windows/launch-verification/wv6-minimum-shelf/` (or a sibling
`transcripts/` directory there) — and update the registrations index paths
in the same records commit. Both source and destination surfaces are
digest-excluded, so the frozen pair `fd96e6d9…/1606` is untouched: no
re-freeze, no re-rebind, no re-acceptance. The same commit should fix the
two records defects in F3 below. Nothing in the acceptance chain reopens.

## Findings (non-blocking)

### F2 — the v2 rolling gate cannot green this merge topology (INDEPENDENTLY FOUND; gate-design, route to governance)

At the head, `garnet_trust_kernel_review_status.py --gate` reports, beyond
the environmental transport finding, two branch-intrinsic graph findings:
the post-review walk from the merged record's `reviewed_head` (`c9e4aa7`)
crosses the integration merges, whose other-parent commits are outside the
reviewed lineage ("no reviewed-lineage parent for 0649d79…"; "post-review
trust touch in merge commit 2bcc6dd…") — even though the trust byte-set and
digest at `reviewed_head` and at the head are identical (verified). The v2
post-review walk was built for linear candidates and structurally cannot
green a review performed on a side branch that is then merged beside
parallel content. Adding a second WV6-specific record cannot cure this
(exactly-one-record law) and this seat does not fix gates. Disposition: a
governance-surface decision — either a reviewed gate amendment (accept a
merge parent whose trust snapshot equals the reviewed one) or Jon's merge
proceeding with the CI component red-by-limitation, documented. The CI
"agent documentation contracts" job will show this regardless of B1's cure.

### F3 — two records defects in the registrations index (INDEPENDENTLY FOUND; ride B1's cure commit)

(a) Line 148's transcript-index hash for the base battery is a 61-hex
truncation (`…ca47a62efd4b…`, dropping `c72`) of the correct 64-hex value
that appears correctly at line 48 of the same record, in the journal, and
in the request — a transcription typo, not a custody break (the true hash
recomputes from the committed file). (b) Two encoding labels are wrong:
`FINAL_ACCEPTANCE_BATTERY` is UTF-16 LE (FF FE BOM) labeled "UTF-8";
`FINAL_FULL_PYTHON_BATTERY` is ANSI/Windows-1252 labeled "UTF-8". Hashes
for both rows are correct.

### F4 — the final battery transcript is not self-attesting (INDEPENDENTLY FOUND; LOW)

Unlike every other preserved transcript, `FINAL_FULL_PYTHON_BATTERY`
(`e48a97d8…`) carries no `head=`/`tree=`/machine/UTC header and no
BEGIN/END markers; its binding to the frozen boundary rests on the
registrations index and the differing capture path the records describe.
The ten-name content verification stands on the transcript's own body;
future ceremonies should require self-attesting headers on the binding
transcript.

### F5 — U-52 and U-53 are used but registered nowhere discoverable (INDEPENDENTLY FOUND; LOW)

The lane claim invokes U-52 (the no-worktree boot law, honored this round)
and the registrations sweep says "U-54 through U-60 unassigned", implying
U-52/U-53 are taken — yet neither appears in any tree of any org head or
any of the fork `mission/*` heads. Under the fleet's own U-50 discipline,
IDs in active use must be sweepable; register both where the allocator
doctrine lands.

### F6 — cosmetic (INDEPENDENTLY FOUND; INFO)

The shelf reporter's inherited `implementer` field still reads
"Codex GPT-5.6 Sol" while this lane's records state the version is not
exposed — a historical constant, not custody; the `ec8ce8f` re-emission
implies an out-of-band deletion of the prior bundle directory (the emitter
refuses to overwrite), with the resulting bytes fully producer-shaped; the
`1e669217…/1527` historical anchor's source objects are absent from this
clone, as squash-durable U-19 anchoring expects; WV-7 is `pending`
fail-closed, as designed.

## Scope and not-verified

- No native-Windows run, no Wasm rebuild, no timing claim on this seat —
  hashes, internal consistency, producer-shape, and re-executed census
  logic stand in, and are identified as such throughout.
- The constituent branches' content was not re-reviewed (carried by this
  seat's five prior verdicts); the two merge commits were verified as pure
  reproductions of their parents.
- This seat fixed nothing, performed no PR, approval, merge, tag, release,
  acceptance emission, or credential action; the only writes are this
  verdict and one journal line, both `ops/wv6-reaccept/**` — which are
  digest-included, so this verdict commit lawfully moves the branch pair
  above the tip while the frozen candidate `410ff11…` and its accepted
  evidence remain exactly bound; the U-35 head-versus-tip shape covers it.

## Consequence

**BLOCKED on B1 only — a byte-preserving relocation of two transcripts into
a fenced evidence path plus the F3 index corrections, all on surfaces
already authorized post-freeze, moving no product byte.** On its landing:
the WV-6 re-acceptance ceremony is verified end-to-end at frozen candidate
`410ff1182cdcefcec9fe046d1346205d8522ec9d` / pair `fd96e6d9…/1606` with the
verifier `accepted 5/5` by this seat's own run, and the record supports
Jon's merge decision with F2's gate-topology limitation disclosed. Approval
then authorizes only that decision — no tag, release, or launch promotion.

## Reviewer stdout summary

Cross-family WV-6 re-acceptance Verdict 01 (Claude Fable 5, Anthropic,
MacBook Air, U-52-clean boot; implementer Codex GPT-5-based on the NUC)
reproduces the entire ceremony with its own hands — both merges byte-exact
via merge-tree, the ten-revision pair chain by two agreeing methods through
diagnostic `43d68dc3…/1604`, superseded `2cb25d0b…/1605` (digits now on the
record), and frozen `fd96e6d9…/1606`; both rebinds pin-movement-only; the
WV-6 verifier run locally to `accepted 5/5, findings []` at the frozen
boundary; the Wasm regeneration's 176-input census re-executed from the
producer's own code, reproducing both the binding `c36f0e45…` and the
falsified manual `556ea1c6…`; and the ten-name U-54 baseline matched
name-for-name in both hash-verified transcripts — and returns **BLOCKED on
one independently-found blocker**: the tip commit violates the repository's
own U-25 text-byte policy by committing two CRLF Windows transcripts
outside the evidence fences, turning the standing gate red at the reviewed
head, undisclosed; the cure is a byte-preserving relocation on
already-authorized surfaces that moves no product byte and reopens nothing.
Findings: the v2 rolling gate structurally cannot green this merge topology
(governance routing), a truncated hash and two wrong encoding labels in the
registrations index, a non-self-attesting final transcript, and
used-but-unregistered U-52/U-53. Native-Windows execution and Wasm rebuild
were impossible on this seat and are claimed nowhere. Verdict authored
under this seat's own identity.
