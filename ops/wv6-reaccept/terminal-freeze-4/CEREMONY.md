# WV-6 terminal freeze 4 ceremony record

## Seat and frozen boundary

- Seat: OpenAI Codex on the native-Windows NUC.
- Checkout: `C:\garnet-u17-freeze4-20260819-60fdde15`.
- Host process count at cold-start preflight: `596`; pre-custody census: `581`.
- OneDrive process count: `0`.
- `core.autocrlf=false`.
- Commit identity available on this seat: `Jon Isaac <Navigata1@gmail.com>`; no Tier-1 identity was configured.
- Branch base: `3ef1e874ff5f6fde14b940441801d5340b85ccea` / tree `a87a2b308cd5a403b5a6059dec99ae7899c9ab98`.
- Frozen cure head: `218047425fd6871d6cb3ad526ef77e3f4df4c669`.
- Frozen cure tree: `9cbd7be6810f1f2852d4908fecd64cd66f75fa9c`.
- Natively computed pair: `056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc / 1640`.

The containing commit is identified as the commit that first introduces this
`CEREMONY.md`; this record does not embed its future commit SHA or tree.

## Occasioning U-17 projection cure

Part A is the dedicated cure commit `218047425fd6871d6cb3ad526ef77e3f4df4c669`, whose only changed paths are
`scripts/garnet_github_governance_gate.py` and
`scripts/test_garnet_github_governance_gate.py`. The adapter projects REST
`check_suite.id` into the bounded internal `check_suite_id`. Missing or
non-positive nested IDs fail the collection closed. Universal shape checks,
exactly-one selection, and the required App tuple remain active. Unrelated
CodeQL App `57789 / github-advanced-security` remains census noise and
receives no required-context binding.

## Field red-before exhibit

The live main-controlled gate run at `2026-08-19T22:36Z` against candidate
head `61e5ab9a8cea6157489e31e6210d48405ffa0ab9` refused with
`check run identity is malformed` followed by the 31-context exactly-one
cascade. Jon retains the verbatim JSON in the PR record.

## Mid-ceremony stop and resume

After preservation, pin rebind, the green Shelf gate, and the single producer
emission, a seat-added assertion expected the evidence manifest state to be
`accepted`. The emitted manifest correctly reported `evidence_complete`, so
the seat stopped before the WV gate, records, staging, or commit.

Jon ratified the stop and authorized evidence-based resumption from that exact
point. The preserved predecessor manifest, SHA-256
`84552a8219cb6ccdeb25ed299ebba1ec92d50adb4a3213a0235ed48c2cad8f3f`,
quotes `"state":"evidence_complete"` under schema `garnet.wv_acceptance_evidence/v2`.
The checked-in producer assigns `"state": "evidence_complete"` at
`scripts/smoke_garnet_minimum_shelf.py:557`, and writes the manifest at line
564. The checked-in WV verifier requires the same token at
`scripts/garnet_wv_acceptance_status.py:271-272`. The seat assertion applied
status vocabulary to the evidence-manifest schema; it was removed without a
replacement. The checked-in WV gate then resumed as the validator and exited
zero.

## Register candidates

1. Seat-side validations must source expected values from checked-in schemas
   or producers, or label them as hypotheses. The corrections doctrine binds
   machine seats as well as chat seats.
2. Tier-1 identity was absent on this seat. The existing cure commit remains
   as authored; history was not rewritten mid-ceremony. This rollout is the
   registered cure and now carries its third identity exhibit.

## Part A local verification

| Suite | Result | Capture SHA-256 |
|---|---:|---|
| `test_garnet_github_governance_gate.py` | 31/31 | stderr `fb10d8e9cf3114986e36605934f066a191533a53f51dd8cb1bbb4bb50934df02` |
| `test_garnet_workflow_identity_policy.py` | 6/6 | stderr `0ddd27f6ced4e37043f59a5445025b49d6553dc79cdf2038f83a650ee4ec9d21` |
| `test_garnet_github_governance_transport.py` | 24/24 | stderr `1eb77452bfa6791b7bb1671d13108ddf77b896eb6ae177c76c30ab5935ebcb52` |
| `test_garnet_base_controlled_trust_status.py` | 4/4 | stderr `19fb1277e806965433f37d1e1695fa8140a3bd96ce5bfc7ce965ba51f3e4e98d` |
| `check-agent-contracts.py` | 24 contracts | stdout `bcd9395d545f1d6f4bcf88b07971f33707999b0536d2239b17c1a12dd679e8fe` |
| `test_check_agent_contracts.py` | 6/6 | stderr `45850d95434291e9ce48c8f10b0775a47fa076a6c32670cf5d08d1fe7e640aa5` |

## Native predicates

| Command | Result | Stdout SHA-256 | Stderr SHA-256 |
|---|---:|---|---|
| `cargo test -p garnet-cli minimum_shelf --no-fail-fast` | 3/3 | `75c537d08661dd9b1cea1f808e9ac5c6e309aa77d31cbcf201ba5bfbf453309d` | `1bd33d4c5aa8d62dfe4a032f0caed67363b4f2f0e0688b779dbcf020b9dc4b06` |
| `cargo test -p garnet-cli --test mcp_stdio --no-fail-fast` | 2/2 | `8d5f231edc3ca264dbe5788b3f795329dbed3c5f8d8cb7a52dcfc3042139883d` | `47cc11deb0ab4a00828c1e84e0c64953685125693c04c5b31ce6ae3500fa3da7` |
| `cargo test -p garnet-cli --test minimum_shelf_package sealed --no-fail-fast` | 1/1 | `9c354624909e99c7a8637d71657af9db6f80ff9bf661783a84b583a4235585d8` | `ff7d51cb25e521717c64086f891c7944c71f68e38204dfd72af138e73636e2a7` |
| `cargo test -p garnet-cli --test minimum_shelf_package rejects --no-fail-fast` | 6/6 | `db20ca5f6c50ed5bc0351a39867f73aacae74e7510cb306dedac6dc2e6cd6cb0` | `7ff8d2cb6456efdfc55803239cbadbbf6f4b3529aa060d3a0044d1e5199d38bc` |

The reporter output, encoding census, byte counts, and hashes are recorded in
`01-f5-red-before.md` and `02-f5-green-after.md`.

## Pin census and producer

The checked-in procedure names nine pins. Eight values moved: four candidate
constants in `scripts/smoke_garnet_minimum_shelf.py` and four matching
mirrors in `proofs/minimum-shelf/lane2b/PROOF.json`. The `.gitattributes`
pin was recomputed as
`b2a14050a850391f8ed1c788f9a6a66155a423ebceb3bb4722478dcaec97dd1b`
and remained byte-identical. Historical reviewed-tree digest and count
values remained byte-identical.

The producer ran exactly once into the vacant live destination. The fresh
`WV_ACCEPTANCE.json` uses schema `garnet.wv_acceptance_evidence/v2`, state
`evidence_complete`, and SHA-256 `e4f5ebdf9f3936765bf20070837f7953538d6a60024da03f755cc87e8c792579`.
Its status and manifest bind the frozen head, tree, and pair above.

| Fresh live file | Bytes | SHA-256 |
|---|---:|---|
| `f1-canonical-reseal.txt` | 2801 | `02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7` |
| `mcp-session.input.hex` | 1073 | `64b2f3e15489b4f06e57a0c51afce960beda3edc9c8009d5f3a530d5d9f92638` |
| `mcp-session.output.hex` | 1797 | `15423fa5ba9697f59f877ea0a9afb3587230fc4d7c3182cee82ce709bbce4799` |
| `minimum-shelf-status.json` | 2070 | `a595cd0895a05427316fee734812adfc20e8b429f1b72a9ecb62401b651268a9` |
| `reporter-cross-checkout.txt` | 1660 | `a4f33e467349bf84a6c9fe04e7ddd486df335a96f2edda0b2cb371787d26425f` |
| `WV_ACCEPTANCE.json` | 1978 | `e4f5ebdf9f3936765bf20070837f7953538d6a60024da03f755cc87e8c792579` |

## Supersession with preservation

State: `SUPERSEDED-WITH-PRESERVATION`.

- Superseded reviewed head: `8659771c5a1828393d2e6ee54e1d679474b6e2ea`.
- Superseded reviewed tree: `4347b6e31d9c681d9d715acd6452cf6cce281416`.
- Superseded pair: `32f3975537470cb1788acfb3365725c4fea208a049e20fb90b1c314b719cba06 / 1637`.
- Preserved `WV_ACCEPTANCE.json` SHA-256: `84552a8219cb6ccdeb25ed299ebba1ec92d50adb4a3213a0235ed48c2cad8f3f`.
- Complete preserved bundle: `proofs/windows/launch-verification/wv6-terminal-freeze-20260819/superseded-8659771c5a1828393d2e6ee54e1d679474b6e2ea/`.
- Raw moved predecessor copy: `C:\garnet-u17-freeze4-capture-20260819-60fdde15\superseded-live-working-copy-8659771c5a1828393d2e6ee54e1d679474b6e2ea`.

The producer-censused six-file bundle was copied byte-for-byte into the
committed preservation path. After all six hashes matched, the original live
directory was moved intact to the raw-capture root, making the producer
destination vacant without deleting any predecessor byte.

| Preserved file | Bytes | SHA-256 |
|---|---:|---|
| `f1-canonical-reseal.txt` | 2801 | `02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7` |
| `mcp-session.input.hex` | 1073 | `64b2f3e15489b4f06e57a0c51afce960beda3edc9c8009d5f3a530d5d9f92638` |
| `mcp-session.output.hex` | 1797 | `15423fa5ba9697f59f877ea0a9afb3587230fc4d7c3182cee82ce709bbce4799` |
| `minimum-shelf-status.json` | 2108 | `c2cdf7176b3fcfc268c429a308fd9db88cf70d20c5d95f6e220c473bb33a2796` |
| `reporter-cross-checkout.txt` | 1660 | `a4f33e467349bf84a6c9fe04e7ddd486df335a96f2edda0b2cb371787d26425f` |
| `WV_ACCEPTANCE.json` | 1978 | `84552a8219cb6ccdeb25ed299ebba1ec92d50adb4a3213a0235ed48c2cad8f3f` |

The complete pair chain is:

1. `2cb25d0b47f55c9dd987bf69fc8a97dade5a4767ef0beda0abc8616808dddc0c / 1605`.
2. `fd96e6d910180f5e33999fbd693ea211e336389a13535930d89b2a870ff54727 / 1606`.
3. `1b45387599223adbe8766ae2e04ddc70e4ecc359712a4867ef1973dac9bd42e4 / 1629`.
4. `8ea996129074e2e70c5ad2f9abd6082c85f7021e943dc017d3a9ba2859ed3ac7 / 1634`.
5. `32f3975537470cb1788acfb3365725c4fea208a049e20fb90b1c314b719cba06 / 1637`.
6. `056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc / 1640`.

All predecessor records and Git history remain intact.

## Head-versus-tip boundary

The three new ceremony records are product-digest included but belong to the
established post-acceptance record class. The accepted pair is frozen at the
cure head `218047425fd6871d6cb3ad526ef77e3f4df4c669`; the containing tip retains it through the U-35
record-path verifier. The containing commit tree is not claimed to hash to
`056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc / 1640`.

After the single commit, both reporters are rerun against the committed path
delta. A nonzero result stops publication.

No token use, canonical review record, authenticated admin readback,
31-to-32 activation, approval, merge, tag, or release is part of this
ceremony.
