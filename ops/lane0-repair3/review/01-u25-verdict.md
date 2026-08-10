# Lane 0 Repair 3 — Review Verdict 01: U-25

request: `ops/lane0-repair3/review/01-u25-request.md` (the request names this
  file `01-u25-verdict.md`; the lane claim named `01-verdict.md` — this file
  follows the request's own destination)
reviewer: Claude Code on **Claude Fable 5** (`claude-fable-5`, Anthropic)
reviewer_machine: `Pulses-MacBook-Air.local`; Darwin 25.5.0; arm64 (Apple M5);
  fanless — functional and byte-level claims only, no timing claims
implementer_identity_as_found: OpenAI Codex, GPT-5-based runtime (exact
  version unavailable), on `Hughs-MacBook-Pro.local`, macOS 26.5, arm64 —
  recorded consistently in the request, `U25_SCOPE.json`, and the lane journal;
  cross-family separation holds for this review
branch: `mission/l0-repair3` (fetched from the fork by explicit refspec;
  zero `refs/pull/*` in the clone)
reviewed_base: `efd4f6bae8b3afaba74594e57944b2548142aeae` (verified still
  exact `origin/main` before any other action)
implementation_head: `be89de21684c84bb0aaae8382906864a48637d20`
  (tree `1faeed401996985d9b6d8412225ab706783c3355` — both reproduced)
reviewed_head: `5e5a24c917fab7a31f061cfa99609f72de746baf`
  (tree `44403069f2284dea6e298e4752e1a9e044f154d7` — both reproduced)
lineage: two commits, linear, single-parent, merge-base exact `origin/main`;
  both authored/committed `OpenAI Codex <codex@openai.com>` (own-seat
  identity per the ceremony authorship ruling); `IDC-Trust-Review` absent
swept_at: `2026-07-31T09:39:16Z` boot; Friday morning America/Chicago —
  Sabbath fence not armed at any point during this review; push not held
verdict: **BLOCKED on exactly one blocker (B1 — undisclosed standing-gate
  movement; records-only cure). The U-25 cure itself is correct, complete,
  and verified at byte level; every number the request asserts was
  independently recomputed and none was accepted on faith.**

## Provenance discipline

Every item below is marked **INDEPENDENTLY FOUND** (first surfaced by this
seat this round) or **INHERITED-AND-CONFIRMED** (asserted by the request or
lane claim, then reproduced by this seat's own commands). Markings record
where this seat got each item, not who else may hold it.

## Recomputed assertions (all INHERITED-AND-CONFIRMED unless noted)

1. **CR census, by bytes, over committed blobs** — never the worktree, never
   `git ls-files --eol`. Method: `git ls-tree -r -z <rev>` piped through
   `git cat-file --batch`, counting `\r\n` and bare `\r` over every blob's
   full bytes (script preserved at the review seat; both trees ~3,410 blobs).
   - Branch tree `5e5a24c9`: exactly **6** CR-bearing text blobs, all
     CRLF-only, **0 lone-CR** — all six under
     `proofs/validation/s114-review/windows-20260628-lane2/` (byte-exact
     evidence, excluded by policy). Verified as claimed.
   - Base tree `efd4f6ba`: **10** CR-bearing text blobs by bytes (9 CRLF-only
     + 1 lone-CR-only) while `git ls-files --eol` reports only **9** (i/crlf).
     Verified as claimed. The file `--eol` structurally cannot see is
     `ops/lane2b/state-of-the-union.html` (blob `68a818b8`, 21 bare CR, zero
     CRLF, no NUL): git's `convert_is_binary()` counts any bare CR as binary,
     bucketing the file into `i/-text` alongside real binaries — it never
     appears in any EOL class. (Root-cause mechanism: INDEPENDENTLY FOUND;
     the 10-vs-9 numbers: INHERITED-AND-CONFIRMED.) At head the same path is
     LF-only (blob `193b1b0b`, 0 CR): fixed by regeneration, not deletion.
   - Scope note: over ALL blobs (no text heuristic) the census is 140 (base)
     / 136 (head) CR-bearing blobs; the extra 130 are NUL-bearing binaries
     (PDFs, media, wasm, .db), identical path sets in both trees. The
     6/10/9 numbers hold exactly under git's own text heuristic.
2. **Sealed S114 proofs untouched.** The entire
   `windows-20260628-lane2` bundle is byte-identical between base and head —
   identical tree object, all 161 tracked blobs, empty diff — which subsumes
   the six-member requirement. Additionally all six `U25_SCOPE.json` SHA-256
   values were recomputed from the actual blob bytes at BOTH revisions:
   12/12 match. `.gitattributes` precedence verified: the four new
   root-anchored `text eol=lf` pins cannot capture `proofs/**`;
   `proofs/** -text` still wins by last-match (`git check-attr` confirms
   `text: unset` on all six sealed paths).
3. **Checker exclusions vs `.gitattributes`.** The checker excludes
   `proofs/` by prefix and `ops/**` paths with `evidence` as an intermediate
   directory. This agrees with the `-text` fences in every load-bearing
   direction, with one precise, safe-direction divergence recorded as F2
   below. The exclusion cannot hide non-evidence `ops/**` paths: `evidence`
   must be an intermediate component (`parts[1:-1]`), so `ops/lane2c/foo.txt`
   or `ops/evidence-notes/x` are in scope.
4. **No repository-wide count anywhere.** Read line-by-line: the checker has
   no total, no expected-N, no length assertion against a constant; every
   numeric literal is plumbing (40-char SHA validation, 30s git timeout, 124
   timeout rc, exit codes, JSON indent). `paths != sorted(set(paths))` is an
   ordering/duplication tripwire, not a count. The tests likewise pin no
   count. Enumeration only — U-45's point holds, and the docstring states
   the rationale. FINDINGS.md registers U-45 as **PROPOSED**, not accepted,
   and openly supersedes the wrong "eleven files" premise with the
   exact-tree ten-path enumeration (6 sealed + 3 CRLF + 1 lone-CR = 10,
   matching my census exactly).
5. **Renderer rejects CR on INPUT as well as output.** Both of its only two
   inputs go through `readLfText()` which fail-closes on any `\r`
   (state.json at the parse site, journal.md in `journalTail`), and the
   rendered OUTPUT is guarded by `if (html.includes("\r")) fail(...)` before
   `writeFileSync`. At base there were no CR guards at all; both sides are
   new in this diff. The committed HTML regenerates byte-identically except
   the producer-written timestamp (`new Date().toISOString()` in the
   renderer). The new byte test (`test_garnet_lane2b_sotu_bytes.py`) passes
   and forecloses CR-input, CR-output, and CRLF-in-committed-HTML.
6. **RED/GREEN reproduction.** Running the head checker with my own hands:
   at base — exactly the four-path RED (`.dogfood/windows-audit-goal.json`,
   `.dogfood/windows-core-audit.json`,
   `D_Executive_and_Presentation/garnet-website.html`,
   `ops/lane2b/state-of-the-union.html`), gate exit 1; at `be89de2` and at
   `5e5a24c9` — PASS, empty violations, gate exit 0. Consumers of the three
   normalized assets verified semantic-or-absent (the .dogfood JSONs are
   parsed, never byte-hashed; the website HTML has zero programmatic
   readers) — the request's semantic-consumers claim survives an
   independent sweep. The trust-path digest the request quotes
   (`sha256:7e3ba611…`) was independently recomputed from git objects and
   matches.

## Gates run (the full set, not the request's subset)

All 47 `scripts/garnet_*_status.py` gates advertising `--gate`, plus the
shelf smoke gate, the WV-6 acceptance reporter, the new text-byte gate at
three revisions, `cargo +1.95.0 fmt --all -- --check`, `git diff --check`
over the range, both new test files, and the full isolated Python battery at
base AND head (pinned venv: PyYAML 6.0.3, jsonschema 4.26.0; symmetric
no-build state). Results: 38 gates exit 0; the failures classify as — three
focused-cargo gates red only under the seat's sub-MSRV default toolchain
(green under `+1.95.0`, the standing seat artifact), launch-readiness red as
the standing HOLD design, trust-kernel rolling gate red as REVIEW REQUIRED
(correct fail-closed: this branch adds three trust-path scripts and no
review record exists until this verdict), workflow-action-integrity green
under the pinned venv (system python lacks PyYAML), **and the Minimum Shelf
smoke gate red at head — which is blocker B1 below**. Battery: base 1,130
tests / 5 failures; head 1,140 / 6. The five shared failures cancel (three
toolchain + two pre-existing repo-state: adoption-surface pointer,
release-assets sha256 line — the latter two are standing at BASE too,
INDEPENDENTLY FOUND as a correction to this seat's own three-failure
expectation). Predecessor-only set: empty. Successor-only set: exactly the
WV-6 acceptance test — see B1.

## BLOCKER B1 — undisclosed standing-gate movement (INDEPENDENTLY FOUND; records-only cure)

The request's diffstat understates the blast radius of `.gitattributes`, and
the packet is silent about two standing surfaces this branch flips red:

```sh
python3 -I scripts/smoke_garnet_minimum_shelf.py --gate        # exit 1 at 5e5a24c9
# findings: product digest cd9c080f…/1553 != pinned ea38d354…/1544 (path count
# 1544→1553), and ".gitattributes SHA-256 does not match the reviewed artifact"
# (head blob b2a14050… vs pin b8b22a96… at scripts/smoke_garnet_minimum_shelf.py:51)

<venv>/python -I scripts/test_garnet_wv_acceptance_status.py   # FAILED at head
# test_current_repository_tracks_wv6_acceptance_and_wv7_pending: 'partial' != 'accepted'
```

Cause: this branch changes 14 digest-included paths (the three normalized
assets, `.gitattributes`, `AGENTS.md`, the new checker + two test files, and
six `ops/lane0-repair3/**` records — `ops/lane0-repair3/` is not an excluded
prefix), moving the product pair from the pinned `ea38d354…/1544` to
`cd9c080f…/1553`, and changes the `.gitattributes` blob whose SHA-256 the
shelf reporter pins per-file. Both failures are correct fail-closed tripwire
behavior — the pins are doing their job — and the U-25 code needs no change.
But the packet does not disclose that merging it turns the Minimum Shelf gate
and the Python battery red on main until a separate, review-gated rebind and
WV-6 re-acceptance land (the exact sequence Lane 1 slice 5 ran, and which
Lane 1 disclosed in its request). An approval issued on this packet as
written would hand Jon a merge that reddens main's standing gates without
that consequence on the record. Nothing crosses a gate on claims alone — and
nothing should cross one on silence either.

Cure (records only; no product-tree change): amend the lane record
(request, `FINDINGS.md`, or journal) to (a) disclose the expected-red
shelf/WV-6/battery state at this head with the exact mismatching pairs, and
(b) name the successor step that cures it — a freeze/rebind slice updating
the four candidate constants AND the `.gitattributes` per-file pin, plus the
NUC WV-6 re-acceptance, each under its own review. With that disclosure
committed, this blocker lifts without re-review of the U-25 change itself.

## Findings (non-blocking)

### F1 — the checker's one real false-green channel (INDEPENDENTLY FOUND; MEDIUM, latent)

`git grep -I` consults the WORKTREE's `.gitattributes` for `diff`/`binary`
attributes even when scanning an exact commit: a future committed — or
merely uncommitted — `<path> -diff` line silently removes that path from the
scan (empirically confirmed in a fixture repo on git 2.50.1). No live
exposure at `5e5a24c9` (the committed `.gitattributes` has no diff/binary
lines, and gates run on clean checkouts), but this is the one genuine
hiding vector the request's check #2 asks about. Cure for a future slice:
pin `GIT_ATTR_SOURCE=<commit>` (git ≥ 2.40) in the checker's git env, or
assert the absence of `-diff`/`binary` attribute lines.

### F2 — exclusion-set divergence, safe direction (INDEPENDENTLY FOUND; LOW)

A blob literally named `evidence` under `ops/**` (e.g. `ops/x/evidence` as a
file) is `-text` per `.gitattributes` (`ops/**/evidence -text`) but is NOT
excluded by the checker (`evidence` must be an intermediate component). The
checker is stricter than the fences — it would fail RED, never false-green —
so the sets do not agree exactly, but the disagreement direction is safe. No
such blob exists at head.

### F3 — pre-existing mojibake in the renderer (INDEPENDENTLY FOUND; LOW, not this slice's defect)

`render-sotu.mjs` head line 109 contains a double-encoded em dash
(`â€”`) in the mission-complete cold-start string, present at base too, and
it propagates into the committed HTML. Not a CR/U-25 issue; recommend a
follow-up regeneration after fixing the source string.

### F4 — sealed-evidence attribute residue and manifest verification footnote (INDEPENDENTLY FOUND; INFO)

Three sealed bundle paths retain an inert `eol: lf` attribute from the
earlier `proofs/**/*.json` lines (`text` is unset, so no conversion occurs —
empirically confirmed by hashing a checked-out copy against its sealed
SHA-256). And the bundle's own `MANIFEST.sha256` uses CRLF terminators, so a
naive `shasum -c` fails on `filename\r`; verification requires stripping CR
from the manifest stream only, after which all 160 entries pass.

### F5 — evidence files record `ref: HEAD` (INDEPENDENTLY FOUND; INFO)

`05-u25-scope-red.txt` / `06-u25-green.txt` embed `ref: HEAD` rather than
the SHA; the commit/tree fields pin them exactly, and regeneration matches
byte-for-byte apart from that field's source. Cosmetic.

## Scope, and what was not verified

- This seat modified no implementation code and performed no fix, PR,
  approval, merge, or acceptance action; the only writes are this verdict
  and one journal line, both `ops/lane0-repair3/**`.
- The implementer did not author this verdict (request seat rule holds).
- No timing claims; no measurement on this fanless seat.
- The `windows-audit` JSON ledgers' SEMANTIC content equality
  (beyond line-ending normalization) was verified structurally: JSON parse
  of base and head blobs yields equal objects for both ledgers, and the
  website HTML diff is line-ending-only (byte diff after CRLF→LF mapping is
  empty). [Method: parse-and-compare / normalize-and-compare on committed
  blobs.]

## Consequence

**BLOCKED on B1 only — a records-only disclosure cure.** The U-25 cure
itself is verified at byte level: exact ten-path RED scope honestly
recorded, four-path violation set cured, six sealed blobs untouched
(12/12 hash matches), enumeration-only checker with no count contract,
fail-closed renderer on both input and output, and a clean differential
apart from the disclosed-by-me WV-6 tripwire. When the disclosure lands,
approval follows without re-review of the product change; the rebind and
WV-6 re-acceptance remain separate, review-gated successor steps binding
implementation head `be89de21684c84bb0aaae8382906864a48637d20` (tree
`1faeed40…`).

## Reviewer stdout summary

Cross-family Lane 0 Repair #3 Verdict 01 (Claude Fable 5, Anthropic, MacBook
Air; implementer Codex GPT-5-based on Hughs-MacBook-Pro) verifies every
number in the U-25 packet by independent recomputation — byte census 6/0 on
the branch tree and 10-vs-9 at base with the `--eol` blindness root-caused
to git's bare-CR binary heuristic, whole-bundle sealed-proof identity with
12/12 recomputed hashes, exclusion parity with one safe-direction
divergence, a genuinely count-free enumerating checker, and a renderer that
fail-closes on CR at both input and output — and returns **BLOCKED on one
independently-found blocker**: the packet does not disclose that its 14
digest-included changes flip the Minimum Shelf gate (digest `cd9c080f…/1553`
vs pin `ea38d354…/1544`, plus the `.gitattributes` per-file pin) and the
WV-6 battery test red at head, so approval as written would redden main's
standing gates silently; the cure is a records-only disclosure naming the
rebind/re-acceptance successor steps. Five non-blocking findings ride along,
including one latent false-green channel in the checker (`git grep`
worktree-attribute trust; pin `GIT_ATTR_SOURCE`) — independently found, as
were the mojibake, the attribute residue, and the standing base battery
failures beyond this seat's own prior expectation. Verdict authored under
this seat's own identity.
