# Freeze-6 F5 red-before capture

Both native-Windows reporters ran at the pristine A3 boundary
`ba9fa6fe3b3581541cd66ef36334a7235e8e699e` before preservation or pin
movement. Both exited 1 solely on movement from the predecessor pair to the
live pair.

The raw streams are ASCII-only, BOM-free, and CRLF-terminated. The text below
is their UTF-8-compatible decoding. Raw capture root:
`C:\garnet-freeze6-capture-20260821-a7fbf88b\freeze6-red-before`.

| Stream | Bytes | SHA-256 |
|---|---:|---|
| Shelf stdout | 2391 | `774e411547da5ff8e2d8d62561cb248e1a04752364632cb47df6fd54005db1de` |
| Shelf stderr | 343 | `ad3057f9fcf8a32ab9698019da5d7324c9fb7306f03536785f9ee3ec273ca09f` |
| WV stdout | 856 | `72d11120119eadf2d9704116911eac8a11907341b7a51a47a90dc1f626312608` |
| WV stderr | 242 | `4a3d1046346b96c2b0a75b2fca440392cd7624f448f6e6f6489b2d03b3ac5b7a` |

## Shelf reporter — exit 1

Stdout:

```json
{"artifact_sha256":{".gitattributes":"b2a14050a850391f8ed1c788f9a6a66155a423ebceb3bb4722478dcaec97dd1b","examples/minimum-shelf-flagship/SHELF_PACKAGE.json":"dc12370de087c8beeb8885b4d51d4e37b8b02b8d7af90a3957bcb7c1e111b618","examples/minimum-shelf-flagship/tool.garnet":"25ebd3dc02c8ab7d17e343c867e6becda9918c84a567f56f5b14ba4aba08a967","examples/minimum-shelf-flagship/tool.seal.json":"526ac0f63f8ac487f6c38fd947defe1b80e1c3c14d80e3cfb38f5a66355b9cbd","garnet-interp-v0.3/src/prelude.rs":"784864f2ccf16eb5494a39e42c5c4bf15b6ac0e09b7f729ca8ecc0e23ad81c62","ops/lane2b/evidence/10-f1-canonical-reseal-green.txt":"02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7","proofs/minimum-shelf/lane2b/mcp-session.input.hex":"2328d55497368e0a351cfbd0e5421ab46c3826abf4f313c1a497e95a9fbfd769","proofs/minimum-shelf/lane2b/mcp-session.output.hex":"dac0eea7138d0f58865eecefc8db0c64490605068b956f757f420d6b284ba15f"},"checks":{"core-ring-tier1":true,"deterministic-shelf-reporter":false,"mcp-raw-byte-stdio":true,"reject-without-seal":true,"sealed-baseline":true},"current_commit":"ba9fa6fe3b3581541cd66ef36334a7235e8e699e","current_tree":"e78afada66baad26a20cc967e7141bbbd57ae084","findings":["product content digest does not match reviewed bytes","product path count does not match reviewed index","product content digest mismatch (449ba9b7aa948cb6fe5e1320385025ea563bc9e5c5ac69cc5ae6b6670bb2a9ee != 056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc)","product path count mismatch (1643 != 1640)"],"implementer":"Codex GPT-5.6 Sol","landed_main_commit":null,"ok":false,"package":"examples/minimum-shelf-flagship","platform":"windows","product_content_sha256":"449ba9b7aa948cb6fe5e1320385025ea563bc9e5c5ac69cc5ae6b6670bb2a9ee","product_path_count":1643,"request_frame_count":5,"response_frame_count":4,"reviewed_head":"218047425fd6871d6cb3ad526ef77e3f4df4c669","reviewed_tree":"9cbd7be6810f1f2852d4908fecd64cd66f75fa9c","reviewed_tree_product_sha256":"1e6692175ea8fe2dd5b04fad4a492dc8ce48767dd07d88fd11a0847ce96749d5","reviewer":"Claude Code Fable 5","ring":"core","schema":"garnet.minimum_shelf_status/v2","scope_limits":["one Garnet-owned local package","Core Ring Tier 1 only","raw-byte stdio only","no hosted registry or network transport","reviewed local content, not signer identity"],"state":"partial","tier":1,"tool":"garnet.core.double","unsigned_predicate":true}
```

Stderr:

```text
Minimum Shelf gate FAILED: product content digest does not match reviewed bytes; product path count does not match reviewed index; product content digest mismatch (449ba9b7aa948cb6fe5e1320385025ea563bc9e5c5ac69cc5ae6b6670bb2a9ee != 056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc); product path count mismatch (1643 != 1640)
```

## WV reporter — exit 1

Stdout:

```json
{
  "artifact_count": 5,
  "contract_base_main_sha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
  "evidence_destination": "proofs/windows/launch-verification/wv6-minimum-shelf/",
  "findings": [
    "product content digest mismatch (449ba9b7aa948cb6fe5e1320385025ea563bc9e5c5ac69cc5ae6b6670bb2a9ee != 056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc)",
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

Stderr:

```text
WV-6 acceptance gate PARTIAL: product content digest mismatch (449ba9b7aa948cb6fe5e1320385025ea563bc9e5c5ac69cc5ae6b6670bb2a9ee != 056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc); product path count mismatch (1643 != 1640)
```
