# Lane 2C - Memory Teardown Integrity

Status: Verdict 01 blocked on B1 only; the ops-only Memcheck cure is ready for
independent re-review. Jon-owned merge remains pending.

## Evidence header

- Implementer: OpenAI Codex, GPT-5-based model; exact submodel/version was not
  exposed by the harness.
- Implementer machine: `NUCBOX_M2PRO_S`, GMKtec NucBox_M2Pro_S.
- Independent reviewer: Claude Code on Claude Fable 5 (`claude-fable-5`,
  Anthropic), on `Pulses-MacBook-Air.local`, Darwin 25.5.0 arm64 (Apple M5).
  This is a different model family and machine from the implementer.
- Review carrier: IDC-Trust-Review only.
- Merge authority: Jon (IslandDevCrew) only.
- Boot head: `efd4f6bae8b3afaba74594e57944b2548142aeae`.
- Boot UTC: `2026-07-28T19:16:25Z`.
- Product head: `5cd113617acd35307bb028463833a8da2bbd6ad2`.
- Product tree: `85faad1de5a2c47cb632bedea78dfb89d209001a`.
- Host OS/architecture: Windows 11 Pro `10.0.26200`, 64-bit.
- Measurement guest: Ubuntu WSL2, Linux
  `6.6.87.2-microsoft-standard-WSL2`, x86_64.
- Checkout setting: global `core.autocrlf=false` before the fresh clone.
- Measurement filesystem: `/dev/sdd` ext4. WSLg and `/mnt/c` were not used.

## Result

The base reproduced a quadratic release curve in all three memory-store paths.
For each 2x input increase, Callgrind instruction counts rose by 4.24x to
4.38x. The product curves rose by 1.97x to 2.30x.

| Case | Roots | Base instructions | Product instructions | Reduction |
|---|---:|---:|---:|---:|
| Working clear | 256 | 43,600,112 | 98,738 | 441.57x |
| Working clear | 512 | 189,450,246 | 226,879 | 835.03x |
| Working clear | 1,024 | 804,658,305 | 460,159 | 1,748.65x |
| Episodic drop | 256 | 43,329,138 | 118,270 | 366.36x |
| Episodic drop | 512 | 189,766,227 | 233,214 | 813.70x |
| Episodic drop | 1,024 | 806,325,886 | 463,283 | 1,740.46x |
| Semantic drop | 256 | 43,650,642 | 168,904 | 258.43x |
| Semantic drop | 512 | 189,873,799 | 334,815 | 567.10x |
| Semantic drop | 1,024 | 805,185,166 | 666,079 | 1,208.84x |

Wall-clock time is retained only as environmental context; it is not claim
evidence.

## Leak-disposition evidence

Verdict 01 resolved the cycle-correctness challenge in the repair's favor and
blocked on exactly B1: no leak evidence for the cheaper teardown. The original
base and product probe binaries were reused after their SHA-256 values matched
the hashes already bound in `measurement.json`. Six Valgrind Memcheck captures
now cover the same three cases at size 1,024:

| Phase | Case | Definitely lost | Indirectly lost | Possibly lost | Still reachable |
|---|---|---:|---:|---:|---:|
| Before | working-clear | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 544 B / 1 block |
| Before | episodic-drop | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 544 B / 1 block |
| Before | semantic-drop | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 544 B / 1 block |
| After | working-clear | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 544 B / 1 block |
| After | episodic-drop | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 544 B / 1 block |
| After | semantic-drop | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 544 B / 1 block |

Every before-to-after byte and block delta is zero. Memcheck leak accounting is
deterministic, so no quiet-machine ritual was performed and no quiet window is
claimed for these six captures. The raw output and evidence header are under
`ops/lane2c/evidence/memcheck/`; the exact replay is
`ops/lane2c/replay_memcheck.sh`.

## Cause and repair

Every isolated root release called `should_buffer_candidate`, which rebuilt
whole-graph rooted reachability and reference counts even though a node with no
incoming ARC edge cannot be a cycle candidate.

`CycleGraph` now maintains exact incoming managed-ARC edge counts. An isolated
store root takes an O(1) rejection path. A node with an actual incoming ARC peer
still uses the rooted-reachability and bounded trial-deletion path.

## Shipped harness and lockfile

The measurement harness ships at
`garnet-memory-v0.3/examples/lane2c_teardown_probe.rs`. It adds no manifest and
no dependency. A reviewer can run a plain replay against the crate:

```sh
cargo run --locked -p garnet-memory \
  --example lane2c_teardown_probe --release -- working-clear 256
```

The exact Callgrind loop is `ops/lane2c/replay_callgrind.sh`; raw profiles are
under `ops/lane2c/evidence/callgrind/`. Every profile was regenerated after the
harness moved into the crate. No count from the superseded probe remains in
this record.

The superseded transient manifest did contain an empty `[workspace]` stanza,
so Cargo treated it as a separate nested workspace and wrote its lockfile
outside the repository. Root `Cargo.lock` was unchanged:
`01b8986b1cee0ef6a53ac439bd018b54fc1dca825a8f845a259ed8001e6715fa`
before and after. `ops/lane2c/PROPOSED-DOCTRINE.md` records proposed U-46 and the
effective placement rule.

## Quiet measurement window

The replacement run used one Callgrind process at a time from
`2026-07-29T08:34:09.8071444Z` through
`2026-07-29T08:35:39.5943570Z`.

- OneDrive PID 9296 and OneDrive.Sync.Service PID 22364 were stopped at
  `2026-07-29T08:34:00.5984747Z`.
- Ubuntu `cron.service` was stopped at
  `2026-07-29T08:34:00.7146380Z`.
- Start and end checks found no concurrent build, Valgrind, sync, cron, or
  spawned-agent workload; `/dev/sdd` ext4 was reconfirmed.
- Cron and OneDrive were active again by
  `2026-07-29T08:35:50.7129927Z`; the OneDrive sync helper was active again by
  `2026-07-29T08:36:10.1339188Z`.
- The ignored stress set ran inside the same quiet window and passed 4/4.

## Verify

```sh
python3 -I ops/lane2c/verify_evidence.py --gate
```

The gate checks the exact Callgrind and Memcheck capture sets and hashes,
counter and command bindings, all curves, the six leak summaries and deltas,
the shipped harness hash, unchanged root lockfile, 4/4 stress output, and the
absence of an active manifest under `ops/lane2c/`.

## Claim boundary

This record does not claim production ARC, a portable wall-clock threshold,
review approval, merge approval, launch acceptance, or global registration of
U-46. The global frozen backlog remains untouched. Verdict 01 is preserved as
BLOCKED; this successor supplies its sole requested evidence cure for
independent re-review. Only the named reviewer and Jon-owned merge path can
advance the repository claim state.
