# U-31 RED — machine-dependent bytes in `08.source` poison the certified tree

- Recorded: 2026-07-28T03:17Z, at HEAD `f3876c5a78beb31d6cfe8cc5a115bf264af8008f`
  (tree `006f06f2b3af451d292116de5e59a14735d4ec6b`)
- Implementer: Claude Code Opus 5 (`claude-opus-5`) — Jon's macOS seat
  (macOS 26.5 / Darwin 25.5.0 / arm64), fleet-fork identity Jon Isaac
  `<Navigata1@gmail.com>`; Python 3.14.5; git 2.50.1 (Apple Git-155);
  `core.autocrlf=false` verified global before any run
- Finding: U-31 as registered — verdict 01 F4 (`ops/lane1/review/01-verdict.md`,
  "absolute path in `08.source`"), reproduced as verdict 02 F5
  (`equal_without_source=true`); slice-5 blocker per BLOCKED.md item 3
- Discipline: RED recorded BEFORE any cure. No cure is implemented in this
  commit or anywhere on this branch. `ops/lane1/**` is product-digest-excluded
  (U-35, verdict 08), so this evidence moves no product byte.

## 1. The machine byte already in the certified tree

`ops/lane0/evidence/08-launch-readiness.json` line 3, committed, digest-included:

```
"source": "/private/tmp/garnet-l0-truth-freeze-231aefa/scripts/garnet_launch_readiness_status.py",
```

That is the truth-freeze seat's absolute path. Producer:
`scripts/garnet_launch_readiness_status.py:509`:

```python
source=str(Path(__file__).resolve()),
```

## 2. Regeneration on this machine (clone A)

```
cd /Users/IDC2.5/garnet-lane1-fresh   # clone A, HEAD f3876c5
python3 scripts/garnet_launch_readiness_status.py --format json > A.json   # exit 0, stderr empty
```

- A.json sha256 `3f902587e2f40cd819a74710da6ae56cfb6125392508e962a0c62968d21ae96d`
- A.json `.source` = `/Users/IDC2.5/garnet-lane1-fresh/scripts/garnet_launch_readiness_status.py`

Committing this regeneration would stamp THIS seat's absolute path into the
certified tree, replacing the freeze seat's.

## 3. Second seat divergence (clone B — same machine, same commit, different path)

```
git clone --no-hardlinks /Users/IDC2.5/garnet-lane1-fresh <scratch>/garnet-B
cd <scratch>/garnet-B && git checkout f3876c5a78beb31d6cfe8cc5a115bf264af8008f
python3 scripts/garnet_launch_readiness_status.py --format json > B.json   # exit 0, stderr empty
```

- B.json sha256 `2440a196c253e7b1d90c7ddbeb4a721019a596ebc92e6581c5fe341dd512d17c`
- B.json `.source` = `<scratch>/garnet-B/scripts/garnet_launch_readiness_status.py`
- `diff A.json B.json` → exactly one hunk: line 3, `source`
- Canonical comparison after deleting only `source`: **`equal_without_source: True`**
  (verdict 02 F5 reproduced at the current tip)

## 4. Digest-level poison (the U-31 crux)

Independent replay of the documented construction
(`mode SP type SP oid TAB path` records from
`git --no-replace-objects ls-tree -r -z HEAD`, frozen exclusions applied,
sort by raw path bytes, SHA-256 over `path NUL blob-OID LF`):

| scenario | product_content_sha256 | count |
|---|---|---|
| baseline at HEAD `f3876c5` | `e89cb2996def7eec8e419dac235d55f985734e84be6f991c565da35d46feb64f` | 1544 |
| seat A commits its regeneration (blob `47a7295b…`) | `824e1e8faeb11d27c558d803b048012da66310ef503f7f754897b3458e622bad` | 1544 |
| seat B commits its regeneration (blob `66062687…`) | `b232031b5f213513f29ba5293f47c641a9447b4946a3664ff075afb0c4ba1360` | 1544 |

Baseline reproduces the verdict-08 pin exactly (module
`tracked_content_digest` agrees from both clones). Seats A and B performing
the **identical lawful act** would certify **different trees**. `ops/lane0/`
is digest-INCLUDED (31 tracked paths) and stays so: only its bytes are
seat-dependent, which is the disease.

## 5. Committed-vs-regenerated separation (two axes, kept distinct)

`diff ops/lane0/evidence/08-launch-readiness.json A.json` → exactly two hunks:

1. `source` — U-31 (seat-dependent byte; the subject of this RED);
2. `live_wasm_playground.blockers` `[3 items] → []` — real dependency drift
   since freeze base `231aefa` (state remains `"remaining"`). This is lawful
   evidence evolution a slice-5 regeneration would legitimately publish; it is
   NOT part of U-31 and is reported here only so the axes cannot be conflated.

Gate id/state tuple is unchanged between committed and regenerated output:
derived denominators are still 3/6 accepted (launch-critical) and 3/8
(ledger) at today's producer.

## 6. Cure-shape simulation (packet evidence only — nothing implemented)

Normalizing `source` to the repository-relative producer path
`scripts/garnet_launch_readiness_status.py` in both seats' outputs
(re-serialized exactly as the emitter does, `json.dumps(..., indent=2) + "\n"`):

- A-cured sha256 = B-cured sha256 =
  `e44eb5b22cdb4a85379d60a553b51b0c77fc270a18ccce24432fa3ea5b60203d` —
  **byte-identical across seats**
- single blob OID `44cae2519dc4eaf2fd70aa64817ef7eeb075e8ba`
- single would-be pair `3aa7ecc6d7f4a2520235c4f80bada08afe98b4fc0a37c9a15edf297cfb043650` / 1544
  — one value regardless of seat (computed at this tip with today's dependency
  truth, including the axis-2 blocker drift; this demonstrates determinism, it
  does NOT pin the slice-5 landed digest)
