# MacBook Pro Codex Fleet Report - 2026-06-10

## Findings

### P0 - Do not describe `v0.8.1` as a signed Git tag

`gh release view v0.8.1` confirms a published GitHub Release with `SHA256SUMS`,
`SHA256SUMS.asc`, `.deb`, `.rpm`, two macOS tarballs, SBOM, and two VSIX
assets. That supports a signed-checksum / signed-release-artifact claim.

It does not support a signed-tag claim. `git tag -v v0.8.1` failed with:

```text
error: no signature found
object 8107c01a59a3f84925e9a4880a29a07a32b408eb
type commit
tag v0.8.1
tagger Jon Isaac <jon-isaac@islanddevcrew.com> 1780818780 -0500
```

The tag ref is `4a6442d439e636bdfcfc2f2ee2c8c2f9b2c8e1c7`; the peeled commit is
`8107c01a59a3f84925e9a4880a29a07a32b408eb`. Current `origin/main` is
`366e69f28812732df2f18fa6b80e1581dd7ac2d9`, two commits after the tag target.

Required wording: use "signed release checksums/assets" or "signed v0.8.1
binaries" only where backed by `SHA256SUMS.asc`; do not say "signed tag" unless
the tag is replaced by a signed tag under Jon's explicit release authority.

### P1 - Command-center and Fable-intake baselines are stale against live main

Live repo truth after `git fetch origin main --tags --prune`:

```text
HEAD        366e69f28812732df2f18fa6b80e1581dd7ac2d9
origin/main 366e69f28812732df2f18fa6b80e1581dd7ac2d9
latest      docs: add W-REBUILD foundation-rebuild workstream pack (#380)
```

`F_Project_Management/GARNET_S129_S200_ECC_DOGFOOD_COMMAND_CENTER.md` and
`F_Project_Management/GARNET_100X_FABLE_INTAKE_2026_06_10.md` are untracked in
this checkout and still cite `5161e64eeb796f5764909a1e5643eb47d5d72430` as the
verified baseline. They were accurate for the S130/#379 moment, but not for the
current post-#380 checkout. Refresh those baselines before using them as
coordination truth.

`F_Project_Management/W_REBUILD/` is tracked on current `HEAD` via #380. The
command-center/intake docs should now acknowledge that W-REBUILD is already on
main, while still blocking execution until its deploy gate is satisfied.

### P1 - Public-truth drift is real and verified

The W-REBUILD punch-list's main public-drift claims reproduce on current
`origin/main`:

- `README.md:145` says `24 registry primitives`; current
  `garnet-stdlib/src/registry.rs` has 80 primitive rows matching
  `rg -n '^\s*p\(' garnet-stdlib/src/registry.rs | wc -l`.
- `README.md:193` still says current main is post-v0.5.0 source.
- `README.md:212` and `FAQ.md:57` still say 24 bridged stdlib primitives.
- `FAQ.md:55` still says `v0.5.0 is research-grade and not production-complete`.
- `FAQ.md:57` still says S1 LSP source surfaces and v0.5.0 release assets.
- `docs/index.html` still hardcodes 1193 workspace tests, 136 security tests,
  `0.93x`, `92.3%`, and `87/87`.
- `python3 scripts/garnet_mit_readiness_status.py --format json` now reports
  `active-partial` at `92.8%`, so the public `92.3%` is stale.

The published release still includes stale VSIX asset names:

- `garnet-0.7.0-lsp-mvp-darwin-arm64.vsix`
- `garnet-0.7.0-lsp-mvp-linux-x64.vsix`

The W-REBUILD "truth guard" direction is correct, but CI/gate wiring is a
Jon-gated gate change and must not be slipped into a source-truth report PR.

### P1 - W-REBUILD deploy gate is not satisfied; do not implement slices

`F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md` correctly blocks execution
unless all three are true:

1. core fleet reports exist,
2. the S131-S134 source-of-truth consolidation PR is merged,
3. the lead machine's working tree is clean on current `origin/main`.

Current state fails that gate:

- only the template and this MacBook Pro Codex report exist locally under
  `F_Project_Management/FLEET_REPORTS/`;
- there are no open PRs on `Island-Dev-Crew/garnet`, so no S131-S134
  consolidation PR is open or merged yet;
- the working tree contains untracked coordination files:
  `FLEET_REPORTS/`, `GARNET_100X_FABLE_INTAKE_2026_06_10.md`,
  `GARNET_ECC_PRIME_ASSIMILATION.md`,
  `GARNET_ECC_PRIME_OVERLAY_RECOMMENDATION.md`,
  `GARNET_GOAL_MODE_KICKOFF_2026_06_10.md`, and
  `GARNET_S129_S200_ECC_DOGFOOD_COMMAND_CENTER.md`.

Verdict: W-REBUILD is safe as a tracked future workstream pack; executing
RB-0/RB-1/etc. now would violate its own deploy gate.

### P1 - `README_PROPOSED.md` is strong but not adoption-ready

Verified blockers and claim-tightening needs:

- `docs/assets/garnet-logo.png` is missing; nearby existing assets are
  `docs/assets/garnet-promo-poster.png`, `docs/icons/garnet-192.png`, and
  `docs/icons/garnet-512.png`.
- "Release v0.8.1 - signed" must be scoped to checksums/assets, not a signed
  Git tag.
- "every crossing is logged" needs deterministic evidence or softer wording.
- "One grammar, two registers, no FFI" risks overstating host-boundary reality;
  the intake doc already recommends narrowing this.
- "No ambient authority, ever" and "enforced at the language layer" need the
  existing boundary: declared capability propagation/diff-caps traps are proven,
  but OS sandbox enforcement is Linux-only where proven and deferred elsewhere.
- "An enforced kernel" is acceptable only if kept tightly scoped to deterministic
  `@caps`, `@max_depth`, and diff-caps traps; it must not imply macOS/Windows
  OS-sandbox enforcement.

The proposed README correctly keeps "research-grade prototype (v0.8.1), not
production-complete" and lists independent verification of the self-found
red-team fix as still open.

### P2 - S114 independence remains correctly unblessed

Live repo text still says the S114/high red-team result was self-found and
self-fixed, not independently verified. The command-center, Fable intake, and
W-REBUILD pack all preserve that boundary:

- keep S114 self-verified until an actually independent adversarial
  re-verification is complete;
- ECC/Fable may package evidence, but cannot self-grade independence;
- do not treat self-authored red-team work as independent verification.

No inspected source-truth input attempted to reclassify S114 as independent.

### P2 - ECC/Fable authority posture is acceptable; one stale prompt is red-zone only

The command-center, Fable intake, kickoff, and ECC recommendation docs all keep
ECC/Fable/Opus/Codex in an advisory role. I found no current attempt to replace
Garnet gates with ECC/Fable authority.

The stale `GARNET_ECC_PRIME_ASSIMILATION.md` contains an old "Signed Re-Cut
Lane" prompt that describes destructive release-tag replacement only after Jon
explicitly says `re-cut`. Under the current session hard stop, that prompt is
not executable. Keep it as historical episodic context only or supersede it in
the consolidation PR.

## Report Header

- Machine: MacBook Pro
- Agent/model: Codex / GPT-5
- Date/time: 2026-06-10 15:59:19 CDT
- OS: macOS 26.5 (25F71), arm64
- Hardware: MacBook Pro, Mac17,8, Apple M5 Pro, 48 GB memory; serial redacted
- Repo path: `/Users/IDC2.5/Desktop/Garnet`
- Active GitHub account: `Navigata1` via `gh auth status`; `IslandDevCrew`
  also authenticated but inactive
- Network state: GitHub fetch and `gh` API calls succeeded
- Report scope: report-only verification of command center, Fable intake,
  W-REBUILD pack, live GitHub release/PR truth, AGENTS contracts, and readiness
  gates

## Repo Truth

Commands run:

```sh
git fetch origin main --tags --prune
git status --short --branch
git rev-parse HEAD origin/main
git remote -v
git log --oneline -8
git tag --points-at HEAD
git show-ref --tags v0.8.1 || true
git tag -v v0.8.1
gh pr list --repo Island-Dev-Crew/garnet --state open --json number,title,headRefName,author,updatedAt,url
gh release view v0.8.1 --repo Island-Dev-Crew/garnet --json tagName,publishedAt,url,targetCommitish,assets
```

Summarized results:

- Fetch: succeeded.
- Branch: `main...origin/main`.
- HEAD: `366e69f28812732df2f18fa6b80e1581dd7ac2d9`.
- origin/main: `366e69f28812732df2f18fa6b80e1581dd7ac2d9`.
- Latest commit: `docs: add W-REBUILD foundation-rebuild workstream pack (#380)`.
- v0.8.1 tag ref: `4a6442d439e636bdfcfc2f2ee2c8c2f9b2c8e1c7`.
- v0.8.1 peeled commit: `8107c01a59a3f84925e9a4880a29a07a32b408eb`.
- v0.8.1 tag signature: absent (`git tag -v` returned `error: no signature found`).
- Open PRs: `[]`.
- Release state: published at `2026-06-07T07:55:45Z`,
  `targetCommitish` reported by GitHub as `main`,
  URL `https://github.com/Island-Dev-Crew/garnet/releases/tag/v0.8.1`.
- Release assets: `SHA256SUMS`, `SHA256SUMS.asc`, `.deb`, `.rpm`,
  two `garnet-0.8.1-*apple-darwin.tar.gz` tarballs,
  `garnet-sbom-cyclonedx.tgz`, and two stale `garnet-0.7.0-lsp-mvp-*.vsix`.

Local untracked files:

```text
F_Project_Management/FLEET_REPORTS/
F_Project_Management/GARNET_100X_FABLE_INTAKE_2026_06_10.md
F_Project_Management/GARNET_ECC_PRIME_ASSIMILATION.md
F_Project_Management/GARNET_ECC_PRIME_OVERLAY_RECOMMENDATION.md
F_Project_Management/GARNET_GOAL_MODE_KICKOFF_2026_06_10.md
F_Project_Management/GARNET_S129_S200_ECC_DOGFOOD_COMMAND_CENTER.md
```

`F_Project_Management/W_REBUILD/` is tracked on `HEAD`.

## Toolchain Truth

Commands run:

```sh
rustc --version
cargo --version
rustup show active-toolchain
node --version || true
npm --version || true
python3 --version
gh --version | head -1
gpg --version | head -1
java --version 2>&1 | head -5 || true
xcodebuild -version 2>/dev/null || true
system_profiler SPHardwareDataType | rg "Model Name|Chip|Memory|Serial" || true
```

Summarized results:

- Rust: `rustc 1.95.0`, `cargo 1.95.0`,
  `stable-aarch64-apple-darwin`.
- Node/npm: Node `v22.22.3`, npm `10.9.8`.
- Python: `Python 3.14.5`.
- GitHub CLI: `gh version 2.92.0`.
- GPG: `gpg (GnuPG) 2.5.19`.
- Java: no Java Runtime found.
- Xcode: not recorded in this pass.

## Garnet Readiness Snapshot

Commands run:

```sh
python3 scripts/garnet_readiness_status.py --format json
python3 scripts/garnet_mit_readiness_status.py --format json
```

Summarized results:

- Tracked implementation plan: 87/87 slices, `100.0%`.
- MIT/productization readiness: `active-partial`, `92.8%`.
- Current truth from the MIT reporter:
  - tracked implementation plan is complete;
  - goal remains active;
  - 100% tracked slices is not full MIT/productization completion;
  - S1 LSP source is tracked separately from full editor dogfood completion.

Non-verified or incomplete MIT/productization lanes include:

- macOS Studio DMG: active-partial 75.0%; blocked by missing Developer ID and
  notarization profile.
- Developer ID notarization: blocked 0.0%.
- Windows/Linux distribution: active-partial 71.0%.
- Mobile distribution: planned 0.0%.
- Promo video: public-site-embedded 95.0%; human/aesthetic acceptance open.
- LLM assist: active-partial 40.0%.
- Official Layer-2 package seed: local-registry-source-ready 85.0%.
- Compiler-as-agent LLM tier: feature-gated-source-ready 85.0%.
- Broad converter frontends: planned 0.0%.
- Proof and empirical validation: active-partial 45.0%.

## Verification

Commands run:

```sh
python3 scripts/check-agent-contracts.py
python3 scripts/test_check_agent_contracts.py
git diff --check
```

Results:

- `python3 scripts/check-agent-contracts.py`: pass,
  `agent-contracts: ok (21 contracts)`.
- `python3 scripts/test_check_agent_contracts.py`: pass, 5 tests, OK,
  `agent-contracts: ok (21 contracts)`.
- `git diff --check`: pass, no output.

## Local Evidence Inventory

This mission did not request a full disk-wide evidence sweep. I inspected the
repo-local source-truth inputs listed in the mission plus related untracked
coordination docs already present in the checkout. No release assets, keys,
credentials, VM images, screenshots, or private proof bundles were copied or
modified.

Relevant repo-local artifacts:

- `F_Project_Management/W_REBUILD/`: tracked on `HEAD`; durable source-truth
  workstream pack.
- `F_Project_Management/GARNET_S129_S200_ECC_DOGFOOD_COMMAND_CENTER.md`:
  untracked; useful, but baseline stale to `5161e64`.
- `F_Project_Management/GARNET_100X_FABLE_INTAKE_2026_06_10.md`: untracked;
  useful, but baseline stale to `5161e64`.
- `F_Project_Management/GARNET_ECC_PRIME_ASSIMILATION.md`: untracked historical
  episodic context; contains old re-cut prompt, not executable under this
  session's hard stops.
- `F_Project_Management/GARNET_ECC_PRIME_OVERLAY_RECOMMENDATION.md`: untracked
  report-only recommendation; aligns with advisory/no-hooks posture.
- `F_Project_Management/GARNET_GOAL_MODE_KICKOFF_2026_06_10.md`: untracked;
  contains this verifier mission and broader fleet prompts.

## S129-S200 Recommendations From This Machine

1. Refresh and commit the command center/intake/fleet reports as a report-only
   S131-S134 consolidation PR after all required machine reports arrive.
2. In that PR, correct all baselines to current `origin/main` and explicitly
   state W-REBUILD is already tracked on main but not executable until its deploy
   gate passes.
3. Keep release wording scoped: signed checksums/assets, not signed tag.
4. Use W-REBUILD Part C / `xtask truth` as a separate guarded implementation
   slice, not a source-truth report mutation.
5. Keep S114 self-verified until an independent attacker/reviewer produces
   adversarial re-verification evidence.

## Safe Next Action

Stop this lane after the report. MacBook Pro Claude consolidation should ingest
this report, refresh stale baselines in the untracked coordination docs, and
assemble a single report-only source-truth PR. Do not implement W-REBUILD
slices from this lane.

## Claim Boundaries

This report proves:

- Live fetch, GitHub PR, GitHub release, tag, readiness, and AGENTS contract
  evidence from this MacBook Pro checkout on 2026-06-10.
- W-REBUILD exists on current `origin/main`.
- Command-center/Fable-intake docs in this checkout are currently untracked and
  stale to `5161e64`.
- Public truth drift exists in README/FAQ/docs and release VSIX names.
- Required lightweight verification commands passed.

This report does not prove:

- production readiness;
- v1.0 readiness;
- a signed Git tag;
- independent S114 adversarial re-verification;
- macOS/Windows OS-sandbox enforcement;
- Wasmtime fuel enforcement;
- release asset correction;
- W-REBUILD deploy-gate success.

Any "enforced" claim in downstream work must name the deterministic trap or
gate that proves it.
