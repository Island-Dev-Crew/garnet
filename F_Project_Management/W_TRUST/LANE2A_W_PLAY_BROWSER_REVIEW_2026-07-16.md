# Lane 2A W-PLAY browser trust-kernel review - 2026-07-16

This is the rolling S114 trust-kernel companion for P7-T2 on
`mission/l2a-wplay-browser`.

- Integration base: `1a0e5d729164ab30ae40523db206b1c36ee80045`.
- Scope: reproducible browser package, live page adapter, Playwright success
  and declared-proc denial traps, evidence-backed reporter promotion, and the
  human/machine capability-diff verdict.
- Launch remains `HOLD`; audit ceiling remains Band 3 while U-17 is open.
- Merge, FIRE, tags, publishing, the 31-to-32 ceremony, base-merge activation,
  and promo QA remain Jon-only.

This companion does not reopen or relabel S114, WV-1, WV-2, or WV-3. WV-4
remains frontend proof and WV-5 remains Wasm plus Node proof. Lane 2A adds a
separate browser proof and does not rewrite either earlier result.

## Trigger paths

The rolling gate is expected to enumerate these trust-kernel paths:

- `.github/workflows/ci.yml` - comments corrected to describe the existing
  reporter steps accurately; no job, trigger, permission, or required context
  is added or removed.
- `scripts/garnet_wasm_readiness.py` - browser promotion now requires package,
  source-digest, artifact-hash, tracked-request, Playwright-journey, denial,
  machine-verdict, layout, and screenshot evidence.
- `scripts/test_garnet_wasm_readiness.py` - reporter v3 and malformed-proof
  regressions.
- `scripts/test_garnet_playground_browser_contract.py` - pins relative package
  loading, browser controls, tracked assets, and the absence of fallback or
  external runtime paths.
- `scripts/test_garnet_playground_browser_proof.py` - pins the strict committed
  proof, denial semantics, duration ceiling, and shared human/machine verdict.
- `scripts/garnet_playground_readiness.py` - explicitly limited to static
  preset fallback validity; it cannot promote browser status.
- `scripts/test_garnet_playground_readiness.py` - pins that ownership split.

The changed `garnet-wasm/tests/run_source.rs` test proves that declaring
`@caps(proc)` does not mint a browser host native. The interpreter returns
`runtime_error`, empty stdout, and a diagnostic naming `proc`.

## Recorded RED

`ops/lane2a/evidence/00-red-tests.txt` binds the pre-implementation failures:

- no package materialization or two-build verification mode;
- no committed `live.js` or browser package;
- no strict browser-proof validity or promotion field;
- no browser proof reader or human/machine verdict check.

The declared-proc runtime control was already green, establishing that page
wiring did not need or authorize a new host capability.

## Threats and fail-closed responses

| Threat | Required response |
|---|---|
| Package file added, removed, or modified | Exact inventory and SHA-256 validation fails |
| Current Rust input differs from package provenance | Canonical source-tree digest validation fails |
| Proof exists but is malformed or partial | `browser_proof_valid` remains false |
| Browser path falls back to Node | `node_global_present` must be false |
| Browser requests external or untracked bytes | Playwright trap and reporter both fail |
| Denied `proc` authority warns or partially runs | Exact `runtime_error`, empty stdout, visible `Denied`, and `proc` diagnostic required |
| Human and machine diff verdicts diverge | Exact shared-result parity validation fails |
| Screenshot or responsive boundary drifts | Tracked screenshot hash or overflow validation fails |
| Pre-squash branch ancestry disappears | Package identity remains source/tree-digest based; branch SHA is diagnostic only |
| A green proof is replayed after browser or harness inputs change | Exact ten-file runtime-input digest validation fails |
| Playwright resolves from ambient or unlocked state | Harness requires direct integrity-locked `@playwright/test` from the Studio `npm ci` tree |

## Fresh local evidence

- `python3 -I scripts/test_build_playground_wasm.py`: 10/10 passed.
- `python3 scripts/build_playground_wasm.py --verify-reproducible`: two clean
  builds matched each other and the committed package.
- `python3 -I scripts/test_garnet_playground_browser_contract.py`: 6/6 passed.
- Playwright browser proof: passed in 2,637 ms with six committed requests,
  zero external requests, zero untracked requests, no console/page errors,
  no desktop/mobile horizontal overflow, and the declared-proc denial green.
- `python3 -I scripts/test_garnet_playground_browser_proof.py`: 3/3 passed.
- `python3 -I scripts/test_garnet_wasm_readiness.py`: 13/13 passed, including
  expected-red malformed proof, stale runtime digest, and changed-file behavior.
- `python3 scripts/garnet_wasm_readiness.py --gate`: schema v3,
  `browser_package_valid: true`, `browser_proof_valid: true`,
  `browser_ready: true`, blockers empty.
- Browser proof runtime identity: ten exact files at
  `e21134ae261f064ccb9db42a1d4150b2375fbb8c9146539be6439f3c56f75f70`,
  including the smoke harness plus Studio `package.json` and lockfile; direct
  `@playwright/test` 1.61.1 is bound to its lock integrity and documented
  `npm ci --ignore-scripts` install.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D
  warnings`, and `cargo test --workspace --no-fail-fast`: passed under Rust
  1.95.0 on macOS arm64.
- `cargo run -p xtask -- truth --check`: passed; Lane 0 closeout, frozen
  backlog, MSRV, trust-kernel companion, and agent-contract gates also passed.
- `npm audit --json --omit=dev`: zero production vulnerabilities. The full
  audit separately reports the existing direct development-only Vite advisory
  for Windows dev-server path handling (`GHSA-fx2h-pf6j-xcff`). This proof does
  not start Vite; the lane records but does not silently repair that dependency.

The post-repair full gate record is
`ops/lane2a/evidence/65-final-local-verification.json`; it preserves the earlier
verification record instead of rewriting it.

Raw browser evidence is
`F_Project_Management/LAUNCH/W_PLAY_BROWSER_PROOF.json` plus
`ops/lane2a/evidence/30-playground-browser.png`. The reporter, not this note,
is the verdict authority.

## Review and cross-OS boundary

A separate security reviewer identified the stronger declared-capability host
denial case before page implementation; that correction is now test-pinned.
Two final-tree review passes then independently caught the stale-proof replay
gap; one also caught ambient Playwright path resolution. The old proof became
RED under the repaired reporter, and commit `2f1d177` plus the regenerated
proof correct both findings. A final rereview of the repaired sealed tree is
still required before merge.

Fresh cross-OS required-check evidence is pending the fork-to-upstream PR. This
local companion is sufficient only for the structural rolling-review gate; it
does not pre-claim remote Windows, macOS, or Ubuntu success. The PR must remain
draft/not-ready for Jon until the ordinary three-OS jobs pass and their exact
run/job links are appended here on a subsequent evidence-only commit.
