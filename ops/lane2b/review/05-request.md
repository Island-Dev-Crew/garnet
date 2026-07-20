# Lane 2B Review Request 05 — final Air execution and content verdict

## Seats and immutable review boundary

- Implementer: Codex GPT-5.6 Sol
- Independent native reviewer: Claude Fable 5, MacBook Air
- Authenticated carrier / ceremony seat: Jon
- Branch: `mission/l2b-sealed-shelf-mcp`
- Reviewed base: `cede73c03c5d535306ed179b5882e99e4d17b050`
- Reviewed head: `927ad221d33668d458499a26f49d96ed4586563d`
- Reviewed tree: `4d9374991bf265b78a0108e0bb62c317a43b8028`
- Diffstat vs base: 67 files, +5,343/−61
- Pull-request refs fetched by implementer: 0
- Launch: HOLD
- Band: 3, capped while U-17 remains open
- Denominators: S114 100.0%; truth pulse 93.1%; launch-critical 50.0%;
  launch ledger 37.5%

Post-reviewed-head commits are evidence/request/state under `ops/lane2b/**`
only. They are excluded by the frozen content contract and make no product-byte
change. Review the exact head/tree above; do not infer approval from branch tip.

## Verdict 04 authorization satisfied

The implementation uses one deterministic SHA-256 over sorted `(path,
blob-OID)` pairs from `git --no-replace-objects ls-files -s -z`. Its exclusion
list is exactly the Verdict-04-frozen set:

1. `ops/lane2b/**`
2. `proofs/**`
3. `F_Project_Management/W_TRUST/**`
4. `scripts/smoke_garnet_minimum_shelf.py` itself

No fifth exclusion exists. The current product digest is
`810f256bcf9304999975120224419216422996ff3b804d1a9a8836d5bcc4c339`
over 1,529 paths. The separately preserved reviewed-tree baseline at
`f3272b9610dba756bd414cafc825fd7462d7a294` is
`1e6692175ea8fe2dd5b04fad4a492dc8ce48767dd07d88fd11a0847ce96749d5`
over 1,527 paths. The count increase is the authorized shared provenance module
and its adversarial test; existing authorized truth/provenance paths also change
content without adding exclusions.

Exact protected and bound blobs at reviewed head:

| Path | Git blob | Bytes |
|---|---|---:|
| `scripts/smoke_garnet_minimum_shelf.py` | `57b91324221a1ba6cf326b0b74607b3248e4693f` | 23438 |
| `scripts/garnet_content_provenance.py` | `dffa7f7887e9ddda9dcc8c2925291a531b1a6724` | 6548 |
| `scripts/garnet_wv_acceptance_status.py` | `e69a61f25a136c5303c427e63607114a828003e2` | 19511 |
| `scripts/test_garnet_minimum_shelf_provenance.py` | `451c4fa7cf5d8beb776730d0beb2c9aec242fec4` | 4230 |
| `scripts/test_garnet_wv_acceptance_status.py` | `d607d654ce98be263fb9591f84f0bd1a8cab5a38` | 9318 |
| `F_Project_Management/W_TRUST/LANE2B_MINIMUM_SHELF_MCP_REVIEW_2026-07-19.md` | `cc2cad21d69a77de3db178d570a70e7933b1e02b` | 8781 |
| `proofs/minimum-shelf/lane2b/PROOF.json` | `277e4ec8f068834318aa106fb9c45348833483c7` | 1856 |
| `proofs/windows/launch-verification/wv6-minimum-shelf/WV_ACCEPTANCE.json` | `8ea687543fcbcf7ae200f8f2b558559cfaa37b4d` | 1978 |

The UNSIGNED predicate language is unchanged: this proves reviewed local
content, not external signer identity.

## RED-before-GREEN and squash proof

`ops/lane2b/evidence/16-verdict04-provenance-red.txt` records all three required
traps RED before the mechanism existed. They are now 3/3 GREEN:

1. any product blob change → RED;
2. absent `a6f0da2...` and `e2820ce...` branch objects → still GREEN on main;
3. evidence/content digest mismatch → RED.

`ops/lane2b/evidence/18-squash-main-only-green.txt` transfers only the exact
reviewed tree and its blobs into a new repository, creates one root commit, then
clones it fresh. The clone has zero pull refs and neither discarded branch
commit. Shelf and WV-6 both pass, identify the new first-parent main commit, and
WV-7 remains pending. There is no red window and no post-squash rebind ceremony.

## Implementer gate results

- Content provenance traps: 3/3
- WV truth tests: 6/6
- Shelf reporter: accepted, 5/5, findings `[]`
- WV-6: accepted, 5/5, 5 artifacts, findings `[]`
- WV-7: pending, 0/5, exit 1 as required
- `garnet-cli`: 460/460
- minimum-shelf native package: sealed 1/1, rejection traps 6/6
- native raw-byte stdio: 2/2
- fmt: pass
- strict clippy: pass
- Lane 0 / MSRV / frozen backlog: pass
- trust gate: `ok:true`, `problems:[]`
- `xtask truth --check`: pass
- full Python: lane 931/17F/8E/3S vs main 928/17F/8E/3S; the three
  added provenance tests pass, with zero failure/error/skip delta

The raw-byte implementer replay at the reviewed head used persisted
`core.autocrlf=false` and `true` before checkout. Four runs were byte-identical:
2,071 bytes, SHA-256
`77c3cb058efda5441985e3f9bb86cd172b454470f0a3ddbc4d84a9b321f8c0fd`,
zero stderr, zero pull refs. See
`ops/lane2b/evidence/20-final-raw-byte-cross-checkout.txt`. This is not the
required independent proof.

## Required Air execution — Verdict 04 Decision 4

Use two fresh, main-object-independent checkouts of the fleet-fork branch. Fetch
no `refs/pull/*`. Set each local config before checkout:

```sh
git clone --no-checkout --branch mission/l2b-sealed-shelf-mcp \
  https://github.com/Navigata1/garnet.git lane2b-lf
git -C lane2b-lf config core.autocrlf false
git -C lane2b-lf checkout --detach 927ad221d33668d458499a26f49d96ed4586563d

git clone --no-checkout --branch mission/l2b-sealed-shelf-mcp \
  https://github.com/Navigata1/garnet.git lane2b-windows
git -C lane2b-windows config core.autocrlf true
git -C lane2b-windows checkout --detach 927ad221d33668d458499a26f49d96ed4586563d
```

In each checkout, run the reporter twice with raw shell redirection, record exit
codes, byte counts, SHA-256, stderr size, and `refs/pull/*` count. `cmp` both
runs and then compare across checkouts. Also run:

```sh
python3 -I scripts/test_garnet_minimum_shelf_provenance.py -v
python3 -I scripts/test_garnet_wv_acceptance_status.py -v
python3 -I scripts/garnet_wv_acceptance_status.py --wv WV-6 --gate
python3 scripts/garnet_trust_kernel_review_status.py --gate
cargo test -p garnet-cli --test minimum_shelf_package --no-fail-fast
cargo test -p garnet-cli --test mcp_stdio --no-fail-fast
```

## Questions requiring the immutable Verdict 05

1. Does the frozen four-exclusion digest exactly implement Verdict 04 Decision
   2a, with no content blind spot or replacement-ref dependency?
2. Does the reviewed-tree baseline plus final authorized digest satisfy Decision
   2b without silently treating implementer-added product bytes as pre-reviewed?
3. Does the first-parent main check pass at the squash instant without either
   discarded branch object, a pull ref, or a red-main rebind window?
4. Do all three adversarial traps fail/green in the required directions without
   weakening an assertion?
5. Are the Air LF and `core.autocrlf=true` raw outputs byte-identical, and do the
   native sealed positive, six negative, and stdio legs remain green?
6. Is W_TRUST exact, identity-true, and sufficient for the protected paths?

Commit `ops/lane2b/review/05-verdict.md` immutably as APPROVE or with exact
blockers. Only APPROVE authorizes Jon to open the PR. Jon remains the only merge
seat; FIRE, tag, publish, and launch stay out of scope.
