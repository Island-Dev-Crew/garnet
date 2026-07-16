## Summary

Completes the Lane 0 truth freeze after absorbed PR #499. This successor
archives the exact first-parent mapping through #506, reconciles U-18 onto the
materialized P7/P7-T1..T4 model, records U-16 and U-17, settles Rust 1.95,
freezes evidence-tied lane/WV contracts, canonicalizes the repo-held research
corpus, and closes the namespaced ARCHIPELAGO session at S6.

Launch remains **HOLD**. The audit is band 3/5 with an `advisory` governance
verdict and no waivers. Lane 2C remains partial because current deterministic
three-case, greater-than-four-minute evidence is absent.

## Dogfood Readiness

### Current truth

- [x] Exactly four denominators are recorded: S114 19/19 (100.0%), truth pulse
  931/1000 (93.1%), launch-critical 3/6 (50.0%), launch ledger 3/8 (37.5%).
- [x] Launch recommendation is HOLD; browser, Shelf, WV, activation, FIRE,
  tag, publish, and public-launch claims remain blocked or Jon-only.
- [x] U-18 is internally consistent at P7 with P7-T1 through P7-T4; no
  P8/P9/P10 phase is invented.

### Local verification

- [x] `python3 -I scripts/garnet_lane0_closeout_status.py --gate`
- [x] `cargo run -p xtask -- truth --check`
- [x] `cargo +1.95.0 test --workspace --no-fail-fast`
- [x] `cargo test --workspace --no-fail-fast`

### Remote verification

- [x] This PR is deliberately parked for Jon; remote CI and the final
  independent review are required before merge and are not claimed complete
  by the local evidence bundle.

### Evidence bundle

- [x] `ops/lane0/evidence/MANIFEST.sha256` gives exact sorted SHA-256 coverage,
  and `ops/lane0/ledger.jsonl` verifies from the ARCHIPELAGO zero-hash genesis.
- [x] `ops/lane0/AUDIT.md` records band 3, G4 -> S2, S6 advisory, no waivers,
  Playwright degradation, and the Lane 2C evidence gap.

### Deferred / out of scope

- Jon provisions the dedicated admin-authoritative token and owns merge,
  activation, FIRE, tags, publishing, promo QA, and the 31-to-32 ceremony.
- Lane 1 closes fresh/exact-head/outcome/live-policy clauses.
- Lane 2A closes package/page/Playwright/denial proof.
- Lane 2B closes bounded tool/raw-byte/seal/rejection/reporter proof.
- Lane 2C must produce current deterministic duration evidence before approval.
- No Studio, converter, provider-LLM, mobile, promo, hosted registry, or new
  language-surface expansion is introduced here.
