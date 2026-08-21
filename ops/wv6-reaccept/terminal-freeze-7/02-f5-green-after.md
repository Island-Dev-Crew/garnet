# Freeze-7 F5 green-after capture

After supersession-with-preservation, eight pin/mirror movements, and one
producer emission into a vacant destination, both native-Windows reporters
exited 0 at the final-cure boundary
`511e0fabad7335d14e972ffb968c7ac5e9b57ca8` / tree
`b7131198e99b01ab23aa75008e1e25acfca906c8` and pair
`87d5204c6b0c989d09e06dc176ab36096cd7a02fb31a2c7b600bcc1f7dd88058 / 1643`.

Raw capture root: `C:\garnet-freeze7-capture-20260821\green-after`.
Every stream passed strict UTF-8 decoding and was BOM-free. Both nonempty
stdout streams used CRLF only; both stderr streams were empty.

| Command | Exit | Stream | Bytes | SHA-256 | BOM | CRLF | bare LF | bare CR |
|---|---:|---|---:|---|---|---:|---:|---:|
| `python -I scripts/smoke_garnet_minimum_shelf.py --gate` | 0 | stdout | 2071 | `e42298fbf3783a365cd48fb909e8e2314dbcca90a770714d532b5562de37c503` | no | 1 | 0 | 0 |
| same | 0 | stderr | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | no | 0 | 0 | 0 |
| `python -I scripts/garnet_wv_acceptance_status.py --wv WV-6 --gate` | 0 | stdout | 627 | `ecf9032eea1ac5a608109c7e8fcf03ddcf55c8c34a917cde0eeec752cac53fe1` | no | 16 | 0 | 0 |
| same | 0 | stderr | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | no | 0 | 0 | 0 |

## Minimum Shelf stdout — exit 0

```json
{"artifact_sha256":{".gitattributes":"b2a14050a850391f8ed1c788f9a6a66155a423ebceb3bb4722478dcaec97dd1b","examples/minimum-shelf-flagship/SHELF_PACKAGE.json":"dc12370de087c8beeb8885b4d51d4e37b8b02b8d7af90a3957bcb7c1e111b618","examples/minimum-shelf-flagship/tool.garnet":"25ebd3dc02c8ab7d17e343c867e6becda9918c84a567f56f5b14ba4aba08a967","examples/minimum-shelf-flagship/tool.seal.json":"526ac0f63f8ac487f6c38fd947defe1b80e1c3c14d80e3cfb38f5a66355b9cbd","garnet-interp-v0.3/src/prelude.rs":"784864f2ccf16eb5494a39e42c5c4bf15b6ac0e09b7f729ca8ecc0e23ad81c62","ops/lane2b/evidence/10-f1-canonical-reseal-green.txt":"02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7","proofs/minimum-shelf/lane2b/mcp-session.input.hex":"2328d55497368e0a351cfbd0e5421ab46c3826abf4f313c1a497e95a9fbfd769","proofs/minimum-shelf/lane2b/mcp-session.output.hex":"dac0eea7138d0f58865eecefc8db0c64490605068b956f757f420d6b284ba15f"},"checks":{"core-ring-tier1":true,"deterministic-shelf-reporter":true,"mcp-raw-byte-stdio":true,"reject-without-seal":true,"sealed-baseline":true},"current_commit":"511e0fabad7335d14e972ffb968c7ac5e9b57ca8","current_tree":"b7131198e99b01ab23aa75008e1e25acfca906c8","findings":[],"implementer":"Codex GPT-5.6 Sol","landed_main_commit":null,"ok":true,"package":"examples/minimum-shelf-flagship","platform":"windows","product_content_sha256":"87d5204c6b0c989d09e06dc176ab36096cd7a02fb31a2c7b600bcc1f7dd88058","product_path_count":1643,"request_frame_count":5,"response_frame_count":4,"reviewed_head":"511e0fabad7335d14e972ffb968c7ac5e9b57ca8","reviewed_tree":"b7131198e99b01ab23aa75008e1e25acfca906c8","reviewed_tree_product_sha256":"1e6692175ea8fe2dd5b04fad4a492dc8ce48767dd07d88fd11a0847ce96749d5","reviewer":"Claude Code Fable 5","ring":"core","schema":"garnet.minimum_shelf_status/v2","scope_limits":["one Garnet-owned local package","Core Ring Tier 1 only","raw-byte stdio only","no hosted registry or network transport","reviewed local content, not signer identity"],"state":"accepted","tier":1,"tool":"garnet.core.double","unsigned_predicate":true}
```

## Minimum Shelf stderr — exit 0

```text
```

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
  "product_content_sha256": "87d5204c6b0c989d09e06dc176ab36096cd7a02fb31a2c7b600bcc1f7dd88058",
  "required_check_count": 5,
  "reviewed_head_sha": "511e0fabad7335d14e972ffb968c7ac5e9b57ca8",
  "reviewed_tree_sha": "b7131198e99b01ab23aa75008e1e25acfca906c8",
  "schema": "garnet.wv_acceptance_status/v2",
  "state": "accepted",
  "wv": "WV-6"
}
```

## WV-6 stderr — exit 0

```text
```
