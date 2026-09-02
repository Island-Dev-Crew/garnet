# Landing Arc 3 register sweep — 2026-09-02 (autonomous run under delegation)

Records lane (L2), swept at `origin/main` = `76b6a34145dea9f29ee00a5abf456599e6da9685` after the day's merges. Every candidate below was
raised inside the 2026-09-02 autonomous run, confirmed by a seat of the opposite family (Codex via the
local wrapper, read-only, detached worktrees) or by the twelve-agent recompute workflow, and carries the
command that reproduces it. Ids allocated after sweeping every advertised head for collisions
(`git grep -hoE 'U-[0-9]+' <head> -- '*.md' '*.json' '*.txt' '*.py' | sort -u`; note `\b` is not a git-grep
token — anchor with `([^0-9]|$)`). The distinct token set ran U-04 … U-83 (census 65) at `fbd64bc5`;
main `76b6a341` counts 67 because the front-door review record (#545) carries two more tokens — `U-1`,
a prose token inside a quoted, corrected sentence (the U-60/U-61 class, not an allocation), and `U-90`,
an id that record put in circulation by naming a finding before this sweep — so U-90 is a **backfill**
here, not a fresh allocation. After this sweep the set runs U-04 … U-90 (census 73, the prose token
included).

## U-84 — SECURITY.md carried four stale present-tense claims on a public truth surface

- Substance: the supported-versions table named 0.4.2 as the current release (tags: v0.8.1, v0.8.0,
  v0.5.0, v0.4.2); the "136 security-specific tests" figure persisted although `docs/truth.json`
  records it has no trusted derivation; a `@mailbox(N)` "BoundedMail bypass" and an unqualified
  `@sandbox` escape were listed as vulnerability classes while both fences are declared-not-enforced
  (CLAUDE.md; enforcement scope); crypto crate versions lagged `Cargo.lock`.
- Raised by: Claude Fable 5.1 (records seat), reading `SECURITY.md` at `fbd64bc5`.
  Confirmed by: Codex (SECURITY review v1 — REJECT on the `@sandbox` bullet, which the v2 cure
  addressed; v2 verdict on record).
- Command: `git show fbd64bc5:SECURITY.md | sed -n '4,10p;57p;76p'`; `git show fbd64bc5:docs/truth.json | grep -A2 security_test_count`.
- Route: L5 public truth. Status: **fixed** — PR #544 merged as `f77a3ebd6d5890d0d46eb71633446b7275995956` (Codex v2 CONFIRM; content byte-identical after the U-74 rebase). Disposition: fixed.

## U-85 — Two action pins still declared a Node 20 runtime ahead of GitHub's 2026-09-23 removal

- Substance: `actions/setup-node` v4.4.0 (`49933ea…`, vscode-extension.yml) and
  `softprops/action-gh-release` v2.6.2 (`3bb1273…`, the tag-only release jobs of linux-packages.yml
  and vscode-extension.yml) declare `runs.using: node20`. GitHub's changelog (editor's note of
  2026-08-25) sets Node 20 removal at **2026-09-23**; the forced Node 24 default began 2026-06-16.
  The release-job pin is silent today only because those jobs skip on non-tag runs; it would first
  fail on the next release tag.
- Reviewer-error instance (U-70 class): the chat seat's assessment named the wrong date
  (September 16) and the wrong actions (`checkout@v4`, `upload-artifact@v4`); the only such
  annotation comes from GitHub's own `pages-build-deployment` workflow, which the repository cannot
  edit. Recomputed by the workflow verifier (C3) from the changelog text and the run annotations.
- Mechanics learned: `scripts/garnet_workflow_action_integrity_status.py` discards the whole pin
  manifest on any single-entry fault (a future-dated `resolved_at` reads as "invalid" and every pin
  then reads "absent"); manifest must be canonical JSON.
- Raised by: chat seat (partially wrong) → Claude Fable 5.1 (recomputed). Confirmed by: workflow
  verifier C3; Codex v1/v2/v3 (record on PR #543).
- Command: `git ls-remote --tags https://github.com/actions/setup-node | grep 49933ea`; `curl -sL https://raw.githubusercontent.com/softprops/action-gh-release/3bb12739c298aeb8a4eeaf626c5b8d85266b0e65/action.yml | grep -A1 '^runs:'`.
- Split on evidence: `build-vsix` is a required-context producer, so moving its `setup-node` pin
  moves the producer's semantic fingerprint in `.github/rulesets/required-context-producers.json`
  and the 31-fingerprint aggregate pinned in `scripts/garnet_github_governance_gate.py` (Jon's
  readback instrument) — that is its own ceremony. The release jobs are not required producers, so
  the `action-gh-release` pin moves without touching any fingerprint (evaluator: 31 bindings, zero
  problems). PR #543 therefore carries only the release-job pin, the registry re-point, the
  release-assets test pin, and the readiness test extended to cover `softprops/action-gh-release`
  (red-before against the base tree names all three v2 pins; green-after at the tip). `setup-node`
  v4.4.0 stays pinned for now; it runs forced on Node 24 today.
- Route: CI hardening (trust-kernel paths; human-merge-only). Status: cure PR #543 (branch `mission/ci-setup-node-v6-pin`, content `f6572fd3` at sweep time; the structured Codex record follows its final rebase onto this sweep's merge, since a rebase moves `reviewed_head`); Jon approves (IDC-Trust-Review) and merges. The `setup-node` producer ceremony is queued for Jon. Disposition: open until merged.

## U-86 — The site's front door was the June feature tour with a stale footer stamp

- Substance: `docs/index.html` at `fbd64bc5` carried the June skeleton (tagline "Rust Rigor. Ruby
  Velocity.", three pillars, a numbers wall, a 390 KB inline poster) and the footer "Site updated
  2026-06-07" although the file had nine later commits (latest #534, 2026-09-01). The live page and
  the repo file were byte-identical (sha256 `1515551d…`). The page did not link the CRA page.
- Raised by: chat seat (claim held; framing corrected by verifier C1). Confirmed by: workflow
  verifier C1 (byte comparison live vs main); Codex front-door review v1 (two blocking copy findings
  on the replacement, cured) and v2.
- Command: `git log --format='%h %ad %s' --date=short fbd64bc5 -- docs/index.html | head -3`; `curl -s https://garnet-lang.org/ | shasum -a 256`.
- Route: L5 public truth / R1 front door. Status: **fixed** — replaced by the evidence front door, PR #545 merged as `76b6a34145dea9f29ee00a5abf456599e6da9685` after Codex v1/v2 REJECT → v3 CONFIRM → CI caught two dropped gate surfaces (positioning themes; promo embed tokens) → v4 REJECT → v5 CONFIRM bound to `b36f613e`. Disposition: fixed.

## U-87 — CRA page rendered its U-32/U-33 citations as literal Markdown

- Substance: `docs/cra-article-14.html` line 95 carried `[U-32](url)` / `[U-33](url)` inside HTML,
  so browsers showed brackets and raw URLs. Cosmetic; the citations were present as text.
- Raised by: workflow verifier C2. Confirmed by: Codex (CONFIRM, bound to `9aa4893e`).
- Command: `git show fbd64bc5:docs/cra-article-14.html | sed -n '95p'`.
- Route: L5. Status: **fixed** — PR #542 merged as `6826ce8b80a3ddec287e7150dd16deff445d4635` (Codex CONFIRM bound to `9aa4893e`). Disposition: fixed.

## U-88 — The September release pack's prototypes carried fabricated evidence (pack errata)

- Substance: the front-door and union prototypes in the pack (outside the repository) showed a
  "sample capture" with `gate 2/5 · bounds enforced at runtime … pass` (bounds are declared, not
  enforced), `evidence bundle written` (no such flag), palette hex codes (`9b1b30`, `c9a227`,
  `d64550`) as seal and commit digests, `garnet seal --plugin` (no such flag), a **locked**
  Playground door while a committed WebAssembly playground (check/run/diff-caps) is live on main,
  and a meta description "the receipts are live". None of it shipped; recorded so no seat
  transcribes a pack line into a public surface. The real capture on the new front door was run
  from the binary built at `fbd64bc5`.
- Raised by: workflow extraction agent (claims-audit of the pack). Confirmed by: the binary run
  (`garnet check` exit 1; `diff-caps --machine` exit 1; `seal` UNSIGNED note) and Codex front-door
  review item 4 (capture verbatim).
- Command: `target/release/garnet --help | grep -E 'seal|diff-caps|build'`; `ls docs/playground/pkg`.
- Route: pack errata (Jon's copy). Status: open — the pack's own errata note is Jon's edit. Disposition: recorded.

## U-89 — A structured review record cannot precede its pull request

- Substance: the rolling-review record schema binds `pull_request_id`, `pull_request_number`,
  `head_repository` and `head_repository_id`; a record authored on an unpublished branch cannot be
  truthful, and the reviewing seat refused to write one twice (Node 24 cure, v1 and v2) until the
  branch was pushed and PR #543 existed. Ceremony order therefore is: content push → PR open →
  record commit → carrier approval → merge. The console's CONFIRM/RECORD blocks assume the PR
  exists; this makes the assumption explicit.
- Raised by: Codex (refusals, on record). Confirmed by: the record validator's required keys
  (`scripts/garnet_trust_kernel_review_status.py`; precedent records under `F_Project_Management/W_TRUST/`).
- Command: `python3 -c "import json;print(sorted(json.load(open('F_Project_Management/W_TRUST/landed/LANE1_GOVERNANCE_ACTIVATION.landed-review.json'))))"`.
- Route: doctrine (CONSOLE.md/LANES.md addendum, Jon's copy). Status: recorded. Disposition: doctrine.

## U-90 — The public promo embed was described as manifest-backed while its files are not hash-bound (backfill)

- Substance: the June `docs/index.html` promo caption said the embed carries "manifest-backed evidence".
  The promo lane's four evidence manifests verify, but they bind 60-second render files; the embedded
  `docs/assets/garnet-promo.{mp4,webm}` are 30-second files whose hashes appear in no current manifest.
  `scripts/garnet_promo_video_status.py` reaches `public_site_embed_present: true` through existence
  and token checks, never hash equality, so the reporter cannot catch the gap. The replacement front
  door's caption was reworded to what the manifests bind.
- Raised by: Codex (front-door review v4, finding 3). Confirmed by: the reporter's `_public_site_embed_passed`
  predicate (token checks only) read by the records seat.
- Command: `sed -n '/def _public_site_embed_passed/,/^def /p' scripts/garnet_promo_video_status.py`; `shasum -a 256 docs/assets/garnet-promo.mp4`.
- Route: promo lane / L5 public truth (reporter hardening: bind the embed by hash). Status: caption cured on the front door; reporter gap open. Disposition: open.

## Amendments without reallocation

- U-75: two-seat measurement now on record (#541); the L4 cure target is the per-object `git`
  subprocess traversal.
- U-70 (reviewer-seat error class): three instances added — the chat seat's Node 20 date and action
  names (see U-85); the records seat's initial front-door copy naming the post-approval
  re-evaluation as an operative act and the register as "U-1 … U-83" (Codex v1 REJECT, cured before
  push); the records seat's retired word transported inside the CRA review record (elided and marked
  before merge).
- U-82: the Base-controlled composite red reproduced on #540, #541, #542 in the registered shape
  ("rolling review v2 does not bind the exact base/candidate boundary", `candidate_policy_ok true`);
  verifier C10 confirms Base-controlled trust policy is not a required context of ruleset 18936562.

## Reconciliation

- Candidates processed: 6 new allocations (U-84 … U-89), 1 backfill (U-90, in circulation via the #545 record), 3 amendments.
- Census: 67 (main, incl. the `U-1` prose token) → 73 at the tip. Collision sweep: no id in
  U-84 … U-89 occurs on any advertised head before this file; U-90 occurs only in the #545 record.
- Every entry recomputed from a command the records seat ran or a cross-family verdict transported
  verbatim; no measurement transcribed from another seat's prose.
