# Garnet Language Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn Garnet's original dual-mode, agent-native language ambition into a sequence of executable milestones that can be reviewed, falsified, and eventually presented as a stable MIT-grade language/toolchain.

**Architecture:** Complete Garnet by vertical language slices. A feature is not current truth until parser/AST support, checker or type semantics, interpreter/runtime behavior, conformance tests, dogfood examples, documentation status, and CI gates line up.

**Tech Stack:** Rust workspace, `garnet-parser-v0.3`, `garnet-check-v0.3`, `garnet-interp-v0.3`, `garnet-actor-runtime`, `garnet-memory-v0.3`, `garnet-cli`, Cargo integration tests, GitHub Actions, Markdown evidence ledgers, dogfood-readiness reports.

---

## Completion Ledger

This table is the current truth as of the v0.5 readiness-remediation branch. It separates what is already executable from work that is parsed-only, partial, or still future research.

| Ambition | Current status | Evidence | Next executable gate |
|---|---|---|---|
| 10 MVP app corpus | Complete for current v0.4.2 examples | `garnet-cli/tests/dogfood_readiness_examples.rs`; CI `canonical MVP examples` job | Keep `cargo test -p garnet-cli --test dogfood_readiness_examples` green |
| Current-state/reviewer guide | Complete first pass | `CURRENT_STATE.md`; `F_Project_Management/GARNET_CURRENT_VS_HISTORICAL_LEDGER.md` | Review before each public/MIT packaging pass |
| Repo IA truth separation | Complete first pass | `CURRENT_STATE.md`; `archive/history/`; roadmap index | Finish link-rewrite cleanup before a public main-page launch |
| v0.4.2 release assets | Org release published and release-backed installer smoke passed | `Island-Dev-Crew/garnet` release `v0.4.2`; `./scripts/verify_org_release_smoke.sh`; Desktop bundle `/Users/idc2.0/Desktop/dogfood/garnet-phase4bi-org-release-closed-20260515-1843` | Keep release assets/checksums intact; signed macOS `.pkg` and Windows MSI remain separate authority work |
| Agentic dogfood stress matrix | Active for audited agent-facing workflows, packaged-app mode, diagnostics, adversarial input boundaries, web/PWA productization, macOS CI artifact capture, packaged-resource source-workspace boundaries, and per-domain coverage adequacy | `scripts/run_agentic_dogfood_matrix.py --copy-to-desktop --strict`; `.github/workflows/agentic-dogfood-matrix.yml` runs the headless-safe matrix with `--skip-app-workbench`; Desktop bundles under `/Users/idc2.0/Desktop/dogfood/`; `F_Project_Management/DOGFOOD/GARNET_AGENTIC_DOGFOOD_STRESS_PLAN.md` | Keep the 3-5 probe bar visible while keeping functional failures, documented skips, and coverage debt separate; all current source-checkout domains are adequate after template scaffold-run-test, doc/fmt check/fmt repair, adversarial parser/capability/safe-mode rejection, advertised log-analyzer parse/check/run, offline/local/browser PWA probes, and Garnet Studio self/smoke/XCTest app probes |
| Garnet Studio agentic matrix surface | Active for source-checkout, packaged app, mounted-DMG install-smoke, packaged docs/PWA resources, converter assist-plan packaged resource staging, Converter-panel assist-plan UI, source-checkout Run-button workflow, source-workspace-only cargo probe skip accounting, manifest-backed DMG smoke evidence, credential-gated Developer ID hardened-runtime signing, and notarization-preflight workflows | `python3 scripts/test_garnet_studio_run_button.py`; `./script/build_and_run.sh --verify`; `swift run --package-path apps/garnet-studio-macos GarnetStudio --agentic-matrix-test`; `swift test --package-path apps/garnet-studio-macos`; `swift run --package-path apps/garnet-studio-macos GarnetStudio --self-test`; `swift run --package-path apps/garnet-studio-macos GarnetStudio --smoke-test`; `./scripts/package_garnet_studio_macos.sh`; `scripts/smoke_garnet_studio_dmg.sh --copy-to-desktop target/macos/GarnetStudio.dmg`; `scripts/preflight_garnet_studio_notarization.sh --copy-to-desktop`; `security find-identity -p codesigning -v`; Desktop bundles `/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-040631`, `/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-045430`, `/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-045440`, `/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-074756`, `/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-074816`, and `/Users/idc2.0/Desktop/dogfood/garnet-studio-notarization-preflight-20260516-021713` | Configure a valid Developer ID identity plus notary profile, then run strict notarization preflight and real-browser web/PWA/mobile gates as separate productization slices |
| Garnet web/PWA readiness | Active seed installable docs surface plus offline service-worker, Studio workbench adoption section, Continuation Pulse hook, readiness status page, live Pages Studio/status-copy smoke, and local real-browser offline gate | `docs/manifest.webmanifest`; `docs/service-worker.js`; `docs/index.html`; `docs/status.html`; `scripts/smoke_garnet_web_pwa.sh --copy-to-desktop --strict`; `scripts/smoke_garnet_web_pwa_offline.mjs --docs-dir docs`; `scripts/smoke_garnet_web_pwa_browser.mjs --docs-dir docs`; `scripts/smoke_garnet_pages_pwa.sh --copy-to-desktop --strict`; `python3 scripts/test_smoke_garnet_pages_pwa.py`; `.github/workflows/web-pwa-readiness.yml`; Browser smoke of `http://127.0.0.1:8765/index.html#studio` | Broaden beyond static docs into a richer web workbench only after the shell, offline service-worker fallback, browser offline gate, Pages deployment smoke, Studio adoption-copy guard, Continuation Pulse guard, and status-page guard stay green |
| Converter adoption truth | Active for deterministic Rust/Ruby/Python/Go migration assistance plus explicit advisory language, native-boundary, backend-lowering, and Garnet-aware LLM/agentic assist gates | `scripts/garnet_converter_status.py`; `python3 scripts/test_garnet_converter_status.py`; `scripts/run_agentic_dogfood_matrix.py` probe `report-converter-adoption-status`; `docs/index.html`; `docs/status.html`; `F_Project_Management/GARNET_CONVERTER_AND_PLATFORM_STRATEGY.md` | Add JavaScript/TypeScript or other high-value deterministic frontends only as separate tested slices; keep advisory planning provider-optional, sandboxed, lineage-preserving, `garnet check`-verified, human-audited, native-boundary honest, and dogfood-gated |
| Garnet-aware assist context pack | Active for deterministic current-truth context and provider-neutral prompt packaging before any provider-backed converter assist is enabled | `scripts/garnet_assist_context_pack.py`; `python3 scripts/test_garnet_assist_context_pack.py`; `scripts/run_agentic_dogfood_matrix.py` probes `report-assist-context-*`; `docs/index.html` | Use this pack as the future LLM/agentic assist input contract; do not claim model-backed conversion until provider/runtime boundaries and dogfood gates exist |
| Converter assist planning | Active for deterministic per-file advisory-language migration risk planning without activating broad conversion | `scripts/garnet_converter_assist_plan.py`; `python3 scripts/test_garnet_converter_assist_plan.py`; `scripts/run_agentic_dogfood_matrix.py` probes `report-assist-plan-*`; `docs/index.html`; `docs/status.html` | Keep this advisory-only: it may inventory safe-mode, memory, CapCaps, actor/orchestration, shell/process, SQL/data, and migration risks for TypeScript, JavaScript, Swift, Java, C, C++, C#, Perl, Kotlin, Shell, SQL, Other, and native-boundary sources, but deterministic frontends and provider-backed conversion remain separate gated slices |
| Converter LLM feasibility | Active for answering whether an LLM belongs in the converter path without overclaiming active conversion | `scripts/garnet_converter_llm_feasibility.py`; `python3 scripts/test_garnet_converter_llm_feasibility.py`; `scripts/run_agentic_dogfood_matrix.py` probes `report-converter-llm-feasibility-*`; `docs/index.html`; `docs/status.html` | Treat provider-neutral advisory planning as feasible; keep autonomous/provider-backed LLM conversion inactive until secure runtime, deterministic frontend, native-boundary, lineage, `@sandbox`, `garnet check`, dogfood, and human-audit gates exist |
| Converter advisory bundle | Active for provider-neutral local handoff packaging before any provider-backed model lane exists | `scripts/garnet_converter_advisory_bundle.py`; `python3 scripts/test_garnet_converter_advisory_bundle.py`; `scripts/run_agentic_dogfood_matrix.py` probes `report-converter-advisory-bundle-*`; `docs/index.html` | Combine feasibility, context, and per-file assist-plan evidence in a manifested bundle; omit source by default, require `--include-source` for explicit local/provider handoff, and keep conversion inactive |
| Converter advisory review | Active for checking a manifested advisory bundle before any model/agent handoff | `scripts/garnet_converter_advisory_review.py`; `python3 scripts/test_garnet_converter_advisory_review.py`; `scripts/run_agentic_dogfood_matrix.py` probes `report-converter-advisory-review-*`; `docs/index.html` | Verify the bundle manifest, block source-included bundles unless explicitly approved, emit a human-review checklist, and keep provider-backed conversion inactive |
| Converter advisory handoff | Active for source-free provider-neutral prompt packaging after the review gate | `scripts/garnet_converter_advisory_handoff.py`; `python3 scripts/test_garnet_converter_advisory_handoff.py`; `scripts/run_agentic_dogfood_matrix.py` probes `report-converter-advisory-handoff-*`; `docs/index.html` | Consume the reviewed advisory bundle, refuse blocked/source-included reviews, emit a no-source handoff packet, and keep model calls plus conversion inactive |
| Converter advisory bundle UX | Active in Garnet Studio as a local package action, not provider-backed conversion | `apps/garnet-studio-macos/Sources/GarnetStudio/GarnetStudioApp.swift`; `swift test --package-path apps/garnet-studio-macos`; `scripts/run_agentic_dogfood_matrix.py` probes `report-studio-advisory-bundle-*`; `docs/index.html` | Expose `Advisory Bundle` beside `Assist Plan`, write manifested local handoff output under `~/Desktop/dogfood/`, and keep source omitted by default |
| Converter advisory review UX | Active in Garnet Studio as a local review action, not provider-backed conversion | `apps/garnet-studio-macos/Sources/GarnetStudio/GarnetStudioApp.swift`; `swift test --package-path apps/garnet-studio-macos`; `scripts/run_agentic_dogfood_matrix.py` probes `report-studio-advisory-review-*`; `docs/index.html` | Expose `Advisory Review` beside `Advisory Bundle`, create a no-source bundle, run the review gate, and preserve the review report under `~/Desktop/dogfood/` |
| Converter advisory handoff UX | Active in Garnet Studio as the final source-free packet action, not provider-backed conversion | `apps/garnet-studio-macos/Sources/GarnetStudio/GarnetStudioApp.swift`; `swift test --package-path apps/garnet-studio-macos`; `scripts/run_agentic_dogfood_matrix.py` probes `report-studio-advisory-handoff-*`; `docs/index.html` | Expose `Advisory Handoff` beside `Advisory Review`, create the default no-source bundle, pass it through review, and preserve the source-free handoff packet under `~/Desktop/dogfood/` |
| MIT readiness objective accounting | Active for broader productization truth beyond the complete tracked implementation-plan ledger plus a public-site progress pulse | `scripts/garnet_mit_readiness_status.py`; `python3 scripts/test_garnet_mit_readiness_status.py`; `scripts/run_agentic_dogfood_matrix.py` probes `report-mit-readiness-*`; `docs/index.html` now surfaces the current `58.6%` local MIT/productization checkpoint beside `87/87 tracked slices` | Use this reporter when discussing public/MIT readiness so `87/87` tracked slices are not confused with notarization, mobile distribution, promo video, broad converter frontends, LLM assist, proof, or empirical validation completion |
| Mac-side continuation accounting | Active for goal-prompt continuation after PR #141 without claiming blocked/delegated gates | `scripts/garnet_mac_side_continuation_status.py`; `python3 scripts/test_garnet_mac_side_continuation_status.py`; `scripts/run_agentic_dogfood_matrix.py` probe `report-mac-side-continuation-boundaries` | Continue Mac-actionable repo, website, converter-advisory, unsigned Studio, proof, and evidence slices; keep Apple Developer ID notarization account-holder blocked, Windows/Linux Studio target-platform delegated, provider-backed LLM conversion inactive, and native backend lowering unimplemented |
| Mac continuation pulse UX | Active in Garnet Studio as a read-only continuation/status action | `apps/garnet-studio-macos/Sources/GarnetStudio/GarnetStudioApp.swift`; `swift test --package-path apps/garnet-studio-macos`; `scripts/run_agentic_dogfood_matrix.py` probes `report-studio-mac-continuation-pulse-*`; `./scripts/package_garnet_studio_macos.sh` | Expose `Continuation Pulse` in the Release panel, run the Mac-side continuation reporter from source or packaged resources, and keep Mac-actionable work separate from Apple Developer ID and Windows/Linux gates |
| MIT demo-route evidence | Active as a bounded presentation route, not final MIT/productization acceptance | `scripts/garnet_mit_demo_route.py`; `python3 scripts/test_garnet_mit_demo_route.py`; `scripts/run_agentic_dogfood_matrix.py` probes `report-mit-demo-route-*`; Desktop bundle `/Users/idc2.0/Desktop/dogfood/garnet-mit-demo-route-20260516-171836` | Use this route to rehearse/demo current verified surfaces and blocked gates; keep Developer ID, Windows/Linux, provider-backed LLM conversion, native backend lowering, mobile distribution, production-ready language, and final acceptance claims forbidden until separate evidence exists |
| MIT demo-route UX | Active in Garnet Studio as a manifested Release-panel action, not final MIT/productization acceptance | `apps/garnet-studio-macos/Sources/GarnetStudio/GarnetStudioApp.swift`; `swift test --package-path apps/garnet-studio-macos`; `scripts/run_agentic_dogfood_matrix.py` probes `report-studio-mit-demo-route-*`; Desktop bundle `/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-174234`; packaged DMG smoke `/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-174457` | Expose `Demo Route` beside `Objective Pulse` and `Continuation Pulse`, run the packaged/source route reporter with `--output-dir`, and preserve a `~/Desktop/dogfood/garnet-studio-mit-demo-route-<stamp>` bundle without claiming notarization, Windows/Linux proof, provider-backed conversion, native lowering, or final acceptance |
| MIT deck-outline evidence | Active as a bounded reviewer deck outline, not final MIT/productization acceptance | `scripts/garnet_mit_deck_outline.py`; `python3 scripts/test_garnet_mit_deck_outline.py`; `scripts/run_agentic_dogfood_matrix.py` probes `report-mit-deck-outline-*`; `scripts/test_garnet_studio_packaging_resources.py`; `./scripts/package_garnet_studio_macos.sh` | Generate a manifested JSON/Markdown 8-slide outline from the current demo route, adoption surface, readiness pulse, blocked gates, and evidence notes; stage the reporter into packaged Studio resources while keeping provider-backed conversion, notarization, Windows/Linux proof, native backend lowering, mobile distribution, and final acceptance unclaimed |
| MIT deck-outline UX | Active in Garnet Studio as a manifested Release-panel action, not final MIT/productization acceptance | `apps/garnet-studio-macos/Sources/GarnetStudio/GarnetStudioApp.swift`; `swift test --package-path apps/garnet-studio-macos`; `scripts/run_agentic_dogfood_matrix.py` probes `report-studio-mit-deck-outline-*` | Expose `Deck Outline` beside the existing presentation/status buttons, run the packaged/source deck-outline reporter with `--output-dir`, and preserve a `~/Desktop/dogfood/garnet-studio-mit-deck-outline-<stamp>` bundle without claiming notarization, Windows/Linux proof, provider-backed conversion, native lowering, mobile distribution, or final acceptance |
| MIT deck-preview artifact | Active as a browser-smokeable HTML review artifact, not final MIT/productization acceptance | `scripts/garnet_mit_deck_preview.py`; `python3 scripts/test_garnet_mit_deck_preview.py`; `scripts/run_agentic_dogfood_matrix.py` probes `report-mit-deck-preview-*` | Render the current deck outline into a self-contained HTML preview plus JSON, outline Markdown, and checksum manifest so reviewers can inspect the slide story, evidence, speaker notes, blocked gates, and forbidden claims without treating it as human/aesthetic deck approval |
| MIT deck-preview UX | Active in Garnet Studio as a manifested Release-panel action, not final MIT/productization acceptance | `apps/garnet-studio-macos/Sources/GarnetStudio/GarnetStudioApp.swift`; `swift test --package-path apps/garnet-studio-macos`; `scripts/run_agentic_dogfood_matrix.py` probes `report-studio-mit-deck-preview-*`; `scripts/test_garnet_studio_packaging_resources.py` | Expose `Deck Preview` beside the existing presentation/status buttons, run the packaged/source deck-preview reporter with `--output-dir`, and preserve a `~/Desktop/dogfood/garnet-studio-mit-deck-preview-<stamp>` bundle without claiming human/aesthetic deck approval, notarization, Windows/Linux proof, provider-backed conversion, native lowering, mobile distribution, or final acceptance |
| MIT deck-preview packaged smoke | Active in copied-DMG app smoke as executable and manifest-verified deck-preview proof, not final deck approval | `apps/garnet-studio-macos/Sources/GarnetStudio/GarnetStudioApp.swift`; `swift test --package-path apps/garnet-studio-macos`; `scripts/smoke_garnet_studio_dmg.sh`; `scripts/run_agentic_dogfood_matrix.py` probes `report-studio-mit-deck-preview-smoke-*` | Add `GarnetStudio --mit-deck-preview-smoke`, require manifested HTML/JSON/outline/manifest outputs, verify the generated preview manifest with `shasum -a 256 -c MANIFEST.sha256`, and run a dedicated copied-DMG manifest verification log with output preserved under `studio-deck-preview/` without claiming human/aesthetic deck approval, notarization, Windows/Linux proof, provider-backed conversion, native lowering, mobile distribution, or final acceptance |
| Public Studio continuation hook | Active on the landing page and Pages smoke contract | `docs/index.html`; `scripts/smoke_garnet_pages_pwa.sh`; `python3 scripts/test_smoke_garnet_pages_pwa.py`; Desktop bundle `/Users/idc2.0/Desktop/dogfood/pages-pwa-readiness-20260516-165857` | Keep the landing-page hook concise and product-facing; verify live `garnet-lang.org` after merge instead of claiming Pages deployment from local source |
| Promo video readiness contract | Active as public-site embedded evidence for the 30-second Garnet promo/ad lane when Desktop render, visual-QA, website-export, and site-sync bundles are present | `scripts/garnet_promo_video_status.py`; `scripts/render_garnet_promo_video.mjs`; `scripts/qa_garnet_promo_video.mjs`; `scripts/export_garnet_promo_video_site.mjs`; `scripts/sync_garnet_promo_video_site.mjs`; `docs/promo/DESIGN.md`; `docs/promo/composition.html`; `docs/assets/garnet-promo.mp4`; `docs/assets/garnet-promo.webm`; `docs/assets/garnet-promo-poster.png`; `python3 scripts/test_garnet_promo_video_status.py`; `python3 scripts/test_render_garnet_promo_video.py`; `python3 scripts/test_qa_garnet_promo_video.py`; `python3 scripts/test_export_garnet_promo_video_site.py`; `python3 scripts/test_sync_garnet_promo_video_site.py`; `scripts/run_agentic_dogfood_matrix.py` probes `report-promo-video-*` | Keep human/aesthetic acceptance as the remaining promo gate before claiming final marketing creative |
| Parser parity for old ambition | Partial, Phase 1 active | `protocol`, `dyn Trait`, `yield`, `next`, `@dynamic`, `@nonsendable`, and `do ... end` parser tests | Keep runtime gaps explicit and activate Phase 2 only with executable semantics |
| Blocks, `yield`, `next` runtime semantics | Phase 2A active | `do ... end` parses as a trailing closure argument; `deferred_blocks_and_yield` runs a managed-mode block/yield/next program | Keep `cargo test -p garnet-cli --test conformance_skeleton deferred_blocks_and_yield` green; add richer block edge cases later |
| Dynamic method dispatch tables | Partial Phase 2H | `deferred_dynamic_dispatch` covers per-instance method tables; `static_impl_dispatch_and_method_missing` covers static inherent impl fallback and `method_missing`; `dynamic_impl_dispatch_tables` covers `@dynamic impl Type for Protocol` registration and dispatch | Add richer dispatch precedence and ambiguity probes |
| Structural protocol satisfaction and runtime casts | Partial Phase 2H | `Item::Protocol` and `Expr::Cast` parse; `deferred_structural_protocols` checks protocol-typed managed parameters, runtime `as Protocol` casts, static/dynamic methods, mode/arity/parameter/return annotation mismatches, generic protocol substitution, core built-in typed method signatures, and `@dynamic impl` methods | Add broader trait/generic coherence |
| Actor protocol enforcement and `Sendable` | Partial Phase 3D | actor runtime crate exists; `actor_sendable_rejects_nonsendable_protocol_payloads` rejects `@nonsendable` actor protocol payloads before runtime; managed interpreter now registers actors, dispatches `spawn Actor.handler(args)` synchronously, creates `spawn Actor` addresses with persistent actor-local state, enforces bounded source mailboxes through `Actor.spawn(capacity)`, and ships a generated `agent-orchestrator` actor template that runs/tests through managed actor addresses; full async OS-thread bridge remains partial | Bridge generated actor projects to the full async `garnet-actor-runtime` OS-thread address/mailbox runtime |
| Rust-grade NLL and borrow rules | Partial Phase 4L | `garnet-check-v0.3/src/borrow.rs`; `garnet-check-v0.3/src/lib.rs`; `garnet-check-v0.3/tests/borrow.rs`; `garnet-check-v0.3/tests/extended.rs`; `partial_borrow_rule_suite` rejects direct use-after-move, direct mut-aliasing, `own self` method receiver moves, method receiver aliasing, simple typed receiver disambiguation, simple field-place aliasing/field use-after-move, and conservative index-place aliasing/index use-after-move while checking nested index operands; `deferred_full_borrow_rule_suite` now covers B5 same-call overlapping `own` drop discipline, direct-returning branch liveness, direct `return` block termination, direct-returning loop-body liveness, scoped `for` loop-variable liveness, scoped `match` pattern binding liveness, match guard move merging, and match-arm block statement preservation; `deferred_nll_lifetime_inference` covers conservative reference-return lifetime elision | Activate full CFG NLL, dynamic place tracking, generic/trait impl dispatch, broader drop elaboration, general loop fixed-point analysis, and two-phase borrows |
| Pattern match exhaustiveness/reachability | Partial Phase 4BI | `garnet-check-v0.3/src/match_coverage.rs`; `garnet-check-v0.3/tests/match_coverage.rs`; `deferred_match_exhaustiveness_and_reachability` rejects non-exhaustive safe-mode `Bool`, same-module enum, finite nested-constructor, and scoped named/glob/module-qualified imported enum alias matches, treats unknown guarded arms as non-covering, counts literal `if true` arms as coverage, rejects literal `if false` arms as statically unreachable, rejects duplicate finite covered arms, rejects open-domain duplicate literal arms plus arms after unguarded catch-all patterns, infers finite match domains from immutable local boolean/enum variant initializers, tracks direct mutable-local finite assignments plus non-finite assignment invalidation, joins finite match-domain evidence across conservative `if`/`elsif`/`else` assignment branches, carries nested `if` all-path assignment joins inside branch bodies, explicitly invalidates finite evidence after compound assignments, conservatively invalidates after possible loop-body assignments with ordered shadowing checks, invalidates after possible `try`/`rescue`/`ensure` writes, prevents uninvoked closure literal bodies from merging assignment domains into enclosing flow, invalidates after direct closure-literal invocations, invalidates after directly called local closure-literal bindings, invalidates after branch-joined local closure-literal binding calls, invalidates after all-path branch rebindings of local closure-literal bindings, invalidates after direct aliases of known local closure-literal bindings, invalidates after all-path branch-selected direct aliases of known local closure bindings, invalidates after direct calls to all-path branch-selected closure expressions, recognizes immutable local boolean guard constants while keeping mutable guard locals unknown, and recognizes same-module, scoped named/glob imported, and path-qualified top-level boolean `const` guard constants, narrow boolean const aliases, and basic boolean const expressions, including left-decisive short-circuit `and`/`or` plus boolean equality/inequality comparisons, the same conservative boolean folding directly in match guard expressions, checked integer arithmetic plus equality/inequality and relational comparisons, same-module bare-name plus scoped named/glob imported integer const identifier forms, static boolean relational const guards, static nil/symbol/string equality/inequality const guards including static interpolated strings, static string relational const guards, static nil relational const guards, mixed known-literal equality/inequality facts, finite float equality/inequality, int-float equality, finite float/int-float relational facts, finite float/int-float arithmetic facts, and immutable local boolean/integer const-expression guard aliases, including aliases that reference path-qualified top-level constants, while preserving parameter shadowing and mutable-local expression invalidation | Add cross-file/package imports, recursive/open payload reasoning, non-finite floats, call-backed/dynamic interpolated strings, broader non-boolean non-string non-numeric comparison, broader float edge-case reasoning, function-call, and broader const expression evaluation beyond immutable local aliases and path-qualified const references, loop fixed-point domain inference, broader mutable/escaped/general higher-order closure invocation/call-effect analysis, broader expression/type inference, open-domain exhaustiveness/range reasoning, and richer non-literal guard-aware diagnostics |
| Trait coherence | Partial Phase 5C | `garnet-check-v0.3/src/coherence.rs`; `garnet-check-v0.3/tests/coherence.rs`; `deferred_trait_coherence` rejects exact duplicate trait impls, orphan-rule violations, simple generic blanket-vs-concrete overlaps, renamed generic blanket overlaps, and qualified external type short-name collisions while allowing local-trait, local-type, and qualified local-module impls | Activate specialization and imported-package coherence solving |
| Generic instantiation / monomorphization | Partial Phase 5B | `generic_instantiation_runs_without_monomorphization_claims` runs generic struct construction, a generic impl method, and a generic function through the managed interpreter | Keep native zero-cost monomorphization deferred until a compiler backend exists |
| Memory Core ARC/cycles and allocator integration | Partial Phase 6Q + Phase 6R + Phase 6S + Phase 6U | `garnet-memory-v0.3/src/{alloc,cycle,working,episodic,semantic,procedural}.rs`; `garnet-memory-v0.3/tests/{cycle,properties,persistence}.rs`; active `deferred_arc_cycle_detection`; `CycleAllocatorFixture` owns graph + root buffer for root/edge decrement scheduling; all four stores expose kind-aware allocator stats; policy-configured episodic/semantic stores evict lazily on read/search; `CycleAwareKindAllocator` observes store-root retain/release lifecycles on write, clear, eviction, replacement, and drop, exposes allocator-facing root release collection evidence for finalization order plus safe-affine exclusion (Phase 6Q), exposes allocator-facing buffered edge-removal collection evidence with threshold-driven `CycleAwareKindAllocator::remove_edge` reports across all four memory kinds (Phase 6R), and records allocator-owned finalizer-log evidence for plain release/collect/remove-edge calls (Phase 6S); `EpisodeStore::save_text` / `load_text` prove versioned episodic text snapshot recovery, delimiter-safe payload encoding, malformed-file non-mutation, and cycle-aware root rehydration; the typed `.garnet-cache/episodic/episodes.mnemos` backend now also has opt-in BLAKE3-keyed record MAC append/load APIs that reject tampered payloads and foreign keys before live store mutation (Phase 6U) | Promote the bounded allocator-owned fixture model into production allocator-integrated ARC and broaden signed persistence/backend hardening beyond the reference episodic backend slice |
| Compiler-as-agent cache privacy/replay | Partial Phase 6I | `garnet-cli/src/{cache,cmd,provenance}.rs`; `garnet-cli/tests/cache_episodes.rs`; cache episode logs redact external absolute paths, collapse project-local absolute paths to stable relative labels, warn while ignoring same-cache foreign-key plus copied-cache replay episodes, bind verified episodes to a keyed source-tree identifier, quarantine copied/stale strategy rows whose provenance does not re-verify in the current source tree, and preserve bounded concurrent plus 16-writer soak appends; CacheHMAC and ProvenanceStrategy tests remain active | Add extended release-duration/cross-platform cache soak and keep production Memory Core ARC integration separate |
| Native compiler | Long-horizon scaffold only | no backend crate | Create backend design PR before claiming compiled language status |
| Formal RustBelt/Iris/Coq proof | Long-horizon scaffold only | Paper V theorem sketches | Open proof repo or `proofs/` workspace with checked theorem stubs |
| Signed cross-platform installers | Partial | Linux packages and checksums exist; macOS/Windows signing remains separate authority work | Signed/notarized macOS and Authenticode Windows install smokes |
| Empirical PLDI-grade validation | Long-horizon scaffold only | benchmarking and empirical protocols exist | Run pre-registered studies with archived datasets/scripts |

## Done Means Executable

Do not advance public status for any row unless all applicable evidence lands in the same PR:

1. A failing conformance or dogfood test that names the missing behavior.
2. Parser/AST support when syntax is involved.
3. Checker/type-system support when the spec promises safety or conformance.
4. Runtime/interpreter support when the feature is user-visible.
5. A canonical example, template, or MVP app smoke when users or agents touch it.
6. Updated conformance matrix and current-state docs.
7. Local verification plus GitHub Actions verification before merge.

## Milestone 1: Parser Parity Baseline

**Purpose:** Accept the old design syntax in a controlled parser-stage form without claiming runtime semantics.

**Files:**

- Modify: `garnet-parser-v0.3/src/token.rs`
- Modify: `garnet-parser-v0.3/src/ast.rs`
- Modify: `garnet-parser-v0.3/src/grammar/mod.rs`
- Modify: `garnet-parser-v0.3/src/grammar/user_types.rs`
- Modify: `garnet-parser-v0.3/src/grammar/types.rs`
- Modify: `garnet-parser-v0.3/src/grammar/stmts.rs`
- Modify: `garnet-parser-v0.3/src/grammar/expr.rs`
- Test: `garnet-parser-v0.3/tests/parse_v1_parser_parity.rs`
- Test: `garnet-parser-v0.3/tests/properties.rs`
- Test: `garnet-cli/tests/conformance_skeleton.rs`
- Docs: `C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md`

- [x] **Step 1: Land parser-stage support for `protocol`, `dyn Trait`, `yield`, `next`, `@dynamic`, and `@nonsendable`**

Run:

```sh
cargo test -p garnet-parser --test parse_v1_parser_parity
cargo test -p garnet-cli --test conformance_skeleton
```

Expected: parser-parity tests pass; deferred runtime/type-system handles remain ignored.

- [x] **Step 2: Add failing parser test for `do ... end` block arguments**

Add this test to `garnet-parser-v0.3/tests/parse_v1_parser_parity.rs`:

```rust
#[test]
fn parses_do_end_block_argument() {
    let src = r#"
def main() {
  each([1, 2, 3]) do |x|
    yield x + 1
  end
}
"#;
    parse_ok(src);
}
```

Run:

```sh
cargo test -p garnet-parser --test parse_v1_parser_parity parses_do_end_block_argument
```

Observed before implementation: failed with `UnexpectedToken` at the newline
after the `do |x|` header because block arguments were not parsed.

- [x] **Step 3: Implement `do ... end` parser support**

Parse `do ... end` as an `Expr::Closure` appended to the call or method-call
argument list. Phase 2A now tags syntactic `do...end` closures so runtime
block dispatch can distinguish them from ordinary first-class closure
arguments.

Run:

```sh
cargo test -p garnet-parser --test parse_v1_parser_parity
cargo test -p garnet-cli --test conformance_phase_gates
```

Expected after implementation: parser test passes; phase gates still prove runtime rows are not silently marked complete.

## Milestone 2: Managed Runtime Semantics

**Purpose:** Make parser-stage surfaces run in managed mode with explicit behavior.

**Files:**

- Modify: `garnet-interp-v0.3/src/value.rs`
- Modify: `garnet-interp-v0.3/src/env.rs`
- Modify: `garnet-interp-v0.3/src/eval.rs`
- Modify: `garnet-interp-v0.3/src/stmt.rs`
- Modify: `garnet-interp-v0.3/src/control.rs`
- Test: `garnet-cli/tests/conformance_skeleton.rs`
- Example: `examples/mvp_06_multi_agent.garnet`

- [x] **Step 1: Replace the ignored block/yield placeholder with a failing executable test**

Change `deferred_blocks_and_yield` so it runs a Garnet program that passes a block and yields a value through it.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton deferred_blocks_and_yield
```

Observed before implementation: failed with `arity mismatch: expected 0, got 1` because the trailing block was still treated as a normal argument.

- [x] **Step 2: Add runtime support for callable block objects**

Use the existing closure-backed `Value::Fn` representation as the block object, bind a trailing closure into the call frame, and route `yield`/`next` through `stmt.rs`.
Only syntactic `do...end` closures may become implicit blocks; ordinary closure
arguments continue to go through normal arity checks.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton deferred_blocks_and_yield
```

Expected after implementation: pass without `#[ignore]`.

- [x] **Step 3: Add managed-mode dynamic dispatch tables**

Change `deferred_dynamic_dispatch` into an active test that constructs a dynamic receiver and dispatches through the method table.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton deferred_dynamic_dispatch
```

Observed before implementation: failed at runtime with `struct method dispatch for 'def_method' requires Rung 4 impl resolution`.

Expected after implementation: pass without `#[ignore]` for the per-instance dynamic method-table slice. Static `impl` fallback and `method_missing` remain deferred.

- [x] **Step 3D: Add static impl fallback and method_missing**

Add an active dispatch-order test proving static inherent impl methods resolve after per-instance dynamic methods and `method_missing` handles unresolved calls.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton static_impl_dispatch_and_method_missing
```

Observed before implementation: failed at runtime with `struct method dispatch for 'label' requires Rung 4 impl resolution`.

Expected after implementation: pass with static inherent impl method registration and `method_missing` fallback in managed mode. `@dynamic impl` tables remain follow-up work.

- [x] **Step 4: Add structural protocol satisfaction and casts**

Change `deferred_structural_protocols` into a test that proves a struct satisfies a protocol by method shape and rejects a missing method.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton deferred_structural_protocols
```

Observed before implementation: failed because protocol-typed parameters were accepted without checking required methods.

Expected after implementation: pass without `#[ignore]` for protocol-typed managed parameter checks, including static inherent impl-backed satisfaction.

- [x] **Step 4E: Tighten protocol method signature compatibility**

Add negative probes to `deferred_structural_protocols` that prove name-only method presence is insufficient.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton deferred_structural_protocols
```

Observed before implementation: a method with the required name but incompatible arity was accepted.

Expected after implementation: protocol satisfaction checks method mode, receiver-adjusted arity, annotated parameter types, and required return types. Runtime `as Protocol` casts, generic protocol substitution, and built-in typed signatures remain follow-up work.

- [x] **Step 4F: Execute runtime `as Protocol` casts**

Add parser and runtime probes that prove `value as Protocol` is not parsed as a stray identifier and that cast success/failure uses the same structural protocol gate as protocol-typed parameters.

Run:

```sh
cargo test -p garnet-parser --test parse_v1_parser_parity parses_protocol_cast_expression
cargo test -p garnet-cli --test conformance_skeleton deferred_structural_protocols
```

Observed before implementation: the compatible cast path failed at runtime with `undefined variable: as`.

Expected after implementation: `Expr::Cast` parses, structurally compatible values pass through unchanged, and incompatible values fail with `does not satisfy protocol ...`. Generic protocol substitution and built-in typed signatures remain follow-up work.

- [x] **Step 4G: Substitute protocol type parameters and type core built-in signatures**

Add positive and negative probes to `deferred_structural_protocols` proving `Protocol<T>` signatures are instantiated before method compatibility checks and that core built-in methods can satisfy typed protocol signatures.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton deferred_structural_protocols
```

Observed before implementation: `BoxLike<String>` rejected a `TextBox.value() -> String` method with `does not satisfy protocol BoxLike` because the required return type remained unresolved as `T`.

Expected after implementation: generic protocol type arguments substitute into required method signatures, incompatible concrete methods still fail, and core built-in methods such as `String#len`, `String#upcase`, and `String#starts_with` satisfy compatible typed protocol signatures while rejecting incompatible return types.

- [x] **Step 4H: Register `@dynamic impl` dispatch tables**

Add a positive dispatch and protocol-satisfaction probe plus a negative probe proving ordinary trait impls remain deferred.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton dynamic_impl_dispatch_tables
```

Observed before implementation: `@dynamic impl TraitWidget for Renderable` was parsed but did not satisfy a `Renderable` parameter, failing with `Struct does not satisfy protocol Renderable: missing method render`.

Expected after implementation: `@dynamic impl` methods satisfy protocol-typed managed parameters and dispatch after per-instance dynamic methods but before static inherent impl fallback. Ordinary non-dynamic trait impl coherence remains deferred.

## Milestone 3: Actor Runtime Bridge And Sendable

**Purpose:** Make the agent-native story executable from Garnet source instead of Rust-only actor-runtime tests.

**Files:**

- Modify: `garnet-parser-v0.3/src/grammar/actors.rs`
- Modify: `garnet-check-v0.3/src/lib.rs`
- Modify: `garnet-check-v0.3/src/borrow.rs`
- Modify: `garnet-actor-runtime/src/runtime.rs`
- Modify: `garnet-interp-v0.3/src/eval.rs`
- Modify: `garnet-cli/templates/agent-orchestrator/src/main.garnet`
- Test: `garnet-cli/tests/new_cmd.rs`
- Test: `garnet-cli/tests/conformance_skeleton.rs`

- [x] **Step 1: Add a failing managed actor bridge smoke**

Extend executable example coverage so source actor/protocol syntax must survive `garnet run`, not only parsing.

Run:

```sh
cargo test -p garnet-cli --test examples multi_agent_builder_runs_with_managed_actor_bridge
cargo test -p garnet-interp c5_actor_handler_dispatches_via_spawn_bridge
```

Observed before implementation: `garnet run examples/multi_agent_builder.garnet` failed with `runtime error: undefined variable: Planner`.

Expected after implementation: actor declarations register as managed runtime actor values and `spawn Actor.handler(args)` dispatches the matching handler synchronously. Full `garnet-actor-runtime` address/mailbox bridging remains a follow-up.

- [x] **Step 2: Enforce `Sendable` at actor message boundaries**

Reject `@nonsendable` message payloads before runtime dispatch.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton
```

Expected after implementation: the actor/sendable test is active and rejects the bad case.

Observed before implementation: `garnet-check` returned no errors for an actor protocol carrying an `@nonsendable` payload type.

Expected after implementation: actor protocol and handler parameters reject `@nonsendable` named or nested payload types before runtime while ordinary sendable payload structs remain accepted. Phase 3B adds the first managed source-to-runtime actor handler dispatch; async runtime bridging remains the next step.

- [x] **Step 3: Add managed actor addresses and bounded mailbox calls**

Make source-level actor state more tangible without overstating the async runtime bridge.

Run:

```sh
cargo test -p garnet-parser parses_spawn_keyword_as_member_method_name
cargo test -p garnet-interp c5_spawn_actor_returns_address_with_persistent_state
cargo test -p garnet-interp c5_actor_address_enforces_bounded_mailbox
cargo test -p garnet-interp c5_actor_address_tell_reports_full_mailbox
cargo test -p garnet-interp c5_actor_spawn_rejects_extra_capacity_args
```

Observed before implementation: `spawn Counter` evaluated to an actor type bridge only, so `counter.tell(:incr, 1)` failed with `actor Counter has no handler 'tell'`; `Counter.spawn(1)` also failed to parse because `spawn` was reserved after `.`.

Expected after implementation: managed actor addresses preserve actor-local `let` and `memory` state, `ask` dispatches immediately, `tell` enqueues or reports a full mailbox, `try_tell` returns false on backpressure, `drain()` processes queued messages and returns the drained count, and `Actor.spawn(capacity)` enforces bounded mailbox capacity. Full `garnet-actor-runtime` OS-thread execution remains the next actor milestone because managed `Value` is still `Rc`/`RefCell`-backed rather than `Send + 'static`.

- [x] **Step 4: Make the generated agent-orchestrator template use actor syntax**

Acceptance: `garnet new --template agent-orchestrator` emits a project whose
`src/main.garnet` declares Researcher / Synthesizer / Reviewer actors, uses
`spawn Actor`, `Actor.spawn(capacity)`, `ask`, `try_tell`, `mailbox_size`, and
`drain`, and whose generated tests pass through actor-local
episodic/semantic/procedural memory.

Evidence: `cargo test -p garnet-cli --test cli_smoke new_agent_orchestrator_template_runs_and_tests`; fresh `garnet new --template agent-orchestrator` smoke with `garnet run` returning `=> 25` and `garnet test` reporting 3 passed.

## Milestone 4: Safe-Mode Ownership Hardening

**Purpose:** Move safe mode from a useful skeleton toward conservative language law.

**Files:**

- Modify: `garnet-check-v0.3/src/borrow.rs`
- Modify: `garnet-check-v0.3/src/lib.rs`
- Test: `garnet-check-v0.3/tests/borrow.rs`
- Test: `garnet-cli/tests/conformance_skeleton.rs`

- [x] **Step 1: Activate the partial B1/B2/B4 borrow-rule suite**

Replace `partial_borrow_rule_suite` with concrete CLI-level cases for direct
use-after-move through `own` parameters, direct mutable aliasing, and managed
ARC behavior that must remain outside affine checking.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton partial_borrow_rule_suite
```

Remaining: full place-granular B1-B5, method-call ownership, B3 lifetime
containment, B5 drop discipline, and two-phase borrows are still deferred.

- [x] **Step 1B: Add unambiguous method receiver ownership**

Extend the partial borrow suite so same-module methods with an unambiguous
`self` ownership contract participate in direct move and alias checks, while
same-named methods with conflicting receiver signatures remain deferred until
type resolution exists.

Run:

```sh
cargo test -p garnet-check --test borrow
cargo test -p garnet-cli --test conformance_skeleton partial_borrow_rule_suite
```

Remaining: type-resolved impl dispatch and full place-granular receiver/field
borrows are still deferred.

- [x] **Step 1C: Disambiguate simple typed method receivers**

Use simple declared receiver types from safe parameters and annotated locals to
select the matching same-module impl method before falling back to unambiguous
method names for untyped receivers.

Run:

```sh
cargo test -p garnet-check --test borrow
cargo test -p garnet-cli --test conformance_skeleton partial_borrow_rule_suite
```

Remaining: generic receiver types, trait impl dispatch, inference for untyped
locals, and full place-granular receiver/field borrows are still deferred.

- [x] **Step 1D: Track simple field places for aliasing and moves**

Teach the borrow checker to treat `root.field` chains as places for direct
ownership and alias checks. The slice rejects same-field `mut`+`borrow`
aliasing, parent/child aliasing, and same-field use-after-move while preserving
valid sibling-field use.

Run:

```sh
cargo test -p garnet-check --test borrow
cargo test -p garnet-cli --test conformance_skeleton partial_borrow_rule_suite
```

Remaining: index/dynamic places, generic receiver types, trait impl dispatch,
inference for untyped locals, drop discipline, two-phase borrows, and NLL are
still deferred.

- [x] **Step 1E: Track indexed places conservatively**

Treat `root[index]` as a wildcard index sub-place for direct ownership and
alias checks. Indexes under the same receiver now conflict, matching the
Mini-Spec prefix rule for undecidable `i = j`, while indexes under distinct
sibling fields remain distinct. Nested index receiver operands are still
evaluated so moved index expressions cannot hide inside a recognized place.

Run:

```sh
cargo test -p garnet-check --test borrow
cargo test -p garnet-cli --test conformance_skeleton partial_borrow_rule_suite
```

Remaining: dynamic places, generic receiver types, trait impl dispatch,
inference for untyped locals, drop discipline, two-phase borrows, and NLL are
still deferred.

- [x] **Step 2: Implement a conservative NLL subset**

Implement the first conservative lifetime-elision gate before full CFG region
solving. Over-reject ambiguous reference returns; do not under-reject unsafe
cases.

Run:

```sh
cargo test -p garnet-check --test extended return_ref
cargo test -p garnet-cli --test conformance_skeleton deferred_nll_lifetime_inference
```

Evidence: Phase 4F implements the conservative Mini-Spec §8.5.2 lifetime
elision subset for reference returns. A safe function returning a reference
must tie the output to exactly one borrowed input lifetime, or to borrowed
`self`; no-input and multiple-borrowed-input reference returns reject until
explicit lifetime syntax and full region solving exist.

Remaining: full CFG region solving, closure capture lifetimes, variance,
dynamic places, generic receiver types, trait impl dispatch, drop discipline,
and two-phase borrows are still deferred.

- [x] **Step 2B: Reject same-call double-own drop hazards**

Implement the first B5 drop-discipline gate by rejecting calls where overlapping
places are passed to more than one `own` parameter in the same expression. This
prevents the checker from accepting an expression that would drop the same
binding, parent/child place, or conservative index family twice.

Run:

```sh
cargo test -p garnet-check --test borrow double_own
cargo test -p garnet-cli --test conformance_skeleton deferred_full_borrow_rule_suite
```

Evidence: Phase 4G rejects `consume_pair(b, b)` and `consume_pair(p, p.left)`
while allowing distinct sibling fields such as `consume_pair(p.left, p.right)`.

Remaining: full CFG region solving, closure capture lifetimes, variance,
dynamic places, generic receiver types, trait impl dispatch, broader drop
elaboration at scope/branch boundaries, and two-phase borrows are still
deferred.

- [x] **Step 2C: Add direct-returning branch liveness**

Implement the first CFG-liveness gate by checking each `if`/`elsif`/`else`
branch against the same pre-branch snapshot and only merging moves from branch
bodies that can continue past the `if`. Preserve moves that happen while
evaluating conditions.

Run:

```sh
cargo test -p garnet-check --test borrow returning
cargo test -p garnet-cli --test conformance_skeleton deferred_full_borrow_rule_suite
```

Evidence: Phase 4H allows a value moved inside a direct-returning branch to be
borrowed on later paths that still continue, while continuing branches still
merge moved state conservatively.

Remaining: full CFG region solving, nested/non-local terminators, loops,
closure capture lifetimes, variance, dynamic places, generic receiver types,
trait impl dispatch, broader drop elaboration, and two-phase borrows are still
deferred.

- [x] **Step 2D: Stop borrow scans at direct returns and returning loop bodies**

Extend the first CFG-liveness gate so direct `return` terminates scanning of
the current block and loop bodies that move then immediately return do not
poison later paths that only exist when the loop body does not execute.

Run:

```sh
cargo test -p garnet-check --test borrow return
cargo test -p garnet-cli --test conformance_skeleton deferred_full_borrow_rule_suite
```

Evidence: Phase 4I routes function bodies and `while`/`loop` bodies through the
same branch-outcome helper used by Phase 4H. Unreachable statements after a
direct `return` are not borrow-checked, and values moved in a direct-returning
loop body can still be borrowed after the loop on paths where the body never
runs.

Remaining: full CFG region solving, nested/non-local terminators, general loop
fixed-point analysis, for-loop fixed-point liveness, closure capture lifetimes,
variance, dynamic places, generic receiver types, trait impl dispatch, broader
drop elaboration, and two-phase borrows are still deferred.

- [x] **Step 2E: Scope `for` loop variables and returning for bodies**

Extend the direct-return loop-body liveness gate to `for` loops and prevent a
loop variable from rebinding an outer safe-mode binding after the loop body has
been checked.

Run:

```sh
cargo test -p garnet-check --test borrow for_
cargo test -p garnet-cli --test conformance_skeleton deferred_full_borrow_rule_suite
```

Evidence: Phase 4J checks `for` bodies against a loop-local environment where
the loop variable is rebound only for the body. Direct-returning `for` bodies
do not poison later non-executed paths, and a loop variable with the same name
as a moved outer binding no longer erases that outer moved state.

Remaining: full CFG region solving, nested/non-local terminators, general loop
fixed-point analysis, closure capture lifetimes, variance, dynamic places,
generic receiver types, trait impl dispatch, broader drop elaboration, and
two-phase borrows are still deferred.

- [x] **Step 2F: Scope `match` pattern bindings before arm move merging**

Prevent moves of match-arm pattern-local bindings from poisoning same-named
outer bindings after the `match`, while preserving diagnostics for real moves
of outer bindings performed inside arms.

Run:

```sh
cargo test -p garnet-check --test borrow match_
cargo test -p garnet-cli --test conformance_skeleton deferred_full_borrow_rule_suite
```

Evidence: Phase 4K records identifiers introduced by each match arm pattern,
checks the guard/body in an arm-local environment, and restores those names
from the pre-match snapshot before merging arm moves back into the outer
environment. A moved pattern-local `item` no longer causes a later `read(item)`
of an outer binding to fail, while `_ => consume(item)` still reports
use-after-move for a real outer move.

Remaining: full CFG region solving, nested/non-local terminators, general loop
fixed-point analysis, closure capture lifetimes, variance, dynamic places,
generic receiver types, trait impl dispatch, broader drop elaboration, and
two-phase borrows are still deferred.

- [x] **Step 2G: Preserve `match` arm block statements before arm tail values**

Keep every `match` arm body as a full block in the parser and downstream
walkers instead of reducing `{ stmt* tail }` arms to only the tail expression.
This makes managed execution, capability inventory, and safe-mode borrow
checking observe statements before the arm tail.

Run:

```sh
cargo test -p garnet-parser --test parse_control_flow parses_match_arm_block_with_statements_and_tail
cargo test -p garnet-interp --test eval_control match_arm_block_preserves_statements_before_tail
cargo test -p garnet-check --test borrow match_arm_block_statement_move_still_propagates_after_match
cargo test -p garnet-cli --test conformance_skeleton deferred_full_borrow_rule_suite
```

Evidence: Phase 4L changes `MatchArm.body` to `Block`, wraps expression arms
as one-tail blocks, evaluates matched arm blocks with normal block semantics,
walks match-arm blocks for capability/safe-mode inventory, and routes
safe-mode arm bodies through branch-block checking. A `let` before the tail now
affects a matched arm result, a move statement inside a match-arm block now
propagates to later use-after-move diagnostics, and moves in guards are still
merged when a guard can fail before a returning arm body runs.

Remaining: full CFG region solving, nested/non-local terminators, general loop
fixed-point analysis, closure capture lifetimes, variance, dynamic places,
generic receiver types, trait impl dispatch, broader drop elaboration, and
two-phase borrows are still deferred.

- [x] **Step 2H: Add finite-domain match exhaustiveness and reachability**

Reject safe-mode `match` expressions over finite domains when they omit a
`Bool` case or same-module enum variant. Treat guarded arms as non-exhaustive
coverage, and reject duplicate covered arms plus arms after an unguarded
catch-all.

Run:

```sh
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4M adds a dedicated `match_coverage` checker pass that uses
function parameter and local type annotations to identify `Bool` and
same-module enum subjects. It rejects a missing `false` arm, a missing enum
variant, a guarded enum arm that would otherwise hide missing coverage,
duplicate unguarded variant arms, and arms made unreachable by an unguarded
catch-all, while preserving complete enum matches.

- [x] **Step 2I: Add finite nested-constructor match coverage**

Reject safe-mode `match` expressions over finite nested constructor payloads
when they omit a nested finite case, while allowing wildcard payload patterns
to cover that nested finite domain.

Run:

```sh
cargo test -p garnet-check --test match_coverage safe_nested_enum_match
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4N enumerates finite nested constructor payload products in the
same scoped `match_coverage` pass. `Outer::Wrap(Inner::Left)` and
`Outer::Wrap(Inner::Right)` are tracked as distinct coverage cases; missing
nested payload cases are reported; and `Outer::Wrap(_)` covers the nested
finite payload domain without claiming imported enum, recursive/open payload,
or guard-proof completeness.

- [x] **Step 2J: Add imported enum alias match coverage**

Resolve named, glob, module-qualified, and module-relative enum imports for the
same scoped safe-mode match coverage pass.

Run:

```sh
cargo test -p garnet-check --test match_coverage safe_imported_
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4O adds a per-module import scope for `match_coverage`. Named
imports such as `use Types::{Status}` and glob imports such as `use Types::*`
resolve `Status` to `Types::Status`, including when `Types` is relative to the
current module, and pattern coverage accepts the source alias prefix
(`Status::Ready`) as coverage for the canonical `Types::Status::Ready` finite
case. This avoids the previous global short-name fallback and keeps ambiguous
or cross-file imports outside the hard-error gate.

- [x] **Step 2K: Add literal guard match coverage reasoning**

Treat literal `if true` and `if false` match guards as decidable in the
safe-mode match coverage pass while keeping non-literal guards conservative.

Run:

```sh
cargo test -p garnet-check --test match_coverage guard
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4P counts `Status::Ready if true` as coverage for the
`Status::Ready` finite-domain case, rejects `Status::Ready if false` as an
unreachable match arm with a statically false guard, and still reports the
false-guarded variant as missing coverage. Dynamic/non-literal guards remain
non-covering because the checker does not yet prove arbitrary guard predicates.

Remaining: cross-file/package imports, recursive/open payload reasoning,
open-domain exhaustiveness/range reasoning, and non-literal guard reasoning are still
deferred.

- [x] **Step 2L: Add open-domain literal match reachability**

Treat duplicate literal arms and arms after catch-all patterns as unreachable
in safe-mode matches even when the subject type is not a finite `Bool` or enum
domain. Keep unknown guarded literal arms non-covering so later unguarded
literal arms remain reachable.

Run:

```sh
cargo test -p garnet-check --test match_coverage safe_open_
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4Q rejects repeated open-domain literals such as two `1`
arms, rejects literal arms after `_`, and preserves conservative behavior for
`1 if ok` because a non-literal guard can fail.

Remaining: cross-file/package imports, recursive/open payload reasoning,
open-domain exhaustiveness/range reasoning, and non-literal guard reasoning are
still deferred.

- [x] **Step 2M: Infer immutable local finite match domains from initializers**

Use immutable local boolean literal initializers and enum variant
constructor/path initializers to seed the safe-mode match coverage environment
when a local binding does not carry an explicit type annotation.

Run:

```sh
cargo test -p garnet-check --test match_coverage safe_match_uses_local_
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4R rejects non-exhaustive matches over `let flag = true` and
`let status = Status::Ready()` locals, while preserving the existing explicit
type-annotation evidence.

Remaining: direct mutable-local assignment tracking is covered in Step 2N
below; cross-file/package imports, recursive/open payload reasoning, broader
expression/type inference, open-domain exhaustiveness/range reasoning, and
non-literal guard reasoning are still deferred.

- [x] **Step 2N: Track direct mutable-local match-domain assignments**

Use direct `let mut` assignment flow to seed or clear the safe-mode match
coverage environment. Finite boolean/enum assignments seed the subject domain;
non-finite assignments clear inferred finite-domain state so open-domain
matches do not receive false exhaustiveness errors.

Run:

```sh
cargo test -p garnet-check --test match_coverage mutable_
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4S rejects non-exhaustive matches after finite mutable
assignment (`flag = true`), keeps mutable enum initializers finite, and stops
reporting finite-domain missing cases after `flag = 1` invalidates the inferred
domain.

Remaining: direct `if`/`elsif`/`else` branch-merged assignment flow is covered
in Step 2O below; compound-assignment invalidation is covered in Step 2Q
below; cross-file/package
imports, recursive/open payload reasoning, broader expression/type inference,
open-domain exhaustiveness/range reasoning, and non-literal guard reasoning are
still deferred.

- [x] **Step 2O: Join branch-local match-domain assignments**

Use conservative `if`/`elsif`/`else` branch joins to carry direct mutable-local
match-domain evidence forward only when every possible branch preserves the
same finite domain. Mixed finite/non-finite branches and missing-else paths
clear inferred domains rather than reporting false finite-domain
exhaustiveness errors.

Run:

```sh
cargo test -p garnet-check --test match_coverage if_else_assignments
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4T rejects non-exhaustive matches after both branches assign a
finite Bool or enum domain, and it accepts an open-domain match after one
branch assigns a non-finite value.

Remaining: nested all-path `if` branch assignment flow is covered in Step 2P
below; compound-assignment invalidation is covered in Step 2Q below;
loop-body invalidation is covered in Step 2R below; `try`/`ensure`
invalidation and uninvoked closure-definition boundaries are covered in Step
2S below; direct closure-literal invocation invalidation is covered in Step
2T below; direct local closure-literal binding call invalidation is covered in
Step 2U below; branch-joined local closure-literal binding call invalidation
is covered in Step 2V below; branch-rebound local closure-literal binding call
invalidation is covered in Step 2W below; direct local closure-alias binding
call invalidation is covered in Step 2X below; branch-joined local closure-alias
call invalidation is covered in Step 2Y below; direct branch-selected closure
expression call invalidation is covered in Step 2Z below; loop fixed-point and broader
mutable/escaped/general higher-order closure invocation/call-effect
flow, cross-file/package imports, recursive/open payload reasoning, broader
expression/type inference, open-domain exhaustiveness/range reasoning, and
non-literal guard reasoning are still deferred.

- [x] **Step 2P: Join nested if assignment domains inside branch bodies**

Extend the branch-join eligibility proof from only direct branch-body
assignments to nested `if` / `elsif` / `else` expressions when every nested
path definitely assigns the same outer match subject. Missing nested `else`
paths remain open-domain and branch-local bindings remain ineligible.

Run:

```sh
cargo test -p garnet-check --test match_coverage nested_if_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4U rejects a non-exhaustive match after a nested `if/else`
inside one outer branch assigns `true`/`false` on every nested path, while it
accepts the missing-nested-else variant as open-domain.

- [x] **Step 2Q: Make compound assignments an explicit invalidation boundary**

Document and test that `+=`, `-=`, `*=`, `/=`, and `%=`-style assignments do
not preserve finite match-domain evidence. Direct and all-branch compound
assignments clear stale `Bool`/enum domains before later matches because their
result depends on operator and type semantics outside the finite-domain proof.

Run:

```sh
cargo test -p garnet-check --test match_coverage compound_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4V accepts matches after direct compound assignment and after
compound assignments in every `if`/`else` branch without reporting stale
finite-domain non-exhaustiveness diagnostics.

- [x] **Step 2R: Invalidate domains after possible loop-body assignments**

Loops may execute and assign through only some iterations or nested branches.
Conservatively clear finite match-domain evidence for outer bindings assigned
inside `while`, `for`, or `loop` bodies instead of preserving stale pre-loop
domains. Loop-local bindings remain excluded so shadowing does not erase the
outer domain, while assignments before a later loop-local shadow still clear
the outer finite-domain evidence.

Run:

```sh
cargo test -p garnet-check --test match_coverage loop_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4W accepts matches after possible `while`/`for` body
assignments and after conditional assignments inside loop bodies without
reporting stale finite-domain diagnostics, while ordered shadowing tests prove
that loop-local declarations neither erase the outer domain nor hide earlier
outer assignments in the same loop body.

- [x] **Step 2S: Invalidate try-flow domains and isolate closure definitions**

`try` bodies, `rescue` handlers, and `ensure` blocks can write through paths
that safe-mode rejects separately from match coverage. Conservatively clear
outer finite match-domain evidence for possible writes in those blocks so the
checker does not report stale non-exhaustiveness after an invalidated value.
Uninvoked closure literals are also a boundary: defining a closure must not
merge its body assignments into the enclosing statement flow before any call
effect analysis exists.

Run:

```sh
cargo test -p garnet-check --test match_coverage try_
cargo test -p garnet-check --test match_coverage uninvoked_closure
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4X removes stale finite-domain diagnostics after
`try`/`rescue`/`ensure` writes while preserving the existing safe-mode
`try`/`rescue` rejection, and accepts matches after an uninvoked closure
definition whose body would otherwise assign a finite `Bool` domain.

- [x] **Step 2T: Invalidate direct closure-literal invocation domains**

Direct invocation of a closure literal runs the closure body immediately, but
the checker still does not attempt a general stored-closure call-effect model.
Conservatively clear finite match-domain evidence for outer bindings assigned
inside directly invoked closure literals, including block-body and expression
body closures, while keeping uninvoked closure definitions isolated.

Run:

```sh
cargo test -p garnet-check --test match_coverage immediate_closure
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4Y accepts matches after direct closure-literal calls whose
bodies assign the match subject without reporting stale finite-domain
non-exhaustiveness diagnostics. Broader stored closure invocation/call-effect
analysis remains deferred.

- [x] **Step 2U: Invalidate direct local closure-literal binding calls**

Local closure literals bound in the current block and called directly run their
closure body at the call site. Track the conservative set of outer bindings
that such a local closure may assign, and clear finite match-domain evidence
when the local binding is invoked. Keep branch-joined and branch-rebound closure handling covered by later steps;
escaped closure, general higher-order call-effect analysis, and broader mutable closure
flow remain deferred.

Run:

```sh
cargo test -p garnet-check --test match_coverage local_closure_call_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4Z accepts matches after directly calling a local closure
literal binding whose body assigns the match subject without reporting stale
finite-domain non-exhaustiveness diagnostics.

- [x] **Step 2V: Invalidate branch-joined local closure binding calls**

When an `if` / `elsif` / `else` expression assigned to a local binding returns
closure literals from every branch, conservatively join the possible outer
writes from those closure bodies. A direct call to that local binding clears
finite match-domain evidence for the joined write set. Branch rebinding is covered by the next step; escaped closures, higher-order
calls, and broader mutable closure flow remain deferred.

Run:

```sh
cargo test -p garnet-check --test match_coverage branch_joined_local_closure_call_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4AA accepts matches after directly calling a local closure
binding produced by an all-branch `if` expression whose closure bodies can
assign the match subject, without reporting stale finite-domain diagnostics.

- [x] **Step 2W: Invalidate branch-rebound local closure binding calls**

When every branch of an `if` / `elsif` / `else` flow leaves a local closure
binding with a known closure-literal effect, conservatively join those possible
outer writes. A later direct call to that rebound local binding clears finite
match-domain evidence for the joined write set. Escaped closures, higher-order
calls, and broader mutable closure flow remain deferred.

Run:

```sh
cargo test -p garnet-check --test match_coverage branch_rebound_local_closure_call_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4AB accepts matches after all branches rebind a local closure
binding to closure literals that can assign the match subject, without
reporting stale finite-domain diagnostics after the direct local call.

- [x] **Step 2X: Invalidate direct local closure-alias calls**

When a local binding aliases another local binding with a known closure-literal
effect, copy that conservative outer-write set to the alias. A later direct
call to the alias clears finite match-domain evidence for the copied write set.
Escaped closures, general higher-order calls, and broader mutable closure flow
remain deferred.

Run:

```sh
cargo test -p garnet-check --test match_coverage local_closure_alias_call_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4AC accepts matches after directly calling a local alias of a
known local closure-literal binding whose body can assign the match subject,
without reporting stale finite-domain diagnostics after the alias call.

- [x] **Step 2Y: Invalidate branch-joined local closure-alias calls**

When a local binding is assigned from an all-path branch expression whose tails
are direct aliases of known local closure bindings, copy the union of those
conservative outer-write sets to the alias. A later direct call to the alias
clears finite match-domain evidence for the copied write set. Branch-local
shadowing before the tail keeps the alias unknown.

Run:

```sh
cargo test -p garnet-check --test match_coverage branch_joined_local_closure_alias_call_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4AD accepts matches after directly calling a branch-selected
local alias of known local closure-literal bindings whose bodies can assign the
match subject, without reporting stale finite-domain diagnostics after the
alias call, while preserving diagnostics for shadowed unknown branch tails.

- [x] **Step 2Z: Invalidate direct branch-selected closure expression calls**

When a call callee is itself an all-path branch expression whose tails resolve
to known local closure bindings, reuse the same conservative closure-effect
extraction as local aliases. A direct call to that branch-selected expression
clears finite match-domain evidence for the joined write set, while
branch-local shadowing before a tail keeps the callee unknown.

Run:

```sh
cargo test -p garnet-check --test match_coverage direct_branch
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4AE accepts matches after directly calling an all-path
branch-selected closure expression whose closure bodies can assign the match
subject, without reporting stale finite-domain diagnostics, while preserving
diagnostics for shadowed unknown branch tails.

- [x] **Step 2ZA: Recognize immutable local boolean guard constants**

Track immutable local boolean constants in a separate guard-fact map. This lets
safe-mode match coverage treat `let always = true` guards as coverage and
`let never = false` guards as statically false/non-covering without broadening
into general expression evaluation. Mutable guard locals stay unknown.

Run:

```sh
cargo test -p garnet-check --test match_coverage bool_guard
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4AF accepts matches covered by immutable local true guards,
rejects immutable local false guards as statically unreachable/non-covering,
and keeps mutable boolean guards conservative.

- [x] **Step 2ZB: Recognize same-module top-level boolean guard constants**

Seed match-guard facts from same-module top-level boolean `const` items. This
lets safe-mode match coverage treat `const ALWAYS = true` guards as coverage
and `const NEVER = false` guards as statically false/non-covering without
const aliases, arithmetic, comparison, function-call, broader const expression
evaluation, or general expression evaluation. Function parameters with the same
name shadow the module const fact.

Run:

```sh
cargo test -p garnet-check --test match_coverage const_bool_guard
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AG accepts matches covered by same-module true const guards,
rejects same-module false const guards as statically unreachable/non-covering,
and keeps parameter-shadowed const guard names conservative.

- [x] **Step 2ZC: Recognize imported top-level boolean guard constants**

Resolve scoped named and glob imports of top-level boolean `const` facts into the
match-guard fact map. This lets safe-mode match coverage treat `use
Flags::{ALWAYS}` and `use Flags::*` boolean guards the same way as local
top-level constants while preserving parameter/local/pattern shadowing.

Run:

```sh
cargo test -p garnet-check --test match_coverage imported
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AH accepts matches covered by named-imported and
module-relative imported true const guards, rejects glob-imported false const
guards as statically unreachable/non-covering, and keeps parameter-shadowed
imported const guard names conservative.

- [x] **Step 2ZD: Recognize path-qualified boolean guard constants**

Resolve path-qualified top-level boolean `const` facts in match guards through
the same scoped const-fact index. This lets safe-mode match coverage treat
`Flags::ALWAYS` guards as coverage and `Flags::NEVER` guards as statically
false/non-covering, while keeping ambiguous paths and broad const expression
evaluation conservative.

Run:

```sh
cargo test -p garnet-check --test match_coverage path_qualified
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AI accepts matches covered by path-qualified true const
guards, rejects path-qualified false const guards as statically
unreachable/non-covering, and keeps arithmetic, comparison, function-call, and
broader const expression evaluation deferred.

- [x] **Step 2ZE: Resolve narrow boolean const guard aliases**

Resolve direct boolean const aliases through the scoped const-fact index without
general const expression evaluation. This lets safe-mode match coverage treat
path-valued aliases such as `Flags::ALWAYS = Core::RAW` as coverage when they
resolve to `true` and statically false/non-covering when they resolve to
`false`.

Run:

```sh
cargo test -p garnet-check --test match_coverage const_bool_alias
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AJ accepts matches covered by path-qualified true const
aliases, rejects path-qualified false const aliases as statically
unreachable/non-covering, and leaves arithmetic, comparison, function-call, and
broader const expression evaluation deferred.

- [x] **Step 2ZF: Fold basic boolean const guard expressions**

Fold basic boolean `not`, `and`, and `or` const expressions over
already-resolved boolean facts. This lets safe-mode match coverage treat
`Core::RAW and not false` as coverage when it resolves to `true`, and
`not Core::RAW or false` as statically false/non-covering when it resolves to
`false`, without general const evaluation.

Run:

```sh
cargo test -p garnet-check --test match_coverage boolean_const_expression
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AK accepts matches covered by true boolean const
expressions, rejects false boolean const expressions as statically
unreachable/non-covering, and leaves arithmetic, comparison, function-call,
recursive, and broader const expression evaluation deferred.

- [x] **Step 2ZG: Honor short-circuit boolean const guard expressions**

Honor decisive left operands for boolean `or` and `and` const expressions
without requiring the right operand to resolve. This lets safe-mode match
coverage treat `true or Missing::VALUE` as coverage and `false and
Missing::VALUE` as statically false/non-covering while still deferring general
const evaluation.

Run:

```sh
cargo test -p garnet-check --test match_coverage short_circuit
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AL accepts matches covered by true short-circuit boolean
const expressions, rejects false short-circuit boolean const expressions as
statically unreachable/non-covering, and leaves arithmetic, comparison,
function-call, recursive, cross-file/package, and broader const expression
evaluation deferred.

- [x] **Step 2ZH: Fold boolean const equality/inequality guard expressions**

Fold boolean `==` and `!=` const expressions over already-resolved boolean
facts. This lets safe-mode match coverage treat `Core::RAW == true` as
coverage and `Core::RAW != true` as statically false/non-covering while still
deferring arithmetic, relational comparison, function-call, recursive, and
cross-file/package const evaluation.

Run:

```sh
cargo test -p garnet-check --test match_coverage boolean_const_equality
cargo test -p garnet-check --test match_coverage boolean_const_inequality
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AM accepts matches covered by true boolean equality const
expressions, rejects false boolean inequality const expressions as statically
unreachable/non-covering, and leaves arithmetic, relational comparison,
function-call, recursive, cross-file/package, and broader const expression
evaluation deferred.

- [x] **Step 2ZI: Fold direct boolean match guard expressions**

Apply the same conservative boolean fact folding directly to match guard
expressions. This lets safe-mode match coverage treat
`Status::Ready if Core::RAW == true` as coverage and
`Status::Ready if Core::RAW != true` as statically false/non-covering without
requiring an intermediate alias const.

Run:

```sh
cargo test -p garnet-check --test match_coverage direct_true_boolean_const_equality
cargo test -p garnet-check --test match_coverage direct_false_boolean_const_inequality
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AN accepts direct match guards covered by true boolean const
equality expressions, rejects direct false boolean const inequality guards as
statically unreachable/non-covering, and leaves arithmetic, relational
comparison, function-call, recursive, cross-file/package, and broader const
expression evaluation deferred.

- [x] **Step 2ZJ: Fold integer const equality/inequality guard facts**

Extend the narrow guard fact domain from booleans to integer literals for
`==`/`!=` comparisons. This lets safe-mode match coverage treat
`Core::LIMIT == 2` as coverage and `Core::LIMIT != 2` as statically
false/non-covering while still deferring arithmetic, relational comparison,
function-call, recursive, and cross-file/package const evaluation.

Run:

```sh
cargo test -p garnet-check --test match_coverage integer_const_equality
cargo test -p garnet-check --test match_coverage false_integer_const_inequality
cargo test -p garnet-check --test match_coverage direct_integer_const_equality
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AO accepts alias and direct match guards covered by true
integer const equality expressions, rejects false integer const inequality
guards as statically unreachable/non-covering, and leaves arithmetic,
relational comparison, function-call, recursive, cross-file/package, and
broader const expression evaluation deferred.

- [x] **Step 2ZK: Fold integer const relational guard facts**

Extend the same narrow integer guard fact domain from equality/inequality to
literal integer `<`, `<=`, `>`, and `>=` comparisons. This lets safe-mode
match coverage treat `Core::LIMIT < 3` and `Core::LIMIT >= 2` as coverage and
`Core::LIMIT > 3` as statically false/non-covering while still deferring
arithmetic, broader non-numeric comparison, broader float edge-case reasoning, function-call, recursive, and
cross-file/package const evaluation.

Run:

```sh
cargo test -p garnet-check --test match_coverage integer_const_less_than
cargo test -p garnet-check --test match_coverage false_integer_const_greater_than
cargo test -p garnet-check --test match_coverage direct_integer_const_greater_equal
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AP accepts alias and direct match guards covered by true
integer const relational expressions, rejects false integer const relational
guards as statically unreachable/non-covering, and leaves arithmetic,
broader non-numeric comparison, broader float edge-case reasoning, function-call, recursive, cross-file/package,
and broader const expression evaluation deferred.

- [x] **Step 2ZL: Fold checked integer arithmetic guard facts**

Extend the same narrow integer guard fact domain from comparisons to checked
literal integer arithmetic (`+`, `-`, `*`, `/`, `%`, unary `-`) inside const
guard expressions. This lets safe-mode match coverage treat
`Core::LIMIT + Core::OFFSET == 3` and `Core::LIMIT + 1 >= 3` as coverage and
`Core::LIMIT * 2 < 4` as statically false/non-covering while still deferring
broader non-numeric comparison, broader float edge-case reasoning, function-call, recursive, cross-file/package,
and broader const expression evaluation.

Run:

```sh
cargo test -p garnet-check --test match_coverage integer_const_arithmetic
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AQ accepts alias and direct match guards covered by checked
integer arithmetic plus comparison/equality expressions, rejects false checked
integer arithmetic guards as statically unreachable/non-covering, and leaves
broader non-numeric comparison, broader float edge-case reasoning, function-call, recursive, cross-file/package,
and broader const expression evaluation deferred.

- [x] **Step 2ZM: Carry integer const facts through scoped guard identifiers**

Extend the scoped guard-fact environment from boolean-only facts to the existing
`ConstFact` domain so same-module bare integer `const` names and scoped
named/glob imported integer `const` names can feed the checked arithmetic and
comparison logic already used by path-qualified guards. This lets safe-mode
match coverage treat `LIMIT + OFFSET == 3` and imported `use Core::{LIMIT,
OFFSET}` guard expressions as coverage while preserving function-parameter
shadowing and keeping cross-file/package imports, non-integer comparison,
function-call evaluation, recursion, and broader const expression evaluation
deferred.

Run:

```sh
cargo test -p garnet-check --test match_coverage integer_const_identifiers
cargo test -p garnet-check --test match_coverage imported_glob_integer_const
cargo test -p garnet-check --test match_coverage function_parameter_shadows_imported_integer_const
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AR accepts same-module and named-imported integer const
identifier guards covered by checked arithmetic, rejects glob-imported false
integer identifier guards as statically unreachable/non-covering, and keeps
parameter-shadowed imported integer identifiers unknown/non-covering.

- [x] **Step 2ZN: Fold literal symbol and string const equality facts**

Extend the narrow `ConstFact` domain to static symbols and plain
non-interpolated strings for equality and inequality guard checks. This lets
safe-mode match coverage treat `Core::MODE == :ready` as coverage and
`Core::LABEL != "ready"` as statically false/non-covering while keeping
interpolated strings, ordering comparisons, function calls, cross-file/package
imports, recursion, and broader const expression evaluation deferred.

Run:

```sh
cargo test -p garnet-check --test match_coverage symbol_const_equality
cargo test -p garnet-check --test match_coverage false_string_const_inequality
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AS accepts symbol const equality guards, rejects false plain
string const inequality guards as statically unreachable/non-covering, and
keeps interpolated string facts conservative.

- [x] **Step 2ZO: Fold nil const equality facts**

Extend the narrow `ConstFact` equality/inequality domain to `nil`. This lets
safe-mode match coverage treat `Core::EMPTY == nil` as coverage and
`Core::EMPTY != nil` as statically false/non-covering while keeping broader
const expression evaluation deferred.

Run:

```sh
cargo test -p garnet-check --test match_coverage nil_const_equality
cargo test -p garnet-check --test match_coverage false_nil_const_inequality
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AT accepts true nil const equality guards and rejects false
nil const inequality guards as statically unreachable/non-covering.

- [x] **Step 2ZP: Fold mixed literal const equality facts**

Apply Garnet's existing runtime equality rule for distinct known literal kinds
inside the narrow `ConstFact` domain. This lets safe-mode match coverage treat
`Core::EMPTY != false` as coverage and `Core::EMPTY == false` as statically
false/non-covering while keeping float arithmetic, non-finite floats, call-backed/dynamic interpolated strings, ordering
comparisons, function calls, cross-file/package imports, recursion, and broader
const expression evaluation deferred.

Run:

```sh
cargo test -p garnet-check --test match_coverage mixed_literal_const_inequality
cargo test -p garnet-check --test match_coverage false_mixed_literal_const_equality
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AU accepts true mixed-literal const inequality guards and
rejects false mixed-literal const equality guards as statically
unreachable/non-covering.

- [x] **Step 2ZQ: Fold finite float const equality facts**

Extend the narrow `ConstFact` domain to finite `Float` literals and align
literal equality with Garnet runtime equality for `Float == Float` and
`Int == Float`. This lets safe-mode match coverage treat `Core::RATIO == 1.5`
and `Core::COUNT == 1.0` as coverage, while false float inequalities become
statically false/non-covering. Non-finite float facts, float arithmetic,
ordering comparisons, interpolated strings, function calls, cross-file/package
imports, recursion, and broader const expression evaluation remain deferred.

Run:

```sh
cargo test -p garnet-check --test match_coverage float_const
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AV accepts true finite-float and int-float const equality
guards, rejects false finite-float inequality guards as statically
unreachable/non-covering, and keeps non-finite float facts unknown/non-covering.

- [x] **Step 2ZR: Fold finite float const relational facts**

Align the narrow `ConstFact` relational comparison rule with Garnet runtime
numeric comparison for finite `Float`/`Float`, `Int`/`Float`, and `Float`/`Int`
pairs. This lets safe-mode match coverage treat `Core::RATIO < 2.0` and
`Core::COUNT <= 2.0` as coverage while `Core::RATIO > 2.0` becomes statically
false/non-covering. Non-finite float facts, float arithmetic, interpolated
strings, function calls, cross-file/package imports, recursion, broader
non-numeric comparison, and broader const expression evaluation remain
deferred.

Run:

```sh
cargo test -p garnet-check --test match_coverage float_const_relational
cargo test -p garnet-check --test match_coverage non_finite_float_relational
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AW accepts true finite-float and int-float const relational
guards, rejects false finite-float relational guards as statically
unreachable/non-covering, and keeps non-finite float facts unknown/non-covering.

- [x] **Step 2ZS: Fold finite float const arithmetic facts**

Align the narrow `ConstFact` arithmetic rule with checked integer arithmetic
plus Garnet runtime numeric arithmetic for finite `Float`/`Float`,
`Int`/`Float`, and `Float`/`Int` pairs. This lets safe-mode match coverage
treat `Core::RATIO + 0.5 == 2.0` and `Core::COUNT * 1.5 >= 3.0` as coverage
while `Core::RATIO * 2.0 < 3.0` becomes statically false/non-covering.
Overflow-to-infinity, non-finite float facts, interpolated strings, function
calls, cross-file/package imports, recursion, broader non-numeric comparison,
broader float edge-case reasoning, and broader const expression evaluation
remain deferred.

Run:

```sh
cargo test -p garnet-check --test match_coverage float_const_arithmetic
cargo test -p garnet-check --test match_coverage non_finite_float_arithmetic
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AX accepts true finite-float and int-float const arithmetic
guards, rejects false finite-float arithmetic guards as statically
unreachable/non-covering, and keeps overflow-to-infinity facts unknown.

- [x] **Step 2ZT: Fold immutable local guard expression aliases**

Carry the existing narrow `ConstFact` evaluator through immutable local guard
bindings so a local alias can be more than a literal or direct identifier.
This lets `let always = limit + 1 == 3` count as match coverage when `limit`
is an immutable local integer fact, while `let never = limit + 1 < 3` is
statically false/non-covering. Mutable local expression sources, path-qualified
local alias expressions, function calls, cross-file/package imports, recursion,
broader non-numeric comparison, and broader const expression evaluation remain
deferred.

Run:

```sh
cargo test -p garnet-check --test match_coverage local_boolean_const_expression
cargo test -p garnet-check --test match_coverage local_integer_const_expression
cargo test -p garnet-check --test match_coverage mutable_local_expression_source
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AY accepts immutable local boolean and integer expression
aliases as safe-mode guard facts, rejects false local integer expression guards
as statically unreachable/non-covering, and keeps mutable local expression
sources unknown/non-covering.

- [x] **Step 2ZU: Resolve path-qualified consts in local guard expression aliases**

Carry scoped path-qualified constant resolution into immutable local guard
expression aliases. This lets `let always = Core::LIMIT + 1 == 3` count as
coverage and `let never = Core::LIMIT + 1 < 3` become statically
false/non-covering while preserving the Phase 4AY mutable-local invalidation
boundary. Function calls, cross-file/package imports, recursion, broader
non-numeric comparison, and broader const expression evaluation remain
deferred.

Run:

```sh
cargo test -p garnet-check --test match_coverage local_path_integer_const_expression
cargo test -p garnet-check --test match_coverage local_
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AZ accepts true path-qualified local integer expression
aliases as safe-mode guard facts and rejects false path-qualified local integer
expression guards as statically unreachable/non-covering.

- [x] **Step 2ZV: Fold static interpolated string const facts**

Extend the narrow `ConstFact` string domain to interpolated strings when every
interpolation body resolves through the same conservative fact evaluator. This
lets `"re#{"ad"}y" == "ready"` count as safe-mode match coverage and
`"re#{"ad"}y" != "ready"` become statically false/non-covering while preserving
the boundary for function-call backed interpolation, recursion,
cross-file/package imports, broader non-numeric comparison, and broader const
expression evaluation.

Run:

```sh
cargo test -p garnet-check --test match_coverage interpolated_string
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4BA accepts true static interpolated string equality guards,
rejects false static interpolated string inequality guards as statically
unreachable/non-covering, and keeps call-backed interpolated string facts
unknown/non-covering.

- [x] **Step 2ZW: Fold static string relational const facts**

Extend the runtime-aligned ordered fact domain to static string-to-string
relational comparisons. This lets `Core::LABEL < "rust"` count as safe-mode
match coverage and `Core::LABEL > "ready"` become statically false/non-covering
while preserving the boundary for mixed string/symbol relational comparisons,
function calls, cross-file/package imports, recursion, broader non-string
non-numeric comparison, and broader const expression evaluation.

Run:

```sh
cargo test -p garnet-check --test match_coverage string_const_relational
cargo test -p garnet-check --test match_coverage mixed_string_symbol_relational
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4BB accepts true static string relational guards, rejects false
static string relational guards as statically unreachable/non-covering, and
keeps mixed string/symbol relational facts unknown/non-covering.

- [x] **Step 2ZX: Fold static boolean relational const facts**

Extend the runtime-aligned ordered fact domain to static boolean-to-boolean
relational comparisons. This lets `Core::RAW < true` follow the managed
runtime's `false < true` ordering and count as safe-mode match coverage while
`Core::RAW < false` becomes statically false/non-covering. Mixed boolean/nil,
symbol, and other non-runtime-comparable relational facts stay unknown.

Run:

```sh
cargo test -p garnet-check --test match_coverage boolean_const_relational
cargo test -p garnet-check --test match_coverage mixed_boolean_nil_relational
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4BC accepts true static boolean relational guards, rejects false
static boolean relational guards as statically unreachable/non-covering, and
keeps mixed boolean/nil relational facts unknown/non-covering.

- [x] **Step 2ZY: Fold runtime-aligned nil relational const facts**

Extend the runtime-aligned ordered fact domain to static `nil`-to-`nil`
relational comparisons. The managed runtime aligns `nil <=> nil` to `Equal`
(`Value::partial_compare`), so `Core::EMPTY <= nil` and `Core::EMPTY >= nil`
count as safe-mode match coverage while `Core::EMPTY < nil` and
`Core::EMPTY > nil` become statically false/non-covering. Mixed nil/integer,
nil/symbol, and other non-runtime-comparable relational facts stay unknown
because the runtime raises a type error on those pairs rather than yielding a
boolean.

Run:

```sh
cargo test -p garnet-check --test match_coverage nil_const_relational
cargo test -p garnet-check --test match_coverage false_nil_const_relational
cargo test -p garnet-check --test match_coverage mixed_nil_int_relational
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4BI accepts true static nil relational guards, rejects false
static nil relational guards as statically unreachable/non-covering, and keeps
mixed nil/integer relational facts unknown/non-covering.

## Milestone 5: Traits, Coherence, And Generic Instantiation

**Purpose:** Make the Rust-rigor side credible without claiming native zero-cost compilation.

**Files:**

- Modify: `garnet-check-v0.3/src/lib.rs`
- Create: `garnet-check-v0.3/src/coherence.rs`
- Create: `garnet-check-v0.3/src/generics.rs`
- Test: `garnet-cli/tests/conformance_skeleton.rs`

- [x] **Step 1: Add a conservative orphan-rule coherence checker**

Reject conflicting trait impls and impls where neither the trait nor the type is local.

Run:

```sh
cargo test -p garnet-check --test coherence
cargo test -p garnet-cli --test conformance_skeleton deferred_trait_coherence
```

Evidence: Phase 5C rejects exact duplicate trait impls, orphan impls where
neither the trait nor the type is local, simple generic blanket-vs-concrete
overlaps, renamed generic blanket overlaps, and qualified external type
short-name collisions. It preserves the Rust-compatible positive cases where
either the trait or the type is defined locally, plus qualified local-module
type impls.

Remaining: specialization, imported-package coherence, and native
monomorphization remain deferred.

- [x] **Step 2: Add interpreter-level generic instantiation evidence**

Treat generic instantiation as runtime/interpreter evidence only. Do not claim monomorphized zero-cost behavior until a compiler backend exists.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton generic_instantiation_runs_without_monomorphization_claims
```

Evidence: Phase 5B runs a generic `Box<T>` struct, a generic `impl<T>
Box<T>` method, and a generic `identity<T>` function through `garnet parse`,
`garnet check`, and `garnet run`, returning `=> 43`.

- [x] **Step 3: Defer native zero-cost claims until a compiler backend exists**

Native Monomorphization and the zero-cost theorem remain future work. Phase 5B
claims only interpreter-level generic instantiation evidence.

## Milestone 6: Memory Core Productization

**Purpose:** Move Mnemos from reference stores toward the Memory Core and ARC/cycle ambitions in the Mini-Spec.

**Files:**

- Modify: `garnet-memory-v0.3/src/lib.rs`
- Create: `garnet-memory-v0.3/src/cycle.rs`
- Modify: `C_Language_Specification/MEMORY_CORE_ROADMAP.md`
- Test: `garnet-cli/tests/conformance_skeleton.rs`

- [x] **Step 1: Add observable cycle fixtures**

Evidence: Phase 6A adds a memory fixture with retained roots, an unrooted
collectable cycle, an unrooted acyclic component that remains available for
ordinary eviction, a self-cycle, and a kind-scheduled cross-kind cycle.

Run:

```sh
cargo test -p garnet-memory --test cycle
cargo test -p garnet-cli --test conformance_skeleton deferred_arc_cycle_detection
```

Observed before implementation: failed because `CycleGraph`, `CycleNodeId`, and
`CycleScan` did not exist. Expected after implementation: active pass.

- [x] **Step 2: Implement bounded Bacon-Rajan-style trial deletion reference path**

Evidence: Phase 6B exposes trial candidates and scan-black retained candidates,
then runs a bounded mark-gray / scan / collect-white pass over the deterministic
cycle graph. This still does not claim the production allocator root buffer.

Run:

```sh
cargo test -p garnet-memory
cargo test -p garnet-cli --test conformance_skeleton deferred_arc_cycle_detection
```

Expected after Phase 6B: cycle fixtures pass and the conformance handle is
active with trial-candidate assertions. Expected after full implementation:
allocator-integrated ARC cycle
collection has separate positive and negative tests for the Bacon-Rajan
root-buffer algorithm.

- [x] **Step 3: Add finalization-order and safe-mode interaction reference fixtures**

Evidence: Phase 6C exposes `finalization_order` from the bounded collect-white
pass and adds `CycleAllocationMode::SafeAffine` nodes that are retained while
excluded from ARC trial candidates.

Run:

```sh
cargo test -p garnet-memory --test cycle
cargo test -p garnet-cli --test conformance_skeleton deferred_arc_cycle_detection
```

Expected after Phase 6C: cycle fixtures pass for deterministic finalization
order and safe-mode non-ARC exclusion. Expected after full implementation:
allocator-integrated finalizer invocation and safe-mode boundary checks prove
the same invariants inside the runtime allocator path.

- [x] **Step 4: Add bounded root-buffer/decrement-event reference path**

Evidence: Phase 6D adds `CycleRootBuffer` and `release_root_to_buffer`, proving
that a root decrement can enqueue a still-referenced object and that collection
can scan buffered candidates without sweeping every unrooted graph node.

Run:

```sh
cargo test -p garnet-memory --test cycle
cargo test -p garnet-cli --test conformance_skeleton deferred_arc_cycle_detection
```

Expected after Phase 6D: buffered root release passes for threshold-triggered
collection and buffered-only scans.

- [x] **Step 5: Add allocator-owned root/edge decrement fixture**

Evidence: Phase 6E adds `CycleAllocatorFixture`, proving that the allocator
surface can own the graph plus root buffer and route root releases and ARC edge
decrements through the same buffered trial-deletion scheduling path.

- [x] **Step 6: Add kind-aware allocator surface and policy-configured store eviction**

Evidence: Phase 6J adds `KindAllocator`, `HeapKindAllocator`, `AllocRequest`,
and `AllocStats`, then threads the allocator surface through working,
episodic, semantic, and procedural stores without breaking existing `new()` /
`Default` callers. Policy-configured `EpisodeStore` and `VectorIndex` now
compact lazily at read/search time using `MemoryPolicy::score` and
`should_retain`.

Run:

```sh
cargo test -p garnet-memory --test properties
cargo test -p garnet-memory
cargo test -p garnet-cli cache
```

Expected after Phase 6J: allocator stats and policy eviction tests pass while
existing Memory Core and CLI cache callers remain compatible.

- [x] **Step 7: Connect store root lifecycles to the cycle-aware allocator adapter**

Evidence: Phase 6K adds `CycleAwareKindAllocator`, object-safe root hooks on
`KindAllocator`, and `AllocRootStats`. Working, episodic, semantic, and
procedural stores now retain observable roots when values are stored and release
them on clear, policy eviction, workflow replacement, and drop. This connects
store behavior to the bounded allocator-owned cycle fixture while still keeping
production ARC finalizers out of scope.

Run:

```sh
cargo test -p garnet-memory --test properties cycle_aware
cargo test -p garnet-memory --test properties dropping_stores_releases_cycle_aware_roots
cargo test -p garnet-memory
```

Expected after Phase 6K: cycle-aware root lifecycle tests pass while existing
Memory Core callers remain compatible.

- [x] **Step 8: Add fenced episodic text snapshot persistence**

Evidence: Phase 6L adds `EpisodePersistenceError` plus
`EpisodeStore::save_text` / `load_text` as a versioned text snapshot boundary
for episodic memory. The snapshot format hex-encodes payload text, writes
through a sibling temp file before rename, rejects malformed files before
touching the existing store, and rehydrates cycle-aware roots for recovered
episodes.

Run:

```sh
cargo test -p garnet-memory --test persistence
cargo test -p garnet-memory --test properties
cargo test -p garnet-memory
```

Expected after Phase 6L: episodic recovery survives delimiter-control
payloads, malformed persistence files are loud and non-mutating, and existing
Memory Core property tests remain compatible.

- [x] **Step 9: Promote fixture-backed roots to production ARC allocator roots**

Wire the trial-deletion pass to production ARC roots, decrement events, and
runtime finalizer invocation inside the Memory Core allocator backend instead
of only the deterministic fixture graph and cycle-aware adapter.

Phase 6Q now has completed the production-facing allocator facade evidence:
`CycleAwareKindAllocator` exposes a bounded root lifecycle surface through
default and injected store backends. Root release through the concrete
allocator can report buffered trial candidates, deterministic finalization
order, collected nodes, root stats, and safe-affine exclusion without callers
manually managing a `CycleAllocatorFixture`.

Phase 6R sibling partial pass: `CycleAwareKindAllocator::remove_edge` now
exposes allocator-facing buffered edge-removal collection evidence at the
wrapper layer. Threshold-crossing edge decrements return the same trial
candidate, finalization-order, collected, and `root_stats` reporting as the
existing release_root path, with safe-affine allocations preserved as
non-collectible. Below-threshold decrements correctly buffer without
collection. The behavior is exercised across all four `MemoryKind` variants.
This is fixture-level observable evidence promoted into the public allocator
test surface.

Phase 6S sibling partial pass: the concrete allocator now accepts an
allocator-owned finalizer log. Plain `release_root`, `collect_roots`, and
`remove_edge` calls record deterministic finalization order without requiring
callers to pass per-call callbacks, including delayed callback evidence after a
below-threshold root release is flushed. This is allocator-boundary
runtime-finalizer evidence; production allocator-integrated ARC and user
payload destructor semantics remain explicitly deferred.

Phase 6U sibling partial pass: the typed episodic cache backend now has an
opt-in signed path. `append_cache_text_with_mac` writes BLAKE3-keyed record
MACs for the canonical source-tree binding, timestamp, and encoded payload,
and `load_cache_text_with_mac` verifies every record before mutating live
memory. Tampered payloads and foreign keys are rejected as `MacMismatch`. The
existing unsigned typed-cache API remains for compatibility, so broad signed
persistence and production allocator-integrated ARC remain deferred.

Phase 6V dogfood evidence pass: the agentic dogfood matrix now includes a
`memory persistence integrity` domain with three signed typed-cache probes:
round-trip with the same key, tampered payload rejection, and foreign-key
rejection before live mutation. This raises source/app dogfood coverage for the
Phase 6U trust boundary without broadening the underlying runtime claim.

Phase 6W dogfood evidence pass: the agentic dogfood matrix now includes an
`agent adversarial boundaries` domain with three rejection probes: parser
depth-budget bombs, `main` entrypoints without explicit `@caps`, and legacy
`var` declarations inside `@safe` code. The first strict run after adding the
domain failed `58/60` because two checker diagnostics were asserted on stderr
instead of the CLI's stdout diagnostic stream; the corrected strict source
matrix passes `60/60` in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-034939`.

Phase 6X packaged-app dogfood evidence pass: the packaged Garnet Studio matrix
now treats signed-cache cargo probes as source-workspace-only when the app
bundle lacks `Contents/Resources/Cargo.toml`. The negative bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-040023`
preserves the original `57/60` failure. The corrected source bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-040631`
passes `60/60` with `skipped=0`; the copied-DMG app bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-040415`
passes `60/60` with `skipped=3` and per-probe skip logs; and
`/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-040414`
proves mounted-DMG copy-install smoke for DMG SHA-256
`e00d8e246fb10e339adf04c98b3f8654d841e8dd709a3ab213bd09cf7f1b34aa`.

Phase 6Z packaged assist-plan resource pass: after Phase 6Y added the
converter assist-plan reporter, packaged Garnet Studio now copies and chmods
`scripts/garnet_converter_assist_plan.py`, and the DMG smoke treats that
reporter as a required executable bundled asset before running copied-app
matrix probes. Verified local evidence:
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-045430`
(packaged app),
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-045440`
(copied app), and
`/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-071720`
for DMG SHA-256
`98d32b6b9970090f219074e41cc51e4163041b4f829c1e308d889fcb95cecfec`.
This remains ad-hoc local packaged evidence, not Developer ID notarization or
clean-machine Gatekeeper proof.

Phase 6AA Studio assist-plan UI pass: the macOS Converter panel now exposes a
separate `Assist Plan` action backed by `scripts/garnet_converter_assist_plan.py`.
The language picker includes advisory adoption targets such as TypeScript,
JavaScript, Swift, Java, C, C++, C#, Perl, Kotlin, Shell, SQL, and Other for
planning, while normal `Convert` remains limited to active deterministic
converter frontends (Rust/Ruby/Python/Go). This is a productization pass over
deterministic planning evidence; it does not make advisory languages active
converters and does not claim provider-backed LLM conversion.

Phase 6AB Studio Run-button workflow pass: the source checkout now includes
`script/build_and_run.sh`, which builds the SwiftPM Studio package, stages
`dist/Garnet Studio.app`, and launches it via `/usr/bin/open -n` with
`--verify`, `--debug`, `--logs`, and `--telemetry` modes. `.codex/environments/environment.toml`
wires the Codex app `Run` action to that script so local app iteration does not
depend on a remembered terminal command. This is source-checkout developer UX,
not signed/notarized distribution evidence.

Phase 6AC public adoption-surface pass: `scripts/garnet_adoption_surface_status.py`
and `docs/index.html` now expose the same current Studio truth: Codex Run,
`dist/Garnet Studio.app`, and planned-language `Assist Plan` are useful current
workflows, while Developer ID notarization, clean-machine Gatekeeper, broad
planned-language conversion, and provider-backed LLM conversion remain open
gates. The site was checked through static PWA smokes and a Browser smoke of
the Studio section.

Phase 6AD live Pages smoke pass: `scripts/smoke_garnet_pages_pwa.sh` now
checks the live deployment for the same Studio adoption phrases, and
`python3 scripts/test_smoke_garnet_pages_pwa.py` gives CI a local failing
fixture so the strict contract can be tested before Pages has published the
new site.

Phase 6AE provider-neutral prompt-pack pass: `scripts/garnet_assist_context_pack.py`
now emits a provider-neutral prompt contract plus
`garnet-assist-prompt-pack.md` in manifested output directories. This moves the
future LLM/agentic converter-assist lane toward a usable handoff artifact while
keeping provider, network, and conversion activation false and preserving
forbidden claims plus lineage, sandbox, `garnet check`, dogfood, and human-audit
gates.
Local Phase 6AE evidence: `scripts/run_agentic_dogfood_matrix.py
--copy-to-desktop --strict --skip-app-workbench` passes `64/64` with
`skipped=0` in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-054543`.

Phase 6AF planned-language assist breadth pass: the agentic matrix now carries
JavaScript, Swift, Java, and C++ assist-plan fixtures alongside TypeScript, and
`scripts/garnet_converter_assist_plan.py` recognizes Java
`CompletableFuture`/executor-style orchestration vocabulary as an
actor/orchestration migration risk. Phase 6AX extends the same taxonomy to keep
Kotlin, Shell, SQL, and Other advisory-only, while C/C++/Objective-C/Assembly/CUDA/platform-specific
code are labeled native-boundary first. This expands advisory planned-language
evidence without activating broad deterministic conversion or provider-backed
LLM conversion. Local evidence passes `68/68` with `skipped=0` in Desktop
bundle `/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-055816`.

Phase 6AO converter LLM feasibility pass: `scripts/garnet_converter_llm_feasibility.py`
now makes the current feasibility decision executable. Advisory, provider-neutral
planning is feasible; autonomous/provider-backed LLM conversion is not active and
is blocked on secure runtime boundaries, deterministic frontend gates, native
boundary review, lineage, `@sandbox`, `garnet check`, dogfood evidence, and
human audit. The agentic matrix adds C, C#, and Perl planned-language assist
fixtures plus a four-probe `converter LLM feasibility` domain so the advertised
broad-language assist surface is dogfood-probed without activating those
languages as deterministic frontends. Local evidence passes `84/84` with
`skipped=0` in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-084611`.

Phase 6AP converter advisory bundle pass: `scripts/garnet_converter_advisory_bundle.py`
now builds a manifested provider-neutral handoff package from the converter
LLM feasibility decision, deterministic context pack, and per-file assist plan.
The bundle keeps conversion inactive, records that provider/model/network use
is not required, omits source text by default for privacy/provider-boundary
safety, and only embeds source when `--include-source` is explicit. The
agentic matrix adds a four-probe `converter advisory bundle` domain covering
current truth, default source omission, explicit source inclusion, and manifest
verification. Local evidence passes `88/88` with `skipped=0` in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-090352`.

Phase 6AQ converter advisory bundle UX pass: Garnet Studio now exposes an
`Advisory Bundle` action in the Converter panel. The action locates
`scripts/garnet_converter_advisory_bundle.py` from packaged app resources,
`GARNET_REPO_ROOT`, or the source checkout, writes a temporary source file and
manifested bundle directory under `~/Desktop/dogfood/`, and never passes
`--include-source` from the app surface. The agentic matrix preserved the failed
`88/91` run at
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-091859` before
fixing the probe path, then passed `91/91` with `skipped=0` in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-091949`.

Phase 6AR durable Studio advisory evidence pass: `GarnetStudioEvidenceDirectory`
now makes the Studio advisory-bundle output path explicit and stable under
`~/Desktop/dogfood/garnet-studio-advisory-bundle-<stamp>`. The transient source
input still lives outside the preserved bundle, and the app still omits
`--include-source`. Local evidence passes `91/91` with `skipped=0` in Desktop
bundle `/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-093319`;
packaged app/DMG evidence passes in
`/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-093402` with
DMG SHA-256
`8a64b563a9a6b7de976d2d479ecabe52eed9b0e08f311a05643a2a3351823d4c`.

Phase 6AS converter advisory review gate pass:
`scripts/garnet_converter_advisory_review.py` now reviews manifested advisory
bundles before model/agent handoff. It verifies the bundle manifest, confirms
the no-source privacy boundary, blocks source-included bundles unless
explicitly approved, keeps provider-backed conversion disallowed, and writes a
manifested review report with the required lineage, `@sandbox`, `garnet check`,
dogfood evidence, and human-audit checklist. The agentic matrix adds a
three-probe `converter advisory review` domain and passes `94/94` with
`skipped=0` in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-100014`.
The packaged app/DMG smoke passes in
`/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-100034` with
DMG SHA-256
`793aaec45055c338fda45386006f545356b4d0a2adad79ad8e6048ef012dee36`.

Phase 6AT converter advisory review UX pass: Garnet Studio now exposes an
`Advisory Review` action beside `Advisory Bundle`. The action creates a
default no-source advisory bundle, runs the provider-neutral review gate, and
preserves manifested review output under
`~/Desktop/dogfood/garnet-studio-advisory-review-<stamp>` without passing
`--allow-source-included`. The agentic matrix adds a three-probe `converter
advisory review UX` domain and passes `97/97` with `skipped=0` in Desktop
bundle `/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-101247`.
Refreshed web/PWA evidence lives at
`/Users/idc2.0/Desktop/dogfood/web-pwa-readiness-20260516-101246`.
The packaged app/DMG smoke passes in
`/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-101312` with
DMG SHA-256
`68babfa7974a4b31173651eee27f0e5f6844637486478c42802b53bd34a80863`.

Phase 6AU Studio objective pulse pass: Garnet Studio now exposes an
`Objective Pulse` action in the Release panel. The action runs the repo-native
`scripts/garnet_mit_readiness_status.py --format markdown` reporter from
packaged resources, `GARNET_REPO_ROOT`, or the source checkout so the app can
show the overall MIT/productization percentage separately from the completed
tracked-slice ledger. The agentic matrix adds a three-probe `MIT objective
pulse UX` domain and passes `100/100` with `skipped=0` in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-102355`.
Refreshed web/PWA evidence lives at
`/Users/idc2.0/Desktop/dogfood/web-pwa-readiness-20260516-102355`.
The packaged app/DMG smoke passes in
`/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-102422` with
DMG SHA-256
`88c1cbe96fe3bc3fa38d44fcf574970400e7739156bb569106790d4e801a22b6`.

Phase 6AV converter advisory handoff pass:
`scripts/garnet_converter_advisory_handoff.py` now turns a manifested advisory
bundle plus `garnet_converter_advisory_review.py` output into a final
source-free, provider-neutral handoff packet. The packet refuses blocked or
source-included reviews, never calls a provider, keeps conversion inactive, and
records the required lineage, `@sandbox`, `migrate_todo`, `garnet check`,
dogfood evidence, and human-audit gates before any candidate output can be
trusted. The agentic matrix adds a three-probe `converter advisory handoff`
domain for current truth, source-included blocking, and manifest verification.
Current source-checkout evidence passes `103/103` with `skipped=0` in Desktop
bundle `/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-103651`.
Refreshed web/PWA evidence lives at
`/Users/idc2.0/Desktop/dogfood/web-pwa-readiness-20260516-103731`.
The packaged app/DMG smoke passes in
`/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-103813` with
DMG SHA-256
`396c5afc7e5409fc9977d0dcd582a363851f65f1479ceb7be8f1c838d94cb932`.

Phase 6AW converter advisory handoff UX pass: the macOS Studio Converter panel
exposes `Advisory Handoff` after `Advisory Review`. The action creates a
default no-source advisory bundle, runs the provider-neutral review gate, then
packages the reviewed no-source context through
`garnet_converter_advisory_handoff.py` into
`~/Desktop/dogfood/garnet-studio-advisory-handoff-<stamp>`. This keeps
provider/model execution, autonomous conversion, and candidate-output trust
outside the active lane until lineage, `@sandbox`, `garnet check`, dogfood
evidence, and human audit are complete.

Phase 6AX converter/platform strategy pass: the converter and public adoption
surface now use an explicit three-label language taxonomy. Active deterministic
conversion remains Rust/Ruby/Python/Go. Advisory planning now covers
JavaScript/TypeScript/Swift/Java/C/C++/C#/Perl/Kotlin/Shell/SQL/Other through
risk inventory, Garnet-aware context, advisory bundle, review, and handoff.
C/C++/Objective-C/Assembly/CUDA/platform-specific code are labeled
native-boundary recommended so low-level ABI, timing, layout, GPU, and platform
runtime behavior is wrapped through native modules or FFI instead of promised
as lossless source conversion. The new
`F_Project_Management/GARNET_CONVERTER_AND_PLATFORM_STRATEGY.md` records the
two-way architecture: import high-level agent/product logic into Garnet where
it fits, keep precision-native code beside Garnet with CapCaps, lineage,
memory, and sandbox declarations, and later lower Garnet out to Wasm or
LLVM-style native targets only after compiler-backend evidence exists.
`docs/index.html` now stays landing-page focused while `docs/status.html`
carries detailed readiness gates. The same slice adds Apple distribution and
Windows/Linux Studio handoff packets so account-owner signing work and
cross-platform Studio MVP work can proceed without overclaiming App Store,
notarization, signed MSI, provider-backed LLM conversion, or native backend
completion. Local evidence passes `106/106` with `skipped=0` in
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-124251`,
web/PWA readiness passes with blockers/warnings at zero in
`/Users/idc2.0/Desktop/dogfood/web-pwa-readiness-20260516-124308`, and
`swift test --package-path apps/garnet-studio-macos` passes `26/26`.

Phase 6AY Mac-side continuation pass: `scripts/garnet_mac_side_continuation_status.py`
adds a goal-prompt-friendly pulse for the work that can still proceed from the
current macOS checkout. It imports the active MIT/productization percentage,
lists the published reusable `Navigata1/dogfood-readiness` toolkit, unsigned
Garnet Studio quality, website/status/presentation, converter advisory, and
proof/benchmark evidence as Mac-actionable lanes, and keeps Apple Developer ID
notarization `blocked-external` plus Windows/Linux Studio `handoff-only`.
The agentic dogfood matrix now probes this boundary so future slices do not
turn Mac-side actionability into notarized distribution, Windows/Linux runtime
completion, provider-backed LLM conversion, or native backend readiness claims.

Phase 6AZ Studio continuation pulse pass: Garnet Studio now exposes a
read-only `Continuation Pulse` action in the Release panel. The action locates
`scripts/garnet_mac_side_continuation_status.py` from packaged resources,
`GARNET_REPO_ROOT`, or the source checkout and runs it with
`--format markdown`, making the post-PR #141 Mac-actionable/deferred split
visible inside the app. The Release panel now names `Mac continuation` as a
separate line and keeps Developer ID signing/notarization plus Windows/Linux
Studio in the deferred boundary. The agentic dogfood matrix adds a three-probe
`Mac continuation pulse UX` domain, and the packaging/DMG smoke stages
`garnet_mac_side_continuation_status.py` as an executable app resource.
Local source-checkout evidence passes `110/110` with `skipped=0` in Desktop
bundle `/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-164602`.
Packaged app/DMG evidence passes in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-164632` for
DMG SHA-256
`4a7fbd77361e6c4d684032fc5f486e3b1abd40598b8bf87f9178b88c657e397e`.
This is unsigned/local Studio productization evidence, not Developer ID
notarization, Windows/Linux runtime completion, provider-backed LLM conversion,
or native backend lowering.

Phase 6BA public Studio continuation hook pass: `docs/index.html` now carries
the Studio `Continuation Pulse` as a landing-page hook beside Objective Pulse,
Assist Plan, Advisory Bundle/Review/Handoff, Codex Run, and the staged
`dist/Garnet Studio.app` workflow. `scripts/smoke_garnet_pages_pwa.sh` now
requires `Continuation Pulse` in the Studio adoption copy, and
`python3 scripts/test_smoke_garnet_pages_pwa.py` locks that requirement with a
local failing fixture. Local Pages evidence passes with blockers/warnings at
zero in `/Users/idc2.0/Desktop/dogfood/pages-pwa-readiness-20260516-165857`,
and refreshed web/PWA smoke evidence passes in
`/Users/idc2.0/Desktop/dogfood/web-pwa-readiness-20260516-165857`. This is
repo/public-site source evidence; live Pages publication is verified after the
PR lands and the deployment has caught up.

Phase 6BB MIT demo-route pass: `scripts/garnet_mit_demo_route.py` now turns the
current objective pulse, Studio continuation, converter advisory workflow,
agentic dogfood matrix, public web/PWA evidence, and blocked-gate closeout into
a seven-minute JSON/Markdown route. It keeps the route presentation-ready while
forbidding final claims about Developer ID notarization, Windows/Linux Studio
runtime proof, provider-backed LLM conversion, native backend lowering, mobile
distribution, production-ready language status, or final MIT/productization
acceptance. The agentic dogfood matrix adds a three-probe `MIT demo route`
domain and passes `113/113` with `skipped=0` in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-171902`;
standalone route evidence verifies in
`/Users/idc2.0/Desktop/dogfood/garnet-mit-demo-route-20260516-171836`.
`scripts/package_garnet_studio_macos.sh` and
`scripts/smoke_garnet_studio_dmg.sh` now stage and require the route reporter
as a packaged executable resource so packaged matrix runs cannot silently lose
the new presentation proof surface. Packaged app/DMG evidence passes in
`/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-172150` for
DMG SHA-256
`52fec2da40597de6dbaecb4b21562cf19007e4ec584f0f8174545f9bd13fe9f3`.

Phase 6BC Studio MIT demo-route UX pass: the macOS Studio Release panel now
exposes `Demo Route` beside `Objective Pulse` and `Continuation Pulse`. The
action locates `garnet_mit_demo_route.py` from packaged resources,
`GARNET_REPO_ROOT`, or the source checkout; runs it with `--output-dir` and
`--format markdown`; and preserves the manifested route bundle under
`~/Desktop/dogfood/garnet-studio-mit-demo-route-<stamp>`. This improves the app
rehearsal loop without turning presentation readiness into final acceptance,
Apple notarization, Windows/Linux runtime proof, provider-backed LLM conversion,
or native backend lowering. SwiftPM covers the locator, runner command, and
Desktop evidence path, while the agentic matrix adds a three-probe `MIT demo
route UX` domain and passes `116/116` with `skipped=0` in
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-174234`.
The route-shaped Desktop bundle verifies in
`/Users/idc2.0/Desktop/dogfood/garnet-studio-mit-demo-route-20260516-224439`,
packaged app/DMG smoke passes in
`/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-174457`,
and the landing-page copy change passes local Pages/Web PWA smokes in
`/Users/idc2.0/Desktop/dogfood/pages-pwa-readiness-20260516-174644` and
`/Users/idc2.0/Desktop/dogfood/web-pwa-readiness-20260516-174627`.

Phase 6AG promo-video readiness pass: `scripts/garnet_promo_video_status.py`
now records the requested 30-second Garnet promo as a planned-contract lane
with storyboard beats, HyperFrames/Remotion composition, rendered-artifact,
visual-QA, website-export, Desktop-bundle, and overclaim gates. This prepares
the ad lane for a later rendering slice without claiming that a video artifact
or website-ready export exists. Local source-checkout matrix evidence passes
`71/71` with `skipped=0` in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-061302`.
Packaged app/DMG evidence also passes after staging the promo reporter, with
DMG smoke bundle
`/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-061434`.

Phase 6AH promo-source lock pass: `scripts/garnet_promo_video_status.py`
now reports the promo lane as `source-locked` at 35.0% by proving canonical
logo/PWA icon assets, the public site, Garnet Studio, the dogfood matrix, and
the MIT readiness reporter are present and hash/phrase checked before a render
slice starts. The remaining gates are still HyperFrames/Remotion composition,
rendered MP4/WebM, visual QA, website export, Desktop dogfood evidence, and
repo/site overclaim checks.
Packaged app/DMG evidence also passes after staging the promo source-lock
assets and Garnet Studio source surface, with DMG smoke bundle
`/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-063506`
for DMG SHA-256
`88126b95c38fd08a7587100dd3299738c52d79e720f7c0ce06bfed5767347a46`.

Phase 6AI promo-composition source pass: the promo lane now reports
`composition-ready` at 50.0% by adding `docs/promo/DESIGN.md` and a
HyperFrames-compatible `docs/promo/composition.html` that registers the
`garnet-promo-main` timeline, declares 30 seconds, and uses locked repo assets.
The matrix adds a fifth promo-video probe for the composition source, and the
source-checkout matrix passes `73/73` with `skipped=0` in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-065332`. The
DMG smoke requires both promo source files to be present in packaged docs and
passes in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-071720` for
DMG SHA-256
`98d32b6b9970090f219074e41cc51e4163041b4f829c1e308d889fcb95cecfec`. This is
still not a rendered MP4/WebM, not a visual-QA verdict, and not a website-ready
export.

Phase 6AJ promo-render harness pass: `scripts/render_garnet_promo_video.mjs`
now renders `docs/promo/composition.html` through headless Chrome/CDP and
`ffmpeg`, producing MP4, WebM, poster, JSON, Markdown, and `MANIFEST.sha256`
evidence. The local Desktop render evidence at
`/Users/idc2.0/Desktop/dogfood/garnet-promo-video` verifies a 30.000-second
1920x1080 render at 12 fps for both H.264 MP4 and VP9 WebM. When those local
artifacts are present, `scripts/garnet_promo_video_status.py` reports
`rendered-artifact-ready` at 65.0% and `scripts/garnet_mit_readiness_status.py`
reports the broader objective at 55.9%. The source-checkout matrix adds a sixth
promo-video readiness probe for the render harness contract and passes `74/74`
with `skipped=0` in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-071841`. This
is still not a visual-QA verdict and not a website-ready export.

Phase 6AK promo automated visual-QA pass: `scripts/qa_garnet_promo_video.mjs`
now verifies the local MP4/WebM render with `ffprobe`, extracts representative
frames with `ffmpeg`, writes JSON/Markdown plus `MANIFEST.sha256`, and records
the caveat that website export and public-site embedding remain separate gates.
The corrected composition replaces the brittle `73/73` proof count with a
`74+` growing-probe-set claim. Local visual-QA evidence at
`/Users/idc2.0/Desktop/dogfood/garnet-promo-video-visual-qa` passes manifest
verification, `scripts/garnet_promo_video_status.py` reports `visual-qa-ready`
at 80.0%, and `scripts/garnet_mit_readiness_status.py` reports the broader
objective at 57.3% when that local evidence is present. The refreshed
source-checkout matrix passes `75/75` with `skipped=0` in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-074756`, and
the copied-DMG smoke passes in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-074816` for
DMG SHA-256
`64bafd2ae61f79a156c7715e23b857f6a95b190d1a84bb491e736d06935b5b2f`. This is
still not a website-ready export.

Phase 6AL promo website-export package pass:
`scripts/export_garnet_promo_video_site.mjs` now requires passing visual-QA
evidence, copies MP4/WebM/poster assets into a website-export bundle, writes an
`embed-snippet.html`, JSON/Markdown evidence, and `MANIFEST.sha256`, and records
that the package is still not embedded on the public site. Local export evidence
at `/Users/idc2.0/Desktop/dogfood/garnet-promo-video-website-export` passes
manifest verification, `scripts/garnet_promo_video_status.py` reports
`website-export-ready` at 90.0%, and `scripts/garnet_mit_readiness_status.py`
reports the broader objective at 58.2% when that local evidence is present. The
source-checkout matrix passes `76/76` with `skipped=0` in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-074756`, and
the refreshed copied-DMG smoke passes in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-074816` for
DMG SHA-256
`64bafd2ae61f79a156c7715e23b857f6a95b190d1a84bb491e736d06935b5b2f`. This is
still not public-site embedding or human/aesthetic acceptance.

Phase 6AM promo public-site sync pass:
`scripts/sync_garnet_promo_video_site.mjs` now requires passing website-export
evidence, copies MP4/WebM/poster assets into `docs/assets/`, verifies the
public-site `<video>` block plus service-worker cache references, and writes
manifested JSON/Markdown sync evidence. The promo lane reports
`public-site-embedded` at 95.0% when render, visual-QA, website-export, and
site-sync bundles are present; `scripts/garnet_mit_readiness_status.py`
reports the broader objective at 58.6%. The public site embeds the video while
keeping human/aesthetic acceptance, notarization, mobile distribution,
provider-backed LLM conversion, and full MIT/productization completion open.
The source-checkout matrix passes `88/88` with `skipped=0` in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-090352`, and the
refreshed copied-DMG smoke passes in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-090543` for DMG
SHA-256
`72af0dc3155fb9c7897167645b10ed4f3caca0b7680bd15a40826cc06d8cc720`.

## Milestone 7: Release, Proof, Native Backend, And Empirical Evidence

**Purpose:** Give the too-large ambitions their own rigorous tracks so they stop being confused with current runtime truth.

**Files:**

- Modify: `F_Project_Management/GARNET_v0_4_2_RELEASE_PUBLICATION_RUNBOOK.md`
- Modify: `C_Language_Specification/GARNET_v0_4_2_Installer_Release_Contract.md`
- Create: `F_Project_Management/ROADMAPS/GARNET_NATIVE_BACKEND_PLAN.md`
- Create: `F_Project_Management/ROADMAPS/GARNET_FORMAL_PROOF_PLAN.md`
- Create: `F_Project_Management/ROADMAPS/GARNET_EMPIRICAL_VALIDATION_PLAN.md`
- Test: release workflow, installer smoke, and dogfood-readiness Part 1

- [x] **Step 1: Publish the org release using an org-authorized browser or desktop session**

Use the already-built v0.4.2 assets. The CLI token for `Navigata1` has `push: false` on `Island-Dev-Crew/garnet`, but the current browser session can open the org release form and shows `Publish release`.

Run after publication:

```sh
gh release view v0.4.2 --repo Island-Dev-Crew/garnet --json tagName,url,assets
```

Observed 2026-05-15: org release exists at
`https://github.com/Island-Dev-Crew/garnet/releases/tag/v0.4.2`, points at
merge commit `6e945d6a151c2b97ae842c21aeeaa5a678ae65f5`, and lists the `.deb`,
`.rpm`, macOS tarball, and `SHA256SUMS` assets.

- [x] **Step 2: Rerun a network-backed installer smoke against the org release**

Run:

```sh
GARNET_INSTALL_MODE=release GARNET_VERSION=v0.4.2 sh installer/sh.garnet-lang.org/install.sh
./scripts/verify_org_release_smoke.sh
garnet --version
```

Observed 2026-05-15: `./scripts/verify_org_release_smoke.sh` passed against
`Island-Dev-Crew/garnet`. On macOS it tried the unsigned `.pkg`, fell back to
the uploaded `aarch64-apple-darwin` tarball, verified SHA-256, installed into a
temporary prefix, and reported `garnet 0.4.2`.

- [x] **Step 3: Create native backend, proof, and empirical plans**

Milestone 7 step 3 now has scaffolded plans:

- `F_Project_Management/ROADMAPS/GARNET_NATIVE_BACKEND_PLAN.md`
- `F_Project_Management/ROADMAPS/GARNET_FORMAL_PROOF_PLAN.md`
- `F_Project_Management/ROADMAPS/GARNET_EMPIRICAL_VALIDATION_PLAN.md`

Each plan must define a falsifiable first milestone:

- native backend: parse/check/run one integer arithmetic program through backend output,
- formal proof: one checked mechanized lemma for a tiny safe-mode core,
- empirical validation: one archived pilot dataset with script-reproducible metrics.

Run:

```sh
cargo test --workspace --no-fail-fast
python3 -m json.tool /tmp/dogfood-readiness-*/dogfood-readiness-data.json
```

Expected: implementation tests stay green and readiness artifacts remain parseable.

## MIT Release Gate

Garnet is ready to be spoken about as an MIT-grade prototype when these are true:

1. The current-state guide is the first reviewer path.
2. Historical claims are separated from current executable truth.
3. The 10 MVP apps parse, check, run, and are wired into CI.
4. Starter templates scaffold, test, and run frictionlessly.
5. The conformance matrix marks parser-only and deferred rows honestly.
6. The org release has assets and checksums, not just a fork release.
7. A dogfood-readiness report/deck/data bundle exists for the current commit.

Garnet is ready to be called a complete language/toolchain only when, in addition to the prototype gate, the runtime, checker, actor, memory, trait/generic, release, and empirical/proof tracks above have executable evidence.

Phase 6BD presentation pass: `scripts/garnet_mit_deck_outline.py` turns the
current demo route, adoption surface, readiness percentage, blocked gates, and
evidence notes into a manifested JSON/Markdown 8-slide reviewer outline. The
agentic dogfood matrix adds a three-probe `MIT deck outline` domain, and
packaging resource tests require the reporter to be staged into Garnet Studio
DMG resources so packaged matrix smoke can still run from `Contents/Resources`.
This is a presentation planning artifact only; it does not claim final
MIT/productization acceptance, Developer ID notarization, Windows/Linux Studio
runtime proof, provider-backed LLM conversion, native backend lowering, or
mobile distribution.

Phase 6BE Studio deck-outline UX pass: the macOS Studio Release panel exposes
`Deck Outline` beside the existing presentation/status actions. The action
locates the packaged or source `garnet_mit_deck_outline.py` reporter, writes a
manifested `~/Desktop/dogfood/garnet-studio-mit-deck-outline-<stamp>` bundle,
and prints the output path in the Studio console. SwiftPM tests cover the
locator, runner, and Desktop evidence path; the agentic dogfood matrix adds a
three-probe `MIT deck outline UX` domain. This remains presentation/evidence
UX only, not final MIT acceptance, notarized distribution, Windows/Linux proof,
provider-backed conversion, native lowering, or mobile distribution.

Phase 6BF deck-preview pass: `scripts/garnet_mit_deck_preview.py` renders the
current deck outline into a self-contained HTML review artifact plus JSON,
outline Markdown, and `MANIFEST.sha256`. The artifact includes the slide story,
evidence paths, speaker notes, blocked gates, and forbidden claims, and the
agentic dogfood matrix adds a `MIT deck preview` domain for current truth, HTML
content, output contract, and verified manifest evidence. This remains a
browser-smokeable review preview, not final MIT/productization acceptance,
human/aesthetic deck approval, notarization, Windows/Linux proof,
provider-backed conversion, native lowering, or mobile distribution.

Phase 6BI packaged deck-preview manifest pass: `GarnetStudio
--mit-deck-preview-smoke` now verifies the generated deck-preview bundle's own
`MANIFEST.sha256` before reporting success. A stale checksum manifest returns
through the deck-preview smoke failure path, the source-checkout matrix expands
the `MIT deck preview smoke` domain to four probes, and packaged DMG smoke
preserves a nested `studio-deck-preview/` bundle whose HTML, JSON, and outline
Markdown checksums verify. This is still unsigned/local presentation evidence,
not human/aesthetic deck approval, Developer ID notarization, Windows/Linux
proof, provider-backed conversion, native lowering, mobile distribution, or
final MIT/productization acceptance.

Phase 6BJ copied-DMG manifest-log pass: `scripts/smoke_garnet_studio_dmg.sh`
now records a dedicated `copied-app-mit-deck-preview-manifest-verify` command
after the copied app creates the deck preview. The evidence bundle preserves
stdout/stderr logs for `shasum -a 256 -c MANIFEST.sha256` inside
`studio-deck-preview/`, and the checks table points directly at the nested
manifest. This is an auditability improvement for unsigned/local DMG smoke only;
it still does not claim human/aesthetic deck approval, Developer ID
notarization, Windows/Linux proof, provider-backed conversion, native lowering,
mobile distribution, or final MIT/productization acceptance.
