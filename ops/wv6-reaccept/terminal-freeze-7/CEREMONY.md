# WV-6 terminal freeze 7 ceremony record

## Seat and frozen boundary

- Seat: OpenAI Codex on the native-Windows NUC.
- Checkout: `C:\garnet-freeze7-20260821` on NTFS, outside OneDrive.
- Cold-start process count: `453`; a pre-custody reconnaissance count was
  `470`, and the post-emission count was `468`. All were below the directing
  baseline of approximately `482`, and long gates were serialized.
- OneDrive was user-declared paused. One OneDrive process was present; this
  seat did not independently inspect the application's pause UI.
- `core.autocrlf=false` globally before clone and in the fresh clone before
  the first repository file read.
- Commit identity: `OpenAI Codex <codex@openai.com>`.
- Main base: `1d765cdb2e69bc097cd33db30f9919ad8e969208` / tree
  `a7de550d8f853c6a50d69f87d3fb6d5919cfef29`.
- Final cure / reviewed head:
  `511e0fabad7335d14e972ffb968c7ac5e9b57ca8`.
- Reviewed tree: `b7131198e99b01ab23aa75008e1e25acfca906c8`.
- Natively computed pair:
  `87d5204c6b0c989d09e06dc176ab36096cd7a02fb31a2c7b600bcc1f7dd88058 / 1643`.

The checked-in `tracked_content_digest` producer and an independent raw-byte
implementation over `git --no-replace-objects ls-tree -r -z` produced the same
pair. The independent method applied the four frozen mutable prefixes and the
exact reporter-path exclusion directly, sorted `(raw_path, blob_oid)`, and
hashed `path + NUL + oid + LF`. The containing commit is the commit that first
introduces this record; this file intentionally does not embed its future SHA
or tree.

## Authenticated live ruleset observation and custody

- Ruleset: ID `18936562`, `Garnet main - human-gated trust kernel`.
- Authenticated reader: Jon (`IslandDevCrew`) using the dedicated U-17
  credential on `2026-08-21`.
- Token custody: Jon only. The NUC seat neither received, inherited, logged,
  printed, nor persisted that credential.
- Jon-attested raw response: `5193` bytes, SHA-256
  `379ae4e509777945fd04b7dc9ec9555b44a5cf286c6cc4b274cb1fb00186e628`.
  Jon retains the raw-byte source.
- Out-of-band transport: attachment `pasted-text.txt`, strict UTF-8, no BOM,
  `7422` bytes, SHA-256
  `d0cf82302a6881d762c7819385f47ca770bb6a5a80162d2b1967e118ab86978d`.
  The required-status rows were deliberately reflowed, so this attachment
  hash is not represented as an independent reproduction of Jon's raw hash.
- Object timestamps: `created_at=2026-07-14T10:13:05.778-05:00` and
  `updated_at=2026-07-14T10:41:47.664-05:00`.
- Projection: the gate's own exact key set, `name`, `target`, `enforcement`,
  `bypass_actors`, `conditions`, and `rules`; `_strict_equal` was unchanged.
- Required-status observations: exactly 31 unique rows, exact checked order and
  values, every `integration_id=15368`,
  `strict_required_status_checks_policy=true`, and
  `do_not_enforce_on_create=false`.
- Bypass observations: `bypass_actors=[]` and
  `current_user_can_bypass="never"`.

The complete live-versus-checked projected divergence was exactly one field:

| JSON path | Checked value | Live value | Classification | Reasoning |
|---|---|---|---|---|
| `rules[3].parameters.require_extra_approval_for_unattributed_changes` | absent | `true` | STRENGTHENING | Adds an approval requirement rather than relaxing one. |

There was no other projected divergence. Ignored response metadata remained
outside `RULESET_KEYS`; no authorization header, token, cookie, or other
credential appears in repository evidence or raw stream captures.

## Occasioning changes

### A1 — first additive ruleset evolution

Commit `ac2060b4b7d52d17856053b59c6203b7f67327af` cleanly carries source
`b955f997f7f47868502aaa5c4ac38468a8b7e45a`. It binds the live additive
`dismissal_restriction {enabled: false, allowed_actors: []}` and
`required_reviewers: []` fields without weakening `_strict_equal` or the
governance posture.

### A2 — external registry yank and reversal

Commit `5e80ed2fd3c64144dccbaeb3eaf43f4e7e25e5ca` cleanly carries source
`415667a39e2e49f981e9848b21ab59f6a3b0044e` and the exact
`arrayref@0.3.9` cargo-deny exception. Commit
`78336e428e591aa113af455631796a16fe53cbd4` cleanly carries source
`7ddb5911095fbbb631545294b89a5f3b4aada869` and records its disjunctive
removal condition. The reviewing seat observed the registry yank reversed on
`2026-08-20`; the exact-version exception is presently inert and remains a
timestamped, non-widening allowance. Global `yanked = "deny"` and
`Cargo.lock` remain unchanged.

### A3 — floating-toolchain lint activation with pin cure

Rust/Clippy 1.98.0 activated `chunks_exact_to_as_chunks` under `-D warnings`.
Commit `e0ce757fd9c16c9f897447edfcd6310a03f78ff7` cleanly carries source
`096626fdafe390c3b112fce49337cb660ea434ac`, adopts
`as_chunks::<2>().0` after the existing odd-length guard, pins the existing
Clippy job to exact Rust `1.98.0`, and moves the producer-derived
required-context digests. No root `rust-toolchain.toml` or lint suppression
was added.

Commit `9cd8198a433529d3411b033d2473b628cf44da62` cleanly carries source
`52bd88b82e29be6878e0995069ff4bb14b26ba30` and refreshes the source-bound
Wasm provenance. Freeze-6's browser-proof refresh `ba9fa6fe...`, ceremony
record `017c5dde...`, and structured review `8af3f4f9...` were intentionally
not carried.

### A4 — second additive ruleset evolution

Commit `2e7b263ec359a6f45413826e1f9602972f4b4ca5` adds the sole divergent
field with its exact live value, keeps `_strict_equal` unchanged, and binds
the 31-context document SHA-256
`ff27197848678f5472788c111c84a3e8545ffe0391f9ab429f12bb26e9d6b727`
plus derived 32-context document SHA-256
`7e5df0fc5aac0b518f5c9cacf7eace1cc05737a55182feee728af6fda85f1dea`.
The live-shaped positive fixture returns zero problems and 31 bindings; the
field-absent and `false` fixtures both fail closed with
`live ruleset policy mismatch`.

Review statement, verbatim from the A4 commit:

> require_extra_approval_for_unattributed_changes: true is STRENGTHENING — it adds an approval requirement rather than relaxing one. The live governance posture is at least as strict as the originally reviewed contract, and this is the second additive-evolution event on this object within 48 hours. The ruleset was created at 2026-07-14T10:13:05.778-05:00 and last updated at 2026-07-14T10:41:47.664-05:00; that five-week-old updated_at proves the object itself was not modified for this newly visible field, which appeared through additive API serialization evolution rather than a governance change. The dedicated U-17 read attests bypass_actors: [], current_user_can_bypass: "never", and 31 required status checks under strict_required_status_checks_policy: true.

### A5 — dogfood body discipline registrations

Commit `511e0fabad7335d14e972ffb968c7ac5e9b57ca8` registers both findings in
`ops/lane1/evidence/101-dogfood-body-discipline-findings.md`, observed
`2026-08-21` and routed to Repair 3b:

1. `EVIDENCE_HEADINGS` is selected in tuple order rather than document order.
2. A workflow re-run replays the original event payload, so a body-dependent
   cure requires a new event.

This boundary changes no checker, test, workflow, trigger, permission, or
network behavior. The PR body must pass locally with checked items under both
real evidence headings before the PR is opened.

## Local verification

- Governance gate 33/33; workflow identity 6/6; governance transport 24/24;
  base-controlled 4/4; activation 10/10.
- Required-context contract: 14 passed, 2 native-Windows skips; evaluator
  13/13.
- Agent contracts: 24 contracts plus 6/6 tests.
- MSRV: 25/25; live gate `ok=true`, MSRV `1.95`, 18 active manifests.
- Native WV predicates: 3/3, 2/2, 1/1, and 6/6.
- `git diff --check` was clean before each cure commit.

One preliminary capture wrapper invoked bare `cargo` help because its `$Args`
parameter collided with PowerShell's automatic variable. It executed no test
and is excluded from all counts. The authoritative direct-process captures
were:

| Predicate | stdout SHA-256 | stderr SHA-256 |
|---|---|---|
| 3/3 Core Ring Tier 1 | `253f444b15e98718e8a604cf2922bcd5354318fcb3bb43667202825917f12fd1` | `1bd33d4c5aa8d62dfe4a032f0caed67363b4f2f0e0688b779dbcf020b9dc4b06` |
| 2/2 MCP stdio | `d53c3c979f46a4aae639724d8820de1e812e60f842809933ef43019924930d49` | `984af4fb4d11a211c52e09433c959ab1ae27fc50d654f764fd03fd211f331e5c` |
| 1/1 sealed package | `9c354624909e99c7a8637d71657af9db6f80ff9bf661783a84b583a4235585d8` | `d5ff3f06818748f3899dd5b12d457da18f609fba9ac45966e570557c223178c4` |
| 6/6 rejection cases | `a910acc0bd286916cf984d5ea7e75343fc79086e222a856bc6dcef9e39bcd51c` | `d5ff3f06818748f3899dd5b12d457da18f609fba9ac45966e570557c223178c4` |

## Supersession with preservation

State: `SUPERSEDED-WITH-PRESERVATION`.

- Superseded reviewed head:
  `218047425fd6871d6cb3ad526ef77e3f4df4c669`.
- Superseded reviewed tree:
  `9cbd7be6810f1f2852d4908fecd64cd66f75fa9c`.
- Superseded pair:
  `056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc / 1640`.
- Preserved manifest SHA-256:
  `e4f5ebdf9f3936765bf20070837f7953538d6a60024da03f755cc87e8c792579`.
- Complete preserved bundle:
  `proofs/windows/launch-verification/wv6-terminal-freeze-20260821/superseded-218047425fd6871d6cb3ad526ef77e3f4df4c669/`.
- Raw moved predecessor:
  `C:\garnet-freeze7-capture-20260821\superseded-live-working-copy-218047425fd6871d6cb3ad526ef77e3f4df4c669`.

| Preserved file | Bytes | SHA-256 |
|---|---:|---|
| `f1-canonical-reseal.txt` | 2801 | `02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7` |
| `mcp-session.input.hex` | 1073 | `64b2f3e15489b4f06e57a0c51afce960beda3edc9c8009d5f3a530d5d9f92638` |
| `mcp-session.output.hex` | 1797 | `15423fa5ba9697f59f877ea0a9afb3587230fc4d7c3182cee82ce709bbce4799` |
| `minimum-shelf-status.json` | 2070 | `a595cd0895a05427316fee734812adfc20e8b429f1b72a9ecb62401b651268a9` |
| `reporter-cross-checkout.txt` | 1660 | `a4f33e467349bf84a6c9fe04e7ddd486df335a96f2edda0b2cb371787d26425f` |
| `WV_ACCEPTANCE.json` | 1978 | `e4f5ebdf9f3936765bf20070837f7953538d6a60024da03f755cc87e8c792579` |

The live six-file bundle was producer-censused from its manifest, copied, and
matched by filename, byte length, and SHA-256 before the original directory
was moved intact. The producer then emitted exactly once into the vacant live
destination.

## Fresh live bundle

| Live file | Bytes | SHA-256 |
|---|---:|---|
| `f1-canonical-reseal.txt` | 2801 | `02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7` |
| `mcp-session.input.hex` | 1073 | `64b2f3e15489b4f06e57a0c51afce960beda3edc9c8009d5f3a530d5d9f92638` |
| `mcp-session.output.hex` | 1797 | `15423fa5ba9697f59f877ea0a9afb3587230fc4d7c3182cee82ce709bbce4799` |
| `minimum-shelf-status.json` | 2070 | `02e2b46b3c2451126e2c297a049fd224725258a292fabfb63db5d96ad6b2db72` |
| `reporter-cross-checkout.txt` | 1660 | `a4f33e467349bf84a6c9fe04e7ddd486df335a96f2edda0b2cb371787d26425f` |
| `WV_ACCEPTANCE.json` | 1978 | `76c147ab9e49216a3f7ccca552062bf79d7cc8cad00e86839e7bfb043bbce04e` |

Fresh manifest SHA-256:
`76c147ab9e49216a3f7ccca552062bf79d7cc8cad00e86839e7bfb043bbce04e`.

## Acceptance succession

1. `2cb25d0b47f55c9dd987bf69fc8a97dade5a4767ef0beda0abc8616808dddc0c / 1605`.
2. `fd96e6d910180f5e33999fbd693ea211e336389a13535930d89b2a870ff54727 / 1606`.
3. `1b45387599223adbe8766ae2e04ddc70e4ecc359712a4867ef1973dac9bd42e4 / 1629`.
4. `8ea996129074e2e70c5ad2f9abd6082c85f7021e943dc017d3a9ba2859ed3ac7 / 1634`.
5. `32f3975537470cb1788acfb3365725c4fea208a049e20fb90b1c314b719cba06 / 1637`.
6. `056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc / 1640`.
7. `87d5204c6b0c989d09e06dc176ab36096cd7a02fb31a2c7b600bcc1f7dd88058 / 1643`.

All accepted predecessors remain intact. The three never-merged boundaries
below are deliberately not inserted into this succession.

## Superseded never-merged boundaries

- PR #525: pair
  `0513edb99391ad4dfe75dffe3618c8c001f877aa4d4f738709c4b506ec06b425 / 1643`;
  record tip `b31f273022d4e2b411f9650b5123543e7accfb41`.
- PR #526: pair
  `573523661f3569d925a05f4c95549582d765212c6ef650871432d35c00c12675 / 1643`;
  record tip `a88813360f1d550a4d209a4cea441fdb9cba1bd6`.
- PR #527: pair
  `449ba9b7aa948cb6fe5e1320385025ea563bc9e5c5ac69cc5ae6b6670bb2a9ee / 1643`;
  record tip `8af3f4f98ade1959b12938c1139ded81a9b2cf67`.

All three never reached main, are superseded with preservation, and remain
outside acceptance succession. Jon owns their closure after this successor
opens.

## U-55 custody timeline

- Custody breath began: `2026-08-21T22:14:24.1170699Z`.
- Producer-censused preservation copy complete:
  `2026-08-21T22:14:24.2067194Z`.
- Original live directory moved intact:
  `2026-08-21T22:14:24.2205920Z`.
- Rebound Shelf gate green with the producer destination absent:
  `2026-08-21T22:14:46.9083397Z`.
- Sole producer emission: `2026-08-21T22:14:57.7344998Z` through
  `2026-08-21T22:14:58.8701693Z`.
- First post-emission WV-6 acceptance gate green:
  `2026-08-21T22:15:07.6823821Z`.
- Fresh manifest and full bundle census complete:
  `2026-08-21T22:15:31.2396570Z`.

Staged regular-file hashes were computed after this record was complete and
immediately before the containing commit. The containing commit is not
self-referenced. Post-commit U-56 verification must producer-census the exact
`REVIEWED_HEAD..tip` walk and reject every non-record path.

## Head-versus-tip boundary

The accepted pair remains bound to final-cure head
`511e0fabad7335d14e972ffb968c7ac5e9b57ca8`. The reporter, four `PROOF.json`
mirrors, fresh live bundle, preserved predecessor, and these three ceremony
records are established post-acceptance record-class paths. The
`.gitattributes` SHA-256 was recomputed as
`b2a14050a850391f8ed1c788f9a6a66155a423ebceb3bb4722478dcaec97dd1b`
and required no byte change.

No structured review record, Air confirmation, carrier approval, U-17
readback, merge, tag, release, or token action is part of this ceremony.
