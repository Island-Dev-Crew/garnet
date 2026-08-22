# Freeze-7b F5 green-after capture

After supersession-with-preservation and the eight authorized pin/mirror
movements, the Shelf gate exited 0 while the producer destination was still
vacant. The producer then emitted exactly once, and the first post-emission
WV-6 acceptance gate exited 0 at reviewed head
`8426ca761c696c3556190be77cce3e340250b5c7` / tree
`601a368414762646ec9e5ad29b53736e20628474` and pair
`6f2d5f0b2dff0bd800955e0a55b81f6d6f784d71240fe3c906e58a6a3ca8eec6 / 1646`.

Raw capture root: `C:\garnet-freeze7b-capture-20260821\green-after`.
Every stream passed strict UTF-8 decoding and was BOM-free. The nonempty
stdout streams used CRLF only; all stderr streams were empty.

| Command | Exit | Stream | Bytes | SHA-256 | BOM | CRLF | bare LF | bare CR |
|---|---:|---|---:|---|---|---:|---:|---:|
| `python -I scripts/smoke_garnet_minimum_shelf.py --gate` | 0 | stdout | 2071 | `11b6512d72fcc2a3d163bd7db07449005bc8ea7c6d195872700a7e6ac68213e5` | no | 1 | 0 | 0 |
| same | 0 | stderr | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | no | 0 | 0 | 0 |
| `python -I scripts/smoke_garnet_minimum_shelf.py --emit-wv6` | 0 | stdout | 2071 | `11b6512d72fcc2a3d163bd7db07449005bc8ea7c6d195872700a7e6ac68213e5` | no | 1 | 0 | 0 |
| same | 0 | stderr | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | no | 0 | 0 | 0 |
| `python -I scripts/garnet_wv_acceptance_status.py --wv WV-6 --gate` | 0 | stdout | 627 | `5fd2fe575e5a29a41850b6bc34a8e0fa161aa51593be623a79cd4346fca20f7f` | no | 16 | 0 | 0 |
| same | 0 | stderr | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | no | 0 | 0 | 0 |

## Minimum Shelf stdout — exit 0

```json
{"artifact_sha256":{".gitattributes":"b2a14050a850391f8ed1c788f9a6a66155a423ebceb3bb4722478dcaec97dd1b","examples/minimum-shelf-flagship/SHELF_PACKAGE.json":"dc12370de087c8beeb8885b4d51d4e37b8b02b8d7af90a3957bcb7c1e111b618","examples/minimum-shelf-flagship/tool.garnet":"25ebd3dc02c8ab7d17e343c867e6becda9918c84a567f56f5b14ba4aba08a967","examples/minimum-shelf-flagship/tool.seal.json":"526ac0f63f8ac487f6c38fd947defe1b80e1c3c14d80e3cfb38f5a66355b9cbd","garnet-interp-v0.3/src/prelude.rs":"784864f2ccf16eb5494a39e42c5c4bf15b6ac0e09b7f729ca8ecc0e23ad81c62","ops/lane2b/evidence/10-f1-canonical-reseal-green.txt":"02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7","proofs/minimum-shelf/lane2b/mcp-session.input.hex":"2328d55497368e0a351cfbd0e5421ab46c3826abf4f313c1a497e95a9fbfd769","proofs/minimum-shelf/lane2b/mcp-session.output.hex":"dac0eea7138d0f58865eecefc8db0c64490605068b956f757f420d6b284ba15f"},"checks":{"core-ring-tier1":true,"deterministic-shelf-reporter":true,"mcp-raw-byte-stdio":true,"reject-without-seal":true,"sealed-baseline":true},"current_commit":"8426ca761c696c3556190be77cce3e340250b5c7","current_tree":"601a368414762646ec9e5ad29b53736e20628474","findings":[],"implementer":"Codex GPT-5.6 Sol","landed_main_commit":null,"ok":true,"package":"examples/minimum-shelf-flagship","platform":"windows","product_content_sha256":"6f2d5f0b2dff0bd800955e0a55b81f6d6f784d71240fe3c906e58a6a3ca8eec6","product_path_count":1646,"request_frame_count":5,"response_frame_count":4,"reviewed_head":"8426ca761c696c3556190be77cce3e340250b5c7","reviewed_tree":"601a368414762646ec9e5ad29b53736e20628474","reviewed_tree_product_sha256":"1e6692175ea8fe2dd5b04fad4a492dc8ce48767dd07d88fd11a0847ce96749d5","reviewer":"Claude Code Fable 5","ring":"core","schema":"garnet.minimum_shelf_status/v2","scope_limits":["one Garnet-owned local package","Core Ring Tier 1 only","raw-byte stdio only","no hosted registry or network transport","reviewed local content, not signer identity"],"state":"accepted","tier":1,"tool":"garnet.core.double","unsigned_predicate":true}
```

## Minimum Shelf stderr — exit 0

```text
```

The sole `--emit-wv6` stdout is byte-identical to the Shelf gate stdout above;
its stderr is empty. It ran from `2026-08-22T00:25:47.7945359Z` through
`2026-08-22T00:25:49.0460366Z` and was not repeated.

## WV-6 stdout — exit 0

```json
{
  "artifact_count": 5,
  "contract_base_main_sha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
  "evidence_destination": "proofs/windows/launch-verification/wv6-minimum-shelf/",
  "findings": [],
  "landed_main_sha": null,
  "ok": true,
  "passed_check_count": 5,
  "product_content_sha256": "6f2d5f0b2dff0bd800955e0a55b81f6d6f784d71240fe3c906e58a6a3ca8eec6",
  "required_check_count": 5,
  "reviewed_head_sha": "8426ca761c696c3556190be77cce3e340250b5c7",
  "reviewed_tree_sha": "601a368414762646ec9e5ad29b53736e20628474",
  "schema": "garnet.wv_acceptance_status/v2",
  "state": "accepted",
  "wv": "WV-6"
}
```

## WV-6 stderr — exit 0

```text
```
