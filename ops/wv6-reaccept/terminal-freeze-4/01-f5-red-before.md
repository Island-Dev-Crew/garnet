# Terminal freeze 4: F5 red before

## Boundary

- Execution head: `218047425fd6871d6cb3ad526ef77e3f4df4c669`.
- Execution tree: `9cbd7be6810f1f2852d4908fecd64cd66f75fa9c`.
- Natively computed product pair: `056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc / 1640`.
- Prior accepted pair: `32f3975537470cb1788acfb3365725c4fea208a049e20fb90b1c314b719cba06 / 1637`.
- Raw capture root: `C:\garnet-u17-freeze4-capture-20260819-60fdde15`.

The U-17 projection cure is digest-domain content. Before rebind, both
reporters failed closed because the two cure paths were outside the prior
boundary's post-acceptance record class.

Raw files are the byte-level evidence. Every non-empty stream passed strict
UTF-8 decoding, had no BOM, and used only CRLF line endings. Empty streams
contain zero bytes. The fenced rendering preserves decoded characters and
line order. This Markdown record is UTF-8 without BOM using LF.

| Command | Exit | Stream | Bytes | SHA-256 | EOL census |
|---|---:|---|---:|---|---|
| `python -I scripts/smoke_garnet_minimum_shelf.py --gate` | 1 | stdout | 2258 | `c1627c57f9dc119a085dd0e519a599d26d10de6a9854ac51573327695df64470` | 1 CRLF; 0 bare LF |
| same | 1 | stderr | 212 | `c746f39fe823a6a37ac79cab771e26479960f68389c2353d42189c86c3346429` | 1 CRLF; 0 bare LF |
| `python -I scripts/garnet_wv_acceptance_status.py --wv WV-6 --gate` | 1 | stdout | 829 | `eec7984cb00f6598e81649538264db09a32a400e626df433b7d0c99693a70774` | 19 CRLF; 0 bare LF |
| same | 1 | stderr | 215 | `087ee127908298eafec06e4d07e5141634b3b5104b23f0f31d57921529d64bd5` | 1 CRLF; 0 bare LF |

## Minimum Shelf stdout

```json
{"artifact_sha256":{".gitattributes":"b2a14050a850391f8ed1c788f9a6a66155a423ebceb3bb4722478dcaec97dd1b","examples/minimum-shelf-flagship/SHELF_PACKAGE.json":"dc12370de087c8beeb8885b4d51d4e37b8b02b8d7af90a3957bcb7c1e111b618","examples/minimum-shelf-flagship/tool.garnet":"25ebd3dc02c8ab7d17e343c867e6becda9918c84a567f56f5b14ba4aba08a967","examples/minimum-shelf-flagship/tool.seal.json":"526ac0f63f8ac487f6c38fd947defe1b80e1c3c14d80e3cfb38f5a66355b9cbd","garnet-interp-v0.3/src/prelude.rs":"784864f2ccf16eb5494a39e42c5c4bf15b6ac0e09b7f729ca8ecc0e23ad81c62","ops/lane2b/evidence/10-f1-canonical-reseal-green.txt":"02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7","proofs/minimum-shelf/lane2b/mcp-session.input.hex":"2328d55497368e0a351cfbd0e5421ab46c3826abf4f313c1a497e95a9fbfd769","proofs/minimum-shelf/lane2b/mcp-session.output.hex":"dac0eea7138d0f58865eecefc8db0c64490605068b956f757f420d6b284ba15f"},"checks":{"core-ring-tier1":true,"deterministic-shelf-reporter":false,"mcp-raw-byte-stdio":true,"reject-without-seal":true,"sealed-baseline":true},"current_commit":"218047425fd6871d6cb3ad526ef77e3f4df4c669","current_tree":"9cbd7be6810f1f2852d4908fecd64cd66f75fa9c","findings":["post-acceptance drift contains non-record path: scripts/garnet_github_governance_gate.py","post-acceptance drift contains non-record path: scripts/test_garnet_github_governance_gate.py"],"implementer":"Codex GPT-5.6 Sol","landed_main_commit":null,"ok":false,"package":"examples/minimum-shelf-flagship","platform":"windows","product_content_sha256":"32f3975537470cb1788acfb3365725c4fea208a049e20fb90b1c314b719cba06","product_path_count":1637,"request_frame_count":5,"response_frame_count":4,"reviewed_head":"8659771c5a1828393d2e6ee54e1d679474b6e2ea","reviewed_tree":"4347b6e31d9c681d9d715acd6452cf6cce281416","reviewed_tree_product_sha256":"1e6692175ea8fe2dd5b04fad4a492dc8ce48767dd07d88fd11a0847ce96749d5","reviewer":"Claude Code Fable 5","ring":"core","schema":"garnet.minimum_shelf_status/v2","scope_limits":["one Garnet-owned local package","Core Ring Tier 1 only","raw-byte stdio only","no hosted registry or network transport","reviewed local content, not signer identity"],"state":"partial","tier":1,"tool":"garnet.core.double","unsigned_predicate":true}
```

## Minimum Shelf stderr

```text
Minimum Shelf gate FAILED: post-acceptance drift contains non-record path: scripts/garnet_github_governance_gate.py; post-acceptance drift contains non-record path: scripts/test_garnet_github_governance_gate.py
```

## WV-6 stdout

```json
{
  "artifact_count": 5,
  "contract_base_main_sha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
  "evidence_destination": "proofs/windows/launch-verification/wv6-minimum-shelf/",
  "findings": [
    "post-acceptance drift contains non-record path: scripts/garnet_github_governance_gate.py",
    "post-acceptance drift contains non-record path: scripts/test_garnet_github_governance_gate.py"
  ],
  "landed_main_sha": null,
  "ok": false,
  "passed_check_count": 5,
  "product_content_sha256": "32f3975537470cb1788acfb3365725c4fea208a049e20fb90b1c314b719cba06",
  "required_check_count": 5,
  "reviewed_head_sha": "8659771c5a1828393d2e6ee54e1d679474b6e2ea",
  "reviewed_tree_sha": "4347b6e31d9c681d9d715acd6452cf6cce281416",
  "schema": "garnet.wv_acceptance_status/v2",
  "state": "partial",
  "wv": "WV-6"
}
```

## WV-6 stderr

```text
WV-6 acceptance gate PARTIAL: post-acceptance drift contains non-record path: scripts/garnet_github_governance_gate.py; post-acceptance drift contains non-record path: scripts/test_garnet_github_governance_gate.py
```
