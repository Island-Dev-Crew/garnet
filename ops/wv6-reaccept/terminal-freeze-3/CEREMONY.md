# WV-6 terminal freeze 3 ceremony record

## Seat and frozen boundary

- Seat: OpenAI Codex on the native-Windows NUC.
- Checkout: `C:\garnet-freeze3-20260818-f08481fa`.
- Host process count at preflight: `599`.
- OneDrive process count: `0`.
- `core.autocrlf=false`.
- Commit identity: `OpenAI Codex <codex@openai.com>`.
- Frozen main head: `8659771c5a1828393d2e6ee54e1d679474b6e2ea`.
- Frozen tree: `4347b6e31d9c681d9d715acd6452cf6cce281416`.
- Natively computed pair: `32f3975537470cb1788acfb3365725c4fea208a049e20fb90b1c314b719cba06 / 1637`.

The Mac-seat pair hypothesis matched the native computation. The native value
is the binding result.

The containing commit is identified as the commit that first introduces this
`CEREMONY.md`; this record does not embed its future commit SHA or tree.

## Native predicates

All four checked-in native predicates passed before the custody mutation:

| Command | Result | Stdout SHA-256 | Stderr SHA-256 |
|---|---|---|---|
| `cargo test -p garnet-cli minimum_shelf --no-fail-fast` | 3/3 | `635f695183939fd108e9431e0b0aef331722325962b966a2b1fe443e97bda54b` | `eb10d727dc4eec772ae1e01be55636c7db239a95dae091b3dc8ba240ea0422c4` |
| `cargo test -p garnet-cli --test mcp_stdio --no-fail-fast` | 2/2 | `7540a91134fbbfe57223843de80e1089cb30466388abd810ffcf271c5d7a86e8` | `e73c01bf9dd3f7600935ca50f5b17d3939abaa20f74c3d9dfa6542c8977de857` |
| `cargo test -p garnet-cli --test minimum_shelf_package sealed --no-fail-fast` | 1/1 | `f0f4f16918e8ca1ac8c17b99c747f15f4de27395c6dd876c9184ee64ed249f8a` | `d5ff3f06818748f3899dd5b12d457da18f609fba9ac45966e570557c223178c4` |
| `cargo test -p garnet-cli --test minimum_shelf_package rejects --no-fail-fast` | 6/6 | `81f5efeff631b0aba77e39dc1729d55cc2a6e42723b4a9a988db52b81e1d40fe` | `2bdf8e1b1e482a3ec63aa7a8f9bb3897fd32d7384907cbcb23d7cbfe5989494e` |

The red-before and green-after reporter outputs, encodings, byte counts, and
hashes are recorded in `01-f5-red-before.md` and `02-f5-green-after.md`.

## Pin census and producer

The checked-in procedure names nine pins. Eight values moved: four candidate
constants in `scripts/smoke_garnet_minimum_shelf.py` and four matching mirrors
in `proofs/minimum-shelf/lane2b/PROOF.json`. The `.gitattributes` file pin was
recomputed as
`b2a14050a850391f8ed1c788f9a6a66155a423ebceb3bb4722478dcaec97dd1b`
and remained byte-identical. The historical reviewed-tree digest and path
count remained byte-identical.

The committed producer ran once into the vacant live destination. The emitted
`WV_ACCEPTANCE.json` SHA-256 is
`84552a8219cb6ccdeb25ed299ebba1ec92d50adb4a3213a0235ed48c2cad8f3f`.
The emitted manifest binds the frozen head, tree, and pair above. Its five
artifact hashes were recomputed from the emitted files and matched the
manifest.

## Supersession with preservation

State: `SUPERSEDED-WITH-PRESERVATION`.

The superseded boundary was derived from the live producer artifacts:

- Reviewed head: `4a6d1aed9c81a624efa2335b28de12b4bdb82c8f`.
- Reviewed tree: `a4829ce899c7525260c222ed16c14137b228c647`.
- Pair: `8ea996129074e2e70c5ad2f9abd6082c85f7021e943dc017d3a9ba2859ed3ac7 / 1634`.
- Preserved `WV_ACCEPTANCE.json` SHA-256: `41e8b71bc5f34fc4b7deea223eee6c523582b63dc417716a53ab9c902a792448`.
- Complete preserved bundle:
  `proofs/windows/launch-verification/wv6-terminal-freeze-20260818/superseded-4a6d1aed9c81a624efa2335b28de12b4bdb82c8f/`.

The producer-censused six-file bundle was copied byte-for-byte before the live
destination was removed. Source and preserved SHA-256 values matched for all
six files.

The complete pair chain is:

1. `2cb25d0b47f55c9dd987bf69fc8a97dade5a4767ef0beda0abc8616808dddc0c / 1605`.
2. `fd96e6d910180f5e33999fbd693ea211e336389a13535930d89b2a870ff54727 / 1606`.
3. `1b45387599223adbe8766ae2e04ddc70e4ecc359712a4867ef1973dac9bd42e4 / 1629`.
4. `8ea996129074e2e70c5ad2f9abd6082c85f7021e943dc017d3a9ba2859ed3ac7 / 1634`.
5. `32f3975537470cb1788acfb3365725c4fea208a049e20fb90b1c314b719cba06 / 1637`.

All predecessor records and Git history remain intact.

## Head-versus-tip boundary

The three new ceremony records are product-digest included but belong to the
established post-acceptance record class. The accepted pair is frozen at main
head `8659771c5a1828393d2e6ee54e1d679474b6e2ea`; the containing tip is expected
to retain it through the U-35 record-path verifier. The containing commit tree
is not claimed to hash to `32f3975537470cb1788acfb3365725c4fea208a049e20fb90b1c314b719cba06 / 1637`.

After the single commit, both reporters must be rerun against the committed
path delta. A nonzero result stops publication.

No canonical review record, adapter change, approval, merge, tag, release, or
credential generation is part of this ceremony.
