# WV-6 integration re-acceptance — Review Request 01

## Seats and authority

- Actual implementer: OpenAI Codex, GPT-5-based agent; exact model version is
  not exposed by this harness, on `NUCBOX_M2PRO_S` (Windows 11 Pro
  10.0.26200, x86_64; WSL kernel 6.6.87.2 on the ext4 implementation
  checkout).
- Requested independent reviewer: Claude Code on Claude Fable 5 (Anthropic),
  on the MacBook Air reviewer seat. The reviewer is a different model family
  on a different machine.
- Review carrier: `IDC-Trust-Review` only.
- Merge authority: Jon (`IslandDevCrew`) only.
- The implementer is neither carrier nor merge authority, does not grade this
  packet, and does not author `01-verdict.md`.

Jon authorized this integration path in the merge-authority ruling dated
2026-08-09. The earlier two-PR expected-red plan is superseded. No PR,
approval, merge, tag, release, or main mutation is part of this request.

## Construction and boundary

- Integration base: `efd4f6bae8b3afaba74594e57944b2548142aeae`
  (the verified `refs/remotes/origin/main` at boot).
- Lane 2C source tip:
  `1bc64c4061250531b12d08007553d5db0f4b2d98`.
- Lane 2C merge commit:
  `814c0bcb36924c00c392d4e47bc1d61bfd18ee45` / tree
  `2196efcde4e129aa0006585db35765111ebdc918`.
- Repair 3 source tip:
  `9e0f7a452be20e5990fc666e7f62c832cc4bded1`.
- Repair 3 merge commit:
  `2bcc6dd5249445a52558a51e17925bfdccae3fe1` / tree
  `897c808f6579b5d10593f4b0de49565fac15cb05`.
- Both merges were clean `--no-ff` merges. No conflict was resolved by hand.
- The pre-record merged tree recomputed to
  `43d68dc3290ccb194bd1921881dcb313721fc495eb5428bc0eff2b28d9132d85 / 1604`.
  That is a diagnostic pair, not the final frozen pair: this request is
  digest-included and is deliberately committed before the freeze.

This request's commit is the final digest-included content boundary. A commit
cannot contain its own SHA, so no self-referential frozen-head claim appears
inside it. After this request lands, the implementer must derive the exact
frozen head, tree, product digest, and path count from the committed tree.
Every later implementer change is restricted to the already authorized
digest-excluded surfaces:

1. `scripts/smoke_garnet_minimum_shelf.py` (the reporter self-path),
2. `proofs/**`, and
3. `F_Project_Management/W_TRUST/**`.

The reviewer must fetch the advertised fork tip and bind the exact frozen
head/tree from the reporter and `WV_ACCEPTANCE.json`, plus the exact final
branch tip reported by `git ls-remote`. This is the U-35 head-versus-tip
shape used by Phase 0. Any later `ops/wv6-reaccept/**` edit would move the
product pair and invalidate the ceremony.

## Expected-red predecessor state

At merged head `2bcc6dd`, before the authorized rebind:

- Minimum Shelf is red only for the live pair
  `43d68dc3290ccb194bd1921881dcb313721fc495eb5428bc0eff2b28d9132d85 / 1604`
  versus the accepted pin
  `ea38d3547eafa7f56141454df50eaf8084dbc66ee9573ebdd67623df8be97bbe / 1544`,
  plus the expected `.gitattributes` per-file hash mismatch.
- WV-6 is `partial` with all 5/5 checks true and only the product-digest
  mismatch above.
- The current `.gitattributes` SHA-256 is
  `b2a14050a850391f8ed1c788f9a6a66155a423ebceb3bb4722478dcaec97dd1b`;
  the reporter still pins
  `b8b22a96534aa11b02d5d72e5baf2a6cc5dc9481ea5ad85a5441728ffa8d2e5f`.

Those are disclosed fail-closed tripwires, not findings to tune away.

## Authorized rebind

After freezing this request commit, the implementation slice changes no
reporter logic. It rebinds exactly:

1. `REVIEWED_HEAD`,
2. `REVIEWED_TREE`,
3. `EXPECTED_PRODUCT_CONTENT_SHA256`,
4. `EXPECTED_PRODUCT_PATH_COUNT`, and
5. the `.gitattributes` entry in `EXPECTED_FILE_SHA256`.

The matching four canonical mirrors in
`proofs/minimum-shelf/lane2b/PROOF.json` move with the reporter. The
historical `REVIEWED_TREE_PRODUCT_SHA256` and
`REVIEWED_TREE_PATH_COUNT` values and mirrors remain byte-identical, as
Phase 0 established. The candidate values are derived from the frozen commit;
the old Repair 3 branch pair
`bcbae1ea664542498e1b1308c167961486e0ecc096619c9ad9b3ee7836753196 / 1559`
is not a target.

## Native acceptance evidence

The complete WV-6 acceptance run must execute natively on Windows from a
fresh NTFS-local checkout outside OneDrive. WSL/WSLg output is inadmissible.
No quiet-machine claim is made or required. The sanctioned commands are:

```text
cargo test -p garnet-cli minimum_shelf --no-fail-fast
cargo test -p garnet-cli --test mcp_stdio --no-fail-fast
cargo test -p garnet-cli --test minimum_shelf_package sealed --no-fail-fast
cargo test -p garnet-cli --test minimum_shelf_package rejects --no-fail-fast
python -I scripts/smoke_garnet_minimum_shelf.py --gate
python -I scripts/smoke_garnet_minimum_shelf.py --emit-wv6
python -I scripts/garnet_wv_acceptance_status.py --wv WV-6 --gate
python -I -m unittest discover -s scripts -p test_*.py
```

The producer-generated acceptance record is
`proofs/windows/launch-verification/wv6-minimum-shelf/WV_ACCEPTANCE.json`.
Its sibling artifacts and hashes are the evidence bundle; the producer's
overwrite guard must not be bypassed by hand-editing output.

The full Python battery is expected to retain exactly the two U-47 failures
already red on `origin/main`:

1. `test_repo_and_site_point_to_the_adoption_surface_reporter`, and
2. `test_tag_release_publishes_unified_checksummed_assets`.

They are pre-existing adoption-surface and tag-release asset findings and are
not cured here. The U-39 Base-controlled trust-policy workflow is outside the
required set and remains assigned to Repair 3b. Any other battery failure is
a merged-tree finding and requires STOP without inline repair.

## Carried review and requested checks

The two Lane 2C verdicts and three Repair 3 verdicts carry forward as the
independent review record for the constituent branches. This review is for
the integration re-acceptance and its new trust-kernel delta, not a
self-review of those already approved bytes.

Please:

1. Recompute the frozen pair from the exact frozen head and confirm the later
   tip preserves it.
2. Verify the reporter diff changes only the four candidate constants and
   the `.gitattributes` per-file pin, with exactly four matching
   `PROOF.json` mirrors.
3. Verify the native-Windows bundle was generated by the sanctioned producer,
   binds the frozen head/tree/pair, and passes the WV-6 acceptance verifier.
4. Confirm the full native battery has exactly the two disclosed U-47 reds
   and no successor-only failure.
5. Audit the trust-kernel path
   `scripts/smoke_garnet_minimum_shelf.py` independently. This branch has a
   real trust-kernel delta, so approval also requires a canonical
   `garnet.trust_kernel_review_record/v2` record under
   `F_Project_Management/W_TRUST/`. The implementer does not fabricate that
   reviewer/carrier record.
6. Confirm no path outside the authorized integration, reporter, proof, and
   review-record surfaces moved during the re-acceptance slice.

Verdict destination:
`ops/wv6-reaccept/review/01-verdict.md`.

## STOP

The implementer stops after publishing the exact candidate and this request.
Jon-only approval, merge, tag, release, and token actions remain unperformed.
