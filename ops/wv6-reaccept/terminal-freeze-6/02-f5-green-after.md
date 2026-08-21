# Freeze-6 F5 green-after capture

After supersession-with-preservation, eight pin movements, and one producer
emission, both native-Windows reporters exited 0 at the A3 boundary
`ba9fa6fe3b3581541cd66ef36334a7235e8e699e`.

The nonempty raw streams are ASCII-only, BOM-free, and CRLF-terminated; both
stderr streams are empty. The text below is the UTF-8-compatible decoding.
Raw capture root:
`C:\garnet-freeze6-capture-20260821-a7fbf88b\freeze6-green-after`.

| Stream | Bytes | SHA-256 |
|---|---:|---|
| Shelf stdout | 2071 | `936c7debd8353334b3f1a51e0c67ebbdffad1c63b5acd27caf086a679c8fc4a8` |
| Shelf stderr | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| WV stdout | 627 | `5c7f3dadac40b34360c71cb81e504017eaf7d3510fd22bdce1ceb6caadbc5c0b` |
| WV stderr | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

## Shelf reporter — exit 0

```json
{"artifact_sha256":{".gitattributes":"b2a14050a850391f8ed1c788f9a6a66155a423ebceb3bb4722478dcaec97dd1b","examples/minimum-shelf-flagship/SHELF_PACKAGE.json":"dc12370de087c8beeb8885b4d51d4e37b8b02b8d7af90a3957bcb7c1e111b618","examples/minimum-shelf-flagship/tool.garnet":"25ebd3dc02c8ab7d17e343c867e6becda9918c84a567f56f5b14ba4aba08a967","examples/minimum-shelf-flagship/tool.seal.json":"526ac0f63f8ac487f6c38fd947defe1b80e1c3c14d80e3cfb38f5a66355b9cbd","garnet-interp-v0.3/src/prelude.rs":"784864f2ccf16eb5494a39e42c5c4bf15b6ac0e09b7f729ca8ecc0e23ad81c62","ops/lane2b/evidence/10-f1-canonical-reseal-green.txt":"02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7","proofs/minimum-shelf/lane2b/mcp-session.input.hex":"2328d55497368e0a351cfbd0e5421ab46c3826abf4f313c1a497e95a9fbfd769","proofs/minimum-shelf/lane2b/mcp-session.output.hex":"dac0eea7138d0f58865eecefc8db0c64490605068b956f757f420d6b284ba15f"},"checks":{"core-ring-tier1":true,"deterministic-shelf-reporter":true,"mcp-raw-byte-stdio":true,"reject-without-seal":true,"sealed-baseline":true},"current_commit":"ba9fa6fe3b3581541cd66ef36334a7235e8e699e","current_tree":"e78afada66baad26a20cc967e7141bbbd57ae084","findings":[],"implementer":"Codex GPT-5.6 Sol","landed_main_commit":null,"ok":true,"package":"examples/minimum-shelf-flagship","platform":"windows","product_content_sha256":"449ba9b7aa948cb6fe5e1320385025ea563bc9e5c5ac69cc5ae6b6670bb2a9ee","product_path_count":1643,"request_frame_count":5,"response_frame_count":4,"reviewed_head":"ba9fa6fe3b3581541cd66ef36334a7235e8e699e","reviewed_tree":"e78afada66baad26a20cc967e7141bbbd57ae084","reviewed_tree_product_sha256":"1e6692175ea8fe2dd5b04fad4a492dc8ce48767dd07d88fd11a0847ce96749d5","reviewer":"Claude Code Fable 5","ring":"core","schema":"garnet.minimum_shelf_status/v2","scope_limits":["one Garnet-owned local package","Core Ring Tier 1 only","raw-byte stdio only","no hosted registry or network transport","reviewed local content, not signer identity"],"state":"accepted","tier":1,"tool":"garnet.core.double","unsigned_predicate":true}
```

## WV reporter — exit 0

```json
{
  "artifact_count": 5,
  "contract_base_main_sha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
  "evidence_destination": "proofs/windows/launch-verification/wv6-minimum-shelf/",
  "findings": [],
  "landed_main_sha": null,
  "ok": true,
  "passed_check_count": 5,
  "product_content_sha256": "449ba9b7aa948cb6fe5e1320385025ea563bc9e5c5ac69cc5ae6b6670bb2a9ee",
  "required_check_count": 5,
  "reviewed_head_sha": "ba9fa6fe3b3581541cd66ef36334a7235e8e699e",
  "reviewed_tree_sha": "e78afada66baad26a20cc967e7141bbbd57ae084",
  "schema": "garnet.wv_acceptance_status/v2",
  "state": "accepted",
  "wv": "WV-6"
}
```
