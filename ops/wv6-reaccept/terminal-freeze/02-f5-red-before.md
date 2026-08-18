# F5 red-before capture

- Head: `8e88b12eb16c057adac99551c5319289920dc9d3`
- Tree: `b961e73436bec5c4753bda6a43cd93f20a773b60`
- Shelf command: `python3 -I scripts/smoke_garnet_minimum_shelf.py --gate`
- Shelf exit: `1`
- WV command: `python3 -I scripts/garnet_wv_acceptance_status.py --wv WV-6 --gate`
- WV exit: `1`

## Shelf stdout, verbatim

```json
{"artifact_sha256":{".gitattributes":"b2a14050a850391f8ed1c788f9a6a66155a423ebceb3bb4722478dcaec97dd1b","examples/minimum-shelf-flagship/SHELF_PACKAGE.json":"dc12370de087c8beeb8885b4d51d4e37b8b02b8d7af90a3957bcb7c1e111b618","examples/minimum-shelf-flagship/tool.garnet":"25ebd3dc02c8ab7d17e343c867e6becda9918c84a567f56f5b14ba4aba08a967","examples/minimum-shelf-flagship/tool.seal.json":"526ac0f63f8ac487f6c38fd947defe1b80e1c3c14d80e3cfb38f5a66355b9cbd","garnet-interp-v0.3/src/prelude.rs":"784864f2ccf16eb5494a39e42c5c4bf15b6ac0e09b7f729ca8ecc0e23ad81c62","ops/lane2b/evidence/10-f1-canonical-reseal-green.txt":"02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7","proofs/minimum-shelf/lane2b/mcp-session.input.hex":"2328d55497368e0a351cfbd0e5421ab46c3826abf4f313c1a497e95a9fbfd769","proofs/minimum-shelf/lane2b/mcp-session.output.hex":"dac0eea7138d0f58865eecefc8db0c64490605068b956f757f420d6b284ba15f"},"checks":{"core-ring-tier1":true,"deterministic-shelf-reporter":false,"mcp-raw-byte-stdio":true,"reject-without-seal":true,"sealed-baseline":true},"current_commit":"8e88b12eb16c057adac99551c5319289920dc9d3","current_tree":"b961e73436bec5c4753bda6a43cd93f20a773b60","findings":["post-acceptance drift contains non-record path: AGENTS.md","post-acceptance drift contains non-record path: ops/gate-topology/FINDINGS.md","post-acceptance drift contains non-record path: ops/gate-topology/RULING-AMENDMENT.md","post-acceptance drift contains non-record path: ops/gate-topology/RULING-ORDERING.md","post-acceptance drift contains non-record path: ops/gate-topology/RULING.md","post-acceptance drift contains non-record path: ops/gate-topology/W_TRUST.md","post-acceptance drift contains non-record path: ops/gate-topology/evidence/01-unamended-v2-red.txt","post-acceptance drift contains non-record path: ops/gate-topology/evidence/02-u35-record-drift.txt","post-acceptance drift contains non-record path: ops/gate-topology/evidence/03-unit-fixture-red.txt","post-acceptance drift contains non-record path: ops/gate-topology/evidence/03a-single-byte-fixture-red.txt","post-acceptance drift contains non-record path: ops/gate-topology/evidence/04-unit-fixture-green.txt","post-acceptance drift contains non-record path: ops/gate-topology/evidence/05-full-trust-gate-suite-green.txt","post-acceptance drift contains non-record path: ops/gate-topology/evidence/06-amended-v2-exact-ref.txt","post-acceptance drift contains non-record path: ops/gate-topology/evidence/07-provenance-exclusion-red.txt","post-acceptance drift contains non-record path: ops/gate-topology/evidence/08-provenance-pair-expectation-falsified.txt","post-acceptance drift contains non-record path: ops/gate-topology/evidence/09-record-class-tolerance-red.txt","post-acceptance drift contains non-record path: ops/gate-topology/evidence/10-record-class-tolerance-unit-green.txt","post-acceptance drift contains non-record path: ops/gate-topology/evidence/11-third-merge-integration-red.txt","post-acceptance drift contains non-record path: ops/gate-topology/evidence/12-final-local-verification.txt","post-acceptance drift contains non-record path: ops/gate-topology/journal.md","post-acceptance drift contains non-record path: ops/gate-topology/review/01-request.md","post-acceptance drift contains non-record path: scripts/garnet_content_provenance.py","post-acceptance drift contains non-record path: scripts/garnet_trust_kernel_review_status.py","post-acceptance drift contains non-record path: scripts/garnet_wv_acceptance_status.py","post-acceptance drift contains non-record path: scripts/test_garnet_minimum_shelf_provenance.py","post-acceptance drift contains non-record path: scripts/test_garnet_trust_kernel_review_status.py"],"implementer":"Codex GPT-5.6 Sol","landed_main_commit":null,"ok":false,"package":"examples/minimum-shelf-flagship","platform":"windows","product_content_sha256":"fd96e6d910180f5e33999fbd693ea211e336389a13535930d89b2a870ff54727","product_path_count":1606,"request_frame_count":5,"response_frame_count":4,"reviewed_head":"410ff1182cdcefcec9fe046d1346205d8522ec9d","reviewed_tree":"57ce26ae1ab8d24609180486bc5fce6179f37957","reviewed_tree_product_sha256":"1e6692175ea8fe2dd5b04fad4a492dc8ce48767dd07d88fd11a0847ce96749d5","reviewer":"Claude Code Fable 5","ring":"core","schema":"garnet.minimum_shelf_status/v2","scope_limits":["one Garnet-owned local package","Core Ring Tier 1 only","raw-byte stdio only","no hosted registry or network transport","reviewed local content, not signer identity"],"state":"partial","tier":1,"tool":"garnet.core.double","unsigned_predicate":true}
```

## WV stdout, verbatim

```json
{
  "artifact_count": 5,
  "contract_base_main_sha": "231aefa91985e5a0520c493c7f0fc3e54d74efc8",
  "evidence_destination": "proofs/windows/launch-verification/wv6-minimum-shelf/",
  "findings": [
    "post-acceptance drift contains non-record path: AGENTS.md",
    "post-acceptance drift contains non-record path: ops/gate-topology/FINDINGS.md",
    "post-acceptance drift contains non-record path: ops/gate-topology/RULING-AMENDMENT.md",
    "post-acceptance drift contains non-record path: ops/gate-topology/RULING-ORDERING.md",
    "post-acceptance drift contains non-record path: ops/gate-topology/RULING.md",
    "post-acceptance drift contains non-record path: ops/gate-topology/W_TRUST.md",
    "post-acceptance drift contains non-record path: ops/gate-topology/evidence/01-unamended-v2-red.txt",
    "post-acceptance drift contains non-record path: ops/gate-topology/evidence/02-u35-record-drift.txt",
    "post-acceptance drift contains non-record path: ops/gate-topology/evidence/03-unit-fixture-red.txt",
    "post-acceptance drift contains non-record path: ops/gate-topology/evidence/03a-single-byte-fixture-red.txt",
    "post-acceptance drift contains non-record path: ops/gate-topology/evidence/04-unit-fixture-green.txt",
    "post-acceptance drift contains non-record path: ops/gate-topology/evidence/05-full-trust-gate-suite-green.txt",
    "post-acceptance drift contains non-record path: ops/gate-topology/evidence/06-amended-v2-exact-ref.txt",
    "post-acceptance drift contains non-record path: ops/gate-topology/evidence/07-provenance-exclusion-red.txt",
    "post-acceptance drift contains non-record path: ops/gate-topology/evidence/08-provenance-pair-expectation-falsified.txt",
    "post-acceptance drift contains non-record path: ops/gate-topology/evidence/09-record-class-tolerance-red.txt",
    "post-acceptance drift contains non-record path: ops/gate-topology/evidence/10-record-class-tolerance-unit-green.txt",
    "post-acceptance drift contains non-record path: ops/gate-topology/evidence/11-third-merge-integration-red.txt",
    "post-acceptance drift contains non-record path: ops/gate-topology/evidence/12-final-local-verification.txt",
    "post-acceptance drift contains non-record path: ops/gate-topology/journal.md",
    "post-acceptance drift contains non-record path: ops/gate-topology/review/01-request.md",
    "post-acceptance drift contains non-record path: scripts/garnet_content_provenance.py",
    "post-acceptance drift contains non-record path: scripts/garnet_trust_kernel_review_status.py",
    "post-acceptance drift contains non-record path: scripts/garnet_wv_acceptance_status.py",
    "post-acceptance drift contains non-record path: scripts/test_garnet_minimum_shelf_provenance.py",
    "post-acceptance drift contains non-record path: scripts/test_garnet_trust_kernel_review_status.py"
  ],
  "landed_main_sha": null,
  "ok": false,
  "passed_check_count": 5,
  "product_content_sha256": "fd96e6d910180f5e33999fbd693ea211e336389a13535930d89b2a870ff54727",
  "required_check_count": 5,
  "reviewed_head_sha": "410ff1182cdcefcec9fe046d1346205d8522ec9d",
  "reviewed_tree_sha": "57ce26ae1ab8d24609180486bc5fce6179f37957",
  "schema": "garnet.wv_acceptance_status/v2",
  "state": "partial",
  "wv": "WV-6"
}
```
