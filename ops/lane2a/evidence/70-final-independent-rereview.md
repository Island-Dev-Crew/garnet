# Lane 2A final independent rereview

Reviewed candidate: `1115f633953b829a1b6a0cf62099ac62c5b71c77`

Reviewed tree: `5e6386107d51d36adc14f26025aa843515c50691`

Integration base: `1a0e5d729164ab30ae40523db206b1c36ee80045`

Review mode: two independent read-only passes against the sealed candidate;
neither reviewer edited the candidate or treated lane prose as verdict authority.

## Verdicts

- **Implementation review: APPROVE.** No critical, high, or medium findings.
  The reviewer confirmed that the stale-proof replay and ambient/unlocked
  Playwright findings are repaired by the exact runtime-input digest, direct
  dependency/lock checks, and Studio `node_modules` containment.
- **Claim verification: PASS, 5/5.** Real browser run/check/diff, visible
  fail-closed declared-proc denial, two-build package reproducibility,
  reporter-only current-evidence promotion, and shared human/machine verdict
  all passed independent inspection. No high or medium claim blocker remained.

The implementation reviewer also ran the focused Python suites, the Wasm
declared-proc test, both readiness gates, and a temporary clean-browser smoke
proof. The claim verifier separately ran a temporary browser proof in 2,524 ms
with six committed requests and reproduced the package under pinned Node
22.22.2. These temporary outputs corroborate but do not replace the committed
2,637 ms proof.

## Prior findings

| Finding | Final state |
|---|---|
| Stale proof replay across changed runtime inputs | Closed; exact ten-file digest and changed-input regressions fail closed |
| Ambient/unlocked Playwright import | Closed; direct lock/integrity identity and Studio install-root containment required |
| Declared capability accidentally mints browser host authority | Closed; Rust and page proofs require `runtime_error`, empty stdout, `proc` diagnostic, and visible `Denied` |

## Residuals, not gate failures

1. **LOW - browser executable identity.** The harness records Chrome's version
   but selects `CHROME_BIN` or an OS path and does not bind the executable hash.
   This limits whole-browser replay hermeticity; it does not invalidate the
   committed-package claim or the observed clean-browser behavior.
2. **LOW - diagnostic commit label.** The committed proof names implementation
   commit `2f1d177`, while this reviewed candidate adds later evidence-only
   commits. The reporter deliberately treats that label as diagnostic and
   validates the exact current runtime-input digest instead.
3. **PENDING - remote required checks.** Ubuntu, Windows, and macOS PR jobs have
   not run. No cross-OS result is claimed by this local rereview.

`ops/mission/state.json` is unchanged across the candidate diff. S114, WV-4,
and WV-5 labels are unchanged. The four launch denominators, launch HOLD, and
Band 3 ceiling while U-17 remains open are unchanged.
