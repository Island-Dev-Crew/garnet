# Gate Topology W_TRUST Companion

This is an implementer provenance companion, not a review verdict, canonical
structured review record, approval event, or acceptance record.

## Seats

- Implementer: OpenAI Codex, GPT-5-based agent. The harness exposed no exact
  model build or submodel version, so none is asserted.
- Implementer machine: Hugh’s MacBook Pro,
  `Hughs-MacBook-Pro.local`, Darwin 25.6.0 arm64.
- Required independent reviewer: Claude Fable 5 (Anthropic), on the Air seat.
  The reviewer must record the actual harness, model version, machine, OS, and
  architecture in the verdict.
- Review carrier: IDC-Trust-Review only.
- Merge authority: Jon (IslandDevCrew) only.

No implementer-authored file in this branch is a verdict. Review is pending.
U-17 remains OPEN, governance freeze remains not armed, and launch remains
HOLD.

## Boot and lineage boundary

- Clone source: `https://github.com/Island-Dev-Crew/garnet.git`
- Remote names: authoritative repository `origin`; fleet fork `fork`
- `core.autocrlf`: `false` before clone and in the clone
- Base and `origin/main` at lane start:
  `efd4f6bae8b3afaba74594e57944b2548142aeae`
- Base tree: `e9bce10421c1eac2a514291212b87d61a5289037`
- Branch: `mission/gate-topology`
- Pre-review content head:
  `c01383fab3061c71b91e10aa46c29d54f46b667e`
- Pre-review content tree:
  `2ed278c5db57e8d78766fb212938a342e03984aa`
- Verification-record head:
  `9659ce8d580543e9b3295198cd1f514106ab17a0`
- Verification-record tree:
  `db16066c4c17edbfb0b8a06a9d93ffb5619a051b`
- Pull-request refs fetched by implementer: zero.
- Merge commits in `base..content head`: zero.
- Exact Git author/committer identity in `base..verification-record head`:
  `OpenAI Codex <codex@openai.com>`.

Commits above the content head are confined to `ops/gate-topology/**` records.
They do not extend or backdate the content head’s review boundary.

## Slice 1 — topology RED

The corrected target is
`162b96adb0a91c5fdc8c189dc2fcdd22ce996cab`: its sole parent is
`8ae41b6f9660ae0f098d2137f14a1a89397fcfe5`, its author is Claude Fable 5,
and its two-path delta is confined to `ops/wv6-reaccept/**`.

Evidence 01 captures the unamended v2 gate at that target. It exits 1 with the
two intrinsic graph findings:

- no reviewed-lineage parent for `0649d796ac6b78b968d868398b517974838112f3`
- post-review trust touch in merge `2bcc6dd5249445a52558a51e17925bfdccae3fe1`
  versus parent `814c0bcb36924c00c392d4e47bc1d61bfd18ee45`

Evidence 03a preserves the corrected one-byte mutation RED. Evidence 03 is
retained because its topology RED is valid; only its mutation-scope sentence
is superseded by Evidence 03a.

## Slice 2 — bounded topology amendment

The v2 post-review walk now derives the reviewed lineage. At a merge commit,
an outside-lineage parent is admitted only when both the complete trust-path
snapshot and a separately computed SHA-256 digest of that snapshot equal the
`reviewed_head` values. Any path-set or byte inequality remains RED. All other
rolling-review semantics are unchanged.

- Equal-snapshot topology fixture: GREEN.
- `alpha = 1` to `alpha = 2` mutation fixture: RED.
- Full trust-gate unit suite: 112/112 GREEN.
- Exact-ref comparison: Evidence 01 contains both graph findings before the
  amendment; Evidence 06 contains neither after it. Evidence 06 remains RED
  only on its declared diagnostic-head override and absent authenticated
  transport, neither of which this lane weakens.

## Slice 3 — U-35 record-class tolerance

The product digest definition is unchanged. In particular,
`ops/wv6-reaccept/**` remains included when the frozen pair is computed. A
separate post-acceptance record class permits tip drift only after the frozen
head tree, digest, and path count match the recorded values, and only when
every `frozen..tip` path is in the enumerated record class.

- Frozen head: `410ff1182cdcefcec9fe046d1346205d8522ec9d`
- Frozen tree: `57ce26ae1ab8d24609180486bc5fce6179f37957`
- Frozen product digest:
  `fd96e6d910180f5e33999fbd693ea211e336389a13535930d89b2a870ff54727`
- Frozen path count: `1606`
- Exact candidate tip:
  `162b96adb0a91c5fdc8c189dc2fcdd22ce996cab`
- Exact-target result: GREEN with `findings=[]`.
- One non-record blob byte in the drift: hard RED with its path named.
- Full provenance suite: 9/9 GREEN.

Evidence 07 and Evidence 08 preserve the withdrawn literal-exclusion trial and
the pair-expectation falsification that forced the substituted mechanism. They
are not deleted or reinterpreted as GREEN.

## Ordering finding

Evidence 11 demonstrates that a conflict-free third merge still introduces
20 non-record governance paths after the intermediate acceptance. Both the WV
verifier and Minimum Shelf reporter correctly remain RED while continuing to
report the frozen `fd96e6d9…/1606` pair. Jon’s ruling therefore makes
acceptance terminal rather than creating a new exception class.

Proposed U-57 records the law: acceptance is the last content operation on a
candidate. A later non-record content merge supersedes the earlier acceptance
with preservation; it never bends the verifier. Evidence 10 and Evidence 11
remain active exhibits and supersede nothing.

## Verification and remaining authority

Evidence 12 records the exact `c01383f` local battery:

- truth floor 5/5 PASS
- topology amendment traps 2/2 PASS
- U-35 positive, non-record mutation, and digest-definition traps PASS
- trust-gate suite 112/112 PASS
- provenance suite 9/9 PASS
- agent contracts 24/24 and contract tests 6/6 PASS
- default and explicit Rust toolchains both 1.95.0
- `cargo +1.95.0 fmt --all -- --check` PASS
- selected `garnet-cli new_cmd` tests 13/13 PASS

The pre-review rolling gate is RED only because the canonical v2 structured
record is absent. That is the required state before independent review and an
authenticated approval event. Request 01 asks Claude Fable 5 to review the
content boundary. Jon alone may merge this branch into
`mission/wv6-reaccept`, perform or authorize the later PR actions, or accept a
candidate. The NUC terminal freeze, native WV-6 acceptance, and pin rebind are
outside this Mac lane.
