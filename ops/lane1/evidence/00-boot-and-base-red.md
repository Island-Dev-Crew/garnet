# Lane 1 boot and base RED — 2026-07-17

Repository: `Island-Dev-Crew/garnet`

Local platform: macOS, America/Chicago

Observed at: `2026-07-17 00:48:28 CDT (-0500)` — before the Friday-sunset Sabbath fence. No scheduled run, merge, ruleset activation, FIRE, tag, or publish action was performed.

## Exact integration base

- Prompt pin: `1a0e5d729164ab30ae40523db206b1c36ee80045`
- Fresh `HEAD`: `cede73c03c5d535306ed179b5882e99e4d17b050`
- Fresh `origin/main`: `cede73c03c5d535306ed179b5882e99e4d17b050`
- `git merge-base --is-ancestor 1a0e5d729164ab30ae40523db206b1c36ee80045 HEAD`: exit 0
- First-parent delta:
  - `8535e6d feat(playground): close W-PLAY with reproducible browser proof (#509)-JI`
  - `cede73c fix(lane0): make evidence proofs clone-durable (#510)-JI`

The #509 delta closes Lane 2A's committed browser-package, live-page, Playwright denial, reporter, and human/machine verdict scope. The #510 delta makes the Lane 0 evidence proof pass in a fresh clone without pull refs. These facts supersede the stale pre-delta Lane 2A wording; launch status and denominator values still require fresh reporter derivation.

## Lane 0 hard gate

- `python3 -I scripts/garnet_lane0_closeout_status.py --gate`: PASS — evidence 22/22, ledger 37, denominators 4/4, launch HOLD, band 3, S6 advisory.
- `python3 -I scripts/garnet_msrv_status.py --gate`: PASS — `msrv: 1.95`, `ok: true`, no findings.
- `python3 -I scripts/garnet_frozen_backlog_status.py --gate`: PASS — 8 entries, 4 partial, 4 planned, no findings.
- U-18 probe: PASS — materialized phases P0 through P7, only P7 referenced, missing phase set empty.

## Required pre-implementation RED

The exact four policy suites were run from the clean base before Lane 1 behavior changed.

1. `python3 scripts/test_garnet_required_context_contract.py`
   - RED: 13 tests ran with 30 assertion failures.
   - Root symptom in every failure: `producer inventory path contains symlink/reparse point: /var` or the same message for the ruleset mirror.
   - This is the required macOS `/var` symlink parity reproduction.
2. `python3 scripts/test_garnet_workflow_file_policy.py`
   - RED: 10 tests ran with 1 failure.
   - Failing case: `test_casefold_and_unicode_normalization_collisions_fail_closed` accepted the NFD/NFC `café.yml` collision instead of returning an empty record set.
3. `python3 -I scripts/test_garnet_workflow_yaml_policy.py`
   - Tool degradation on the first attempt: system Python lacked the required `PyYAML 6.0.3`, so collection stopped with `RuntimeError: PyYAML 6.0.3 is required`.
   - Remediation: installed only `scripts/garnet_workflow_yaml_requirements.txt` into disposable venv `/private/tmp/garnet-l1-policy-venv-20260717`.
   - Base result under the pinned dependency: GREEN, 4/4 tests.
4. `python3 -I scripts/test_garnet_workflow_schema_policy.py`
   - Same initial PyYAML degradation and disposable-venv remediation.
   - Base result under the pinned dependency: GREEN, 5/5 tests.

`python3 scripts/garnet_trust_kernel_review_status.py --gate` returned `ok: true`, `problems: []`, and `trust_kernel_touched: false` before implementation because the branch had no trust-kernel diff yet.

This evidence records the RED; it does not claim the repairs or cross-OS parity are complete.
