# Terminal freeze 3: F5 red before

## Boundary

- Execution head: `8659771c5a1828393d2e6ee54e1d679474b6e2ea`.
- Execution tree: `4347b6e31d9c681d9d715acd6452cf6cce281416`.
- Native product pair: `32f3975537470cb1788acfb3365725c4fea208a049e20fb90b1c314b719cba06 / 1637`.
- Raw capture root: `C:\garnet-freeze3-capture-20260818-f08481fa`.

Raw files are the byte-level evidence. Every non-empty stream passed strict
UTF-8 decoding, had no BOM, and used only CRLF line endings. The fenced
rendering preserves decoded characters and line order. This Markdown record
is UTF-8 without BOM using LF.

| Command | Exit | Stream | Bytes | SHA-256 | EOL census |
|---|---:|---|---:|---|---|
| `python -I scripts/smoke_garnet_minimum_shelf.py --gate` | 1 | stdout | 2391 | `1378846100984f9955a0da67d08ef9d7543035553b133ac31ebfcb65a56e9e5a` | 1 CRLF |
| same | 1 | stderr | 343 | `78dfd13a6b3145367b895e5c1f766e9daba63f3ae06038c0f3322bee1f144400` | 1 CRLF |
| `python -I scripts/garnet_wv_acceptance_status.py --wv WV-6 --gate` | 1 | stdout | 856 | `8194ca71ed6758341025183e222958650c21a48b8fd747aba69c73e8657e3006` | 19 CRLF |
| same | 1 | stderr | 242 | `df2905ee353fb2ae9161ec2f06abb88c714ae23e2ebaf86cf3e248a4159d27c8` | 1 CRLF |

## Minimum Shelf stdout

```json
{"artifact_sha256":{".gitattributes":"b2a14050a850391f8ed1c788f9a6a66155a423ebceb3bb4722478dcaec97dd1b","examples/minimum-shelf-flagship/SHELF_PACKAGE.json":"dc12370de087c8beeb8885b4d51d4e37b8b02b8d7af90a3957bcb7c1e111b618","examples/minimum-shelf-flagship/tool.garnet":"25ebd3dc02c8ab7d17e343c867e6becda9918c84a567f56f5b14ba4aba08a967","examples/minimum-shelf-flagship/tool.seal.json":"526ac0f63f8ac487f6c38fd947defe1b80e1c3c14d80e3cfb38f5a66355b9cbd","garnet-interp-v0.3/src/prelude.rs":"784864f2ccf16eb5494a39e42c5c4bf15b6ac0e09b7f729ca8ecc0e23ad81c62","ops/lane2b/evidence/10-f1-canonical-reseal-green.txt":"02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7","proofs/minimum-shelf/lane2b/mcp-session.input.hex":"2328d55497368e0a351cfbd0e5421ab46c3826abf4f313c1a497e95a9fbfd769","proofs/minimum-shelf/lane2b/mcp-session.output.hex":"dac0eea7138d0f58865eecefc8db0c64490605068b956f757f420d6b284ba15f"},"checks":{"core-ring-tier1":true,"deterministic-shelf-reporter":false,"mcp-raw-byte-stdio":true,"reject-without-seal":true,"sealed-baseline":true},"current_commit":"8659771c5a1828393d2e6ee54e1d679474b6e2ea","current_tree":"4347b6e31d9c681d9d715acd6452cf6cce281416","findings":["product content digest does not match reviewed bytes","product path count does not match reviewed index","product content digest mismatch (32f3975537470cb1788acfb3365725c4fea208a049e20fb90b1c314b719cba06 != 8ea996129074e2e70c5ad2f9abd6082c85f7021e943dc017d3a9ba2859ed3ac7)","product path count mismatch (1637 != 1634)"],"implementer":"Codex GPT-5.6 Sol","landed_main_commit":null,"ok":false,"package":"examples/minimum-shelf-flagship","platform":"windows","product_content_sha256":"32f3975537470cb1788acfb3365725c4fea208a049e20fb90b1c314b719cba06","product_path_count":1637,"request_frame_count":5,"response_frame_count":4,"reviewed_head":"4a6d1aed9c81a624efa2335b28de12b4bdb82c8f","reviewed_tree":"a4829ce899c7525260c222ed16c14137b228c647","reviewed_tree_product_sha256":"1e6692175ea8fe2dd5b04fad4a492dc8ce48767dd07d88fd11a0847ce96749d5","reviewer":"Claude Code Fable 5","ring":"core","schema":"garnet.minimum_shelf_status/v2","scope_limits":["one Garnet-owned local package","Core Ring Tier 1 only","raw-byte stdio only","no hosted registry or network transport","reviewed local content, not signer identity"],"state":"partial","tier":1,"tool":"garnet.core.double","unsigned_predicate":true}
```

## Minimum Shelf stderr

```text
Minimum Shelf gate FAILED: product content digest does not match reviewed bytes; product path count does not match reviewed index; product content digest mismatch (32f3975537470cb1788acfb3365725c4fea208a049e20fb90b1c314b719cba06 != 8ea996129074e2e70c5ad2f9abd6082c85f7021e943dc017d3a9ba2859ed3ac7); product path count mismatch (1637 != 1634)
```

## WV-6 stdout

```json
{
  "artifact_count": 5,
  "contract_base_main_sha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
  "evidence_destination": "proofs/windows/launch-verification/wv6-minimum-shelf/",
  "findings": [
    "product content digest mismatch (32f3975537470cb1788acfb3365725c4fea208a049e20fb90b1c314b719cba06 != 8ea996129074e2e70c5ad2f9abd6082c85f7021e943dc017d3a9ba2859ed3ac7)",
    "product path count mismatch (1637 != 1634)"
  ],
  "landed_main_sha": null,
  "ok": false,
  "passed_check_count": 5,
  "product_content_sha256": "8ea996129074e2e70c5ad2f9abd6082c85f7021e943dc017d3a9ba2859ed3ac7",
  "required_check_count": 5,
  "reviewed_head_sha": "4a6d1aed9c81a624efa2335b28de12b4bdb82c8f",
  "reviewed_tree_sha": "a4829ce899c7525260c222ed16c14137b228c647",
  "schema": "garnet.wv_acceptance_status/v2",
  "state": "partial",
  "wv": "WV-6"
}
```

## WV-6 stderr

```text
WV-6 acceptance gate PARTIAL: product content digest mismatch (32f3975537470cb1788acfb3365725c4fea208a049e20fb90b1c314b719cba06 != 8ea996129074e2e70c5ad2f9abd6082c85f7021e943dc017d3a9ba2859ed3ac7); product path count mismatch (1637 != 1634)
```
