# Lane 2C Review Request 01 - Memory Teardown Integrity

## Seats and review boundary

- Implementer: OpenAI Codex, GPT-5-based model; exact submodel/version not
  exposed by the harness.
- Implementer machine: `NUCBOX_M2PRO_S`.
- Required reviewer: a different model family on the MacBook Air. Record the
  actual model/version and machine in the verdict.
- Review carrier: IDC-Trust-Review only.
- Merge authority: Jon (IslandDevCrew) only.
- Branch: `mission/l2c-memory-teardown`
- Reviewed base: `efd4f6bae8b3afaba74594e57944b2548142aeae`
- Reviewed product head: `5cd113617acd35307bb028463833a8da2bbd6ad2`
- Reviewed product tree: `85faad1de5a2c47cb632bedea78dfb89d209001a`
- Pull-request refs fetched by implementer: zero.

The product diff is exactly:

```text
 garnet-memory-v0.3/AGENTS.md                       |   5 ++
 .../examples/lane2c_teardown_probe.rs              | 100 +++++++++++++++++++++
 garnet-memory-v0.3/src/cycle.rs                    |  95 ++++++++++++++++++--
 3 files changed, 195 insertions(+), 5 deletions(-)
```

Commits after the product head are `ops/lane2c/**` records only. Review the
exact product head/tree above; do not infer approval from branch-tip state.

## Required review

1. Verify the branch was cloned or fetched without `refs/pull/*`.
2. Verify the product head/tree and exact three-path product diff.
3. Verify there is no active build manifest below `ops/**`, the root
   `Cargo.lock` hash is
   `01b8986b1cee0ef6a53ac439bd018b54fc1dca825a8f845a259ed8001e6715fa`,
   and the MSRV gate still reports the exact 18-manifest set.
4. Run:

   ```sh
   python3 -I ops/lane2c/verify_evidence.py --gate
   python3 -I scripts/garnet_lane0_closeout_status.py --gate
   python3 -I scripts/garnet_msrv_status.py --gate
   python3 -I scripts/garnet_frozen_backlog_status.py --gate
   python3 -I scripts/garnet_capability_scope_status.py --gate
   python3 -I scripts/garnet_evidence_integrity_status.py --gate
   cargo test --locked -p garnet-memory --test cycle
   cargo test --locked -p garnet-memory
   cargo test --locked -p garnet-cli cache
   cargo clippy --locked -p garnet-memory --all-targets -- -D warnings
   cargo doc --locked -p garnet-memory --no-deps
   cargo fmt --all -- --check
   git diff --check efd4f6bae8b3afaba74594e57944b2548142aeae..5cd113617acd35307bb028463833a8da2bbd6ad2
   ```

5. Inspect incoming-edge accounting for duplicate add, missing remove,
   safe-affine exclusion, collected-edge cleanup, self-edge behavior, and
   retained rooted cycles.
6. Verify that isolated store roots take the O(1) rejection path while actual
   ARC-peer candidates retain rooted-reachability and trial deletion.
7. Confirm the harness is the crate example
   `garnet-memory-v0.3/examples/lane2c_teardown_probe.rs`, adds no dependency,
   and can be run plainly with:

   ```sh
   cargo run --locked -p garnet-memory \
     --example lane2c_teardown_probe --release -- working-clear 256
   ```

8. Verify the raw Callgrind profiles, source/lock/profile hashes, three
   before/after curves, and 4/4 stress output. Do not carry numbers from any
   earlier probe.
9. Do not rerun performance measurement on the MacBook Air; the quiet-machine
   law excludes it. Review the committed operation-count evidence and
   deterministic verifier.
10. Confirm `ops/lane2c/DOCTRINE.md` records the U-46 placement rule without
    modifying the frozen backlog or claiming global registration.
11. Confirm no wall-clock, production ARC, review, merge, or launch claim is
    introduced by the artifacts.

## Verdict

Write `ops/lane2c/review/01-verdict.md` as APPROVE or with exact blockers.
Include the reviewer model family/version, MacBook Air identity, reviewed
head/tree, commands and outputs, and any finding severity. The implementer
must not author that file. An approval authorizes only Jon's merge decision;
it does not authorize a tag, release, or acceptance record.
