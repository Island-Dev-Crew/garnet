# Mission Journal — Garnet S114 Closure & Hardening

Append-only session log, newest entry LAST.

## 2026-07-12T21:30Z — session 1 (kickoff)

- Mission bootstrapped from the S114 Plan of Attack (Desktop HTML, 2026-07-12) at HEAD d0d4f7c; orchestration Tier 1 (Workflow tool available, ultracode session).
- 9-agent recon workflow grounded every phase in file:line anchors; digest in session scratchpad; the load-bearing findings are recorded in state.json task notes.
- Two recovered-evidence discoveries: the S114 Windows lane-2 review bundle (commit 6153726, 160-file manifest) and the Codex verdict commit (61cfbae) exist as unreachable objects in C:/garnet and as reachable refs in the nested Desktop repo (branch validation/2026-06-25-codex-s114-review, also on the Navigata1 fork). Protective local refs created: s114/recovered-lane2-review, s114/recovered-codex-verdict (P1-T1 done).
- Confirmed the promo-line machine dependence: this machine renders composition-ready (50.0%) vs committed public-site-embedded (95.0%); only ledger line 84 differs. P0 includes the canonical-snapshot fix.
- gh auth switched Navigata1 -> IslandDevCrew (admin verified). No branch protection on main (verified via API) — green-before-merge is discipline.
- Authorization basis for this mission recorded: Jon's written directive of 2026-07-12 ("execute everything end to end in its completion") over the plan document; S114 acceptance provenance will cite it verbatim. CI-wiring merges remain Jon-only.
- Mission bootstrap merged as #472 (4168582), full CI green (26 pass / 2 skip / 0 fail).

## 2026-07-13T03:20Z — session 1 (Phase 0 complete)

- P0-T1: authored S114 acceptance artifact — LAUNCH/S114_ACCEPTANCE.json (schema garnet.s114_acceptance/v1, state accepted-scoped, scope "first-party CLI/Wasm trust-kernel baseline") + W_TRUST/S114_ACCEPTANCE_RECORD_2026-07-12.md. Provenance explicit: decided by Jon, basis = his execute-end-to-end directive; independence_relabel=false — the S114 verdict language stays "independently-re-verified-with-fixes". Five conditions map to P1/P2/P3; out_of_scope + tracked_debt recorded.
- P0-T2: reporter now READS the artifact (never grades): read_s114_acceptance() -> accepted-scoped, fail-closed to external-pending on missing/malformed. launch_ready membership accepts accepted-scoped for the s114 slot but stays False (playground/wasm/shelf open) — accepting S114 makes no launch claim. Added read_promo_snapshot() + committed PROMO_EVIDENCE_SNAPSHOT.json so ledger regeneration is machine-independent (fixes the PR #466 machine-dependent-promo flag); live probe still invoked. Regenerated LAUNCH_READINESS.md — diff was exactly the S114 section, promo line held at 95.0%.
- Gates: P0-G1 reporter suite 37/37 OK (Windows needs PYTHONUTF8=1 — pin test captures child stdout, cp1252 mangles em-dashes; locale artifact, not content; suite is not in CI). P0-G2 evidence integrity 36/36 ok. Reporter --gate still exits 1 (HOLD) as it must.
- Next: P1-T2 land the recovered Codex verdict + Windows lane-2 review bundles under proofs/ (refs already protected).
- P0 merged as #473 (155dec9), full CI green (29 checks, 0 fail).

## 2026-07-13T03:40Z — session 1 (Phase 1 complete)

- P1-T2: landed both recovered S114 evidence bundles durably; evidence integrity gate 36 -> 38/38 ok, verified from committed index blobs.
  - Codex independent verdict (origin 61cfbae): verdict MD -> W_TRUST byte-identical (blob 34176867...); 784 proof files -> proofs/independent/s114/codex-verdict-20260625/ via raw-blob extraction (git show, not checkout — checkout to a non-proofs/ path smudged CRLF and broke hashes). Manifest re-expressed bundle-relative (path prefix stripped, 784 hashes identical) so the gate's base=manifest.parent resolution verifies it; origin manifest sha256 ba07c1e8... recorded.
  - Windows lane-2 review (origin 6153726, reviewed 2e2fe84): landed byte-identical at proofs/validation/s114-review/windows-20260628-lane2/ (bundle-relative manifest self-verifies) + VALIDATION_REPORTS report. 3 .garnet-cache files (episodes.log, knowledge.db, strategies.db) are .gitignore'd (line 46) — force-added to match the manifest.
  - proofs/independent/s114/RECOVERY_PROVENANCE.md documents SHAs, lineage, the manifest re-expression, and frames both as reviewer captures at their stated base (predating later main), NOT verification of current HEAD.
- Gotcha recorded: `git checkout <sha> -- <path>` smudges EOL for paths OUTSIDE proofs/ (the -text rule is proofs/-scoped); use raw `git show <sha>:<path>` bytes when relocating sealed evidence. And `git add` silently skips .gitignore'd files — verify bundle completeness from the index, force-add as needed.
- Next: P2 language hardening (capability enforcement scope table + bound overbroad copy + claim fixture).
- P1 merged as #474 (0931d12), full CI green.

## 2026-07-13T03:55Z — session 1 (Phase 2 complete)

- P2-T1: C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md — opens with the ONE universal check-time CapCaps guarantee, then 7 bounded classes (declared/checker-only time+uuid, 12 runtime-gated, 3 entry-gated, declared-only ffi/net_internal, unbridged tcp_listen/udp_bind, OS-sandboxed Linux-seccomp-reference-only enforced:false, caps-invisible memory::*), each code-anchored; a "may/may not say" fence. CURRENT_STATE.md index row added (=> dogfood body).
- P2-T2: bounded the two clear runtime overclaims — README (now scoped to the 12+3 gated host-authority prims under the CLI, time/uuid checker-only, links the scope table) and docs/index.html:1040 ('no ambient authority, ever' -> 'undeclared OS authority fails the check'). Bumped SW cache garnet-web-v1 -> v2 so returning PWA visitors get the correction. /why byte-identical (condition #3). Blog 2026-05-20 left as minor follow-up.
- P2-T3: scripts/garnet_capability_scope_status.py (garnet.capability_scope/v1) + 6 tests — asserts the scope doc names every class, /why has exactly 2 'enforced:' claims, cited test anchors exist, and forbidden universal-enforcement phrasing is absent from README/index/why. Runs locally; CI wiring parked for Jon (P4-T2).
- Gates: truth --check ok, caps-enforcement --gate ok, capability-scope --gate ok + 6 tests OK, web-pwa smoke blockers=0 (SW bump clean).
- Next: P3 code hardening — do the two CLI fail-closed lanes first (smaller, independent), then the embedder strict-by-default flip (largest), then the checker-only CLI coverage test.
- P2 merged as #475 (8f794f7), full CI green.

## 2026-07-13T04:12Z — session 1 (Phase 3a: CLI fail-closed lanes)

- P3-T2 (dep preload fail-closed): garnet-cli/src/cmd/run.rs preload_dependencies now returns Result<(),String>; is_authority_trap() classifies by the stable 'capability:' prefix; an authority trap during vendored-dep preload aborts the run (exit 1 + episode record), while benign parse/read/missing-vendor errors stay warn-and-continue (S12 'noisy dep' contract preserved). Fixed the authority-trap mislabel. s114_residual_lanes.rs strengthened with an additive non-zero-exit assertion.
- P3-T3 (test-helper fail-closed): garnet-cli/src/cmd/test.rs Ok(Err) helper-preload arm now fails the file's tests like the panic arm. New tests/test_helper_preload.rs (authority-trap + unparseable helpers fail; well-formed still passes).
- P3-T4 (checker-only coverage): new tests/checker_only_caps.rs pins that time::now_ms (declared+undeclared) and std::uuid::new_v4 run without a runtime trap under garnet run. Empirical note: garnet run does not invoke the checker, so checker-only caps aren't runtime-gated — documented (P2 scope table), unchanged (S90 preserved).
- garnet-cli/AGENTS.md updated with the fail-closed contract. Gates: full garnet-cli suite 0 failed, clippy clean, caps-enforcement + red-team + agent-contracts all pass.
- Next: P3-T1 embedder strict-by-default on branch mission/p3b-embedder-strict (largest slice); then re-run all P3 gates at phase close; then P4.
- P3a merged as #476 (ec39bb1), full CI green incl. agentic dogfood matrix. (One rustfmt retry: multi-line fn signature — fixed and re-pushed.)

## 2026-07-13T04:38Z — session 1 (Phase 3 complete: embedder strict-by-default)

- P3-T1 (P3b): garnet-interp Interpreter::new() is now strict (deny-by-default); Interpreter::new_permissive() is the explicit opt-out. Mechanism: eval::StrictScope (thread-local per-instance strict depth) + deny_no_frame() = strict_no_frame() || instance_strict(). The A5 process-global one-way latch is UNTOUCHED (STRICT_NO_FRAME static stays false) and still dominates — a permissive instance cannot escape a latched process. load_module/eval_expr_src/call/call_entry hold the scope when strict.
- Migrations (trusted internal harnesses -> new_permissive): host_effect_composition.rs, stdlib_s22/s23_dispatch.rs, strict_no_frame_latch.rs baseline (now also proves the latch dominates a permissive instance), and the garnet-vm bench. New strict_by_default.rs (5 tests). lib.rs doctest is pure -> unchanged. CHANGELOG records the deliberate contract reversal.
- Empirical blast radius: exactly the 4 predicted test files failed under strict (proc entry-gate at entry_frames==0 via unframed .call); s24 did not (its main doesn't hit the no-frame path). garnet-cli unchanged (latches the global, uses framed APIs) and garnet-vm unchanged (framed run paths).
- Phase-close gates (branch): full workspace 0 failed, clippy clean, fmt clean, cargo doc -D warnings ok, caps-enforcement/capability-scope/red-team/bounded-enforcement/agent-contracts all pass. P3 phase DONE pending P3b merge.
- Next: P4 rolling S114 (trust-kernel file set + policy + status gate), then the Jon-only CI-wiring PR (parked), then mission complete.
- P3b merged as #477 (499717e), full CI green incl. agentic dogfood matrix.

## 2026-07-13T04:50Z — session 1 (Phase 4a: rolling S114 policy + gate)

- P4-T1: scripts/garnet_trust_kernel_review_status.py (schema garnet.trust_kernel_review/v1) makes S114 a recurring control. Machine-readable trust-kernel trigger set (checker / interp / vm / stdlib registry / wasm / CLI authority flows / capability reporters / /why + scope table). Given a diff (mirrors check_dogfood_pr_body.py's --base/--head/--changed-file), it requires a review companion — a scoped S114_ACCEPTANCE.json update, a proofs/independent/s114 or W_TRUST or VALIDATION_REPORTS artifact, or a named 'Trust-Kernel-Review:' commit trailer. --gate exits 1 on an uncompanioned trust-kernel change. Policy doc GARNET_TRUST_KERNEL_ROLLING_REVIEW.md + CURRENT_STATE.md row. 12 tests OK; gate behavior verified.
- Next: P4-T2 opens the CI-wiring PR for the P2 + P4 gates and PARKS it for Jon (integrity rule 1); then mission complete.
- P4-T1 merged as #478 (ea9e381), full CI green.

## 2026-07-13T05:05Z — session 1 (MISSION COMPLETE)

- P4-T2: opened PR #479 wiring the P2 capability-scope gate + the P4 trust-kernel review gate (and their tests) into .github/workflows/ci.yml, plus fetch-depth:0 for the rolling diff. PARKED for Jon — .github/workflows/ is human-merge-only (integrity rule 1). Its CI is green (the new agent-contracts gates already pass), so Jon has a clean, mergeable PR. NOT self-merged.
- Mission status set to COMPLETE. All five phases done and merged: P0 (#473) scoped S114 acceptance + reporter reads it + machine-independent ledger; P1 (#474) recovered Codex + lane-2 evidence landed durably (integrity 36->38/38); P2 (#475) capability enforcement scope table + bounded two runtime overclaims + claim fixture; P3 (#476 CLI fail-closed lanes, #477 embedder strict-by-default); P4 (#478 rolling trust-kernel review control). Plus bootstrap #472.
- Closing evidence (ops/mission/evidence/MISSION-CLOSE-2026-07-13.txt): S114 row accepted-scoped; launch_ready=False/HOLD preserved (no launch or runtime-safety claim); evidence_integrity/caps_enforcement/capability_scope/trust_kernel_review/bounded_enforcement/red_team gates all exit 0.
- Honesty fences held throughout: S114 independence language unchanged (only Jon relabels); /why two bounded claims byte-identical; no tags pushed; the one gate-changing PR handed to Jon.
- Single remaining follow-up (by design): Jon merges #479 to enable the two new gates in CI.

## 2026-07-13T08:05Z — session 2 (P4-T2 closed by Jon; P5 Foundation Hygiene)

- Jon merged #479 (6087565) — the capability-scope + trust-kernel gates are now CI-wired; both verified green on main post-merge. Last S114 blocker cleared; mission reopened as active for P5.
- Foundation-refresh research sweep (3 agents + adversarial judge, 2026-07-13): PROCEED verdict — zero foundational refutations. Key corrections: the "0.144.1/0.145.0-alpha.4 rust" versions Jon saw are OpenAI Codex CLI release tags (rust-v0.x.y), not Rust (actual stable: 1.97.0, 2026-07-09); Ruby 3.5 never shipped — renumbered to Ruby 4.0 (2025-12-25, current 4.0.5). Garnet's determinism claims are toolchain-immune by design (manifest.rs hand-serialization; gate compares output manifests, not binaries).
- P5-T1: PR #481 opened + PARKED for Jon — supply-chain scans extended to the Studio (435-pkg wry/tao) + fuzz lockfiles (strongest research finding: deny.toml 'release-grade' label vs root-only coverage). P5-T2: FAQ MSRV contradiction (1.75+ vs 1.95+) unified on 1.95+, decision = stay floating (no pin). P5-T3: garnet-wasm verified under 1.97.0 (build clean, 6/6 tests — 1.96 linker hardening non-event). P5-T4: converter input-dialect honesty note (stylized ~3.3-era Ruby subset; 3.4 `it` / 4.0 leading-operators not targeted).
- FROZEN: /why claim surfaces — Jon is bringing a reprocessed claims/attribution report (the cowork report that flagged /why attribution issues was itself wrong; Jon reprocessed right-vs-wrong). No /why edits until it lands; the claim fixture pins exactly 2 'enforced:' claims, so any claim-set change must move fixture + scope doc + ledger in one PR.
- Deferred + recorded: edition 2024 migration (post-launch), benchmark Ruby-framing refresh (next re-run), optional CI wasm32 job (bundle with a future Jon-gated CI PR).
- P5-T1 validation run: the new scans immediately found two live quick-xml 0.39.4 advisories (RUSTSEC-2026-0194/0195) in the previously-unscanned Studio lock — patched on the parked branch (plist 1.10.0 -> quick-xml 0.41.0); cargo-deny --config mechanics fixed (root deny.toml staged beside the Studio manifest). The coverage gap was not theoretical.

## 2026-07-13T10:10Z — session 2 (/why reconciliation)

- Jon delivered the promised re-verification report (cowork stream, July 8-11 contamination audit + 2026-07-13 primary-source re-check). Ingested and mapped to in-repo reality.
- HEADLINE: the audited /why upgrade (Shai-Hulud/Megalodon threat card, Veracode, insurance card, Ronacher card, FDA quote, vocab swaps) NEVER merged into this repo — cowork "PR 479" != repo #479 (our CI-wiring PR; coincidental numbering). Verified: no ref/branch/commit contains it (git log --all -S across Megalodon/Ronacher/Shai; the stale claude/why-page-live branch is the pre-#459 original). Live docs/why.html = the #471-fenced pre-upgrade state. Flags 1-3 have NO live target.
- In-repo corpus was already clean: Ronacher mentions are ecosystem/error-policy citations (S42 era); GARNET_REASSESSMENT_2026-06-11.md already attributes the FDA envelope phrasing to McDermott/Ballard Spahr. Zero shelf-scope commits in the July 8-11 window — nothing committed under the invented CRA crisis; nothing to re-ratify in-repo.
- Independently re-verified Flag 1 against primary sources (WebFetch, lucumr.pocoo.org): permission-checks quote incl. 'amost' typo = "Agentic Coding Recommendations" (2025-06-12); needs{time,rng} sketch (full line: fn issue(sub: UserId, scopes: []Scope) -> Token needs { time, rng }) = "A Language For Agents" (2026-02-09), which does NOT contain the permission quote. The report's Flag 1 is confirmed — with our own eyes, not inherited.
- Corpus-of-record written: F_Project_Management/RESEARCH/WHY_PAGE_RECONCILIATION_2026-07-13.md (checklist mapping, verified corrections, rules for the eventual upgrade PR, the shipped src-line TODO blemish at why.html:287). /why UNFROZEN; upgrade PR is Jon's call, buildable from the note's rules.

## 2026-07-13T16:25Z — session 2 (P6: /why upgrade shipped from verified sources)

- Jon directed the upgrade build. Discovery: the page shipped with TWO explicit placeholder cards (.cv ins src: "verify carrier references before publishing"; .cv build src: "attribute + link exact quotes... before naming anyone publicly") plus the threat-card "link primary sources" TODO — the upgrade completes the page's own design.
- 3-agent source-verification workflow fetched every fact at source. It caught errors IN the audit report itself: Shai-Hulud is 373 malicious package-versions across 169 npm + 2 PyPI (CSA/Snyk), NOT "172/403"; the "48 hours" window is unsourced (real first wave: 84 versions in ~6 min); "valid SLSA BL3" needs slsa.dev's own qualifier (linked); the CISA alert is joint (Nx Console + Megalodon); the insurance "red-team snapshots" framing was wrong per-carrier (only Armilla red-teams; Testudo markets "no integration with your AI systems") — the no-attestable-instrument thesis restated accurately.
- Cards shipped: threat (Shai-Hulud/Megalodon headline, TeamPCP attribution, "days later" never "successor", eBuilder+Veracode additive flat-45% line, 10 source links), insurance (4 carriers + ISO Jan-2026 exclusions + Berkshire/Chubb/Travelers approvals per Wolfe attribution, 6 links), Ronacher two-post card (verbatim quotes, a[l]most bracket, both essays linked). Meta+byline: "honest account" -> "calibrated account". Reg card untouched. The two enforced: claims byte-identical.
- Gates: claim fixture 2/2 + 0 forbidden; trust-kernel gate positive AND negative case proven (why.html alone fails without the W_TRUST companion); web-pwa smoke blockers=0; truth --check ok; visual render check via local server (4 cards, all src lines "verified" with live links).
- Companion (trust-kernel): W_TRUST/WHY_UPGRADE_REVIEW_2026-07-13.md — full correction table + source ledger.

## 2026-07-13T18:15Z — session 2 (MISSION COMPLETE)

- Jon merged the last two parked PRs: #481 (2be1c3a — supply-chain scans across all three lockfiles; the scan found+patched 2 live quick-xml CVEs before it ever merged) and #483 (93b5c3d — the parallel session's dogfood-gate lstrip fix; .github/ paths are now correctly readiness-sensitive, confirmed live).
- Final full-gate pass on merged main: dogfood checker 16 tests OK (+ .github/workflows/ci.yml sensitive=True), trust-kernel 13 tests OK + --gate 0, capability-scope --gate 0 (2/2 enforced claims), evidence-integrity 38/38 ok, truth --check ok, both post-merge Security workflow runs on main SUCCESS (three-lockfile audit + Studio deny-advisories with the committed policy + dual SBOM), launch-readiness reporter suite: first pass caught REAL drift (LedgerPinTests red — tracked ledger still said 36/36 from P0 while P1 raised evidence integrity to 38/38); ledger regenerated via the reporter (single-line diff, promo line held at 95.0% proving the P0 machine-independence fix end to end), suite re-run green 37/37.
- Mission ledger: 7 phases done, 19/19 tasks, all gates passed, 14 PRs (12 autonomous with recorded merge trailers, 2 Jon-gated by design). launch_ready remains honestly HOLD on the playground/live-wasm/shelf gates — this mission never claimed otherwise.
- Fences held end to end: S114 independence never relabelled, no tags pushed, every gate-file change human-merged, /why enforced-claim count pinned at exactly 2 throughout.

## 2026-07-15T13:35Z — session 3 (Launch Convergence opened; GOV/W-PLAY/Shelf parallel)

- Reconciled remote truth through PR #496 at exact `origin/main` `32ec9624d571596decd9ea50c64ca9f06fa49131`. PR #486 closed the prior S114 mission; #487 reconciled S114/Wasm truth; #488-#496 built the GOV-001-008 preactivation governance chain. The committed mission heartbeat had stopped at #485, so the old completion state was stale even though its historical phases remained correct.
- Preserved P0-P6 as completed history and opened one compact P7 parallel-convergence phase with four bounded task/gate families: GOV-009, W-PLAY, Minimum Shelf, and final candidate reconciliation. Launch readiness remains HOLD; no product or governance claim was promoted by this bookkeeping change.
- Grounded three exact-#496 isolated worktrees and branches: `mission/gov009-authenticated-transport`, `mission/w-play-live-browser`, and `mission/minimum-shelf-mcp`. Fresh read-only reconnaissance identified bounded first slices and proved the existing GOV/Wasm/MCP baselines are green.
- Frozen implementation contracts under `docs/superpowers/plans/`: GOV-009 authenticated bounded pagination; W-PLAY checker/capability-diff Wasm adapter; Minimum Shelf MCP 2025-11-25 protocol core. Each requires an observed RED before implementation, fresh GREEN gates, <=400 changed lines where possible, and no workflow/gate change in its first PR.
- Three fresh implementers started in parallel. GOV-009 unit tests use injected responses and no real token/network; W-PLAY preserves WV-4 as Studio proof and WV-5 as Wasm+Node proof; Shelf stays protocol-core-only before stdio/execution. Public browser/Shelf claims remain blocked on their later Playwright/seal/reporter traps.
- Current checkpoint through merged #506: GOV-009 has authenticated object transport, strict Link parsing, and bounded complete page-number collection; W-PLAY has real check/diff Wasm exports plus a hermetic no-publish package probe; Shelf has strict schema, lifecycle, and adversarial foundations. The next reviewed slices are candidate anchoring, package materialization, and one bounded in-process Shelf tool; launch claims remain blocked.
- Live authority caveat: the active `Navigata1` gh credential cannot prove admin-only settings (`null`/403 is RED). The separately installed `IslandDevCrew` credential can read admin fields and showed ruleset `18936562` with `bypass_actors: []`; GOV-009 will require an explicit admin-authoritative token and will never inherit, persist, or print the active gh credential.

## 2026-07-16T04:35Z — session 4 (Lane 0 truth freeze; archive and U-18 reconciled)

- Recorded `mainlineCheckpoint` from the local first-parent history at exact successor pin `231aefa91985e5a0520c493c7f0fc3e54d74efc8`: merge order `497,498,500,501,502,503,504,505,506,499`, with every PR mapped to its full squash-main SHA. No `Navigata1/garnet` main state or ambient credential was read.
- Resolved U-18 by renormalizing the only active mission onto materialized `P7`: `resume.activePhase` is exactly `P7`, the four next actions map to `P7-T1` through `P7-T4`, and stale P8/P9/P10 resume references are removed. Renderer-supported phase/task status tokens are now `in_progress`.
- Validated `ops/lane0/idea.lock.json` and `ops/lane0/plan.lock.json` with pinned ARCHIPELAGO provenance `Navigata1/archipelago@b9f7cee2823f9791503db20f33b22c9e20af7abe`; kickoff created only the namespaced Lane 0 state/journal and left this mission state byte-identical during kickoff.
- Launch remains HOLD. GOV-009 admin-authoritative reads and human 31-to-32 activation remain Jon-blocked; browser and Shelf claims remain blocked on committed Playwright, seal, and reporter evidence.

## 2026-07-16T04:56Z — session 4 review correction (full archive and human surface)

- Supersedes the initial session-4 checkpoint description: the frozen claim spans all 34 first-parent PR commits in `d0d4f7cc..1fe7489`, followed by successor archive merge #499 at `231aefa`. `mainlineCheckpoint` now records that full Git-derived order and every exact full squash-main SHA with unambiguous archived-range and successor fields.
- Corrected the ARCHIPELAGO plan verdict to `warning` with `adversarialFindingsResolved: false`. The pinned kickoff regenerated the namespaced Lane 0 state without changing `ops/mission/state.json`; the original Lane 0 journal history and tool provenance remain preserved.
- Added `scripts/garnet_lane0_truth_freeze_status.py` plus regression tests. The frozen P0 gates now compare the checkpoint to local first-parent Git history and enforce U-18 referential integrity, exact P7-T1 through P7-T4 actions, supported status tokens, and absence of P8/P9/P10 resume references.
- The generated SOTU now labels `prLog` as a compact operational log and separately renders the complete 35-merge checkpoint through #499. Launch remains HOLD and all Jon-only boundaries remain unchanged.

## 2026-07-16T11:35Z — session 4 policy, MSRV, and WV acceptance reconciliation

- Settled one enforced Rust floor: root Cargo metadata declares `rust-version = "1.95"`, all 16 active workspace members inherit it, and the excluded Studio backend plus parser fuzz workspace declare it directly. Current README, contributor, FAQ, getting-started, website, and parser surfaces now say Rust 1.95+ while ordinary CI continues to track moving stable.
- Preserved the TDD sequence: `python3 -I scripts/test_garnet_msrv_status.py` first failed because the reporter did not exist; after implementation the reporter suite and `--gate` passed, `cargo +1.95.0 check --workspace --all-targets --all-features --locked` completed green, and the excluded Studio manifest also passed its exact-1.95 all-target check. The existing required CI and Windows Studio contexts now carry the exact-floor checks; this successor PR therefore remains parked for Jon.
- Decided U-16 explicitly: extend the Jon-merge-only exception list to `scripts/garnet_github_*`. PRs #500, #502, and #506 auto-merged correctly under the prior exception list; this decision is prospective, and the governance freeze remains unarmed until the Lane 1 terminus.
- Recorded U-17 as Jon-blocking: P7-T1 cannot close the live settings/no-bypass clause until Jon provisions a dedicated admin-authoritative token. No credential was probed, copied, inherited, persisted, or printed in this slice; GOV-009 must receive the future token explicitly.
- Added machine-readable planned-only WV-6/WV-7 acceptance contracts at `F_Project_Management/LAUNCH/WV6_WV7_ACCEPTANCE_CONTRACTS.json`, each pinned to base `231aefa91985e5a0520c493c7f0fc3e54d74efc8` with authority sources, acceptance command, evidence destination, scope limits, and Jon-only actions. The commands are intentionally marked not yet implemented, so neither contract is presented as proof.
- Launch remains HOLD. No browser, Shelf, governance-activation, cross-OS, FIRE, tag, publish, or production claim was promoted.

## 2026-07-16T11:53Z — session 4 review correction (WV authority + structural MSRV)

- Supersedes the WV assignments in the first acceptance-contract revision. The tracked authority at `cad91aa:ops/mission/state.json#resume.nextActions[1]` retains WV-6 for the native-Windows proof after Core Ring Tier 1 + the Minimum Shelf/MCP library ship, and WV-7 for winget/Scoop dry-run + devcontainer/Docker + installer happy-path distribution proof. Mac Air reconciliation and frozen-candidate release review remain separate P7-T4 work.
- Replaced the two nonexistent revision-1 commands with one deterministic fail-closed reporter: `scripts/garnet_wv_acceptance_status.py`. Both current gates intentionally exit nonzero with `state=pending` until `proofs/windows/launch-verification/wv6-minimum-shelf/WV_ACCEPTANCE.json` or `wv7-distribution/WV_ACCEPTANCE.json` binds every required check and SHA-256 artifact to one full candidate SHA. The reporter performs no Jon-only action and never self-promotes missing evidence.
- Hardened the Rust MSRV reporter from string presence to structure: it now enumerates all 18 active non-archive manifests, rejects an unlisted manifest, scans every declared current surface for conflicting Rust version claims, and uses the pinned typed workflow-YAML projection to require active steps in exactly `ci.yml:test`, `ci.yml:agent-contracts`, and `macos-studio.yml:windows-studio`. Reviewer mutations for comments, `${{ false }}`, wrong-job placement, misplaced stable tracking, Rust 1.94+, and an extra active manifest all fail closed.
- Launch remains HOLD. WV-6 and WV-7 remain pending acceptance contracts, not proof; signing, package-manager publication, promo QA, FIRE, tag, publish, and launch announcement remain Jon-only.

## 2026-07-16T12:45Z — session 5 (Lane 0 truth freeze closed locally)

- Closed namespaced ARCHIPELAGO Lane 0 P0-P3 at S6 without closing the live Garnet P7 mission. Every Lane 0 gate now points to sealed evidence; the namespaced mission is 4/4 definition-of-done complete and remains prepared for Jon's review/merge, never self-merged.
- Recomputed exactly four denominators on the current tree: S114 bounded mission 19/19 = 100.0%; committed truth pulse 65.2/70 = 93.1% (rounded); launch-critical 3/6 = 50.0%; whole launch ledger 3/8 = 37.5%. Launch remains HOLD; no fifth or blended percentage was introduced.
- The ARCHIPELAGO audit is band 3/5. A missing browser Playwright/runtime journey would cap runtime evidence at 4; the requested Lane 2C approval lacks current deterministic three-case, greater-than-four-minute proof, so it remains partial and lowers the audit to 3. Recorded the evidence failure as G4 -> S2 before proceeding to S6.
- S6 governance verdict is advisory with no waivers. Local contracts, gates, evidence hashes, and ledger chain are enforced; Playwright, Lane 2C duration evidence, remote CI, final integrated review, admin-token provisioning, Jon-only merge, and activation remain pending.
- Next: Lane 1 proves fresh/exact-head/outcome/live-policy clauses after Jon provisions a dedicated admin-authoritative token; Lane 2A closes package/page/Playwright/denial proof; Lane 2B closes bounded tool/raw-byte/seal/rejection/reporter proof; Lane 2C produces current deterministic duration evidence before approval.
