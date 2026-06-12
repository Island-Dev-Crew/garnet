# MacBook Pro Claude (Fable 5) Fleet Report — 2026-06-12 (W-REBUILD RB-band completion refresh)

Band-end refresh of this machine's truth after the RB-1→RB-3 execution. The
prior report (2026-06-10 file, authored 06-11) predates the entire W-REBUILD
build; this one records the post-band machine state. Companion document:
`F_Project_Management/W_REBUILD/RB_BAND_STOP_REPORT_2026-06-12.md` (the
band ledger + the nine queued decisions).

## Report Header

- Machine: MacBook Pro (Mac17,8), Apple M5 Pro, 48 GB memory; serial redacted
- Agent/model: Claude Code / `claude-fable-5[1m]` (ultracode; multi-agent
  recon/review workflows: 4–27 subagents per slice, every finding
  adversarially verified or refuted)
- Date/time: authored 2026-06-12 (CDT morning); covers the 06-11→06-12 band run
- OS: macOS 26.5, arm64
- Repo path: `/Users/IDC2.5/Desktop/Garnet`
- Active account: `gh` active `Navigata1`; `IslandDevCrew` authenticated,
  used only for the five squash-merges, switched back each time
- Report scope: repo truth + machine-local toolchain/evidence deltas from the
  RB band; no broad filesystem sweep this pass

## Repo Truth (verified at authoring)

- Current branch: `main...origin/main` in sync; working tree clean except
  the two known 2026-06-07 untracked ECC-Prime advisory docs (J3, unchanged)
- HEAD = origin/main = `0eecb65` ("docs: W-REBUILD RB-band stop report (#394)")
- This lane's merges this band: #388 (RB-1), #389 (RB-2), #390 (RB-2
  follow-up), #393 (RB-3 keystone), #394 (stop report) — all squash-merged
  by IslandDevCrew with agent/model/gate provenance in the PR bodies
  (integrity Rule 3); other lanes merged #391/#392 (Studio suite) in the
  same window with zero surface overlap
- Open PRs: none. Tags: unchanged (v0.4.2/v0.5.0/v0.8.0/v0.8.1 match origin;
  no tag pushed by this lane — cut acts stay Jon's)
- Verification at HEAD: workspace 1999 passed / 0 failed; clippy/fmt/doc/deny
  clean; agent-contracts 22/22; all 11 trust/readiness gates PASS;
  `xtask truth --check` ok (4 stamped surfaces); mit-readiness 92.8

## Machine-Local Deltas (this band)

- **Toolchain additions:** rustup `nightly` (minimal profile) + `cargo-fuzz`
  0.13.2 installed for the RB-2 fuzz run. Ladder toolchain remains
  stable 1.95.0 (`~/.rustup/toolchains/stable-aarch64-apple-darwin`);
  1.93.0 fires spurious never-read lints in garnet-parser — do not use it
  for clippy.
- **Fuzz corpus growth (gitignored, local-only):** ~64k generated units now
  sit under `garnet-parser-v0.3/fuzz/corpus/parse_input/` beside the 13
  tracked `.garnet` seeds. They improve future local fuzz runs; the smoke
  test filters to tracked seeds, CI unaffected.
- **Evidence bundles (Desktop, each MANIFEST.sha256-verified):**
  `garnet-rb1-caps-bitset-20260612T024936Z/`,
  `garnet-rb2-crash-surface-20260612T053932Z/`,
  `garnet-rb3-registry-dispatch-*/` — differential logs, red→green records,
  fuzz stats, gate-PASS tables, change diffs.
- **Operational lesson (recorded in lane memory):** adversarial-review
  workflow verifiers probe the SHARED working tree with temporary edits;
  never commit while a review fleet is running — reconcile the tree after
  the fleet completes (one probe deliberately miswired a dispatch key and
  was reverted; it also exposed a real macro gap, fixed fail-closed in
  #393's final commit).

## Honest Boundaries

- All performance and fuzz numbers in the band are machine-local to this
  MacBook Pro and labeled as such; nothing broader is claimed.
- The nine spec deviations/decisions accumulated across the band are
  recorded in CHANGELOG + the stop report and await Jon — this lane is
  STOPPED at the post-RB-3 gate; RB-4a does not start without a green-light.
- Research-grade prototype; not production, not v1.0; only `@caps` +
  `@max_depth` carry deterministic traps; seccomp Linux-only; S114 stays
  self-verified.
