# U-31 cure RED — trap suite fails against the un-cured absolute `08.source`

- Recorded: 2026-07-28T09:57:05Z, at HEAD
  `ef6d21b96a3f8ea0b63603dbc11b763ddaf46f40` (verdict 09 authorization tip)
- Implementer: **Claude Code — Opus 5 (`claude-opus-5`)**, running on
  **`Hughs-MacBook-Pro.local`** (macOS / Darwin 25.5.0 / arm64) — Jon's macOS
  seat, fleet-fork commit identity Jon Isaac `<Navigata1@gmail.com>`.
  Python 3.14.5; git 2.50.1 (Apple Git-155); `core.autocrlf=false` verified
  global and effective before any run. (Recorded model identity is my true
  identity, not one this tasking assumes.)
- Seat: implementer, Lane 1. Reviewer of record: Codex GPT-5.6 Sol
  (cross-family). Merge authority: Jon (IslandDevCrew) only; review carrier:
  IDC-Trust-Review only; the implementer is neither.
- Authorization: verdict 09 (`ops/lane1/review/09-verdict.md`) — Option A,
  bounded. RED recorded BEFORE any cure. **No cure is implemented at this
  commit.** The only source change here is the addition of the trap suite to
  `scripts/test_garnet_launch_readiness_status.py`; the reporter is untouched.

## 1. Pre-cure floor (self-recomputed, not transcribed)

| quantity | value |
|---|---|
| tracked product pair (index) | `e89cb2996def7eec8e419dac235d55f985734e84be6f991c565da35d46feb64f` / 1544 |
| tracked product pair (HEAD tree) | `e89cb2996def7eec8e419dac235d55f985734e84be6f991c565da35d46feb64f` / 1544 |
| index == HEAD | True |
| tracked `ops/lane0/` paths | 31, all INCLUDED (0 hit by the frozen predicate) |
| `FROZEN_MUTABLE_PREFIXES` | `(b"ops/lane2b/", b"proofs/", b"F_Project_Management/W_TRUST/", b"ops/lane1/")` |
| `REPORTER_PATH` (Shelf self-path) | `b"scripts/smoke_garnet_minimum_shelf.py"` |

`ops/lane1/**` is digest-EXCLUDED (U-35, verdict 08), so committing this RED
moves no product byte; the pair is recomputed as `e89cb299…/1544` after the
commit.

## 2. The cause under cure — `scripts/garnet_launch_readiness_status.py:509`

```python
source=str(Path(__file__).resolve()),
```

`REPO_ROOT = Path(__file__).resolve().parents[1]` is already defined at line 57
and available in scope; the authorized Option A construction is
`Path(__file__).resolve().relative_to(REPO_ROOT).as_posix()`.

## 3. Trap suite RED (verbatim, run at HEAD `ef6d21b`)

```
$ python3 scripts/test_garnet_launch_readiness_status.py U31CureTrapTests -v
test_trap1_source_is_exact_repo_relative_posix_path ... FAIL
test_trap2_real_dependency_change_moves_artifact_source_constant ... ok
test_trap3_source_only_change_no_collateral_semantics ... ok
test_trap4_frozen_exclusion_tuple_and_lane0_inclusion ... ok

FAIL: test_trap1_source_is_exact_repo_relative_posix_path
AssertionError: 'scripts/garnet_launch_readiness_status.py' !=
'/Users/IDC2.5/garnet-lane1-fresh/scripts/garnet_launch_readiness_status.py'

Ran 4 tests in 2.056s
FAILED (failures=1)
```

- **Load-bearing RED:** `test_trap1_...` — the exact-value trap fails because
  the un-cured reporter emits a host-absolute path. This is the check that
  could fail and did.
- **Standing regression guards (pass in both states, committed alongside):**
  - `test_trap2_...` — a real readiness change (wasm blocker via the existing
    `Dependencies` seam) still moves the serialized artifact while `source`
    stays constant; restores cleanly.
  - `test_trap3_...` — mutating only `source` moves the JSON by exactly one
    line and leaves human/markdown byte-identical; the `source` key and schema
    ordering are retained.
  - `test_trap4_...` — the frozen exclusion tuple is exactly the four
    authorized prefixes plus the Shelf self-path, and no `ops/lane0/` path is
    excluded.

  These three cannot fail pre-cure (they are invariants a source-only cure must
  preserve); they are committed now so any future cure that broke real-state
  sensitivity, added collateral render semantics, or weakened the digest
  exclusion would be caught.

## 4. Live two-root divergence (trap 1, disease reproduced fresh)

Two distinct absolute clone roots, same commit `ef6d21b`, same readiness state:

| root | `.source` | JSON sha256 |
|---|---|---|
| A `/Users/IDC2.5/garnet-lane1-fresh` | `/Users/IDC2.5/garnet-lane1-fresh/scripts/garnet_launch_readiness_status.py` | `3f902587e2f40cd819a74710da6ae56cfb6125392508e962a0c62968d21ae96d` |
| B `…/scratchpad/u31/B_red` (`git clone --no-hardlinks`) | `…/scratchpad/u31/B_red/scripts/garnet_launch_readiness_status.py` | `6ae2a5bc7ea41a41aa7c1ee80bda5584d3a93e0cb90511813b86ebc142d33a6c` |

- `diff A.json B.json` → exactly one hunk: line 3, `source`.
- `equal_without_source: True`.

(The A-vs-B diff isolates only `source`; the second, lawful axis — the
`live_wasm_playground.blockers` drift versus the *committed* `08` — is present
only in a committed-vs-regenerated comparison and is slice-5 content, not
U-31. It is not normalized away by this cure.)

## 5. Pre-cure renders captured for the trap-3 post-cure comparison

| render | sha256 |
|---|---|
| `--format json` | `3f902587e2f40cd819a74710da6ae56cfb6125392508e962a0c62968d21ae96d` |
| `--format human` | `376f4113190a1e9916dbb4bf53ccee622eb4fdb4c383e8c7f715b8ae535145fa` |
| `--format markdown` | `03e5a1fea80593afd7716a5e530d2cddfa0c46fc1575e3b9d7389f3dfd13b34f` |

Post-cure, the human and markdown hashes must be unchanged and the JSON must
differ by exactly the `source` line (§ trap 3 in the GREEN record).

## STOP (RED)

No reporter behavior is changed at this commit. The cure lands in the next
commit and turns `test_trap1_...` GREEN.
