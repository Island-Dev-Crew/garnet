# WV-6 integration re-acceptance — final registrations and evidence index

## Seats and frozen boundary

- Implementer: OpenAI Codex, GPT-5-based agent; exact model version is not
  exposed by this harness, on `NUCBOX_M2PRo_S` (Windows 11 Pro
  `10.0.26200`, `AMD64`).
- Independent reviewer requested: Claude Code on Claude Fable 5 (Anthropic),
  on the MacBook Air reviewer seat.
- Review carrier: `IDC-Trust-Review` only.
- Merge authority: Jon (`IslandDevCrew`) only.

The final digest-included boundary is
`410ff1182cdcefcec9fe046d1346205d8522ec9d`, tree
`57ce26ae1ab8d24609180486bc5fce6179f37957`, product pair
`fd96e6d910180f5e33999fbd693ea211e336389a13535930d89b2a870ff54727 / 1606`.
The reporter rebind is `31e35151d506305817c2c1b344a0a79c77a821ae`.
The committed WV-6 acceptance bundle is
`ec8ce8f249cc820f1da8f1e818da13e8d9f2d33f`; its verifier passes 5/5 and
binds the frozen head, tree, and product digest above. This record remains
under `F_Project_Management/W_TRUST/**`; captured byte streams that violate the
text-byte policy live under the fenced `proofs/**` evidence surface. Both are
product-digest-excluded, so this later U-35 tip does not move the frozen pair.

## Finding registry index

| ID | state | discoverable title |
|---|---|---|
| U-52 | REGISTERED — ACTIVE PROCEDURAL CONSTRAINT | Worktree admissibility |
| U-53 | PROPOSED — DEFERRED | Gate diagnostics swallow underlying messages |
| U-54 | PROPOSED — DEFERRED | Platform-qualified full-battery expected-red sets |
| U-55 | PROPOSED — DEFERRED | Evidence custody window |
| U-56 | PROPOSED — DEFERRED | Repository-wide producer and pin census |

## U-52 — worktree admissibility

**REGISTERED — ACTIVE PROCEDURAL CONSTRAINT.** When a lane or review ceremony
requires a fresh clone, a linked Git worktree is not an admissible substitute.
The boot record must show the clone path, `core.autocrlf`, branch/ref source,
and `git worktree list`; the ceremony stops if another checkout is being used
as the claimed fresh-clone evidence seat. This registers the identifier already
used by the WV-6 reviewer without changing the constraint it names.

## U-53 — gate diagnostics swallow underlying messages

**PROPOSED — DEFERRED.** Some composed gates capture a failing subcommand but
surface only the wrapper result, swallowing the underlying diagnostic that
identifies the actual mismatch. The governance cure is to propagate bounded
stdout/stderr with the subcommand identity while preserving deterministic gate
output. No gate implementation changes in this records-only correction.

## U-54 — platform-qualified full-battery expected-red sets

**PROPOSED — DEFERRED.** The native-Windows full-battery baseline differs
from the Linux/macOS expected-red set. Expected-red sets must be qualified by
platform and measured in the same native environment. Candidate/base
comparisons are by failure/error test name, not count.

The base and final-candidate sets are identical:

| kind | test name |
|---|---|
| ERROR | `test_output_dir_writes_manifested_pack` |
| ERROR | `test_output_dir_writes_manifested_plan` |
| ERROR | `test_symlink_cannot_be_used_as_clause_evidence` |
| ERROR | `test_output_dir_writes_manifested_evidence_bundle` |
| FAIL | `test_repo_and_site_point_to_the_adoption_surface_reporter` |
| FAIL | `test_tracked_ledger_matches_renderer_byte_for_byte` |
| FAIL | `test_all_novel_programs_check_and_run` |
| FAIL | `test_tag_release_publishes_unified_checksummed_assets` |
| FAIL | `test_run_script_stages_swiftpm_gui_app_bundle` |
| FAIL | `test_missing_studio_adoption_copy_is_a_strict_blocker` |

- Base: 1,130 tests in 857.498 s, 6 failures, 4 errors, 5 skips;
  transcript SHA-256
  `aacddc271ed737db4d8c44b0f7b91e6788b0ca47a62c72efd4b88c747cf685ed`.
- Final candidate: 1,141 tests in 889.659 s, 6 failures, 4 errors, 5
  skips; transcript SHA-256
  `e48a97d8d62c46355db7346e764387e19f8c14a5cb59164e339e76b0dcbc5405`.
- Set delta: missing `[]`; additional `[]`; candidate-only `[]`.

The differing test totals reflect added branch tests and are not used as the
baseline discriminator. Wall time is context only and is not an acceptance
claim.

## U-55 — evidence custody window

**PROPOSED — DEFERRED.** Evidence generated but not immediately committed is
one careless command from nonexistence. On evidence-producing branches, the
doctrine is generate -> hash -> commit in the same breath. The battery gate
comes after the acceptance commit, following Phase 0.

Exhibit one: the first generated WV-6 bundle was destroyed by a literal-
wildcard copy followed by checkout restoration before commit. A sanctioned
same-head regeneration reproduced all six recorded hashes exactly, proving
determinism rather than claiming preserved custody.

Exhibit two: the Wasm producer generated a valid reproducible package whose
source census falsified the manual expected digest. The wrapper stopped without
staging but left all three outputs intact. No checkout or wildcard intervened;
Jon bound `c36f0e45.../176`, and all three staged index blobs rehashed to the
recorded values immediately before commit `2082b29`.

## U-56 — repository-wide producer and pin census

**PROPOSED — DEFERRED.** Rebind scope must derive from a repository-wide
producer/pin census, not one reporter's pins or an old provenance input list.

Exhibit one: the first rebind enumerated the Minimum Shelf reporter and its
mirrors but did not discover the independent browser/Wasm source aggregate,
leaving eight correct fail-closed readiness failures.

Exhibit two: the manual browser diagnosis rehashed the historical 175-path
provenance list and produced non-binding
`556ea1c6250dccac2030e2fa42984a411004f8c3a78291f91b9f19029d114507`.
The sanctioned producer's live `INPUT_ROOTS` census found 176 inputs, including
`garnet-memory-v0.3/examples/lane2c_teardown_probe.rs`, and governs with
`c36f0e45ea14dbceaf4c91c969257271d5f7cb662d65fb6ce1d3eede2d7cb562`.
Because the built dependency chain compiles `garnet-memory-v0.3/src/cycle.rs`,
artifact regeneration—not pin-only editing—was required.

## Finding-ID sweep

The fork advertised 462 heads. Fork `main` was listed but not fetched or read.
All 461 non-`main` heads were fetched into collision-free numbered temporary
refs; no `refs/pull/*` was fetched or present. A whole-tip grep found U-54
through U-60 unassigned. U-54, U-55, and U-56 are therefore the next available
registrations for this lane.

## Build and package provenance

- Checkout: `C:\gvr806-a31f2`, NTFS-local, outside OneDrive.
- Quiet state: not required and not claimed.
- `core.autocrlf=false`.
- Node `v22.22.2`; npm `10.9.7`.
- Install command: exactly `npm ci --ignore-scripts`.
- Studio lockfile SHA-256:
  `e729ee69006fd3e6f5aa6171a93b8477ec51e96f12bb476c1a923df46aa93422`.
- `node_modules/` was ignored before and absent from Git status after.
- npm reported three high-severity audit findings. They were recorded and not
  chased because dependency remediation is outside this re-acceptance slice.
- Wasm source aggregate:
  `c36f0e45ea14dbceaf4c91c969257271d5f7cb662d65fb6ce1d3eede2d7cb562 / 176`.
- Wasm package hashes: JS
  `bf72509961525b4eb2e0702f41da61b4e8087ceee98787b7a046d83a85791a6d`,
  Wasm
  `60887f721e57e7309564edfb5eb5a99f4b01d1839fb4c0800e8d7ef9685a737f`,
  provenance
  `156dccd2eb3515125cf400fe9879af8c4f68d35db09cd9759b1c6a10a7fb21a9`.
- Browser proof SHA-256:
  `85cfe5e7376156d2a83e2f30c0e70a4090d5c3d78dd9cd015437605317943779`;
  screenshot SHA-256:
  `52c4e9878aa8354c21bd25bfd28982dbce323748a71b1e167f0bdde5119fe1ee`.

## WV-6 acceptance output hashes

The final native acceptance command transcript hashes to
`1841e151c96ab25adbd1cc550375e747abd1b73789cf415b9387af0864a1b1d2`.
The six sanctioned producer outputs staged at the acceptance commit were:

| output | bytes | SHA-256 |
|---|---:|---|
| `f1-canonical-reseal.txt` | 2801 | `02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7` |
| `mcp-session.input.hex` | 1073 | `64b2f3e15489b4f06e57a0c51afce960beda3edc9c8009d5f3a530d5d9f92638` |
| `mcp-session.output.hex` | 1797 | `15423fa5ba9697f59f877ea0a9afb3587230fc4d7c3182cee82ce709bbce4799` |
| `minimum-shelf-status.json` | 2070 | `a9a682355ee6ca6e0524fa2aeb1e8c4a12968fbfd76a520b1dddc2e02ad19a03` |
| `reporter-cross-checkout.txt` | 1660 | `a4f33e467349bf84a6c9fe04e7ddd486df335a96f2edda0b2cb371787d26425f` |
| `WV_ACCEPTANCE.json` | 1978 | `56fea133501eab5692a484900d3a15c272c406222b8b6877d0b92b1633fd7fe3` |

## Preserved transcript index

| path | role | encoding | SHA-256 |
|---|---|---|---|
| `proofs/windows/launch-verification/wv6-transcripts/WV6_REACCEPTANCE_NATIVE_WINDOWS_2026-08-09.txt` | initial native ceremony transcript; ends at the first full-battery start and is diagnostic only | UTF-8 BOM | `064dd5194fcf8afcd4ca0d7d09d781e08903343f3e10a78c7b443b78643aabc2` |
| `WV6_REACCEPTANCE_FULL_PYTHON_BATTERY_2026-08-09.txt` | pre-acceptance candidate; 18-name set exposing eight candidate-only reds | UTF-16 LE | `7decf0d50e02190aa45382961262120292b0338222bd89242f73b845dc69aa09` |
| `WV6_REACCEPTANCE_BASE_FULL_PYTHON_BATTERY_2026-08-09.txt` | same-platform base discriminator; binding ten-name base set | UTF-16 LE | `aacddc271ed737db4d8c44b0f7b91e6788b0ca47a62c72efd4b88c747cf685ed` |
| `WV6_REACCEPTANCE_POST_ACCEPTANCE_FULL_PYTHON_BATTERY_2026-08-10.txt` | acceptance-commit ordering discriminator; still 18 names before browser artifact regeneration | UTF-16 LE | `b45d770e557399588b46717754e8502c5648cfcae4d35f456131267a764769aa` |
| `WV6_REACCEPTANCE_FINAL_ACCEPTANCE_BATTERY_2026-08-10.txt` | final native WV-6 acceptance commands | UTF-16 LE | `1841e151c96ab25adbd1cc550375e747abd1b73789cf415b9387af0864a1b1d2` |
| `proofs/windows/launch-verification/wv6-transcripts/WV6_REACCEPTANCE_FINAL_FULL_PYTHON_BATTERY_2026-08-10.txt` | final accepted candidate; exact ten-name platform baseline | Windows-1252 | `e48a97d8d62c46355db7346e764387e19f8c14a5cb59164e339e76b0dcbc5405` |

The final full battery ran as one uninterrupted Python process. Its monitoring
wrapper timed out at ten minutes, but the original child process remained
active and continued writing the same transcript; no replacement battery was
started. Process completion was observed before the transcript was hashed and
parsed.

## Review readiness

`ops/wv6-reaccept/review/01-request.md` contains the eight-test diagnostic,
the producer-governed 176-input correction, regeneration decision, and proof
resolution. The exact advertised branch tip must contain this record and all
indexed transcripts. Request 01 is then ready for the Air's independent
Claude Fable 5 review. The implementer writes no verdict.
