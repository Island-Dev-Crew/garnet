# Lane 1 Phase 0 — Independent Review Verdict 05 (Requests 05 + 06)

reviewer: Codex GPT-5.6 Sol (independent, non-implementer)
reviewed_head: f1ec5699703688037c3b6320c630ff160379cf48
reviewed_tree: b35a775dd4a33e94d861e4e6636f11933138c11f
ruling_packet_head: 35cf33e04c414e349a166c6df09c515d2a9f58be
ruling_packet_tree: 2929d637e00e5e069823cd484a4c46839e969f78
request_05: ops/lane1/review/05-request.md (ea1dcf634e7f2ba2c3673334670cfb8155a87f91)
request_06: ops/lane1/review/06-request.md (35cf33e04c414e349a166c6df09c515d2a9f58be)
swept_at: 2026-07-27T22:58:13Z
machine: Pulse's MacBook Air; Mac17,3; Apple M5; macOS 26.5.1; Darwin arm64
model: Codex GPT-5.6 Sol
implementer_self_report: Claude Code Fable 5
verdict: U-35 AUTHORIZE-WITH-CONSTRAINTS; REQUEST 05 APPROVE-AS-REVIEWED-BUT-SUPERSEDED; NUC HOLD
approved_head_for_nuc: NONE
verified_identity: head/tree/diff/digests independently reproduced: yes
lineage: merge-base with 68317ae is 68317ae; merge commits through packet tip: 0
differential: request-05 candidate is exactly 8 value replacements in 2 files; no logic/comment/structure change
security: NOT APPLICABLE

U-35 is real. The existing frozen construction counts Lane 1 operational and
review artifacts as product, so every request, verdict, evidence record, or
heartbeat after a pin moves the product digest. A squash commit carries the
branch-tip tree unchanged; therefore a pin bound before those artifacts cannot
match landed main. This is a deterministic tree-content regression, not a
speculative merge concern.

The correction is authorized as the single enumerated prefix
`b"ops/lane1/"`. It is a consistency correction, not a weakening: all 40
paths currently under that prefix are lane state, locks, ledger, evidence,
requests, verdicts, or journal/blocker records; the product/gate code that
produces or validates them remains in the digest. The existing
`b"ops/lane2b/"` exclusion uses the same namespace-level treatment.

A general predicate such as every `ops/<lane>/` namespace is DENIED. It would
pre-authorize exclusions for future, uninspected lanes. The literal Lane 1
prefix is bounded, matches the existing frozen tuple's style, and can be
tested as exact data.

Request 05 is correct at its exact candidate: the four reporter constants and
four required `PROOF.json` mirrors are the only changed values, and
`5d3e7f72.../1581` is self-consistent at `f1ec569`. But that pin is superseded
by this ruling because the authorized exclusion and its included gate/test
bytes require a new digest and count. No native-Windows run may start at
`f1ec569`, `35cf33e`, or any other existing head.

## Boot and truth floor

- Sabbath fence: not active; local date was Monday 2026-07-27 in
  America/Chicago.
- Fresh clone path: `/tmp/garnet-review-Dj9pWJEc/garnet`; no warm clone was
  reused.
- `core.autocrlf=false` was set globally before cloning.
- Remote order is correct:
  `origin=https://github.com/Island-Dev-Crew/garnet.git`, then
  `fork=https://github.com/Navigata1/garnet.git`.
- No `refs/pull/*` were fetched; the local pull-ref count is zero.
- `origin/main` is
  `68317ae258327aade47fc2c07b7b5b580ec7c6ea`, tree
  `29191aa0e17121c08b73fe12578ee4464559e2ba`.
- Boot UTC: `2026-07-27T22:50:39Z`.
- Main truth floor:
  - Lane 0 closeout: PASS, evidence 22/22, ledger 37 entries,
    denominators 4/4.
  - MSRV: PASS, Rust 1.95, 16/16 workspace members inheriting.
  - Frozen backlog: PASS, eight entries, no findings.
  - Authenticated rolling-review v2 diagnostic: PASS,
    base=head=`68317ae...`, `trust_kernel_touched=false`, problems `[]`.

## Part 1 — U-35 ruling and exact reproduction

The repository implementation excludes exactly:

```text
ops/lane2b/
proofs/
F_Project_Management/W_TRUST/
scripts/smoke_garnet_minimum_shelf.py
```

`ops/lane1/` is absent. I independently implemented the documented raw
construction over
`git --no-replace-objects ls-tree -r -z <revision>` and compared it with
`tracked_content_digest`. Every pair matched:

```text
14a5e456  48c63f8d32ad8f0628aca2a82c31415739bccd0623b10dc45031cf3b99791fab  1578
72ae0246  99c3f2701f0a19b25f2f56e5cca8f59e9f719c8dd03b1bc5f14401cebeb0c3ab  1578
f1ec5699  5d3e7f727b56dbeb22d570c0b61a3b45b5e0b8df4e3b6305be896f9b5ed42b43  1581
ea1dcf63  9f483ce917e657a682965b0abbd208a91abfa6b0a75d53771ed56f46e6e008dc  1582
35cf33e0  726005d3a4c9ca1de5272ecac0e7fdfae0d75c2db6d430f59185eec20e671871  1584
```

From `f1ec569` to the packet tip, the only changed paths are:

```text
M ops/lane1/BLOCKED.md
A ops/lane1/evidence/91-u35-tip-vs-head-digest-red.md
M ops/lane1/journal.md
A ops/lane1/review/05-request.md
A ops/lane1/review/06-request.md
```

There are no product-code changes in that range. Nevertheless the current tip
Shelf gate exits 1 with:

```text
product content digest does not match reviewed bytes
product path count does not match reviewed index
product content digest mismatch
(726005d3... != 5d3e7f72...)
```

A squash changes the commit object and parent, not the tree. The landed
construction over the squash tree therefore remains `726005d3.../1584` while
the old pin remains `5d3e7f72.../1581`; WV-6 cannot remain accepted on landed
main under that pin.

### Consistency-correction proof

As a non-mutating diagnostic, I added only `b"ops/lane1/"` to the predicate in
memory. The exact included-path delta at `35cf33e` was 40 paths, and every one
started with `ops/lane1/`; no other path left the set.
`scripts/garnet_content_provenance.py` remained included.

Before any cure code or test byte is changed, that diagnostic produces:

```text
14a5e456  5edb658ff8c989d67b6bf5bbe4e1c7a1e285fb8146d41d5d0aaa68ba0bdd0d4b  1544
72ae0246  494e48372dfd26d095a57f19de5e0b79f2cf79f770871c20ea53348287180592  1544
f1ec5699  494e48372dfd26d095a57f19de5e0b79f2cf79f770871c20ea53348287180592  1544
ea1dcf63  494e48372dfd26d095a57f19de5e0b79f2cf79f770871c20ea53348287180592  1544
35cf33e0  494e48372dfd26d095a57f19de5e0b79f2cf79f770871c20ea53348287180592  1544
```

This proves both sides of the crux:

- the real `docs/why.html` product-byte change from `14a5e45` to `72ae024`
  still moves the digest (`5edb658f...` to `494e4837...`) and produces a
  product-content mismatch against the old pin;
- the review-only states from `72ae024` through `35cf33e` no longer move the
  digest or count.

`494e4837.../1544` is diagnostic evidence only. It is NOT the authorized final
pin: the cure changes the included provenance script, and committed trap
changes may also change included test bytes or path count. The exact final
pair must be derived after those bytes are finalized.

## Authorized cure scope and mandatory traps

The next implementer series may:

1. append exactly `b"ops/lane1/"` to
   `FROZEN_MUTABLE_PREFIXES` in
   `scripts/garnet_content_provenance.py`;
2. add or strengthen focused tests in the existing provenance/WV test
   surfaces;
3. rederive only `EXPECTED_PRODUCT_CONTENT_SHA256` and
   `EXPECTED_PRODUCT_PATH_COUNT` in
   `scripts/smoke_garnet_minimum_shelf.py`;
4. update only the matching `productContentSha256` and `productPathCount`
   mirrors in `proofs/minimum-shelf/lane2b/PROOF.json`;
5. add the necessary `ops/lane1/` evidence/request/heartbeat artifacts.

No authorization is granted to change reporter logic, `REVIEWED_HEAD`,
`REVIEWED_TREE`, `REVIEWED_TREE_PRODUCT_SHA256`,
`REVIEWED_TREE_PATH_COUNT`, either `reviewedTree*` proof field, another
exclusion, a workflow, a ruleset, or implementation code outside the focused
provenance tests.

The proposed four traps are necessary but not sufficient without the
following clarification and fifth trap:

- **Trap (a), required:** mutate an included product blob; the digest must
  move and both the content verifier and pin-dependent gate must fail.
- **Trap (b), required:** add and modify only `ops/lane1/` artifacts; digest
  and count must remain byte-identical.
- **Trap (c), corrected and required:** compare the final U-35 cure/rebind
  candidate with the later review-request tip. They must have identical
  digest/count. Do not compare the new code against old `f1ec569`: the
  included provenance-script blob must differ.
- **Trap (d), strengthened and required:** assert the exact exclusion tuple
  is the three old prefixes plus `b"ops/lane1/"`, with the reporter self-path
  unchanged. Assert the included-path set difference contains only
  `ops/lane1/` paths. A general `ops/<lane>/` predicate fails this trap.
- **Trap (e), additional and required:** after all included cure and test
  bytes are final, independently recompute the digest/count at the exact
  candidate tree; prove the two new reporter constants and two proof mirrors
  equal that pair, the old `5d3e7f72.../1581` pair fails, the Shelf gate is
  green, and both historical `reviewedTree*` values remain byte-identical.

The new expected pair must travel in the same reviewed series as the
exclusion. Separating them would deliberately commit a red reporter window,
the failure mode this correction is intended to remove.

## Part 2 — Request 05 disposition

The exact candidate identity reproduces:

```text
head      f1ec5699703688037c3b6320c630ff160379cf48
tree      b35a775dd4a33e94d861e4e6636f11933138c11f
parent    a33a76b9e2e8feec0cfbb96422ce2f54dc9b8a88
mergebase 68317ae258327aade47fc2c07b7b5b580ec7c6ea
merges    0
```

The commit author and committer are both
`Jon Isaac <Navigata1@gmail.com>`.

### Differential and byte identity

`git diff-tree` reports exactly two modified files:

```text
M proofs/minimum-shelf/lane2b/PROOF.json
M scripts/smoke_garnet_minimum_shelf.py
```

Each file is `4` insertions / `4` deletions. A zero-context diff contains only:

- reporter: `REVIEWED_HEAD`, `REVIEWED_TREE`,
  `EXPECTED_PRODUCT_CONTENT_SHA256`, `EXPECTED_PRODUCT_PATH_COUNT`;
- proof: `reviewedHead`, `reviewedTree`, `productContentSha256`,
  `productPathCount`.

That is exactly eight changed values. There is no logic, comment, key-set,
format, or structural change, and the candidate diff is `git diff --check`
clean.

The historical pair is byte-identical in both reporter and proof:

```text
REVIEWED_TREE_PRODUCT_SHA256 / reviewedTreeProductSha256
1e6692175ea8fe2dd5b04fad4a492dc8ce48767dd07d88fd11a0847ce96749d5

REVIEWED_TREE_PATH_COUNT / reviewedTreePathCount
1527
```

Those values are not compared to the live tree or current product digest.
They are historical Lane 2B provenance. For precision, Request 05's phrase
"never compared" is too broad: `_validate_proof` does compare the two proof
mirrors against these constants for canonical agreement. That comparison is
reportorial consistency, not a live-content acceptance check, and it does not
alter the disposition.

### Candidate digest and focused gates

The raw construction and repository module independently reproduce at
`f1ec569`:

```text
product_content_sha256 =
5d3e7f727b56dbeb22d570c0b61a3b45b5e0b8df4e3b6305be896f9b5ed42b43
product path count = 1581
```

At that exact detached candidate:

- Shelf gate: exit 0, `ok=true`, state `accepted`, checks 5/5,
  findings `[]`, current head/tree exact, product digest/count exact.
- WV-6 gate: exit 1, state `partial`, checks 5/5, artifacts 5, with exactly
  four expected stale-native-manifest findings: reviewed head mismatch,
  reviewed tree mismatch, manifest product digest mismatch, and live digest
  mismatch against the old manifest digest.
- Provenance focused suite: 3/3 pass.
- WV focused suite: 5/6 pass; the sole failure is the standing assertion that
  expects WV-6 `accepted` while the unchanged native evidence is still
  `partial`.

The Shelf-green/WV-partial combination is therefore correct for Request 05
before the native run. It is not approval to run the NUC because U-35
supersedes the pin.

## Findings and advisories

F1 (BLOCKER UNTIL CURED): U-35 is confirmed. The pre-verdict packet tip has
digest/count `726005d3.../1584`, not the `f1ec569` pin
`5d3e7f72.../1581`, despite only Lane 1 artifacts changing above that head.
The same tip tree would land in the squash.

F2 (REQUIRED CONSTRAINT): authorize only literal `b"ops/lane1/"`. The broader
`ops/<lane>/` predicate is denied because it would silently weaken future
coverage.

F3 (REQUIRED CONSTRAINT): the original trap (c) is ill-posed if "rebind head"
means old `f1ec569`. The cure script is an included product path, so the new
candidate cannot equal old `f1ec569` after implementation. Trap (c) must use
the new final candidate, and Trap (e) must bind its rederived pair.

F4 (NOTE): Request 05's historical `reviewedTree*` pair is unchanged and is
not a live-content comparator. It is nevertheless compared to the proof
mirrors for canonical report agreement; "never compared" should not be
repeated literally.

F5 (NOTE): Request 05 is approved as an exact, internally correct historical
rebind. Its expected product pair is superseded for all forward action by the
authorized U-35 correction.

S-SEC-1 (ADVISORY, non-blocking): carry forward the broad security sweep
across capability/authority surfaces before the Lane 4 frozen candidate and
final red-team, including any such surface touched by Lane 0 repair #3.

S-WS-1 (STYLE ADVISORY, non-blocking): the cumulative branch still contains
trailing whitespace in immutable Requests 02 and 03. The Request 05 candidate
diff is clean. Do not create a standalone formatting round.

## Scope, weakening, security, and provenance

- Scope reviewed: Request 06's design ruling; Request 05's exact two-file
  candidate; the lineage and review-only commits through `35cf33e`.
- No implementation cure exists yet. This verdict does not approve an
  uninspected future code delta.
- The authorized exclusion removes only the enumerated Lane 1 operational
  namespace. The product page, provenance implementation, reporter consumers,
  tests, CLI, Rust crates, dependencies, workflows, rulesets, and all other
  paths remain covered exactly as before.
- Existing exclusions for `ops/lane2b/`, `proofs/`,
  `F_Project_Management/W_TRUST/`, and the reporter self-path remain
  unchanged.
- Security is NOT APPLICABLE to the reviewed Request 05 constant/registry
  delta and the Request 06 design packet. No authority, capability, unsafe,
  process, dependency, network, credential, seal, or MCP-host implementation
  changed. This is a scope ruling, not a waiver of S-SEC-1.
- Raw Git reproduction used `--no-replace-objects`; no replacement refs or
  pull refs were present.
- The old Request 05 pin was recomputed, not transcribed, and is valid only
  at its exact candidate under the old predicate.
- The final U-35 digest/count is intentionally not guessed. It is unknowable
  until the authorized included code/test bytes are final.

## Not verified

- Native Windows WV-6 was not run; this macOS/arm64 Air is not the NUC.
- No timing or performance claim was made or validated.
- No full Cargo, Clippy, or workspace-wide Python battery was run because the
  reviewed Request 05 delta contains no Rust or executable logic. The focused
  provenance, Shelf, and WV legs named by the requests were run.
- No broad security scan was run under the explicit scope ruling.
- No PR, merge, GitHub approval event, workflow, ruleset, implementation
  file, canonical review record, or `ops/mission/state.json` was created or
  modified by this reviewer.

## NUC consequence

**APPROVED HEAD FOR THE NUC: NONE.**

The implementer must now produce one linear U-35 cure/rebind candidate
containing the exact prefix, all five traps, and the final rederived
digest/count plus proof mirrors in the same reviewed series. A new immutable
request must name that exact head/tree and the candidate-to-review-tip
stability proof. Only a subsequent independent verdict may name a NUC head.
