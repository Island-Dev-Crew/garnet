# WV-6 integration re-acceptance journal

## Seat and wake facts

- Implementer: OpenAI Codex, GPT-5-based agent; exact model version is not
  exposed by this harness.
- Machine: `NUCBOX_M2PRo_S`; Microsoft Windows NT `10.0.26200.0`, `AMD64`.
- Native checkout: `C:\gvr806-a31f2`, NTFS-local and outside OneDrive.
- Branch: `mission/wv6-reaccept`.
- Integration base and verified `origin/main`:
  `efd4f6bae8b3afaba74594e57944b2548142aeae`.
- Global `core.autocrlf=false`.
- This wake record was refreshed at `2026-08-10T23:12:08.3859212Z`.

Merge authority remains Jon (`IslandDevCrew`) only. Review carrier remains
`IDC-Trust-Review` only. The implementer does not approve, merge, tag, release,
mint tokens, or write the reviewer verdict.

## Acceptance-bundle custody loss and deterministic regeneration

The original generated WV-6 bundle was destroyed before commit by a literal-
wildcard copy followed by checkout restoration. The committed bundle at
`f6d0239c0ecf5de263ab2c1ebd70a5e28f0658a2` is a hash-verified,
byte-identical regeneration at the same producer head. That is a stronger
claim than preserved custody: producer determinism was demonstrated rather
than assumed. The original recorded hashes and regeneration hashes were:

| output | original recorded SHA-256 | regeneration SHA-256 |
|---|---|---|
| `f1-canonical-reseal.txt` | `02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7` | `02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7` |
| `mcp-session.input.hex` | `64b2f3e15489b4f06e57a0c51afce960beda3edc9c8009d5f3a530d5d9f92638` | `64b2f3e15489b4f06e57a0c51afce960beda3edc9c8009d5f3a530d5d9f92638` |
| `mcp-session.output.hex` | `15423fa5ba9697f59f877ea0a9afb3587230fc4d7c3182cee82ce709bbce4799` | `15423fa5ba9697f59f877ea0a9afb3587230fc4d7c3182cee82ce709bbce4799` |
| `minimum-shelf-status.json` | `b90bfabb8dc2758a3f9d5b0dc43fcfed32312bfaa00844739f6e5a4e22576d2b` | `b90bfabb8dc2758a3f9d5b0dc43fcfed32312bfaa00844739f6e5a4e22576d2b` |
| `reporter-cross-checkout.txt` | `a4f33e467349bf84a6c9fe04e7ddd486df335a96f2edda0b2cb371787d26425f` | `a4f33e467349bf84a6c9fe04e7ddd486df335a96f2edda0b2cb371787d26425f` |
| `WV_ACCEPTANCE.json` | `13b6898b190a716c105fc524779a93998c48f17c2c973a1b34dbfc42323b6c80` | `13b6898b190a716c105fc524779a93998c48f17c2c973a1b34dbfc42323b6c80` |

Doctrine recorded from the incident: on evidence-producing branches,
generate -> hash -> commit in the same breath. The acceptance commit precedes
the full battery gate on re-acceptance branches, following Phase 0; the old
generate-hold-verify-commit order is obsolete. The literal-wildcard copy and
checkout overwrite are the first custody exhibit.

## Rebind slice 2 producer record

The browser/Wasm producer fork resolved to regeneration because the producer
declares `garnet-memory-v0.3` as an input root and builds a dependency chain
that compiles `garnet-memory-v0.3/src/cycle.rs`. The current producer census is
176 inputs and binds
`c36f0e45ea14dbceaf4c91c969257271d5f7cb662d65fb6ce1d3eede2d7cb562`.
The earlier manual 175-input census and `556ea1c6...` digest were wrong because
they reused the historical provenance path list; the producer census governs.

Studio dependencies were installed only with `npm ci --ignore-scripts`.
Build-environment provenance is Node `v22.22.2`, npm `10.9.7`; the lockfile
remained at
`e729ee69006fd3e6f5aa6171a93b8477ec51e96f12bb476c1a923df46aa93422`.
`node_modules/` was ignored before and absent from Git status afterward. npm
reported three high-severity audit findings; they were observed and not chased
because dependency remediation is outside this slice.

The materializing producer attested `reproducible: true`. The wrapper's
rejection of the stale manual digest did not destroy or replace its output.
With no checkout or wildcard operation intervening, Jon bound the producer's
`c36f0e45.../176` result. All three staged index blobs then rehashed exactly to
the recorded JS `bf725099...`, Wasm `60887f72...`, and provenance
`156dccd2...` values before commit `2082b29`. That preserved-byte stop and
index verification is the second custody exhibit.

The sanctioned Playwright producer then passed in 2174 ms and its staged proof
and screenshot bytes were verified before commit `6994237`. The Wasm readiness
gate reports the package, proof, and browser readiness all valid.

Each of the eight formerly candidate-only tests was then invoked alone with
the authorized isolated command. Every filter selected exactly one test and
all eight ended `OK`; the combined capture SHA-256 is
`16025e5745636ef33ff902c7b1613246f571a614e77916faeb418875985c23eb`.

Doctrine recorded from the second exhibit: a rebind scope must derive from a
repository-wide producer/pin census, not from one reporter's pins or a stale
provenance path list.

## Platform baseline and finding-ID sweep

Native-Windows full-battery expected-red sets differ from the Linux/macOS
baseline and must be platform-qualified. The native base transcript contains
exactly ten failure/error test names and hashes to
`aacddc271ed737db4d8c44b0f7b91e6788b0ca47a62c72efd4b88c747cf685ed`.
The final candidate transcript and named-set comparison will be recorded at the
digest-excluded U-35 tip after the acceptance commit.

The fork advertised 462 heads: 461 non-`main` heads plus fork `main`, which was
listed but not fetched or read. All 461 non-`main` tips were fetched into
numbered `refs/id-sweep/*` refs; no `refs/pull/*` was fetched or present. A
whole-tip grep found no assignment of U-54 through U-60. The lane therefore
reserves:

- U-54: native-Windows full-battery baseline differs from Linux/macOS;
  expected-red sets must be platform-qualified.
- U-55: generated evidence not immediately committed is one careless command
  from nonexistence; generate, hash, and commit in one custody window.
- U-56: rebind scope must come from a repository-wide producer/pin census, not
  a single reporter or historical input list.

The completed registrations, including both U-54 transcript hashes, belong in
`F_Project_Management/W_TRUST/WV6_REACCEPTANCE_REGISTRATIONS_2026-08-10.md`
after the final battery. That surface is product-digest-excluded, preserving
the frozen pair while binding the later U-35 tip.
- 2026-08-11T01:05:00Z: INDEPENDENT REVIEW VERDICT 01 (Claude Code on Claude Fable 5, Anthropic, `Pulses-MacBook-Air.local`, Darwin 25.6.0, U-52-clean boot; implementer Codex GPT-5-based on the NUC) — **BLOCKED on exactly one INDEPENDENTLY FOUND blocker; the ceremony itself verifies end-to-end.** Recomputed with own hands: both merges byte-exact via merge-tree; pair chain by two agreeing methods at ten revisions (diagnostic `43d68dc3…/1604`; superseded intermediate `2cb25d0b47f55c9dd987bf69fc8a97dade5a4767ef0beda0abc8616808dddc0c/1605`, digits now on the record; frozen `fd96e6d9…/1606` held through rebind/acceptance/tip); post-freeze confinement exact; both rebinds pin-movement-only with historical anchors byte-identical; WV-6 verifier run locally → **accepted 5/5 findings []** at frozen `410ff11`/`57ce26ae`; shelf ok:true at the frozen pair; WV_ACCEPTANCE producer-shaped field-for-field with all five artifact hashes recomputed; Wasm census re-executed from the producer's own INPUT_ROOTS code reproducing binding `c36f0e45…`/176 AND falsified `556ea1c6…`/175; artifact bytes exact, js-unchanged/wasm-changed consistent with the manifest-verified dependency chain compiling cycle.rs; wasm-readiness browser_ready true; U-54 ten-name baseline matched NAME-FOR-NAME in both hash-verified transcripts (base `aacddc27…`, final `e48a97d8…`); registrations/custody hash claims all reproduce; trust delta = the three Repair-3 scripts, satisfied by the merged canonical record (digest match). Native-Windows runs and Wasm rebuild impossible on this seat — hashes/internal consistency stood in, stated. **B1 (relocation-only cure): the tip commit `35ddc22` itself commits two CRLF Windows transcripts (FINAL_FULL_PYTHON_BATTERY, NATIVE_WINDOWS) under W_TRUST/ outside the U-25 evidence fences — the repo's own text-byte gate exits 1 at the reviewed head (green at the frozen commit), undisclosed; cure = byte-preserving move into the fenced acceptance-evidence path + registrations index update, digest-inert, nothing reopens.** Findings: F2 the v2 rolling gate structurally cannot green this merge topology (post-review walk vs merge parents; governance routing); F3 61-hex truncated hash + two wrong encoding labels in the registrations index; F4 final transcript not self-attesting; F5 U-52/U-53 used but registered nowhere; F6 cosmetics. Verdict authored as this seat's own identity. No fix performed. STOP.
