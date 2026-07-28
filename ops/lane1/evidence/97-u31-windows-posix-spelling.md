# Evidence 97 — U-31 native-Windows POSIX-spelling proof (trap-1 strengthening)

- Date: 2026-07-28
- Mandate: Verdict 09 native-platform prerequisite; Verdict 10 F2 / Leg 7
  ("the native-Windows leg must run at the exact approved head and prove the
  emitted JSON value is byte-for-byte `scripts/garnet_launch_readiness_status.py`")
- Executing agent (true identity, per the identity-check rule): Claude Code on
  **Claude Fable 5** (`claude-fable-5`, Anthropic) — not Codex, not the
  Opus 4.8 / Opus 5 seats that appear elsewhere in this lane's history.
- Machine: NUCBOX_M2PRO_S · Windows 11 Pro build 10.0.26200 · native NTFS
  filesystem, no VM, no WSL.
- Python: 3.11.15, invoked as `python -I` (no `python3` shim on this host;
  same `-I` isolation as the canonical spelling).
- Toolchain note: probe is Python-only; no cargo invocation was required.

## Boot facts

- `git config --global core.autocrlf false` verified BEFORE cloning and
  effective (`false`) inside both roots.
- Root 1 `C:\g1`: fresh clone, `origin` = `Island-Dev-Crew/garnet` (U-34
  ordering), `fork` = `Navigata1/garnet`; fork lane branch fetched by the
  single explicit refspec `+refs/heads/mission/l1-reconcile-post-activation:refs/review/l1`.
- Root 2 `C:\g2`: `git clone --no-hardlinks C:\g1 C:\g2`.
- Both roots: space-free paths, outside any OneDrive/sync-managed directory;
  **zero `refs/pull/*` after all fetches** (verified per root);
  worktrees clean; both detached at the exact approved head.

## Head and freeze verification (STOP-check)

- HEAD (both roots): `c3dc53ee4169ae879647fcb74e7bb524488653ed`
  ("fix(lane1): U-31 cure — emit repo-relative POSIX producer path in 08.source")
- Product pair recomputed per root with the repository's own
  `garnet_content_provenance.tracked_content_digest`:

```text
root1  0b6239c204f893ed627f59c937d83b6cd5c56832bc0260e31a63775d8d568158 / 1544
root2  0b6239c204f893ed627f59c937d83b6cd5c56832bc0260e31a63775d8d568158 / 1544
```

Both equal the Verdict-10-mandated pair. No STOP condition arose.

## Probe runs (read-only; no repository state changed)

Command in each root, stdout captured raw to files outside both repositories:

```text
python -I scripts/garnet_launch_readiness_status.py --format json
```

| root | exit | stderr bytes | stdout bytes | stdout SHA-256 |
|------|------|--------------|--------------|----------------|
| `C:\g1` | 0 | 0 | 5391 | `6d1fb599f35a9cdc4e63aac36734bf819c3703fd65914298c1a5a40fbea70797` |
| `C:\g2` | 0 | 0 | 5391 | `6d1fb599f35a9cdc4e63aac36734bf819c3703fd65914298c1a5a40fbea70797` |

- Byte-level comparison (`cmp root1.json root2.json`): **identical**.
- (The 5391-byte size legitimately differs from the reviewer's macOS 5268-byte
  capture: JSON content includes machine-dependent readiness state. The
  requirement proven here is cross-ROOT byte identity on native Windows plus
  the exact `source` spelling, both of which hold.)

## Assertions (both roots, parsed from the captured bytes)

```text
source = 'scripts/garnet_launch_readiness_status.py'
schema = garnet.launch_readiness/v1
equals_expected      = True   (== "scripts/garnet_launch_readiness_status.py")
no_backslash         = True   ("\" does not occur in source)
no_drive_letter      = True   (no /^[A-Za-z]:/ prefix)
no_absolute_prefix   = True   (does not start with "/" or "\")
source_is_second_key = True   (key order preserved)
```

ALL ASSERTIONS PASS in both roots. On this real Windows filesystem,
`Path(__file__).resolve().relative_to(REPO_ROOT).as_posix()` emits exactly the
repo-relative POSIX spelling with forward slashes, no drive letter, and no
host-absolute prefix — the property Verdict 10 Leg 7 could only demonstrate
lexically on macOS.

## Scope honesty

- Probe only: no reporter regeneration was committed, no `08-*.json` ledger
  touched, no slice-5 action performed, no pin rebind, no PR, no record.
- U-31's slice-5 consumption gate is now evidence-satisfied for the POSIX
  spelling leg only; everything else returns to the implementer seat and Jon.
