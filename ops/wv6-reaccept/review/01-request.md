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

The full Python battery is compared by test name against the measured native-
Windows base transcript, not against the earlier Linux/macOS two-red set. The
binding base set is exactly these ten names:

- errors: `test_output_dir_writes_manifested_pack`,
  `test_output_dir_writes_manifested_plan`,
  `test_symlink_cannot_be_used_as_clause_evidence`, and
  `test_output_dir_writes_manifested_evidence_bundle`;
- failures: `test_repo_and_site_point_to_the_adoption_surface_reporter`,
  `test_tracked_ledger_matches_renderer_byte_for_byte`,
  `test_all_novel_programs_check_and_run`,
  `test_tag_release_publishes_unified_checksummed_assets`,
  `test_run_script_stages_swiftpm_gui_app_bundle`, and
  `test_missing_studio_adoption_copy_is_a_strict_blocker`.

The base transcript SHA-256 is
`aacddc271ed737db4d8c44b0f7b91e6788b0ca47a62c72efd4b88c747cf685ed`.
The two U-47 failures are a subset of this measured platform baseline; the
other eight are also pre-existing on native Windows. The U-39 Base-controlled
trust-policy workflow remains outside the required set and assigned to Repair
3b. Any missing baseline name, additional failure/error name, or candidate-
only name requires STOP without inline repair.

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
4. Confirm the full native battery's failure/error set equals the measured
   ten-name native-Windows base set exactly and has no candidate-only name.
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

## Superseding diagnosis at `f6d0239` and authorized rebind-slice-2 resolution

This section supersedes the platform-unqualified two-red expectation above and
review check 4. The native-Windows base discriminator established a ten-name
platform baseline, and the committed acceptance record did not clear the eight
candidate-only failures. Jon first authorized source diagnosis only. Later
rulings authorized the producer-governed regeneration and proof refresh
recorded below; the isolated results remain the pre-cure evidence.

At exact local head
`f6d0239c0ecf5de263ab2c1ebd70a5e28f0658a2`, every authorized direct-file
invocation selected exactly one named test, ran exactly one test, and ended in
`FAILED (failures=1)` rather than an import or runtime `ERROR`.

### Isolated assertions

| test | capture SHA-256 | assertion text |
|---|---|---|
| `test_proof_is_valid_and_under_thirty_seconds` | `dfad55eb921c861934907902aed94cee31354fc86dab616971cab978bbea4467` | `AssertionError: False is not true` |
| `test_gate_passes_on_real_repo` | `88c3eeb49aa9af47dc2a8b4462ac36233683aa3863fc2a5d088565552785eb21` | `AssertionError: 1 != 0` |
| `test_run_is_cut_ready` | `dbe138fed92d85478a057584a5d4793c59e1e14f6356b849ee918d382e01bb15` | `AssertionError: False is not true` |
| `test_both_bands_merged_and_subgates_pass` | `f335f1a5536a9439e7b9c919204a9d93efebc699cbb6027e934e3cc75fd1db0f` | `AssertionError: False is not true : all anti-rot sub-gates must pass` |
| `test_gate_passes` | `156b4a6468815f8ac1c417f2eccdc17a54254f855d2b038e43aca18fd8984890` | `AssertionError: 1 != 0` |
| `test_committed_browser_package_and_proof_promote_browser_readiness` | `f4feaf854371c8d4241a3750ae5dcd27b9ffe15fc6aaac8a3fc40e86755cc369` | `AssertionError: False is not true` |
| `test_gate_guards_committed_build_execution_and_browser_evidence` | `a83a1d9934da001dcc8dec19653e96464bb66a1d1d691fb9ed693a0497f2cfe6` | `AssertionError: 0 != 1` |
| `test_markdown_separates_node_proof_from_browser_claim` | `23b0a6742f12a572b6de044228744f9268da6e8c425393ac0a7cd4d9a310703f` | The verbatim assertion is recorded below because it includes the complete rendered Markdown. |

The eighth assertion was:

```text
AssertionError: 'browser ready: **yes**' not found in '# Garnet Wasm / W-PLAY readiness\n\n_Schema garnet.wasm_readiness/v3._\n\n## Recorded product evidence\n\n- `garnet-wasm` crate present: True\n- clean-Windows WV-5 proof valid: True\n- proof commit: `098260e48f5625c409d53646889f27922e08c1e9`\n- wasm32 + wasm-pack builds passed: True\n- real Node execution passed: True\n- build/execution owned bits ready: **yes**\n\n## Remaining browser product surface\n\n- `check_source` export: True\n- capability-surface/diff export: True\n- browser adapter: True\n- browser package present: True\n- browser package valid: False\n- Playwright proof present: True\n- Playwright proof valid: False\n- browser ready: **NO**\n- BLOCKER: browser Wasm package is missing, invalid, or does not match current committed inputs\n- BLOCKER: W-PLAY Playwright browser proof is missing or invalid\n\n## Local reproduction convenience (not product truth)\n\n- wasm32 target installed here: True\n- wasm-pack present here: True\n- Node present here: True\n- wasmtime present here (optional): True\n- miette `fancy` detected: True (recorded WV-5 build proves it is not a blocker)\n\nHonest scope: WV-5 alone proves a real interpreter-to-Wasm build and Node execution. Browser readiness is promoted separately only when the committed package and strict Playwright proof both validate.\n'
```

### Eight-test classification table

| test name | gate script | pinned value | pinned versus candidate actual | classification |
|---|---|---|---|---|
| `test_proof_is_valid_and_under_thirty_seconds` | `scripts/garnet_wasm_readiness.py` (`browser_proof_valid`) | package provenance `source_tree_sha256 = 850cc02753dde0a7de3f89cd22d187e97d8641c9df4bdaad6d6387175d198d8c` | candidate aggregate `556ea1c6250dccac2030e2fa42984a411004f8c3a78291f91b9f19029d114507`; package validity returns false before the otherwise-matching proof checks | **STALE-PIN** |
| `test_gate_passes_on_real_repo` | `scripts/garnet_v0_8_0_cut_readiness.py` → release gate → Wasm gate | same package source aggregate through the release sub-gate | ledger complete; all 11 runway gates pass; release fails only because Wasm exits 1 on the aggregate mismatch | **STALE-PIN** (transitive) |
| `test_run_is_cut_ready` | `scripts/garnet_v0_8_0_cut_readiness.py` → release gate → Wasm gate | same package source aggregate through `release_gate.passed` | `ledger_complete=True`, `runway_pass=True`, `release_gate.passed=False` only because Wasm exits 1 | **STALE-PIN** (transitive) |
| `test_both_bands_merged_and_subgates_pass` | `scripts/garnet_v0_8_0_release_readiness.py` → Wasm gate | same package source aggregate through `sub_gates_pass` | both bands complete; ten other anti-rot sub-gates pass; only `wasm-readiness (S55)` exits 1 | **STALE-PIN** (transitive) |
| `test_gate_passes` | `scripts/garnet_v0_8_0_release_readiness.py` → Wasm gate | same package source aggregate through `release_ready` | `bands_complete=True`; `sub_gates_pass=False` only because Wasm exits 1 | **STALE-PIN** (transitive) |
| `test_committed_browser_package_and_proof_promote_browser_readiness` | `scripts/garnet_wasm_readiness.py` (`browser_package_valid`) | package provenance `source_tree_sha256 = 850cc027…` | candidate aggregate `556ea1c6…`; this is the first failed assertion (`browser_package_valid=False`) | **STALE-PIN** |
| `test_gate_guards_committed_build_execution_and_browser_evidence` | `scripts/garnet_wasm_readiness.py` | package provenance `source_tree_sha256 = 850cc027…` | package false; proof false only because proof validation requires a valid package; all other proof predicates match | **STALE-PIN** |
| `test_markdown_separates_node_proof_from_browser_claim` | `scripts/garnet_wasm_readiness.py` | package provenance `source_tree_sha256 = 850cc027…` | reporter renders `browser package valid: False`, `Playwright proof valid: False`, and `browser ready: **NO**` from the same mismatch | **STALE-PIN** |

All eight therefore classify as **STALE-PIN**. No checked value demonstrates a
behavior regression.

### Exhaustive interpreted pin inventory

The Wasm/package/proof path performs 56 interpreted comparisons. Exactly one
is unequal: the package source aggregate above. Satisfying only that validity
boundary in an in-memory diagnostic makes `browser_proof_valid()` return
`True`; no repository byte was edited for that diagnostic.

| pin site or constraint | pinned value | candidate actual | result |
|---|---|---|---|
| package directory file set | `garnet_wasm.js`, `garnet_wasm_bg.wasm`, `provenance.json` | exact same three files | MATCH |
| `artifacts.garnet_wasm.js` | `6581` bytes; `bf72509961525b4eb2e0702f41da61b4e8087ceee98787b7a046d83a85791a6d` | same bytes and SHA-256 | MATCH |
| `artifacts.garnet_wasm_bg.wasm` | `2215266` bytes; `4df19554877167c63fe683e7584a4d31cf10f9f19c2aae576870d00099febd3c` | same bytes and SHA-256 | MATCH |
| provenance source input shape | 175 sorted, unique, repo-safe files | 175/175 sorted, unique, repo-safe | MATCH |
| provenance `source_tree_sha256` | `850cc02753dde0a7de3f89cd22d187e97d8641c9df4bdaad6d6387175d198d8c` | `556ea1c6250dccac2030e2fa42984a411004f8c3a78291f91b9f19029d114507` | **STALE-PIN** |
| canonical `Cargo.lock` SHA-256 | `01b8986b1cee0ef6a53ac439bd018b54fc1dca825a8f845a259ed8001e6715fa` | same | MATCH |
| canonical Studio package-lock SHA-256 | `e729ee69006fd3e6f5aa6171a93b8477ec51e96f12bb476c1a923df46aa93422` | same | MATCH |
| provenance tool constraints | non-empty versions for cargo, esbuild, node, rustc, wasm-pack | all five present | MATCH |
| proof schema/verdict/duration | `garnet.w-play.browser-proof/1`; `pass`; integer `0 < duration_ms < 30000` | same; `6210 ms` | MATCH |
| proof execution | engine `playwright-browser-page`; no Node global; runtime ready; service workers blocked | exact match | MATCH |
| proof Git constraints | runtime inputs clean; tested commit/tree are 40 lowercase hex | `true`; `12afac12…`; `8fd9b7cc…` satisfy the format contract | MATCH (historical provenance, not current-head equality pins) |
| proof package mirror | provenance schema, source aggregate, and both artifact metadata objects | exact mirror of the currently recorded provenance | MATCH internally; its source aggregate is stale versus candidate bytes as identified above |
| proof Playwright identity | `@playwright/test`; declared `^1.49.1`; version `1.61.1`; recorded integrity; package hashes; `npm ci --ignore-scripts` | exact match | MATCH |
| proof network constraints | external/untracked requests `[]`; 1–32 unique repo-safe requested paths; required HTML/JS/Wasm paths present | exact match; every bound path tracked | MATCH |
| run/check/diff/denial journey contracts | exact schemas and outcomes, including `Hello from Garnet!\n`, authority expansion by `fs`, denial `runtime_error`, empty denial stdout, and a diagnostic containing `proc` | exact match | MATCH |
| console/page errors | `[]` / `[]` | `[]` / `[]` | MATCH |
| screenshot | `f7dadd3f6c4fe041403752da95c4fc383ce9de325fc1918bcbe6ed55283908c2`; desktop ready/no overflow; mobile no overflow | exact match | MATCH |

The ten raw browser-runtime file pins also match exactly:

| runtime file | pinned and actual bytes | pinned and actual SHA-256 |
|---|---:|---|
| `apps/garnet-studio/package-lock.json` | `44280` | `e729ee69006fd3e6f5aa6171a93b8477ec51e96f12bb476c1a923df46aa93422` |
| `apps/garnet-studio/package.json` | `495` | `89aedfa9bf42d4b11e3dc073600675f2df8dfaf46f3e4313ae99ad7a97ad6c41` |
| `docs/icons/garnet-192.png` | `67820` | `790a70b22ea92e6f7004eec6196c0807f612d149a2bb40115d31d44ab68927fa` |
| `docs/playground.html` | `7997` | `01b12abd2046ebe61de8b7d0b4fa45430981211e68d99ec32a1b3cf7ae775942` |
| `docs/playground/examples.json` | `3016` | `eb1b7edb207c2a695b7d51c5b3c4acfac51bf42eb706c0da520b5586a0534d18` |
| `docs/playground/live.js` | `5110` | `0c9b32f144a5acbbeb36c29c1822d12ec2293d2ea29a0a10d8e5432d4a958c91` |
| `docs/playground/pkg/garnet_wasm.js` | `6581` | `bf72509961525b4eb2e0702f41da61b4e8087ceee98787b7a046d83a85791a6d` |
| `docs/playground/pkg/garnet_wasm_bg.wasm` | `2215266` | `4df19554877167c63fe683e7584a4d31cf10f9f19c2aae576870d00099febd3c` |
| `docs/playground/pkg/provenance.json` | `8434` | `81476add10c69abaa03dff8d31ef162a6d4ede7f5f05bd702a31e479dcf6f907` |
| `scripts/smoke_garnet_playground_browser.mjs` | `17844` | `ddbd4ac831a8cc108514527f70ae06ea58b0e8064b51a18897d4c08348d3978d` |

Their aggregate runtime SHA-256 is pinned and actual-identical at
`05b141bd3125202a03f45871095820ccb04bc871dd1541b6dde6d060f9952066`.

The historical WV-5 path used by `read_readiness()` also remains valid:
six required command records are `ok=true`/`exit_code=0`, all four required
Node semantic markers are present, and all 15 manifest entries match their
current bytes:

```text
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  commands/node-smoke-stderr.txt
1f945e7647ffd842a3b46da8e9fc5d0063bf69005855b25e30ac7884b57db20b  commands/node-smoke-stdout.txt
5b19407b22ae5cbe3dabd9ac682b453b506ed14d0d75559733096691bda99a0a  commands/output-capture-tests-stderr.txt
8d986291143e29073a1590425c9f5f6340f722c9403bd0ccf811777165205a11  commands/output-capture-tests-stdout.txt
262290194f3689e8826ddbc074c215feb46109d7033e403820c93fd3d312ca29  commands/wasm-native-tests-stderr.txt
97771179cbdcfe3773cf028d46268cea0026d147ab5df2141f5147dd56950b69  commands/wasm-native-tests-stdout.txt
3b4ff43b5e989e603303fa3c288f9db5e6b034f40e5e955e89e018db2653ab36  commands/wasm-pack-nodejs-stderr.txt
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  commands/wasm-pack-nodejs-stdout.txt
3ce25a515d32d1f8512499f8e6d422c4bf4dbd33eded6056f451eb2d071a0a5b  commands/wasm-pack-web-stderr.txt
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  commands/wasm-pack-web-stdout.txt
fafb8a68401aac80e835e6324ad264964686a5f50c5f7183f368f6acff356058  commands/wasm32-build-stderr.txt
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  commands/wasm32-build-stdout.txt
db4db34ca1a077b4c01be0faf2f36ee27086a23b0cb417a5c238c590d45eb4c7  smoke/wv5_node_smoke.js
a674535c271100d003b9143a28c9c9b760a2839542a1bbf2878094a47129868d  wv5-wasm-lane-proof.json
8546906cf14f641ce690b9e4bf26a554d15a366db8911dbbd08c3eba8c9cb14f  wv5-wasm-lane-proof.md
```

### Source-byte attribution and hypothesis result

Of 75 candidate-changed paths, only two intersect the package provenance's
175 source inputs:

| source input | base bytes / SHA-256 | candidate bytes / SHA-256 | attribution |
|---|---|---|---|
| `garnet-memory-v0.3/AGENTS.md` | `1868` / `7aeee21f9939eb0f09e64d1222af6d2eef2212e55db9d98b5bdbcaeed801c481` | `2266` / `6cfb433b6c5e69c698ee51153d184c44b90c0056c0493bd3eb700696cf86201e` | Lane 2C commits `0649d79` and `5cd1136`; substantive doctrine/example-command additions, not EOL conversion |
| `garnet-memory-v0.3/src/cycle.rs` | `20874` / `b5b13e510b6e35c72d61778c1bf841c9e91a71d93b02edef76704d4cff6d3a65` | `24006` / `42c7e3a1c97bd5aabc87f5b18146e291bf947808484c6d71b88f2f776efc3ed4` | Lane 2C commit `0649d79`; reviewed teardown implementation and tests |

Both base and candidate versions contain zero CRLF sequences, and the package
source aggregate canonicalizes CRLF to LF before hashing. The change is lawful
merged product movement, not a line-ending artifact and not evidence of broken
runtime behavior.

The more specific replicated-pin hypothesis is falsified. None of the ten
browser runtime inputs changed between `efd4f6b` and `f6d0239`.
`.gitattributes`, `D_Executive_and_Presentation/garnet-website.html`,
`.dogfood/windows-audit-goal.json`, `.dogfood/windows-core-audit.json`, and
`proofs/minimum-shelf/lane2b/PROOF.json` changed but are not read by this
eight-test gate chain. Release/cut read unchanged `.dogfood/v0_8_goal.json`:
S31–S79 are all `merged`, and S41–S59 all carry merge confidence 5. There are
no `.gitattributes` pins or Markdown-section hashes in these failing paths.

The exact rebind-slice-2 evidence scope is therefore the single stale package
source aggregate and its proof mirrors/cascades: provenance
`source_tree_sha256`; the proof package's mirrored source SHA; and, if the
provenance bytes move under an authorized producer, the proof's pinned
`docs/playground/pkg/provenance.json` byte count/SHA and aggregate runtime SHA.
Artifact, lockfile, browser-runtime, Playwright, screenshot, duration, WV-5,
journey, network, release-ledger, and runway pins are not part of the observed
mismatch. This is a scope enumeration, not authority to edit any of those
sites.

### Producer-governed census correction and resolution

The manual census above was wrong. It rehashed the 175 paths already present
in the historical provenance record and therefore could not discover a new
producer input. The sanctioned producer's current `INPUT_ROOTS` census governs:
it found 176 sorted, unique, repo-safe source inputs and added
`garnet-memory-v0.3/examples/lane2c_teardown_probe.rs`. The binding source
aggregate is therefore
`c36f0e45ea14dbceaf4c91c969257271d5f7cb662d65fb6ce1d3eede2d7cb562`,
not the manual 175-input result
`556ea1c6250dccac2030e2fa42984a411004f8c3a78291f91b9f19029d114507`.

The producer-definition fork resolved **YES — REGENERATION**. In
`scripts/build_playground_wasm.py`, `INPUT_ROOTS` includes
`garnet-memory-v0.3` and the producer builds `garnet-wasm`; the locked target
dependency tree is `garnet-wasm -> garnet-interp -> garnet-memory`, whose
`src/lib.rs` compiles `cycle.rs`. Pin editing alone would therefore have
blessed a stale Wasm artifact over changed compiled sources.

The locked Studio dependency install was exactly
`npm ci --ignore-scripts` on the NTFS-local checkout. `node_modules/` was
git-ignored before and remained absent from Git status afterward. The build
environment was Node `v22.22.2` and npm `10.9.7`; `package-lock.json` remained
byte-identical at
`e729ee69006fd3e6f5aa6171a93b8477ec51e96f12bb476c1a923df46aa93422`.
npm reported three high-severity audit findings. They were observed and left
unchased because this re-acceptance slice authorized the locked install and
artifact regeneration, not dependency remediation.

The sanctioned materializing producer attested `reproducible: true`. Before
commit, the staged index bytes—not merely the worktree—rehash-matched all three
recorded outputs:

| generated output | bytes | staged SHA-256 |
|---|---:|---|
| `docs/playground/pkg/garnet_wasm.js` | `6581` | `bf72509961525b4eb2e0702f41da61b4e8087ceee98787b7a046d83a85791a6d` |
| `docs/playground/pkg/garnet_wasm_bg.wasm` | `2217102` | `60887f721e57e7309564edfb5eb5a99f4b01d1839fb4c0800e8d7ef9685a737f` |
| `docs/playground/pkg/provenance.json` | `8486` | `156dccd2eb3515125cf400fe9879af8c4f68d35db09cd9759b1c6a10a7fb21a9` |

The preserved bytes were committed without regeneration at
`2082b291189ef71017517701b9d01443e6683a75`. The JavaScript blob matched HEAD,
so the commit's file delta contains the Wasm and provenance files while the
index-time three-file hash check covers the complete package.

The sanctioned Playwright producer then passed in `2174 ms` with six committed
requests. Its output was committed at
`6994237941667e06ff9fce7711596aea1e4fbdbb` after staged-byte verification:

| proof output | bytes | staged SHA-256 |
|---|---:|---|
| `F_Project_Management/LAUNCH/W_PLAY_BROWSER_PROOF.json` | `6398` | `85cfe5e7376156d2a83e2f30c0e70a4090d5c3d78dd9cd015437605317943779` |
| `ops/lane2a/evidence/30-playground-browser.png` | `51262` | `52c4e9878aa8354c21bd25bfd28982dbce323748a71b1e167f0bdde5119fe1ee` |

The refreshed proof mirrors the binding `c36f0e45…` source aggregate and the
new artifact metadata. Its runtime-input aggregate is
`1de71131204c1463694343026acb943e938bd28cddefd86a45f1a8a308363f15`.
At `6994237`, `python -I scripts/garnet_wasm_readiness.py --gate` passes with
`browser_package_valid=true`, `browser_proof_valid=true`, and
`browser_ready=true`. Thus the eight-test stale-pin chain has been resolved by
regenerating the artifact that consumes the changed sources and refreshing its
strict browser proof; no behavioral regression was found in that chain.

Because this request is the digest-included freeze boundary, the final native-
Windows battery transcript and the completed U-54/U-55/U-56 registrations are
carried at the later U-35 tip in the digest-excluded record
`F_Project_Management/W_TRUST/WV6_REACCEPTANCE_REGISTRATIONS_2026-08-10.md`.
The reviewer must read that record from the exact advertised branch tip and
verify its transcript hashes and named-set comparison. The request is not
ready until that record exists and the remote tip is confirmed.

## STOP

The implementer stops after publishing the exact candidate and this request.
Jon-only approval, merge, tag, release, and token actions remain unperformed.
