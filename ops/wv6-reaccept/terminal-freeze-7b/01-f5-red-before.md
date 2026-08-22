# Freeze-7b F5 red-before capture

Both native-Windows reporters ran at the pristine regenerated-proof boundary
`8426ca761c696c3556190be77cce3e340250b5c7` / tree
`601a368414762646ec9e5ad29b53736e20628474` before preservation or pin
movement. Both exited 1 on exactly one finding: the regenerated browser proof
is a digest-domain path after the first Freeze-7 reviewed head. This is the
expected fail-closed signal that a new reviewed boundary is required.

The checked-in producer and an independent raw `git ls-tree -r -z` parser both
computed the new pair as
`6f2d5f0b2dff0bd800955e0a55b81f6d6f784d71240fe3c906e58a6a3ca8eec6 / 1646`.
The reporters still truthfully exposed the superseded accepted pair
`87d5204c6b0c989d09e06dc176ab36096cd7a02fb31a2c7b600bcc1f7dd88058 / 1643`
until the authorized rebind.

Raw capture root: `C:\garnet-freeze7b-capture-20260821\red-before`.
Every stream passed strict UTF-8 decoding and was BOM-free. Every nonempty
stream used CRLF only; the table reports the measured byte and line-ending
census.

| Command | Exit | Stream | Bytes | SHA-256 | BOM | CRLF | bare LF | bare CR |
|---|---:|---|---:|---|---|---:|---:|---:|
| `python -I scripts/smoke_garnet_minimum_shelf.py --gate` | 1 | stdout | 2175 | `f2672a5109934cacb83aeb8b1511dab2dd13e20ed3a1dafe0359154f4fef6997` | no | 1 | 0 | 0 |
| same | 1 | stderr | 130 | `291e87181d2de74a1439ad6f2600179ddbd43471d62326ca8c2f066faa2c157c` | no | 1 | 0 | 0 |
| `python -I scripts/garnet_wv_acceptance_status.py --wv WV-6 --gate` | 1 | stdout | 740 | `1c2c8baf4335f66011d763df4f6761f6834ce1c803839f4b55880fd083467bc6` | no | 18 | 0 | 0 |
| same | 1 | stderr | 133 | `0defe37ff371f68983914700bb7b2bf44d49a8325b244fd40ef755936b44958a` | no | 1 | 0 | 0 |

## Minimum Shelf stdout — exit 1

```json
{"artifact_sha256":{".gitattributes":"b2a14050a850391f8ed1c788f9a6a66155a423ebceb3bb4722478dcaec97dd1b","examples/minimum-shelf-flagship/SHELF_PACKAGE.json":"dc12370de087c8beeb8885b4d51d4e37b8b02b8d7af90a3957bcb7c1e111b618","examples/minimum-shelf-flagship/tool.garnet":"25ebd3dc02c8ab7d17e343c867e6becda9918c84a567f56f5b14ba4aba08a967","examples/minimum-shelf-flagship/tool.seal.json":"526ac0f63f8ac487f6c38fd947defe1b80e1c3c14d80e3cfb38f5a66355b9cbd","garnet-interp-v0.3/src/prelude.rs":"784864f2ccf16eb5494a39e42c5c4bf15b6ac0e09b7f729ca8ecc0e23ad81c62","ops/lane2b/evidence/10-f1-canonical-reseal-green.txt":"02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7","proofs/minimum-shelf/lane2b/mcp-session.input.hex":"2328d55497368e0a351cfbd0e5421ab46c3826abf4f313c1a497e95a9fbfd769","proofs/minimum-shelf/lane2b/mcp-session.output.hex":"dac0eea7138d0f58865eecefc8db0c64490605068b956f757f420d6b284ba15f"},"checks":{"core-ring-tier1":true,"deterministic-shelf-reporter":false,"mcp-raw-byte-stdio":true,"reject-without-seal":true,"sealed-baseline":true},"current_commit":"8426ca761c696c3556190be77cce3e340250b5c7","current_tree":"601a368414762646ec9e5ad29b53736e20628474","findings":["post-acceptance drift contains non-record path: F_Project_Management/LAUNCH/W_PLAY_BROWSER_PROOF.json"],"implementer":"Codex GPT-5.6 Sol","landed_main_commit":null,"ok":false,"package":"examples/minimum-shelf-flagship","platform":"windows","product_content_sha256":"87d5204c6b0c989d09e06dc176ab36096cd7a02fb31a2c7b600bcc1f7dd88058","product_path_count":1643,"request_frame_count":5,"response_frame_count":4,"reviewed_head":"511e0fabad7335d14e972ffb968c7ac5e9b57ca8","reviewed_tree":"b7131198e99b01ab23aa75008e1e25acfca906c8","reviewed_tree_product_sha256":"1e6692175ea8fe2dd5b04fad4a492dc8ce48767dd07d88fd11a0847ce96749d5","reviewer":"Claude Code Fable 5","ring":"core","schema":"garnet.minimum_shelf_status/v2","scope_limits":["one Garnet-owned local package","Core Ring Tier 1 only","raw-byte stdio only","no hosted registry or network transport","reviewed local content, not signer identity"],"state":"partial","tier":1,"tool":"garnet.core.double","unsigned_predicate":true}
```

## Minimum Shelf stderr — exit 1

```text
Minimum Shelf gate FAILED: post-acceptance drift contains non-record path: F_Project_Management/LAUNCH/W_PLAY_BROWSER_PROOF.json
```

## WV-6 stdout — exit 1

```json
{
  "artifact_count": 5,
  "contract_base_main_sha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
  "evidence_destination": "proofs/windows/launch-verification/wv6-minimum-shelf/",
  "findings": [
    "post-acceptance drift contains non-record path: F_Project_Management/LAUNCH/W_PLAY_BROWSER_PROOF.json"
  ],
  "landed_main_sha": null,
  "ok": false,
  "passed_check_count": 5,
  "product_content_sha256": "87d5204c6b0c989d09e06dc176ab36096cd7a02fb31a2c7b600bcc1f7dd88058",
  "required_check_count": 5,
  "reviewed_head_sha": "511e0fabad7335d14e972ffb968c7ac5e9b57ca8",
  "reviewed_tree_sha": "b7131198e99b01ab23aa75008e1e25acfca906c8",
  "schema": "garnet.wv_acceptance_status/v2",
  "state": "partial",
  "wv": "WV-6"
}
```

## WV-6 stderr — exit 1

```text
WV-6 acceptance gate PARTIAL: post-acceptance drift contains non-record path: F_Project_Management/LAUNCH/W_PLAY_BROWSER_PROOF.json
```
