# W-PLAY proof refresh — host macOS arm64 — 2026-07-17

The deterministic package rematerialization changed the sealed Wasm bytes and
source digest, so the existing Playwright proof correctly became RED.  This
artifact records the required proof regeneration through the existing browser
harness; no proof field or readiness verdict was edited by hand.

- Package commit: `12afac12d2f71615740bc2773e8ce84406ae844d`.
- Command: `node scripts/smoke_garnet_playground_browser.mjs --proof F_Project_Management/LAUNCH/W_PLAY_BROWSER_PROOF.json` using Node 22.22.2 and the direct integrity-locked Studio `npm ci` tree.
- Harness result: PASS; six committed requests, zero external requests, and
  zero untracked requests.
- Runtime-input aggregate SHA-256:
  `05b141bd3125202a03f45871095820ccb04bc871dd1541b6dde6d060f9952066`.
- Proof SHA-256:
  `9f19d420ddd8fee3c49ab89fae6ff33b695c8ef7fafa7b29d983312d77253590`.
- Screenshot SHA-256:
  `f7dadd3f6c4fe041403752da95c4fc383ce9de325fc1918bcbe6ed55283908c2`.

The proof records its raw harness duration as required by its existing schema.
This Lane 1 repair makes no performance or timing claim; Lane 2C retains that
authority.

Fresh downstream suite results after regeneration:

| Command | Result | Total |
| --- | --- | --- |
| `python3 -I scripts/test_garnet_wasm_readiness.py` | GREEN | 13 run, 0 failures |
| `python3 -I scripts/test_garnet_playground_browser_proof.py` | GREEN | 3 run, 0 failures |
| `python3 -I scripts/test_garnet_v0_8_0_cut_readiness.py` | GREEN | 8 run, 0 failures |
| `python3 -I scripts/test_garnet_v0_8_0_release_readiness.py` | GREEN | 5 run, 0 failures |
| `python3 -I scripts/garnet_wasm_readiness.py --gate --format json` | GREEN | `browser_ready=true`; blockers empty |

These are local host checks.  They do not replace the replacement PR's CI
x86_64 matrix, which remains the acceptance authority.
