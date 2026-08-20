# Terminal freeze 5: F5 green after

## Boundary

- Execution and frozen head: `eecd6a407b64b8b83bd195cf049d3ebd1953da05`.
- Execution tree: `001c7139daad866f5b8a70090930e31765df4cef`.
- Accepted pair: `573523661f3569d925a05f4c95549582d765212c6ef650871432d35c00c12675 / 1643`.
- Raw capture root: `C:\garnet-freeze5-redo-capture-20260820-0ab08c84\freeze5`.

The Shelf gate ran after the eight moving pins were rebound and before the
single producer invocation. The WV-6 gate ran after the fresh six-file bundle
was emitted into the vacant live destination.

Every non-empty raw stream passed strict UTF-8 decoding, had no BOM, and used
only CRLF line endings. Empty streams contain zero bytes. The fenced rendering
preserves decoded characters and line order. This Markdown record is UTF-8
without BOM using LF.

| Command | Exit | Stream | Bytes | SHA-256 | EOL census |
|---|---:|---|---:|---|---|
| `python -I scripts/smoke_garnet_minimum_shelf.py --gate` | 0 | stdout | 2071 | `d9ac8f885767ec5fc8a082790b9d641203b47db6f6a620b8d42ea2974ce2ef86` | 1 CRLF; 0 bare LF |
| same | 0 | stderr | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | empty |
| `python -I scripts/garnet_wv_acceptance_status.py --wv WV-6 --gate` | 0 | stdout | 627 | `c2e01b84c1d1c6f198c6349f97e7c68fac1ee42d7beb3eed066d38356f4595a4` | 16 CRLF; 0 bare LF |
| same | 0 | stderr | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | empty |

The producer command `python -I scripts/smoke_garnet_minimum_shelf.py --emit-wv6`
ran exactly once and exited 0. Its stdout SHA-256 is
`d9ac8f885767ec5fc8a082790b9d641203b47db6f6a620b8d42ea2974ce2ef86`; stderr SHA-256 is
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

## Minimum Shelf stdout

```json
{"artifact_sha256":{".gitattributes":"b2a14050a850391f8ed1c788f9a6a66155a423ebceb3bb4722478dcaec97dd1b","examples/minimum-shelf-flagship/SHELF_PACKAGE.json":"dc12370de087c8beeb8885b4d51d4e37b8b02b8d7af90a3957bcb7c1e111b618","examples/minimum-shelf-flagship/tool.garnet":"25ebd3dc02c8ab7d17e343c867e6becda9918c84a567f56f5b14ba4aba08a967","examples/minimum-shelf-flagship/tool.seal.json":"526ac0f63f8ac487f6c38fd947defe1b80e1c3c14d80e3cfb38f5a66355b9cbd","garnet-interp-v0.3/src/prelude.rs":"784864f2ccf16eb5494a39e42c5c4bf15b6ac0e09b7f729ca8ecc0e23ad81c62","ops/lane2b/evidence/10-f1-canonical-reseal-green.txt":"02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7","proofs/minimum-shelf/lane2b/mcp-session.input.hex":"2328d55497368e0a351cfbd0e5421ab46c3826abf4f313c1a497e95a9fbfd769","proofs/minimum-shelf/lane2b/mcp-session.output.hex":"dac0eea7138d0f58865eecefc8db0c64490605068b956f757f420d6b284ba15f"},"checks":{"core-ring-tier1":true,"deterministic-shelf-reporter":true,"mcp-raw-byte-stdio":true,"reject-without-seal":true,"sealed-baseline":true},"current_commit":"eecd6a407b64b8b83bd195cf049d3ebd1953da05","current_tree":"001c7139daad866f5b8a70090930e31765df4cef","findings":[],"implementer":"Codex GPT-5.6 Sol","landed_main_commit":null,"ok":true,"package":"examples/minimum-shelf-flagship","platform":"windows","product_content_sha256":"573523661f3569d925a05f4c95549582d765212c6ef650871432d35c00c12675","product_path_count":1643,"request_frame_count":5,"response_frame_count":4,"reviewed_head":"eecd6a407b64b8b83bd195cf049d3ebd1953da05","reviewed_tree":"001c7139daad866f5b8a70090930e31765df4cef","reviewed_tree_product_sha256":"1e6692175ea8fe2dd5b04fad4a492dc8ce48767dd07d88fd11a0847ce96749d5","reviewer":"Claude Code Fable 5","ring":"core","schema":"garnet.minimum_shelf_status/v2","scope_limits":["one Garnet-owned local package","Core Ring Tier 1 only","raw-byte stdio only","no hosted registry or network transport","reviewed local content, not signer identity"],"state":"accepted","tier":1,"tool":"garnet.core.double","unsigned_predicate":true}
```

## Minimum Shelf stderr

```text

```

## WV-6 stdout

```json
{
  "artifact_count": 5,
  "contract_base_main_sha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
  "evidence_destination": "proofs/windows/launch-verification/wv6-minimum-shelf/",
  "findings": [],
  "landed_main_sha": null,
  "ok": true,
  "passed_check_count": 5,
  "product_content_sha256": "573523661f3569d925a05f4c95549582d765212c6ef650871432d35c00c12675",
  "required_check_count": 5,
  "reviewed_head_sha": "eecd6a407b64b8b83bd195cf049d3ebd1953da05",
  "reviewed_tree_sha": "001c7139daad866f5b8a70090930e31765df4cef",
  "schema": "garnet.wv_acceptance_status/v2",
  "state": "accepted",
  "wv": "WV-6"
}
```

## WV-6 stderr

```text

```
