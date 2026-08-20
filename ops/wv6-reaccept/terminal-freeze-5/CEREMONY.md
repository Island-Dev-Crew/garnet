# WV-6 terminal freeze 5 ceremony record

## Seat and frozen boundary

- Seat: OpenAI Codex on the native-Windows NUC.
- Checkout: `C:\garnet-freeze5-redo-20260820-0ab08c84`.
- Host process count at cold-start preflight: `562`; pre-custody census:
  `559`.
- OneDrive process count: `0`.
- `core.autocrlf=false`.
- Commit identity: `Jon Isaac <Navigata1@gmail.com>`.
- Branch base: `1d765cdb2e69bc097cd33db30f9919ad8e969208` / tree
  `a7de550d8f853c6a50d69f87d3fb6d5919cfef29`.
- Ruleset-pin cure A1: `7b01726fd1ab9ca9194c9203b9d6d418d9d922b1`.
- Registry-yank cure A2 and frozen head: `eecd6a407b64b8b83bd195cf049d3ebd1953da05`.
- Frozen tree: `001c7139daad866f5b8a70090930e31765df4cef`.
- Natively computed pair: `573523661f3569d925a05f4c95549582d765212c6ef650871432d35c00c12675 / 1643`.

The containing commit is identified as the commit that first introduces this
`CEREMONY.md`; this record does not embed its future commit SHA or tree.

## Occasioning changes

A1 updates the tracked 31-context and activation ruleset documents for two
additive GitHub API fields and refreshes their canonical digest pins. The
review statement is: the two additive fields are reviewed non-weakening
(`dismissal_restriction` disabled with no allowed actors; no required
reviewers); the live governance posture is unchanged from the originally
reviewed contract. The five A1 suites passed 33/33 governance, 6/6 identity,
24/24 transport, 4/4 base-controlled, and 10/10 activation.

A2 retains global `yanked = "deny"` and exempts exactly
`arrayref@0.3.9`. A fresh task-specific registry cache reproduced
`error[yanked]` and `advisories FAILED` before the exception. Crates.io
reported `0.3.5` through `0.3.9` yanked with no yank messages; the newest
non-yanked `0.3.4` cannot satisfy BLAKE3 1.8.4's `^0.3.5` requirement.
The captured RustSec database names no arrayref advisory but predates the
yanks, so it does not establish their reason. Evidence 100 registers removal
when BLAKE3's dependency moves. `Cargo.lock` remained byte-identical at
SHA-256 `01b8986b1cee0ef6a53ac439bd018b54fc1dca825a8f845a259ed8001e6715fa`.

The CI-matching cargo-deny 0.20.2 replay then produced:

```text
advisories ok
advisories ok, bans ok, licenses ok, sources ok
```

Both exited 0. The full workspace/all-targets/all-features build exited 0.
The four native acceptance predicates passed 3/3, 2/2, 1/1, and 6/6.

## Pin census and producer

The checked-in procedure names nine pins. Eight values moved: four candidate
constants in `scripts/smoke_garnet_minimum_shelf.py` and four matching
mirrors in `proofs/minimum-shelf/lane2b/PROOF.json`. The
`.gitattributes` pin was recomputed as
`b2a14050a850391f8ed1c788f9a6a66155a423ebceb3bb4722478dcaec97dd1b`
and remained byte-identical. Historical reviewed-tree digest and count values
remained `1e6692175ea8fe2dd5b04fad4a492dc8ce48767dd07d88fd11a0847ce96749d5 / 1527`.

The producer ran exactly once into the vacant destination. The fresh
`WV_ACCEPTANCE.json` reports schema
`garnet.wv_acceptance_evidence/v2`, state `evidence_complete`, binds the
frozen head/tree/pair above, and has SHA-256
`143a9588642c8e8b5c8946763fffb57a48edd39964b032a4ccab00837449bce3`.

| Fresh live file | Bytes | SHA-256 |
|---|---:|---|
| `f1-canonical-reseal.txt` | 2801 | `02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7` |
| `mcp-session.input.hex` | 1073 | `64b2f3e15489b4f06e57a0c51afce960beda3edc9c8009d5f3a530d5d9f92638` |
| `mcp-session.output.hex` | 1797 | `15423fa5ba9697f59f877ea0a9afb3587230fc4d7c3182cee82ce709bbce4799` |
| `minimum-shelf-status.json` | 2070 | `00fbeafe77e22aacf6ee84a9d4cb838ea01085ca07f2688ecf0f885d74ccf108` |
| `reporter-cross-checkout.txt` | 1660 | `a4f33e467349bf84a6c9fe04e7ddd486df335a96f2edda0b2cb371787d26425f` |
| `WV_ACCEPTANCE.json` | 1978 | `143a9588642c8e8b5c8946763fffb57a48edd39964b032a4ccab00837449bce3` |

## Supersession with preservation

State: `SUPERSEDED-WITH-PRESERVATION`.

- Superseded reviewed head: `218047425fd6871d6cb3ad526ef77e3f4df4c669`.
- Superseded reviewed tree: `9cbd7be6810f1f2852d4908fecd64cd66f75fa9c`.
- Superseded pair: `056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc / 1640`.
- Preserved `WV_ACCEPTANCE.json` SHA-256:
  `e4f5ebdf9f3936765bf20070837f7953538d6a60024da03f755cc87e8c792579`.
- Complete preserved bundle: `proofs/windows/launch-verification/wv6-terminal-freeze-20260820/superseded-218047425fd6871d6cb3ad526ef77e3f4df4c669/`.
- Raw moved predecessor copy: `C:\garnet-freeze5-redo-capture-20260820-0ab08c84\superseded-live-working-copy-218047425fd6871d6cb3ad526ef77e3f4df4c669`.

The producer-censused six-file bundle was copied byte-for-byte into the
committed preservation path. All six hashes matched before the original live
directory was moved intact to the raw-capture root.

| Preserved file | Bytes | SHA-256 |
|---|---:|---|
| `f1-canonical-reseal.txt` | 2801 | `02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7` |
| `mcp-session.input.hex` | 1073 | `64b2f3e15489b4f06e57a0c51afce960beda3edc9c8009d5f3a530d5d9f92638` |
| `mcp-session.output.hex` | 1797 | `15423fa5ba9697f59f877ea0a9afb3587230fc4d7c3182cee82ce709bbce4799` |
| `minimum-shelf-status.json` | 2070 | `a595cd0895a05427316fee734812adfc20e8b429f1b72a9ecb62401b651268a9` |
| `reporter-cross-checkout.txt` | 1660 | `a4f33e467349bf84a6c9fe04e7ddd486df335a96f2edda0b2cb371787d26425f` |
| `WV_ACCEPTANCE.json` | 1978 | `e4f5ebdf9f3936765bf20070837f7953538d6a60024da03f755cc87e8c792579` |

The accepted pair chain is:

1. `2cb25d0b47f55c9dd987bf69fc8a97dade5a4767ef0beda0abc8616808dddc0c / 1605`.
2. `fd96e6d910180f5e33999fbd693ea211e336389a13535930d89b2a870ff54727 / 1606`.
3. `1b45387599223adbe8766ae2e04ddc70e4ecc359712a4867ef1973dac9bd42e4 / 1629`.
4. `8ea996129074e2e70c5ad2f9abd6082c85f7021e943dc017d3a9ba2859ed3ac7 / 1634`.
5. `32f3975537470cb1788acfb3365725c4fea208a049e20fb90b1c314b719cba06 / 1637`.
6. `056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc / 1640`.
7. `573523661f3569d925a05f4c95549582d765212c6ef650871432d35c00c12675 / 1643`.

All accepted predecessors remain intact.

## Abandoned first attempt

PR #525's A1-only boundary is abandoned and outside acceptance succession. Its
content head was `2ca3d81129d22ab827b7574ff992d7c262aaf9e2`, tree
`fc571865bc6c548d9b7bc02f79a1b2d2517d9f2b`, pair
`0513edb99391ad4dfe75dffe3618c8c001f877aa4d4f738709c4b506ec06b425 / 1643`,
and record-only tip `b31f273022d4e2b411f9650b5123543e7accfb41`, tree
`c61fbfcd6709803ad1a8eaf52da4d59349c6e468`. It never became main's live
accepted predecessor. Its branch remains untouched; this ceremony neither
supersedes nor preserves it, and its pair is omitted from the accepted chain.

## Head-versus-tip boundary

The ceremony records belong to the established post-acceptance record class.
The accepted pair is frozen at `eecd6a407b64b8b83bd195cf049d3ebd1953da05`; the containing tip retains it
through the record-path verifier. The containing commit tree is not claimed
to hash to `573523661f3569d925a05f4c95549582d765212c6ef650871432d35c00c12675 / 1643`.

No structured review JSON, approval, merge, tag, release, or token action is
part of this ceremony. The correct pre-Air structured-record state is absent.
