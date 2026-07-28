# Lane 1 · Phase 0 — Review Request 10 (U-31 cure IMPLEMENTED: `08.source` → repo-relative POSIX)

- Date: 2026-07-28 (UTC ~09:57Z)
- Implementer: **Claude Code — Opus 5 (`claude-opus-5`)**, on
  **`Hughs-MacBook-Pro.local`** (macOS / Darwin 25.5.0 / arm64), fresh
  `autocrlf=false` clone, fleet-fork identity Jon Isaac `<Navigata1@gmail.com>`.
  (True model identity, recorded per lane rule — not an identity this tasking
  assumes.)
- Independent reviewer sought: **Codex GPT-5.6 Sol — cross-family verdict of
  record** (an implementation verdict; the authorizing verdict 09 approved a
  cure but no cure head).
- Merge authority: Jon (IslandDevCrew) only; review carrier: IDC-Trust-Review
  only; the implementer is neither.
- Authorization: `ops/lane1/review/09-verdict.md` — Option A, bounded.
- Packet lineage (atop verdict-09 tip `ef6d21b`):
  RED `657f22a8731344ec2d135a56dfb816616f11d1f9`
  → cure `c3dc53ee4169ae879647fcb74e7bb524488653ed`
  → this request (adds `10-request.md`, `96-…`, journal, BLOCKED — all
  digest-excluded `ops/lane1/`).
- **THE CURE IS IMPLEMENTED.** No cure head is approved until an independent
  implementation verdict names one. `08-launch-readiness.json` is NOT
  regenerated here (slice 5).

## 1. The cure — one line, exactly the authorized construction

`scripts/garnet_launch_readiness_status.py:509`:

```diff
-        source=str(Path(__file__).resolve()),
+        source=Path(__file__).resolve().relative_to(REPO_ROOT).as_posix(),
```

The `source` KEY is retained; the serialized value is exactly
`scripts/garnet_launch_readiness_status.py`. `REPO_ROOT` (line 57) is
unchanged. No other reporter line, no other `scripts/garnet_*` file, no
workflow, no ruleset, no digest predicate is touched. `git diff` on the
reporter is this single hunk.

## 2. Product pairs (each self-recomputed via `garnet_content_provenance`)

| commit | what changed | product pair | count |
|---|---|---|---|
| `ef6d21b` (verdict-09 tip) | — | `e89cb2996def7eec8e419dac235d55f985734e84be6f991c565da35d46feb64f` | 1544 |
| `657f22a` (RED) | +trap suite (`scripts/test_…`, digest-included) | `26b0e1f5bc540f8776caa46ccd554257f3e0123d99ea11f083787c6937e2f0cb` | 1544 |
| `c3dc53e` (cure) | reporter line 509 (`scripts/`, digest-included) | `0b6239c204f893ed627f59c937d83b6cd5c56832bc0260e31a63775d8d568158` | 1544 |

The pair legitimately moves because the reporter and its tests are tracked,
digest-INCLUDED `scripts/` files. Count stays 1544 (no included path added;
the evidence/request files are digest-excluded `ops/lane1/`). The
`e89cb299…/1544` value was the pre-cure-series floor (BLOCKED.md "before the
cure commit"), verified at `ef6d21b` before any edit.

## 3. Four traps — outputs (verdict 09 Leg 4, exact strengthening)

### Trap 1 — clone-path determinism (evidence 94 RED, 96 GREEN)
- Pre-cure, two distinct absolute clone roots at `ef6d21b`: `.source` diverged
  (`/Users/IDC2.5/…` vs `/private/tmp/…/B_red`), JSON `3f902587…` vs
  `6ae2a5bc…`, one hunk, `equal_without_source: True`.
- Post-cure, two distinct roots at cure head `c3dc53e`: cured JSON
  **byte-identical** (`e44eb5b2…`), `source == "scripts/garnet_launch_readiness_status.py"`,
  `isabs == False`, no backslash.
- **NATIVE-WINDOWS POSIX SPELLING — PENDING PREREQUISITE, NOT CLAIMED.**
  `.as_posix()` is load-bearing for the Windows NUC seat; this macOS run cannot
  prove the emitted Windows spelling. Verdict 09 Leg 4(1) requires native-Windows
  evidence that the value is exactly `scripts/garnet_launch_readiness_status.py`
  (forward slashes) BEFORE slice 5 consumes a Windows regeneration. That is an
  outstanding NUC leg.

### Trap 2 — real state sensitivity (`test_trap2_…`, GREEN)
A real readiness change through the existing `Dependencies` seam (a wasm
blocker) moves the serialized artifact; `source` stays constant; the original
state restores cleanly (immutable `replace`).

### Trap 3 — no collateral reporter semantics (evidence 95)
At identical dependency state, pre-cure vs post-cure `--format json` differ by
**exactly one line** (`source`); parsed `changed keys: ['source']`; schema
`garnet.launch_readiness/v1` unchanged; key present and ordering unchanged.
`--format human` and `--format markdown` are **byte-identical** pre/post
(`376f4113…`, `03e5a1fe…`) — the renderers never read `.source`.

### Trap 4 — digest determinism without exclusion (evidence 96)
Substituting only the cured `08` blob (OID `44cae2519…`, identical across both
roots) into the tracked set at the cure head yields the **same** simulated pair
`6f8eb413…/1544` from both clone roots (RED had two divergent trees
`824e1e8f…` vs `b232031b…`). All **31** tracked `ops/lane0/` paths remain
INCLUDED; the frozen tuple is exactly
`(b"ops/lane2b/", b"proofs/", b"F_Project_Management/W_TRUST/", b"ops/lane1/")`
plus `REPORTER_PATH = b"scripts/smoke_garnet_minimum_shelf.py"`. No `ops/lane0/`
exclusion, generalized `ops/` predicate, or equivalent bypass exists. (The
simulated `6f8eb413…` is a determinism demo at the cure-head tree, NOT a
slice-5 landed pin, and differs from the RED's `3aa7ecc6…` only because the
base tree changed — see evidence 96 §3.)

## 4. Full suite — one UNCHANGED baseline failure

`python3 scripts/test_garnet_launch_readiness_status.py -v` → 41 tests, one
failure: `test_tracked_ledger_matches_renderer_byte_for_byte` (verdict 09 F3 —
stale `live_wasm_playground` blocker lines in the tracked Markdown ledger).
This is **not** caused by and **not** fixed by this cure: the Markdown bytes are
identical pre/post (§ trap 3), so the failure existed identically before the
cure (37 tests then; 41 now, +4 traps). A ledger regeneration is slice-5 scope,
not absorbed here.

## 5. Denominators — honest Phase-0 values, unchanged

Today's producer still derives **3/6 = 50.0%** launch-critical and
**3/8 = 37.5%** ledger. This cure changes no gate/denominator/dependency
(verdict 09 Leg 5). 4/6 is not chased; U-36 (shelf-gate/WV-acceptance wiring)
is registered for its own reviewed lane and is untouched.

## 6. Requested from the reviewer

1. Independent verification of the exact cure diff (only line 509; key
   retained; value exactly `scripts/garnet_launch_readiness_status.py`).
2. Independent re-execution of the four traps, including a re-derivation of the
   product pairs and the cross-root determinism.
3. Confirmation that the F3 ledger failure is an unchanged baseline the cure
   correctly did not absorb.
4. Confirmation that the native-Windows POSIX-spelling evidence is an
   outstanding prerequisite (a NUC leg) that this macOS packet does not and
   must not claim as satisfied.
5. Confirmation that the frozen exclusion and `ops/lane0/` inclusion are
   untouched, and that `08-launch-readiness.json` is correctly NOT regenerated
   in this cure.

## STOP

Per ceremony the implementer stops here. No cure head is approved until an
independent implementation verdict names one. The reviewable cure head is
`c3dc53ee4169ae879647fcb74e7bb524488653ed` (product pair `0b6239c2…/1544`).
