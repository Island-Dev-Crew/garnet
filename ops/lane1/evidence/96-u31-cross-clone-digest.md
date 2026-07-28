# U-31 cure — trap 1 (cross-clone determinism) & trap 4 (digest without exclusion)

- Recorded: 2026-07-28, at cure head
  `c3dc53ee4169ae879647fcb74e7bb524488653ed`.
- Implementer: **Claude Code — Opus 5 (`claude-opus-5`)**, on
  **`Hughs-MacBook-Pro.local`** (Darwin 25.5.0 / arm64); Python 3.14.5;
  git 2.50.1; `core.autocrlf=false`. Fleet-fork identity Jon Isaac
  `<Navigata1@gmail.com>`.
- All digests recomputed on this seat. Numbers are NOT transcribed from the
  verdict, RED, or any prior summary; where a value legitimately differs from
  the RED it is because the base tree changed (see §3).

## Trap 1 — clone-path determinism (live, cured reporter)

Two distinct absolute clone roots, both checked out at the cure head
`c3dc53e`, same readiness state:

| seat | absolute root | cured `--format json` sha256 | `.source` |
|---|---|---|---|
| A | `/Users/IDC2.5/garnet-lane1-fresh` | `e44eb5b22cdb4a85379d60a553b51b0c77fc270a18ccce24432fa3ea5b60203d` | `scripts/garnet_launch_readiness_status.py` |
| B | `/private/tmp/claude-502/-Users-IDC2-5-Desktop-Garnet/c7ff1606-f08e-49d9-8af5-a866423be499/scratchpad/u31/B_green` (`git clone --no-hardlinks`) | `e44eb5b22cdb4a85379d60a553b51b0c77fc270a18ccce24432fa3ea5b60203d` | `scripts/garnet_launch_readiness_status.py` |

- `diff curedA.json curedB.json` → **byte-identical**.
- `source == "scripts/garnet_launch_readiness_status.py"` on both.
- `os.path.isabs(source) == False`; no backslash in `source`.

The single seat-dependent byte identified in the RED is eliminated. The cured
JSON sha256 `e44eb5b2…` and the resulting blob OID `44cae2519dc4eaf2fd70aa64817ef7eeb075e8ba`
independently reproduce the RED (evidence 93) cure-shape prediction.

### Native-Windows POSIX spelling — PENDING PREREQUISITE (not satisfied here)

`.as_posix()` is load-bearing for the Windows NUC seat: on Windows,
`str(PureWindowsPath(...).relative_to(REPO_ROOT))` would emit backslashes and
re-introduce a seat-dependent byte. This macOS run **cannot** and **does not**
prove the native-Windows emitted spelling. Verdict 09 Leg 4(1) requires
native-Windows evidence that the emitted value is exactly
`scripts/garnet_launch_readiness_status.py` (forward slashes) **before slice 5
consumes a Windows regeneration**. That is an outstanding NUC leg; it is
flagged, not claimed.

## Trap 4 — digest determinism without exclusion (both roots)

Method: at the cure head, take the frozen-namespace-filtered tracked
`(path, blob-OID)` set from `git --no-replace-objects ls-tree -r -z HEAD`
(`garnet_content_provenance._tree_entries`), substitute ONLY the
`ops/lane0/evidence/08-launch-readiness.json` OID with the OID of each root's
own cured reporter output (`git hash-object --stdin`), and recompute the pair.
This simulates the lawful slice-5 regeneration WITHOUT committing it.

| quantity | seat A | seat B | equal |
|---|---|---|---|
| baseline pair (cure-head tree, no substitution) | `0b6239c204f893ed627f59c937d83b6cd5c56832bc0260e31a63775d8d568158` / 1544 | `0b6239c204f893ed627f59c937d83b6cd5c56832bc0260e31a63775d8d568158` / 1544 | yes |
| cured 08 blob OID | `44cae2519dc4eaf2fd70aa64817ef7eeb075e8ba` | `44cae2519dc4eaf2fd70aa64817ef7eeb075e8ba` | yes |
| simulated pair (08 substituted) | `6f8eb413d4672d6c6c3632ce5d7637a5cbd9682867a75bfbbec50d4cc6661b66` / 1544 | `6f8eb413d4672d6c6c3632ce5d7637a5cbd9682867a75bfbbec50d4cc6661b66` / 1544 | **yes** |
| tracked `ops/lane0/` paths | 31 | 31 | yes |

- The two roots certify the **same** tree after the lawful regeneration — the
  U-31 disease (RED: `824e1e8f…` vs `b232031b…`) is cured.
- All **31** tracked `ops/lane0/` paths remain INCLUDED (0 hit the frozen
  predicate). No `ops/lane0/` exclusion, no generalized `ops/` predicate, no
  equivalent bypass was introduced.
- `FROZEN_MUTABLE_PREFIXES == (b"ops/lane2b/", b"proofs/",
  b"F_Project_Management/W_TRUST/", b"ops/lane1/")` and
  `REPORTER_PATH == b"scripts/smoke_garnet_minimum_shelf.py"` — unchanged.

## §3 — Why the simulated pair differs from the RED's `3aa7ecc6…`

RED (evidence 93) computed the cured-shape simulation at tree `ef6d21b`; this
record computes it at the cure head `c3dc53e`, whose tracked tree additionally
contains the changed reporter blob and the added trap-test blob (both under
digest-included `scripts/`). The determinism claim under test is **equality
across the two clone roots at the same commit** — which holds
(`6f8eb413…` == `6f8eb413…`). Neither `6f8eb413…` nor `3aa7ecc6…` is a slice-5
landed pin; both are determinism demonstrations. The committed cure-head
product pair (no 08 regeneration) is `0b6239c2…/1544`.
