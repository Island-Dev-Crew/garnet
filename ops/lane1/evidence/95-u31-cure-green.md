# U-31 cure GREEN — repo-relative POSIX `08.source`; unit traps pass, no collateral semantics

- Recorded: 2026-07-28, at the cure commit (successor of RED
  `657f22a8731344ec2d135a56dfb816616f11d1f9`).
- Implementer: **Claude Code — Opus 5 (`claude-opus-5`)**, on
  **`Hughs-MacBook-Pro.local`** (Darwin 25.5.0 / arm64); Python 3.14.5;
  git 2.50.1; `core.autocrlf=false` verified. Fleet-fork commit identity
  Jon Isaac `<Navigata1@gmail.com>`. (True model identity, recorded per lane
  rule.)
- Authorization: verdict 09, Option A, bounded. Cross-clone determinism and
  digest-inclusion legs are recorded separately in
  `ops/lane1/evidence/96-u31-cross-clone-digest.md`.

## 1. The cure — one line, `scripts/garnet_launch_readiness_status.py:509`

```diff
-        source=str(Path(__file__).resolve()),
+        source=Path(__file__).resolve().relative_to(REPO_ROOT).as_posix(),
```

The `source` KEY is retained. `REPO_ROOT` (line 57) is unchanged. Nothing else
in the reporter, no other reporter, no workflow, no ruleset, no digest
predicate is touched. The serialized value becomes exactly
`scripts/garnet_launch_readiness_status.py`.

## 2. Trap suite: RED → GREEN

```
$ python3 scripts/test_garnet_launch_readiness_status.py U31CureTrapTests -v
test_trap1_source_is_exact_repo_relative_posix_path ... ok
test_trap2_real_dependency_change_moves_artifact_source_constant ... ok
test_trap3_source_only_change_no_collateral_semantics ... ok
test_trap4_frozen_exclusion_tuple_and_lane0_inclusion ... ok
Ran 4 tests in 2.078s
OK
```

`test_trap1_...` flipped from RED (evidence 94) to GREEN; the three standing
guards remain green.

## 3. Full suite — exactly one UNCHANGED baseline failure (verdict 09 F3)

```
Ran 41 tests in 24.319s
FAILED (failures=1)

FAIL: test_tracked_ledger_matches_renderer_byte_for_byte (LedgerPinTests)
AssertionError: … "\n\nBlockers:\n- docs/playground/live.js brow…"
              != … "\n\n### `minimum_sealed_shelf` — Minimum seal…"
```

This is verdict 09 finding **F3**: the tracked Markdown ledger
(`F_Project_Management/LAUNCH/LAUNCH_READINESS.md`) still carries three stale
`live_wasm_playground` blocker lines that live regeneration no longer emits.

**It is not caused by this cure and this cure does not fix it (slice-5 scope).**
Proof: the cure changes only the JSON `source` value, and the Markdown renderer
never reads `.source`. The `--format markdown` output is **byte-identical**
before and after the cure:

| render | pre-cure sha256 | post-cure sha256 | equal |
|---|---|---|---|
| `--format human` | `376f4113190a1e9916dbb4bf53ccee622eb4fdb4c383e8c7f715b8ae535145fa` | `376f4113190a1e9916dbb4bf53ccee622eb4fdb4c383e8c7f715b8ae535145fa` | yes |
| `--format markdown` | `03e5a1fea80593afd7716a5e530d2cddfa0c46fc1575e3b9d7389f3dfd13b34f` | `03e5a1fea80593afd7716a5e530d2cddfa0c46fc1575e3b9d7389f3dfd13b34f` | yes |

Since the Markdown bytes are unchanged, this failure was present identically
before the cure (verdict 09 observed it at 37 tests; the suite is now 41 tests
with the four added traps). The cure may not absorb a ledger regeneration.

## 4. Trap 3 — no collateral reporter semantics (field-by-field)

At identical dependency state, pre-cure vs post-cure `--format json`:

- `diff` → exactly one hunk (line 3):
  ```
  <   "source": "/Users/IDC2.5/garnet-lane1-fresh/scripts/garnet_launch_readiness_status.py",
  >   "source": "scripts/garnet_launch_readiness_status.py",
  ```
- Parsed comparison: `changed keys: ['source']`; key set and order unchanged;
  `schema` unchanged (`garnet.launch_readiness/v1`); `source` key present in
  both.
- post-cure JSON sha256 = `e44eb5b22cdb4a85379d60a553b51b0c77fc270a18ccce24432fa3ea5b60203d`
  — independently reproduces the RED (evidence 93) cured-shape prediction
  `e44eb5b2…`.

## 5. Not done here (slice-5 scope, out of bounds for this cure)

- `ops/lane0/evidence/08-launch-readiness.json` is **NOT** regenerated or
  committed. That is slice 5's act, lawful only after this cure receives its
  own independent implementation verdict.
- No denominator, gate, schema, dependency, rendering, or recommendation logic
  is changed. Today's producer still derives **3/6 = 50.0%** launch-critical and
  **3/8 = 37.5%** ledger; this cure moves neither (verdict 09 Leg 5; the honest
  Phase-0 values — 4/6 is not chased).
