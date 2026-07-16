# Garnet Lane 0 frozen backlog

**Frozen base:** `231aefa91985e5a0520c493c7f0fc3e54d74efc8`

**Launch:** HOLD

**Machine authority:** `ops/lane0/frozen-backlog.json` and
`python3 -I scripts/garnet_frozen_backlog_status.py --gate`

Only `implemented`, `partial`, `planned`, and `research` are valid claim states.
Future evidence destinations are contracts, not evidence.

| Item | Frozen state | Current freeze context | Still open |
|---|---|---|---|
| Lane 1 item 1 / GOV-009 | **partial (3/7)** | Authenticated transport #500 (`52971a5`); strict pagination #502 (`8c07fec`); complete bounded collection #506 (`1fe7489`). | Fresh cohort; exact reviewed head; verified outcomes; live admin-authoritative settings/no-bypass readback. U-17 blocks the last until Jon provisions a dedicated token. |
| Lane 2A / W-PLAY | **partial** | Wasm check/diff adapters #498 (`a021005`); hermetic no-publish package probe #505 (`65da989`). | Package materialization; live adapter/page; Playwright traps; reporter promotion; fail-closed denial proof. |
| Lane 2B / Minimum Shelf | **partial** | Initialize schema #501 (`750e4b6`); lifecycle core #503 (`098b0bd`); adversarial hardening #504 (`46031e7`). | One bounded in-process Garnet tool; raw-byte stdio; sealed baseline; reject-without-seal proof; deterministic reporter. |
| Lane 2C / stress | **partial, not approved** | Six ignored stress fixtures exist across memory and actor runtime. | No tracked evidence proves three reproducible cases exceeding four minutes. The historical memory log records four cases in `0.03s`; exact-candidate long-duration proof and a fail-closed reporter remain open. |
| WV-6 | **planned** | Acceptance contract and red/pending reporter exist. | Native-Windows Core Ring Tier 1 + Minimum Shelf/MCP evidence at the frozen destination. |
| WV-7 | **planned** | Acceptance contract and red/pending reporter exist. | winget/Scoop dry-run, devcontainer/Docker, and installer happy-path evidence on one exact candidate. |
| U-15 / fleet-fork main | **planned, Lane 3** | Lane 0 records the debt and does not read fork main. | Reconcile the curated snapshot or add an explicit public-truth banner. |
| Quarterly competitive watch | **planned** | Cadence, query, evidence, and fail-closed reporter contract are active. | First report is due 2026-09-30; no report is claimed today. |

## Exact next acceptance commands and evidence destinations

| Item | Acceptance command | Evidence destination |
|---|---|---|
| Lane 1 | `python -I scripts/test_garnet_github_governance_gate.py` | `F_Project_Management/W_TRUST/reviews/GOV-009-final/` |
| Lane 2A | `node scripts/smoke_garnet_playground_browser.mjs --gate` | `proofs/playground/lane2a-browser/` |
| Lane 2B | `python scripts/smoke_garnet_minimum_shelf.py --gate` | `proofs/minimum-shelf/lane2b/` |
| Lane 2C | `python3 -I scripts/garnet_lane2c_stress_status.py --gate` | `proofs/performance/lane2c-stress/` |
| WV-6 | `python3 -I scripts/garnet_wv_acceptance_status.py --wv WV-6 --gate` | `proofs/windows/launch-verification/wv6-minimum-shelf/` |
| WV-7 | `python3 -I scripts/garnet_wv_acceptance_status.py --wv WV-7 --gate` | `proofs/windows/launch-verification/wv7-distribution/` |
| U-15 | `python3 -I scripts/garnet_public_truth_status.py --gate` | `F_Project_Management/PUBLIC_TRUTH/NAVIGATA1_MAIN_RECONCILIATION.md` |
| Watch | `python3 -I scripts/garnet_quarterly_competitive_watch_status.py --gate` | `research/competitive-watch/2026-Q3.md` |

The Lane 1, Lane 2A, Lane 2B, Lane 2C, and U-15 commands intentionally name the
terminal contracts even when their scripts do not yet exist. Absence remains
open work; the freeze does not substitute a green partial-component test for
terminal acceptance.

## Jon-only boundary

Jon provisions the U-17 admin-authoritative token; performs any 31→32 or
base-controlled ceremony; approves base merges, promo QA, public claim
promotion, FIRE, tag, and publication; authorizes signing or public
package-manager submission; and approves any public fork-main banner. Reporters
must not inherit, persist, or print ambient credentials.
