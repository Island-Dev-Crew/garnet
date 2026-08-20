# WV-6 terminal freeze 5 ceremony record

## Seat and frozen boundary

- Seat: OpenAI Codex on the native-Windows NUC.
- Checkout: `C:\garnet-freeze5-20260820-4bfc6d21`.
- Host process count at cold-start preflight: `560`; long gates were serialized.
- OneDrive process count: `0`.
- `core.autocrlf=false` before the first repository file read.
- Commit identity: `Jon Isaac <Navigata1@gmail.com>`.
- Branch base: `1d765cdb2e69bc097cd33db30f9919ad8e969208` / tree `a7de550d8f853c6a50d69f87d3fb6d5919cfef29`.
- Frozen cure head: `2ca3d81129d22ab827b7574ff992d7c262aaf9e2`.
- Frozen cure tree: `fc571865bc6c548d9b7bc02f79a1b2d2517d9f2b`.
- Natively computed pair: `0513edb99391ad4dfe75dffe3618c8c001f877aa4d4f738709c4b506ec06b425 / 1643`.

The containing commit is identified as the commit that first introduces this
`CEREMONY.md`; this record does not embed its future commit SHA or tree.

## Occasioning ruleset pin refresh

Part A is the dedicated cure commit
`2ca3d81129d22ab827b7574ff992d7c262aaf9e2`. Its three paths are:

- `.github/rulesets/garnet-main.json`
- `scripts/garnet_github_governance_gate.py`
- `scripts/test_garnet_github_governance_gate.py`

A token-authenticated read of live ruleset `18936562` found exactly two
live-only fields under `rules[3].parameters` and no other structural or value
drift:

```json
"dismissal_restriction": {"enabled": false, "allowed_actors": []}
"required_reviewers": []
```

Review statement: the two additive fields are reviewed non-weakening
(dismissal restriction disabled with no allowed actors; no required
reviewers); the live governance posture is unchanged from the originally
reviewed contract.

The strict-equality function and generic problem strings did not change. The
regression fixture accepts the exact live shape and rejects enabled dismissal,
non-empty allowed actors, and non-empty required reviewers.

The checked-in canonical document producer reproduced the prior pins before
the edit and produced these new pins after the edit:

| Document | Canonical SHA-256 |
|---|---|
| 31-context ruleset | `83b69477d7886c51889e6f55a3b93af6c534f0e9194f2958d0f152ce01fe0b2f` |
| Derived 32-context activation ruleset | `8f8c1e6727e7234579adc698960118b4fe0de5fd1f40edd926f4fff58a448402` |

The producer census found no other operational ruleset-document digest pin.
Historical evidence remained untouched.

## Part A local verification

| Suite | Result |
|---|---:|
| `test_garnet_github_governance_gate.py` | 33/33 |
| `test_garnet_workflow_identity_policy.py` | 6/6 |
| `test_garnet_github_governance_transport.py` | 24/24 |
| `test_garnet_base_controlled_trust_status.py` | 4/4 |
| `check-agent-contracts.py` | 24 contracts |
| `test_check_agent_contracts.py` | 6/6 |
| `test_garnet_governance_activation_ceremony.py` | 10/10 |

## Native predicates

All four native-Windows acceptance predicates ran at the clean frozen Part A
head before custody mutation:

| Command | Result |
|---|---:|
| `cargo test -p garnet-cli minimum_shelf --no-fail-fast` | 3/3 |
| `cargo test -p garnet-cli --test mcp_stdio --no-fail-fast` | 2/2 |
| `cargo test -p garnet-cli --test minimum_shelf_package sealed --no-fail-fast` | 1/1 |
| `cargo test -p garnet-cli --test minimum_shelf_package rejects --no-fail-fast` | 6/6 |

Reporter output, encoding census, byte counts, and hashes are recorded in
`01-f5-red-before.md` and `02-f5-green-after.md`. Raw captures remain outside
the repository at `C:\garnet-freeze5-capture-20260820-4bfc6d21`.

## Pin census and producer

The checked-in procedure names nine pins. Eight values moved: four candidate
constants in `scripts/smoke_garnet_minimum_shelf.py` and four matching mirrors
in `proofs/minimum-shelf/lane2b/PROOF.json`. The `.gitattributes` pin was
natively recomputed as
`b2a14050a850391f8ed1c788f9a6a66155a423ebceb3bb4722478dcaec97dd1b`
and remained byte-identical. Historical reviewed-tree digest and count values
remained byte-identical.

The Shelf gate accepted the rebound state before emission. The committed
producer ran exactly once into the vacant live destination. The canonical WV
gate and both green-after reporters then exited zero with state `accepted` and
empty findings. The fresh `WV_ACCEPTANCE.json` uses schema
`garnet.wv_acceptance_evidence/v2`, state `evidence_complete`, and SHA-256
`b43fedeb86c63b4a0c91eef2937eaa8a0c75dd4ae33941415e89d5ddbdb8d270`.

| Fresh live file | Bytes | SHA-256 |
|---|---:|---|
| `f1-canonical-reseal.txt` | 2801 | `02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7` |
| `mcp-session.input.hex` | 1073 | `64b2f3e15489b4f06e57a0c51afce960beda3edc9c8009d5f3a530d5d9f92638` |
| `mcp-session.output.hex` | 1797 | `15423fa5ba9697f59f877ea0a9afb3587230fc4d7c3182cee82ce709bbce4799` |
| `minimum-shelf-status.json` | 2070 | `ca778509c49e84a068e4e97575062cda813fa28c5c858bf17e68688365ff12cd` |
| `reporter-cross-checkout.txt` | 1660 | `a4f33e467349bf84a6c9fe04e7ddd486df335a96f2edda0b2cb371787d26425f` |
| `WV_ACCEPTANCE.json` | 1978 | `b43fedeb86c63b4a0c91eef2937eaa8a0c75dd4ae33941415e89d5ddbdb8d270` |

## Supersession with preservation

State: `SUPERSEDED-WITH-PRESERVATION`.

- Superseded reviewed head: `218047425fd6871d6cb3ad526ef77e3f4df4c669`.
- Superseded reviewed tree: `9cbd7be6810f1f2852d4908fecd64cd66f75fa9c`.
- Superseded pair: `056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc / 1640`.
- Preserved `WV_ACCEPTANCE.json` SHA-256: `e4f5ebdf9f3936765bf20070837f7953538d6a60024da03f755cc87e8c792579`.
- Complete preserved bundle: `proofs/windows/launch-verification/wv6-terminal-freeze-20260820/superseded-218047425fd6871d6cb3ad526ef77e3f4df4c669/`.
- Raw moved predecessor copy: `C:\garnet-freeze5-capture-20260820-4bfc6d21\predecessor-live-wv6-minimum-shelf`.

The producer-censused six-file bundle was copied byte-for-byte into the
committed preservation path. After all six hashes and sizes matched, the
original live directory was moved intact to the raw-capture root, making the
producer destination vacant without discarding any predecessor byte.

| Preserved file | Bytes | SHA-256 |
|---|---:|---|
| `f1-canonical-reseal.txt` | 2801 | `02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7` |
| `mcp-session.input.hex` | 1073 | `64b2f3e15489b4f06e57a0c51afce960beda3edc9c8009d5f3a530d5d9f92638` |
| `mcp-session.output.hex` | 1797 | `15423fa5ba9697f59f877ea0a9afb3587230fc4d7c3182cee82ce709bbce4799` |
| `minimum-shelf-status.json` | 2070 | `a595cd0895a05427316fee734812adfc20e8b429f1b72a9ecb62401b651268a9` |
| `reporter-cross-checkout.txt` | 1660 | `a4f33e467349bf84a6c9fe04e7ddd486df335a96f2edda0b2cb371787d26425f` |
| `WV_ACCEPTANCE.json` | 1978 | `e4f5ebdf9f3936765bf20070837f7953538d6a60024da03f755cc87e8c792579` |

The complete pair chain is:

1. `2cb25d0b47f55c9dd987bf69fc8a97dade5a4767ef0beda0abc8616808dddc0c / 1605`.
2. `fd96e6d910180f5e33999fbd693ea211e336389a13535930d89b2a870ff54727 / 1606`.
3. `1b45387599223adbe8766ae2e04ddc70e4ecc359712a4867ef1973dac9bd42e4 / 1629`.
4. `8ea996129074e2e70c5ad2f9abd6082c85f7021e943dc017d3a9ba2859ed3ac7 / 1634`.
5. `32f3975537470cb1788acfb3365725c4fea208a049e20fb90b1c314b719cba06 / 1637`.
6. `056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc / 1640`.
7. `0513edb99391ad4dfe75dffe3618c8c001f877aa4d4f738709c4b506ec06b425 / 1643`.

All predecessor records and Git history remain intact.

## Head-versus-tip boundary

The three new ceremony records are product-digest included but belong to the
established post-acceptance record class. The accepted pair is frozen at the
cure head `2ca3d81129d22ab827b7574ff992d7c262aaf9e2`; the containing tip retains it
through the record-path verifier. The containing commit tree is not claimed to
hash to `0513edb99391ad4dfe75dffe3618c8c001f877aa4d4f738709c4b506ec06b425 / 1643`.

After the single acceptance commit, both reporters are rerun against the
committed path delta. A nonzero result stops publication. The changed-path
classifier must identify every path from the frozen head to the containing tip
as record class.

No token use, canonical structured review record, authenticated admin
readback, 31-to-32 activation, approval, merge, tag, or release is part of this
ceremony. The expected PR state before independent Air review is a required
rolling-gate red for the absent structured review record plus the separately
registered non-required Base-controlled U-39 red; these are expectations to be
checked against the actual PR run.
