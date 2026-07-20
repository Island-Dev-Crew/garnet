# Lane 2B Review Verdict 05 — final independent Air execution

reviewer: Claude Code Fable 5 (independent, non-implementer, MacBook Air)
reviewed_head: 927ad221d33668d458499a26f49d96ed4586563d
reviewed_tree: 4d9374991bf265b78a0108e0bb62c317a43b8028
request: 05-request.md
swept_at: 2026-07-20T00:43:38Z
machine: MacBook Air, macOS 26.5.1, aarch64-apple-darwin, rust 1.95.0 (rustup), python3 -I
verdict: APPROVE

Truth floor on origin/main (cede73c0) at boot: lane0 closeout / MSRV / frozen
backlog all PASS. Branch tip 953d4308 fetched from the fork with a single
+refs/heads/... refspec; zero refs/pull/* fetched anywhere in this sweep.
Checkpoint 927ad221 is an ancestor of the tip; post-checkpoint commits touch
ops/lane2b/** only (verified by name-only diff).

## Leg 1 — IDENTITY: reproduced in full

- `git diff --stat cede73c0..927ad221` → 67 files, +5,343/−61 (matches packet).
- All eight bound blobs reproduce exactly (git blob OID and byte count via
  `git ls-tree` / `git cat-file -s`), including reporter blob `57b91324…`
  (23,438 B) and W_TRUST companion blob `cc2cad21…` (8,781 B).
- Product digest re-derived with MY OWN implementation (not the repo module)
  from `git --no-replace-objects ls-tree -r -z`: checkpoint =
  `810f256bcf9304999975120224419216422996ff3b804d1a9a8836d5bcc4c339` over
  1,529 paths; Verdict-04 baseline at tree `f3272b96…` = `1e6692175ea8…c96749d5`
  over 1,527 paths. Both match the packet and the reporter constants.
- Baseline→checkpoint product-set delta enumerated: added exactly
  `scripts/garnet_content_provenance.py` + `scripts/test_garnet_minimum_shelf_provenance.py`;
  removed none; content-changed exactly `scripts/garnet_wv_acceptance_status.py`
  + `scripts/test_garnet_wv_acceptance_status.py`. No implementer-added product
  byte is silently treated as pre-reviewed: every delta path is OID-bound in
  the packet and W_TRUST Binding 4, and each binding was verified (below).

## Leg 2 — REPORTER DOUBLE-RUN (the Air-only leg)

Two fresh `--no-checkout` clones of the fork branch, config set BEFORE
checkout, both detached at 927ad221 (trees verified identical, 0 pull refs):
`core.autocrlf=false` and `core.autocrlf=true`. Reporter run twice in each
with raw shell redirection (`python3 -I scripts/smoke_garnet_minimum_shelf.py
--gate > out 2> err`):

- all four runs: exit 0, stderr 0 bytes, stdout 2,070 bytes,
  SHA-256 `e9051611b99aa330150ceb475cb286dcd5a14e07e3db7f33b0e869e3cf569cb9`
- within-checkout byte equality: true (cmp). cross-checkout byte equality:
  true (cmp). The Verdict-04 Decision-4 property — byte-identical verdict
  output across LF and default-Windows checkout conventions — HOLDS natively
  on this machine.
- against committed evidence 20 (2,071 bytes, SHA-256 `77c3cb05…f8c0fd`):
  see F1. The JSON payload is byte-identical; the single differing byte is the
  stdout line terminator (Windows text-mode CRLF vs POSIX LF), proven by exact
  reconstruction: my 2,069-byte payload + `\r\n` reproduces the committed
  SHA-256 bit-for-bit.
- the committed canonical WV-6 artifact
  `proofs/windows/launch-verification/wv6-minimum-shelf/minimum-shelf-status.json`
  (2,070 bytes, LF-framed although produced on Windows — binary `write_bytes`)
  differs from my stdout ONLY in the `current_commit`/`current_tree` snapshot
  fields, which record its sanctioned emission commit `5af167e0` by design.

## Leg 3 — ADVERSARIAL TRAPS: 3/3 rerun natively, correct directions

(a) product blob mutation: appended bytes to `garnet-cli/src/mcp.rs` and
    staged it (the digest reads the index, per the authorized construction)
    → reporter exit 1, state partial, findings name the exact divergent digest.
(b) squash durability, strongest form: created root commit `5c6491e9` wrapping
    the exact reviewed tree `4d937499…` in a new bare repo, cloned it fresh.
    The clone contains ZERO branch commit objects — `a6f0da2b…`, `e2820ce5…`,
    and even checkpoint `927ad221` are absent (`git cat-file -e` fails for all
    three) — and zero refs/pull/*. Shelf reporter: exit 0, accepted, findings
    [], `landed_main_commit = 5c6491e9…` (first-parent main identified).
    WV-6 gate: exit 0, accepted, 5/5 checks, findings [], same landed SHA.
    No red window, no rebind ceremony, no branch-object dependency.
(c) evidence/content mismatch: one byte appended to
    `ops/lane2b/evidence/10-f1-canonical-reseal-green.txt` → reporter exit 1,
    finding names that exact artifact.

Extra Air-only check: decoded the committed hex transcript and replayed it
through the real binary — `./target/debug/garnet mcp-serve --package
examples/minimum-shelf-flagship < input` at the checkpoint — output is
byte-identical to the committed expectation (898 bytes, exit 0, stderr 0).
The sealed flagship loads natively on macOS: the original Verdict-02 F1
prelude cure is re-proven end-to-end on a second platform.

## Leg 4 — DIFFERENTIAL

- Full python battery (fresh clone worktrees, this machine):
  merge-base cede73c0 → 121 pass / 13 fail; checkpoint → 121 pass / 14 fail.
  File-level delta: `test_garnet_minimum_shelf_provenance.py` newly PASSING
  (the authorized addition), and `test_garnet_novel_compositions.py`
  apparently newly failing — ADJUDICATED PRE-EXISTING, not charged: at the
  merge-base it is SKIPPED when no CLI binary exists (0.000s run); after
  `cargo build -p garnet-cli` at the merge-base the same test
  (`test_all_novel_programs_check_and_run`) fails identically. This is the
  implementer's long-known "novel_07" red, unmasked here only because my
  workspace build left a binary in the checkpoint worktree. Zero genuine
  new-vs-base failures. The packet's unittest-level parity claim
  (931/17F/8E/3S vs 928/17F/8E/3S, +3 tests, zero delta) is consistent.
- Cargo, functional counts only: `rustup run 1.95.0 cargo test --workspace
  --no-fail-fast` at the checkpoint → 2,163 passed, 0 failed, including
  `--test mcp_stdio` 2/2 and `--test minimum_shelf_package` 7/7
  (sealed 1/1 + rejection traps 6/6).
- Packet-required gates, LF checkout: provenance tests 3/3 OK; WV acceptance
  tests 6/6 OK; `garnet_wv_acceptance_status.py --wv WV-6 --gate` exit 0,
  accepted, 5/5 checks, 5 artifacts, findings []; `--wv WV-7 --gate` exit 1,
  pending (required direction); trust-kernel gate `ok: true, problems: []`.
- Corroboration re-run here at the checkpoint: `cargo fmt --check` pass;
  `cargo test -p garnet-cli --no-fail-fast` → 460 passed, 0 failed
  (reproducing the packet's exact count); strict workspace
  `cargo clippy --all-targets -- -D warnings` pass.

## Leg 5 — INTEGRITY

- Weakening scan dcf6008..927ad221: 217 deletions fully accounted for —
  (1) the authorized Verdict-04 truth pairing: `candidateMainSha` commit-
  ancestry mechanism (schema v1) removed from `garnet_wv_acceptance_status.py`
  and its test, REPLACED by the strictly stronger content-digest +
  landed-first-parent-main mechanism (schema v2, `reviewedTreeSha` +
  `productContentSha256` + shared-module squash verification) — this is
  exactly the squash-fragile→squash-durable swap Decision 2c authorized, and
  trap (b) proves the new mechanism survives what broke the old one;
  (2) regenerated sanctioned proof artifacts; (3) reporter rewrite per
  Decision 2a; (4) ops/W_TRUST documents. No assertion removed or loosened
  outside the authorized pairing.
- Exclusion list: `FROZEN_MUTABLE_PREFIXES` + `REPORTER_PATH` defined in
  exactly ONE place (`scripts/garnet_content_provenance.py`) and imported by
  both gates; contents exactly the four authorized entries (`ops/lane2b/**`,
  `proofs/**`, `F_Project_Management/W_TRUST/**`, reporter self-path). A
  checkpoint-wide grep finds no other path-exclusion logic in any gate: NO
  fifth exclusion.
- WV states asserted by the suite:
  `test_current_repository_tracks_wv6_acceptance_and_wv7_pending` pins both
  states with exact counts/exit codes; malformed/missing/hash-mismatch/
  self-promotion traps intact (6/6 pass here).
- W_TRUST companion: all six bound blobs (Bindings 1–4) verified against the
  checkpoint tree — git blob OID, SHA-256, and byte count all match,
  including the Verdict-02 protected `garnet-cli/src/bin/garnet.rs` blob
  `27835ca3…` unchanged. RED-before-mechanism evidence (16) is genuine:
  interface-boundary reds for all three traps, captured before either
  reporter changed.

## Findings

F1 (NOTE, non-blocking): the reporter's STDOUT STREAM is not cross-PLATFORM
byte-identical: Python's text-mode `print` terminates with CRLF on Windows and
LF on POSIX. My four Air runs are 2,070 bytes / SHA-256 `e9051611…`; committed
evidence 20 publishes 2,071 bytes / `77c3cb05…` from Windows runs.
Reproduction of equivalence: take any Air run's stdout, replace the trailing
`\n` with `\r\n` → SHA-256 equals the committed value exactly, proving the
2,069-byte JSON verdict payload is byte-identical across all eight runs
(four implementer Windows, four Air macOS). The property Verdict 04 Decision 4
demanded — byte-identical output across LF and default-Windows CHECKOUTS on
the Air — holds outright. No committed gate or artifact pins the console
stream bytes; the sanctioned WV-6 artifact chain is written in binary mode and
is platform-stable (verified byte-for-byte). Recommendation for any future
cross-platform stdout pinning: emit via `sys.stdout.buffer.write` or publish
per-platform stream digests alongside the payload digest. Not a cure
precondition for this verdict.

## Answers to the six request questions

1. YES — the frozen four-exclusion digest exactly implements Decision 2a:
   single documented construction, one enumeration command,
   `--no-replace-objects` everywhere (no replacement-ref dependency),
   stage-0-only with fail-closed parsing, duplicate-path rejection;
   independently re-derived to the same value.
2. YES — the baseline (1e669217…/1,527 @ f3272b96) plus final digest
   (810f256b…/1,529) are both reproduced; the delta is exactly the four
   OID-bound authorized paths; nothing implementer-added rides in unreviewed.
3. YES — proven in the strongest form: a fresh main-only clone with NO branch
   objects at all and zero pull refs passes Shelf and WV-6 and identifies the
   landed first-parent main commit; no red window exists at the squash instant.
4. YES — 3/3 traps rerun natively in the required directions with exact
   findings; the RED-before-mechanism record is genuine.
5. YES — Air LF and autocrlf=true outputs are byte-identical (4/4, cmp), and
   the native sealed positive 1/1, negative traps 6/6, and stdio 2/2 legs are
   green on this machine (see also the byte-identical committed-transcript
   replay). The one-byte stream-terminator variance against the
   Windows-captured evidence number is documented as F1 with proof of payload
   identity.
6. YES — W_TRUST is exact (every digest verified against the tree),
   identity-true (names this reviewer for execution review, chat-seat verdicts
   for design, Jon as carrier only; explicitly declines self-approval), and
   sufficient for the protected paths (CLI dispatch blob unchanged since its
   Verdict-02 authorization; reporter/provenance/test blobs bound and
   verified; trust gate ok:true, problems:[]).

## not_verified

- Windows-native execution (the implementer's Windows byte captures,
  `_setmode` binary-mode behavior): machine unsuitable (macOS/arm64).
  Corroborated indirectly — the same suites are green here (garnet-cli
  460/460 reproduced exactly), and the payload-identity proof covers their
  reporter stream outputs.
- timing / wall-clock: none claimed, none validated (fanless Air; machine
  honesty).

## Ceremony consequence

APPROVE. Per Request 05 and the standing ceremony, this authorizes Jon (the
only merge seat) to open the PR with the W_TRUST companion. FIRE, tag,
publish, and launch remain out of scope; Launch stays HOLD, Band 3 while U-17
is open.
