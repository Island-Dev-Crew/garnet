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
