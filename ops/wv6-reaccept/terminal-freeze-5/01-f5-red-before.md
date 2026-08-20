# Terminal freeze 5: F5 red before

## Boundary

- Execution head: `2ca3d81129d22ab827b7574ff992d7c262aaf9e2`.
- Execution tree: `fc571865bc6c548d9b7bc02f79a1b2d2517d9f2b`.
- Natively computed product pair: `0513edb99391ad4dfe75dffe3618c8c001f877aa4d4f738709c4b506ec06b425 / 1643`.
- Prior accepted pair: `056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc / 1640`.
- Raw capture root: `C:\garnet-freeze5-capture-20260820-4bfc6d21\red-before`.

The ruleset pin refresh is digest-domain content. Before rebind, both
reporters failed closed on the product digest and path-count movement.

Raw files are the byte-level evidence. Every stream passed strict UTF-8
decoding, had no BOM, and used only CRLF line endings. The fenced rendering
preserves decoded characters and line order. This Markdown record is UTF-8
without BOM using LF.

| Command | Exit | Stream | Bytes | SHA-256 | EOL census |
|---|---:|---|---:|---|---|
| `python -I scripts/smoke_garnet_minimum_shelf.py --gate` | 1 | stdout | 2391 | `e553eaa404cefa87ed419b0493e1df723221a024f449008f5303a678c6d5e32f` | 1 CRLF; 0 bare LF |
| same | 1 | stderr | 343 | `b30e1fd5142f996aa0fe3a9d47f4c1658bd80121402afb28d800af39f9181c6c` | 1 CRLF; 0 bare LF |
| `python -I scripts/garnet_wv_acceptance_status.py --wv WV-6 --gate` | 1 | stdout | 856 | `fd59cc3a3da03baa15a08b11d820dfc4e732262fd01e8c65dbe0f5874b1ef5d0` | 19 CRLF; 0 bare LF |
| same | 1 | stderr | 242 | `a7df856850b70b4dad95e6e1bd7e36dd7690609004d371ce195f34c77928c548` | 1 CRLF; 0 bare LF |

## Minimum Shelf stdout

```json
{"artifact_sha256":{".gitattributes":"b2a14050a850391f8ed1c788f9a6a66155a423ebceb3bb4722478dcaec97dd1b","examples/minimum-shelf-flagship/SHELF_PACKAGE.json":"dc12370de087c8beeb8885b4d51d4e37b8b02b8d7af90a3957bcb7c1e111b618","examples/minimum-shelf-flagship/tool.garnet":"25ebd3dc02c8ab7d17e343c867e6becda9918c84a567f56f5b14ba4aba08a967","examples/minimum-shelf-flagship/tool.seal.json":"526ac0f63f8ac487f6c38fd947defe1b80e1c3c14d80e3cfb38f5a66355b9cbd","garnet-interp-v0.3/src/prelude.rs":"784864f2ccf16eb5494a39e42c5c4bf15b6ac0e09b7f729ca8ecc0e23ad81c62","ops/lane2b/evidence/10-f1-canonical-reseal-green.txt":"02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7","proofs/minimum-shelf/lane2b/mcp-session.input.hex":"2328d55497368e0a351cfbd0e5421ab46c3826abf4f313c1a497e95a9fbfd769","proofs/minimum-shelf/lane2b/mcp-session.output.hex":"dac0eea7138d0f58865eecefc8db0c64490605068b956f757f420d6b284ba15f"},"checks":{"core-ring-tier1":true,"deterministic-shelf-reporter":false,"mcp-raw-byte-stdio":true,"reject-without-seal":true,"sealed-baseline":true},"current_commit":"2ca3d81129d22ab827b7574ff992d7c262aaf9e2","current_tree":"fc571865bc6c548d9b7bc02f79a1b2d2517d9f2b","findings":["product content digest does not match reviewed bytes","product path count does not match reviewed index","product content digest mismatch (0513edb99391ad4dfe75dffe3618c8c001f877aa4d4f738709c4b506ec06b425 != 056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc)","product path count mismatch (1643 != 1640)"],"implementer":"Codex GPT-5.6 Sol","landed_main_commit":null,"ok":false,"package":"examples/minimum-shelf-flagship","platform":"windows","product_content_sha256":"0513edb99391ad4dfe75dffe3618c8c001f877aa4d4f738709c4b506ec06b425","product_path_count":1643,"request_frame_count":5,"response_frame_count":4,"reviewed_head":"218047425fd6871d6cb3ad526ef77e3f4df4c669","reviewed_tree":"9cbd7be6810f1f2852d4908fecd64cd66f75fa9c","reviewed_tree_product_sha256":"1e6692175ea8fe2dd5b04fad4a492dc8ce48767dd07d88fd11a0847ce96749d5","reviewer":"Claude Code Fable 5","ring":"core","schema":"garnet.minimum_shelf_status/v2","scope_limits":["one Garnet-owned local package","Core Ring Tier 1 only","raw-byte stdio only","no hosted registry or network transport","reviewed local content, not signer identity"],"state":"partial","tier":1,"tool":"garnet.core.double","unsigned_predicate":true}
```

## Minimum Shelf stderr

```text
Minimum Shelf gate FAILED: product content digest does not match reviewed bytes; product path count does not match reviewed index; product content digest mismatch (0513edb99391ad4dfe75dffe3618c8c001f877aa4d4f738709c4b506ec06b425 != 056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc); product path count mismatch (1643 != 1640)
```

## WV-6 stdout

```json
{
  "artifact_count": 5,
  "contract_base_main_sha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
  "evidence_destination": "proofs/windows/launch-verification/wv6-minimum-shelf/",
  "findings": [
    "product content digest mismatch (0513edb99391ad4dfe75dffe3618c8c001f877aa4d4f738709c4b506ec06b425 != 056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc)",
    "product path count mismatch (1643 != 1640)"
  ],
  "landed_main_sha": null,
  "ok": false,
  "passed_check_count": 5,
  "product_content_sha256": "056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc",
  "required_check_count": 5,
  "reviewed_head_sha": "218047425fd6871d6cb3ad526ef77e3f4df4c669",
  "reviewed_tree_sha": "9cbd7be6810f1f2852d4908fecd64cd66f75fa9c",
  "schema": "garnet.wv_acceptance_status/v2",
  "state": "partial",
  "wv": "WV-6"
}
```

## WV-6 stderr

```text
WV-6 acceptance gate PARTIAL: product content digest mismatch (0513edb99391ad4dfe75dffe3618c8c001f877aa4d4f738709c4b506ec06b425 != 056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc); product path count mismatch (1643 != 1640)
```
