# Lane 1 successor RED evidence — 2026-07-17

This artifact preserves both independently confirmed blockers before either
repair is committed.  The candidate base is
`057f4aaec310671f77ecf218cc379ddcef1636b5` (tree
`5216d0824d10e582da0094acdd2f8eebe4cbda2e`).  PRs #511 and #512 are closed
unmerged and retained as published evidence; their history is not rewritten.

## F2 — native Linux cross-OS manifest fixtures

- Authority: GitHub Actions run
  `https://github.com/Island-Dev-Crew/garnet/actions/runs/29569855719`, job
  `87850815921`, on PR #512.
- Platform: GitHub-hosted Linux x86_64 runner.
- Command: `python3 -I scripts/test_garnet_cross_os_policy_manifest.py`.
- Result: **RED**, 10 tests run, 6 failures.
- Diagnostic: positive and orthogonal fixtures declared `macOS` while the
  runtime reported `Linux`; the fail-closed runner rejected the mismatch.
- Downstream `rustfmt`, `clippy`, workspace tests, canonical examples, and
  `cargo doc` were starved/skipped by the documented CI dependency chain.
  They were not counted as independent failures.

The six failing fixtures were:

1. clean exact head;
2. parity digest head independence;
3. different passing test-set detection;
4. skipped-test detection;
5. suite-mutation detection;
6. loader-failure fail-closed behavior.

## F1 — W-PLAY provenance seal drift

Platform: host macOS arm64.  The F2 fixture repair was present only as an
uncommitted working-tree patch and does not affect these four suites.

| Command | Result | Total |
| --- | --- | --- |
| `python3 -I scripts/test_garnet_wasm_readiness.py` | RED | 13 run, 3 failures |
| `python3 -I scripts/test_garnet_playground_browser_proof.py` | RED | 3 run, 1 failure |
| `python3 -I scripts/test_garnet_v0_8_0_cut_readiness.py` | RED | 8 run, 2 failures |
| `python3 -I scripts/test_garnet_v0_8_0_release_readiness.py` | RED | 5 run, 2 failures |

The current canonical SHA-256 of `Cargo.lock` is
`01b8986b1cee0ef6a53ac439bd018b54fc1dca825a8f845a259ed8001e6715fa`,
while `docs/playground/pkg/provenance.json` records
`eca75b70a900a21a241fef8b4d6aef3ea6e7fde155cf6fc71750a0a24d7cf38b`.
`garnet_wasm_readiness.py` therefore correctly reports
`browser_package_valid=false`, `browser_proof_valid=false`, and
`browser_ready=false`.  The provenance check remains fail-closed and is not
weakened by this repair.
