# Lane 0 ARCHIPELAGO S6 audit

Status: complete locally; prepared for Jon's review and merge.

Launch: **HOLD**.

Audit band: **Band 3/5**.

Governance verdict: **advisory**.

Waivers: none.

## Band-cap decision

Lane 0 has static, resilience, full-workspace, exact-Rust-1.95, hash-integrity,
and fail-closed reporter evidence. It does not have a clean-browser Playwright
runtime journey, so runtime evidence alone could reach no higher than band 4.
The requested Lane 2C `APPROVED` state also lacks current deterministic
evidence for three exact-candidate stress cases each exceeding four minutes.
That unverified claim lowers the audit to band 3. It is not treated as
falsified or band 1: the frozen backlog keeps Lane 2C `partial` until the
required reporter evidence exists.

## Loopback

The missing Lane 2C duration evidence is a G4 evidence failure and routes to
the earliest affected stage: **G4 -> S2**. The loopback is recorded in
`ops/lane0/state.json` and the hash-chained `ops/lane0/ledger.jsonl`. Lane 0
then proceeds to S6 with the claim downgraded instead of freezing an
unverified approval.

## S6 governance verdict

The verdict is `advisory`:

- Enforced locally: idea and plan contracts, P0-P3 gates, exact evidence
  manifest coverage, SHA-256 verification, ARCHIPELAGO ledger-chain
  verification, the four-denominator contract, launch HOLD, U-18 phase
  integrity, Rust MSRV, frozen-backlog states, and WV fail-closed pending
  contracts.
- Pending: a browser Playwright/runtime journey, current Lane 2C duration
  proof, remote CI on this successor branch, Jon's independent final review,
  Jon-only merge, admin-token provisioning, and any activation.
- Waived: none.

`jsonschema` is available and was used for the pinned ARCHIPELAGO contract
validation. Python Playwright is unavailable on this evidence machine; that
degradation is recorded and no browser proof is claimed.

## Frozen denominators

Exactly four readiness denominators are admitted:

1. S114 bounded mission: 19/19 = 100.0%.
2. Truth pulse: 931/1000 = 93.1%.
3. Launch-critical: 3/6 = 50.0%.
4. Whole launch ledger: 3/8 = 37.5%.

They are not averaged, blended, or supplemented with a fifth percentage.
