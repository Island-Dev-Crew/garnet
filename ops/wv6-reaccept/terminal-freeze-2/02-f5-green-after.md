# F5 green-after capture — terminal freeze 2

- Head: `6e94374556d4d94148c27d2d2edaa3aa839cab6a`
- Tree: `01ff8096fc09fbd2c226a89a83aa5a030115903d`
- Shelf command: `python3 -I scripts/smoke_garnet_minimum_shelf.py --gate`
- Shelf exit: `0`
- WV command: `python3 -I scripts/garnet_wv_acceptance_status.py --wv WV-6 --gate`
- WV exit: `0`
- Raw capture root: `C:\gtf2-capture-01a08f2c`
- Shelf stdout SHA-256: `64f182a20e379a9338762a8f42c1c074c63a59566355aad98d071df86f7ba921`
- Shelf stderr SHA-256: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- WV stdout SHA-256: `c8b659a7d53049434cd3f7e46d7db5b6c2d1bb7c300da6a117c7416b5b76d16a`
- WV stderr SHA-256: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`

## Shelf stdout, verbatim

```json
{"artifact_sha256":{".gitattributes":"b2a14050a850391f8ed1c788f9a6a66155a423ebceb3bb4722478dcaec97dd1b","examples/minimum-shelf-flagship/SHELF_PACKAGE.json":"dc12370de087c8beeb8885b4d51d4e37b8b02b8d7af90a3957bcb7c1e111b618","examples/minimum-shelf-flagship/tool.garnet":"25ebd3dc02c8ab7d17e343c867e6becda9918c84a567f56f5b14ba4aba08a967","examples/minimum-shelf-flagship/tool.seal.json":"526ac0f63f8ac487f6c38fd947defe1b80e1c3c14d80e3cfb38f5a66355b9cbd","garnet-interp-v0.3/src/prelude.rs":"784864f2ccf16eb5494a39e42c5c4bf15b6ac0e09b7f729ca8ecc0e23ad81c62","ops/lane2b/evidence/10-f1-canonical-reseal-green.txt":"02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7","proofs/minimum-shelf/lane2b/mcp-session.input.hex":"2328d55497368e0a351cfbd0e5421ab46c3826abf4f313c1a497e95a9fbfd769","proofs/minimum-shelf/lane2b/mcp-session.output.hex":"dac0eea7138d0f58865eecefc8db0c64490605068b956f757f420d6b284ba15f"},"checks":{"core-ring-tier1":true,"deterministic-shelf-reporter":true,"mcp-raw-byte-stdio":true,"reject-without-seal":true,"sealed-baseline":true},"current_commit":"6e94374556d4d94148c27d2d2edaa3aa839cab6a","current_tree":"01ff8096fc09fbd2c226a89a83aa5a030115903d","findings":[],"implementer":"Codex GPT-5.6 Sol","landed_main_commit":null,"ok":true,"package":"examples/minimum-shelf-flagship","platform":"windows","product_content_sha256":"8ea996129074e2e70c5ad2f9abd6082c85f7021e943dc017d3a9ba2859ed3ac7","product_path_count":1634,"request_frame_count":5,"response_frame_count":4,"reviewed_head":"4a6d1aed9c81a624efa2335b28de12b4bdb82c8f","reviewed_tree":"a4829ce899c7525260c222ed16c14137b228c647","reviewed_tree_product_sha256":"1e6692175ea8fe2dd5b04fad4a492dc8ce48767dd07d88fd11a0847ce96749d5","reviewer":"Claude Code Fable 5","ring":"core","schema":"garnet.minimum_shelf_status/v2","scope_limits":["one Garnet-owned local package","Core Ring Tier 1 only","raw-byte stdio only","no hosted registry or network transport","reviewed local content, not signer identity"],"state":"accepted","tier":1,"tool":"garnet.core.double","unsigned_predicate":true}
```

Shelf stderr was empty; the raw file SHA-256 is the empty-byte digest recorded above.

## WV stdout, verbatim

```json
{
  "artifact_count": 5,
  "contract_base_main_sha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
  "evidence_destination": "proofs/windows/launch-verification/wv6-minimum-shelf/",
  "findings": [],
  "landed_main_sha": null,
  "ok": true,
  "passed_check_count": 5,
  "product_content_sha256": "8ea996129074e2e70c5ad2f9abd6082c85f7021e943dc017d3a9ba2859ed3ac7",
  "required_check_count": 5,
  "reviewed_head_sha": "4a6d1aed9c81a624efa2335b28de12b4bdb82c8f",
  "reviewed_tree_sha": "a4829ce899c7525260c222ed16c14137b228c647",
  "schema": "garnet.wv_acceptance_status/v2",
  "state": "accepted",
  "wv": "WV-6"
}
```

WV stderr was empty; the raw file SHA-256 is the empty-byte digest recorded above.
