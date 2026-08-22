# WV-6 terminal freeze 7b ceremony record

## Convention and boundary

This successor uses the sibling name `terminal-freeze-7b`. The original
`ops/wv6-reaccept/terminal-freeze-7/` ceremony remains append-only and
byte-identical; `7b` identifies a new reviewed boundary within the same
Freeze-7 lineage without rewriting the first ceremony.

- Seat: OpenAI Codex on the native-Windows NUC.
- Checkout: `C:\garnet-freeze7-20260821` on NTFS, outside OneDrive.
- Process count: `457` at this task's cold start and `456` at red capture,
  below the directing baseline of approximately `482`; long gates were
  serialized.
- OneDrive was user-declared paused. One OneDrive process was present; this
  seat did not inspect the application's pause UI.
- `core.autocrlf=false` before the first repository-file read in the clone and
  reconfirmed before regeneration.
- Main base: `1d765cdb2e69bc097cd33db30f9919ad8e969208`.
- Regenerated-proof final cure / reviewed head:
  `8426ca761c696c3556190be77cce3e340250b5c7`.
- Reviewed tree: `601a368414762646ec9e5ad29b53736e20628474`.
- Natively computed pair:
  `6f2d5f0b2dff0bd800955e0a55b81f6d6f784d71240fe3c906e58a6a3ca8eec6 / 1646`.

The checked-in `tracked_content_digest` producer and an independent raw-byte
implementation over `git --no-replace-objects ls-tree -r -z` produced the same
pair. Two independent tree reads (`rev-parse <head>^{tree}` and
`show --format=%T`) also agreed. The count moved from 1643 to 1646 because the
three first Freeze-7 ceremony files are digest-domain content at this new
reviewed head; replacing the already tracked browser proof moved the digest
but added no path.

The containing commit is the commit that first introduces this record, so this
file intentionally does not embed its future SHA or tree.

## Superseded first Freeze-7 boundary and doctrine

State: `SUPERSEDED-WITH-PRESERVATION` before merge.

- Superseded reviewed head:
  `511e0fabad7335d14e972ffb968c7ac5e9b57ca8`.
- Superseded reviewed tree:
  `b7131198e99b01ab23aa75008e1e25acfca906c8`.
- Superseded pair:
  `87d5204c6b0c989d09e06dc176ab36096cd7a02fb31a2c7b600bcc1f7dd88058 / 1643`.
- Superseded record tip:
  `ba20593fab7d664483e9c357992c788df006ff73`.
- Cause: the Freeze-7 carry mandate omitted Freeze-6's browser-proof companion
  refresh `ba9fa6fe3b3581541cd66ef36334a7235e8e699e` after carrying the A3
  provenance refresh.

The red was ours, not the world's. The A3 Clippy cure moved
`garnet-memory-v0.3/src/episodic.rs`; the package producer hashes that source,
so the provenance `source_tree_sha256` and `provenance.json` bytes moved, which
made the committed browser proof's pin for that runtime file stale.

Doctrine registered by this supersession: a cure and its downstream proof
refresh are one atomic unit, and carry lists must be derived from the
predecessor boundary's full commit set, never hand-enumerated.

The old companion commit was not cherry-picked because it captured the
Freeze-6 tree. This boundary includes A4 and the first Freeze-7 records, so a
new real-browser capture at the current content was required.

## Locked browser-proof regeneration

The Studio dependency tree was absent, so the authorized
`npm ci --ignore-scripts` was required. It completed in `2551 ms`, installed
21 packages, and left `apps/garnet-studio/package-lock.json` byte-identical at
SHA-256
`e729ee69006fd3e6f5aa6171a93b8477ec51e96f12bb476c1a923df46aa93422`.
Node was `v22.22.2`, npm was `10.9.7`, and the locked install resolved both
`@playwright/test` and `playwright-core` to `1.61.1`. Three high-severity npm
audit findings were observed but not remediated outside this proof-refresh
authority.

The sanctioned command was:

```text
node scripts/smoke_garnet_playground_browser.mjs --proof F_Project_Management/LAUNCH/W_PLAY_BROWSER_PROOF.json
```

It launched real headless Chrome `151.0.7922.170`, passed in producer-measured
`2472 ms` (`2924 ms` process wall time), and served six committed requests.
The new proof was captured at `2026-08-22T00:21:37.782Z` and committed alone as
`8426ca761c696c3556190be77cce3e340250b5c7` with the causal chain named in the
commit message.

| Regeneration fact | Before | After |
|---|---|---|
| proof bytes / SHA-256 | `6398` / `85cfe5e7376156d2a83e2f30c0e70a4090d5c3d78dd9cd015437605317943779` | `6399` / `24d8ab436ccad574096cfee290261ca5c2febea3d53812b299fdbd07c392cf49` |
| screenshot bytes / SHA-256 | `51262` / `52c4e9878aa8354c21bd25bfd28982dbce323748a71b1e167f0bdde5119fe1ee` | identical |
| runtime-input aggregate | `1de71131204c1463694343026acb943e938bd28cddefd86a45f1a8a308363f15` | `8c43f5b86a2b8fdad43a20beb187c0ebdef4011ca675e8b9417c5ff7ca1f06f0` |
| `provenance.json` runtime pin | `156dccd2eb3515125cf400fe9879af8c4f68d35db09cd9759b1c6a10a7fb21a9` | `e015264242f501ea15efd00dfe9d0e2dd5b750ca0a2e78627ece372984478f2c` |
| package source aggregate | `c36f0e45ea14dbceaf4c91c969257271d5f7cb662d65fb6ce1d3eede2d7cb562` | `5b106c5eaf5c55d372b7b1eecea1a303dfda5b37edc24d1172b262da4087fc57` |

Only `F_Project_Management/LAUNCH/W_PLAY_BROWSER_PROOF.json` changed; the
screenshot reproduced byte-for-byte and was not committed again.

The locked downstream verification was green:

- `python3 -I scripts/test_garnet_wasm_readiness.py`: 13/13 in 2.651 s.
- `python3 scripts/garnet_wasm_readiness.py --gate`: exit 0,
  `browser_package_valid=true`, `browser_proof_valid=true`, and
  `browser_ready=true`.
- `python3 -I scripts/test_garnet_playground_browser_proof.py`: 3/3 in
  0.109 s.

## Occasioning change chain

The original ceremony remains the full review record for A1–A5. This successor
retains and names the complete chain:

| Slice | Freeze-7 commit(s) | Occasion |
|---|---|---|
| A1 | `ac2060b4b7d52d17856053b59c6203b7f67327af` | first additive ruleset evolution |
| A2 | `5e80ed2fd3c64144dccbaeb3eaf43f4e7e25e5ca`, `78336e428e591aa113af455631796a16fe53cbd4` | external registry yank exception and its reversal record |
| A3 | `e0ce757fd9c16c9f897447edfcd6310a03f78ff7`, `9cd8198a433529d3411b033d2473b628cf44da62` | floating-toolchain Clippy activation, pin cure, and Wasm provenance refresh |
| A4 | `2e7b263ec359a6f45413826e1f9602972f4b4ca5` | second additive ruleset evolution |
| A5 | `511e0fabad7335d14e972ffb968c7ac5e9b57ca8` | dogfood body-discipline findings registration |
| C2 | `8426ca761c696c3556190be77cce3e340250b5c7` | missing downstream browser-proof refresh regenerated at current content |

## Red-before and green-after

`01-f5-red-before.md` contains the four raw red streams verbatim with hashes
and encoding census. The sole finding was the regenerated proof's non-record
drift after the superseded reviewed head.

Exactly eight values moved during rebind: `REVIEWED_HEAD`, `REVIEWED_TREE`,
`EXPECTED_PRODUCT_CONTENT_SHA256`, and `EXPECTED_PRODUCT_PATH_COUNT` in the
Shelf reporter plus their four canonical `PROOF.json` mirrors. The historical
reviewed-tree pair and mirrors remained byte-identical. `.gitattributes`
remained SHA-256
`b2a14050a850391f8ed1c788f9a6a66155a423ebceb3bb4722478dcaec97dd1b`.

The rebound Shelf gate passed while the producer destination was absent, the
producer emitted exactly once, and the first post-emission acceptance gate was
green. `02-f5-green-after.md` contains those streams verbatim with hashes and
encoding census.

## Preserved predecessor bundle

State: `SUPERSEDED-WITH-PRESERVATION`.

- Preserved manifest SHA-256:
  `76c147ab9e49216a3f7ccca552062bf79d7cc8cad00e86839e7bfb043bbce04e`.
- Complete preserved bundle:
  `proofs/windows/launch-verification/wv6-terminal-freeze-20260821/superseded-511e0fabad7335d14e972ffb968c7ac5e9b57ca8/`.
- Raw moved predecessor:
  `C:\garnet-freeze7b-capture-20260821\superseded-live-working-copy-511e0fabad7335d14e972ffb968c7ac5e9b57ca8`.

| Preserved file | Bytes | SHA-256 |
|---|---:|---|
| `f1-canonical-reseal.txt` | 2801 | `02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7` |
| `mcp-session.input.hex` | 1073 | `64b2f3e15489b4f06e57a0c51afce960beda3edc9c8009d5f3a530d5d9f92638` |
| `mcp-session.output.hex` | 1797 | `15423fa5ba9697f59f877ea0a9afb3587230fc4d7c3182cee82ce709bbce4799` |
| `minimum-shelf-status.json` | 2070 | `02e2b46b3c2451126e2c297a049fd224725258a292fabfb63db5d96ad6b2db72` |
| `reporter-cross-checkout.txt` | 1660 | `a4f33e467349bf84a6c9fe04e7ddd486df335a96f2edda0b2cb371787d26425f` |
| `WV_ACCEPTANCE.json` | 1978 | `76c147ab9e49216a3f7ccca552062bf79d7cc8cad00e86839e7bfb043bbce04e` |

The live six-file bundle was enumerated from its manifest, copied, and matched
by filename, byte length, and SHA-256 before the original directory was moved
intact. No hand-authored file list governed custody.

## Fresh live bundle

| Live file | Bytes | SHA-256 |
|---|---:|---|
| `f1-canonical-reseal.txt` | 2801 | `02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7` |
| `mcp-session.input.hex` | 1073 | `64b2f3e15489b4f06e57a0c51afce960beda3edc9c8009d5f3a530d5d9f92638` |
| `mcp-session.output.hex` | 1797 | `15423fa5ba9697f59f877ea0a9afb3587230fc4d7c3182cee82ce709bbce4799` |
| `minimum-shelf-status.json` | 2070 | `5a4e41cd6f646ab87f2e60a2878d32f29c182cb3d2b999786f368f687e774aef` |
| `reporter-cross-checkout.txt` | 1660 | `a4f33e467349bf84a6c9fe04e7ddd486df335a96f2edda0b2cb371787d26425f` |
| `WV_ACCEPTANCE.json` | 1978 | `2e7fb2cfa5ceed3a1115405b0d0a17dced50a4fe7e43c94aaea4b07fd6b6bbc7` |

Fresh manifest SHA-256:
`2e7fb2cfa5ceed3a1115405b0d0a17dced50a4fe7e43c94aaea4b07fd6b6bbc7`.

## Acceptance succession

1. `2cb25d0b47f55c9dd987bf69fc8a97dade5a4767ef0beda0abc8616808dddc0c / 1605`.
2. `fd96e6d910180f5e33999fbd693ea211e336389a13535930d89b2a870ff54727 / 1606`.
3. `1b45387599223adbe8766ae2e04ddc70e4ecc359712a4867ef1973dac9bd42e4 / 1629`.
4. `8ea996129074e2e70c5ad2f9abd6082c85f7021e943dc017d3a9ba2859ed3ac7 / 1634`.
5. `32f3975537470cb1788acfb3365725c4fea208a049e20fb90b1c314b719cba06 / 1637`.
6. `056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc / 1640`.
7. `87d5204c6b0c989d09e06dc176ab36096cd7a02fb31a2c7b600bcc1f7dd88058 / 1643` (first Freeze-7 boundary, now superseded pre-merge).
8. `6f2d5f0b2dff0bd800955e0a55b81f6d6f784d71240fe3c906e58a6a3ca8eec6 / 1646` (Freeze-7b candidate).

## Superseded never-merged boundaries

- PR #525: pair
  `0513edb99391ad4dfe75dffe3618c8c001f877aa4d4f738709c4b506ec06b425 / 1643`;
  record tip `b31f273022d4e2b411f9650b5123543e7accfb41`.
- PR #526: pair
  `573523661f3569d925a05f4c95549582d765212c6ef650871432d35c00c12675 / 1643`;
  record tip `a88813360f1d550a4d209a4cea441fdb9cba1bd6`.
- PR #527: pair
  `449ba9b7aa948cb6fe5e1320385025ea563bc9e5c5ac69cc5ae6b6670bb2a9ee / 1643`;
  record tip `8af3f4f98ade1959b12938c1139ded81a9b2cf67`.

These three never reached main, remain outside acceptance succession, and are
unchanged by this re-freeze. The first Freeze-7 PR #528 boundary also never
reached main; it is superseded within the same branch by the head movement
recorded here.

## U-55 custody timeline

- Custody breath began: `2026-08-22T00:24:49.9027449Z`.
- Producer-censused preservation copy verified:
  `2026-08-22T00:24:49.9267659Z`.
- Original live directory moved intact:
  `2026-08-22T00:24:49.9516832Z`.
- Rebound Shelf gate green with the producer destination absent:
  `2026-08-22T00:25:33.7013472Z`.
- Sole producer emission: `2026-08-22T00:25:47.7945359Z` through
  `2026-08-22T00:25:49.0460366Z`.
- First post-emission WV-6 acceptance gate green:
  `2026-08-22T00:26:04.7601737Z`.
- Fresh manifest and complete bundle census:
  `2026-08-22T00:26:13.5686726Z`.

Staged regular-file hashes are computed after this record is complete and
immediately before the containing commit. Post-commit U-56 verification must
use the checked-in producer's exact changed-path census from the reviewed head
to the containing tip and reject every non-record path; the walk is never
hand-listed.

## Stop boundary

The accepted pair remains bound to C2 reviewed head
`8426ca761c696c3556190be77cce3e340250b5c7`. The reporter, four `PROOF.json`
mirrors, fresh live bundle, preserved predecessor bundle, and these three
ceremony records are established post-acceptance record-class paths.

No Air confirmation, structured review record, carrier approval, U-17
readback, merge, tag, release, token action, or workflow rerun is part of this
ceremony.
