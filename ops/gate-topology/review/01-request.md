# Gate Topology Review Request 01

## Seats and exact boundary

- Implementer: OpenAI Codex, GPT-5-based agent; exact build unavailable.
- Implementer machine: `Hughs-MacBook-Pro.local`, Darwin 25.6.0 arm64.
- Required reviewer: Claude Fable 5 (Anthropic), on the Air seat. Record the
  actual model version and machine facts in the verdict.
- Review carrier: IDC-Trust-Review only.
- Merge authority: Jon (IslandDevCrew) only.
- Branch: `mission/gate-topology`
- Base: `efd4f6bae8b3afaba74594e57944b2548142aeae`
- Content head: `c01383fab3061c71b91e10aa46c29d54f46b667e`
- Content tree: `2ed278c5db57e8d78766fb212938a342e03984aa`
- Verification-record head:
  `9659ce8d580543e9b3295198cd1f514106ab17a0`
- Pull-request refs fetched by implementer: zero.

The pushed tip adds only this request, the W_TRUST implementer companion, and
the corresponding journal entry above `9659ce8`. Record the exact fetched tip
and tree in the verdict. Review the content head named above; do not infer a
content approval from a later record-only tip.

## Rulings and evidence to read first

Read every file in this order:

1. `ops/gate-topology/RULING.md`
2. `ops/gate-topology/RULING-AMENDMENT.md`
3. `ops/gate-topology/RULING-ORDERING.md`
4. `ops/gate-topology/FINDINGS.md`
5. `ops/gate-topology/W_TRUST.md`
6. `ops/gate-topology/journal.md`
7. `ops/gate-topology/evidence/01-unamended-v2-red.txt` through
   `ops/gate-topology/evidence/12-final-local-verification.txt`

Evidence 03a supersedes only Evidence 03’s inaccurate mutation-scope sentence.
Evidence 08’s exclusion proposal is withdrawn but preserved as the arithmetic
that forced the Slice 3 substitution. Evidence 10 and Evidence 11 are active
exhibits and supersede nothing.

## Required independent review

1. Set `core.autocrlf=false` before a fresh clone, fetch the fleet branch
   without any `refs/pull/*`, and verify the fetched tip by `git ls-remote`.
2. Verify the branch is linear from the exact base:

   ```sh
   git rev-list --merges \
     efd4f6bae8b3afaba74594e57944b2548142aeae..HEAD
   git merge-base \
     efd4f6bae8b3afaba74594e57944b2548142aeae HEAD
   ```

   The first command must print nothing and the second must print the exact
   base.
3. Verify `c01383f` resolves to tree `2ed278c`, and every later path before
   your verdict is under `ops/gate-topology/**`.
4. Reproduce the unamended topology RED from Evidence 01 using the historical
   module at `efd4f6b` and target `162b96a`. Confirm both exact graph finding
   classes are present before evaluating the amendment.
5. Review the v2 walk amendment in
   `scripts/garnet_trust_kernel_review_status.py`. Confirm that an
   outside-lineage merge parent is admitted if and only if the complete trust
   byte-set and its separately computed digest equal `reviewed_head`, and that
   inequality stays hard RED. Confirm no other review, identity, append-only,
   or authenticated-transport semantic was weakened.
6. Run the topology positive and one-byte negative fixtures:

   ```sh
   python3 -I scripts/test_garnet_trust_kernel_review_status.py \
     -k 'merge_parent_outside_reviewed_lineage'
   python3 -I scripts/test_garnet_trust_kernel_review_status.py
   ```

7. Review `scripts/garnet_content_provenance.py`,
   `scripts/garnet_wv_acceptance_status.py`, and
   `scripts/smoke_garnet_minimum_shelf.py`. Confirm the four-prefix product
   digest exclusion tuple is byte-for-byte unchanged and that
   `ops/wv6-reaccept/**` is added only to the separate post-acceptance record
   class.
8. Reproduce the exact `410ff11..162b96a` proof from Evidence 10. It must
   report the unchanged frozen pair
   `fd96e6d910180f5e33999fbd693ea211e336389a13535930d89b2a870ff54727`
   / `1606`, `findings=[]`, and GREEN.
9. Run all Slice 3 traps and the full provenance suite:

   ```sh
   python3 -I scripts/test_garnet_minimum_shelf_provenance.py
   python3 -I scripts/test_garnet_launch_readiness_status.py \
     U31CureTrapTests.test_trap4_frozen_exclusion_tuple_and_lane0_inclusion
   ```

   Confirm record-only drift is GREEN, one non-record byte is named and RED,
   and WV-6 records remain included in the frozen digest definition.
10. Reproduce or independently inspect Evidence 11. Confirm the synthetic
    third-merge tree is conflict-free, remains RED on the enumerated non-record
    paths, and still reports the frozen pair. Confirm U-57’s attribution and
    terminal-acceptance disposition match Jon’s ordering ruling.
11. Run the final local ladder:

    ```sh
    python3 -I scripts/garnet_lane0_closeout_status.py --gate
    python3 -I scripts/garnet_msrv_status.py --gate
    python3 -I scripts/garnet_frozen_backlog_status.py --gate
    python3 -I scripts/garnet_capability_scope_status.py --gate
    python3 -I scripts/garnet_evidence_integrity_status.py --gate
    python3 scripts/check-agent-contracts.py
    python3 scripts/test_check_agent_contracts.py
    cargo +1.95.0 fmt --all -- --check
    cargo +1.95.0 test -p garnet-cli new_cmd
    git diff --check \
      efd4f6bae8b3afaba74594e57944b2548142aeae..HEAD
    ```

12. Run the rolling trust gate without a credential. Its expected pre-review
    state is RED only for the absent canonical structured record. Do not supply
    a credential, invent PR fields, or describe that expected RED as approval.

## Verdict

Write `ops/gate-topology/review/01-verdict.md` as APPROVE or with exact
blockers. Include:

- actual reviewer model family/version, machine, OS, and architecture
- fetched branch tip/tree and reviewed content head/tree
- exact commands and outputs
- RED-before/GREEN-after comparison for topology
- the equal-snapshot and both mutation dispositions
- the exact frozen product pair and path count
- U-57 attribution and ordering-law disposition
- any Critical, Important, or advisory findings

Do not edit the implementation, this request, the W_TRUST companion, or prior
evidence. The implementer must not author the verdict. Approval authorizes only
the next Jon-directed integration step; it does not authorize a PR, merge,
acceptance, tag, release, or launch promotion.
