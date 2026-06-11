# Script walkthroughs — repo reporters and advisory pipeline

Relocated from the root README by W-REBUILD RB-0b (the README links here; the
walkthrough text below is preserved verbatim from the pre-RB-0b README — these
are the operational reporters' usage notes, with their claim boundaries
intact). For current public numbers, read `docs/truth.json` (RB-0a) — these
walkthroughs explain *how to run* the reporters, not what today's values are.

## Converter status reporter

`python3 scripts/garnet_converter_status.py` reports the active converter
lanes, advisory planning languages, native-boundary languages, future
Wasm/LLVM-style lowering posture, and the planned Garnet-aware assist contract
for future LLM/agentic guidance.

## Converter advisory pipeline, Studio surface, adoption truth, readiness, promo

For future LLM or agentic converter guidance, run `python3 scripts/garnet_converter_llm_feasibility.py` first. It reports the current decision: provider-neutral advisory planning is feasible, but autonomous/provider-backed LLM conversion is not active and is not feasible until secure runtime, deterministic frontend, `@sandbox`, lineage, `garnet check`, dogfood, and human-audit gates exist. `python3 scripts/garnet_assist_context_pack.py` creates a deterministic local context pack from the current truth, public README, Mini-Spec, conformance matrix, and dogfood ledger; it does not enable provider-backed conversion or make planned language frontends active. For a single active or planned source file, run `python3 scripts/garnet_converter_assist_plan.py --language typescript --source path/to/file.ts`; it emits advisory safe-mode, memory, CapCaps, actor/orchestration, migration-risk, and gate evidence without executing source code or claiming conversion. The advisory language menu is JavaScript, TypeScript, Swift, Java, C, C++, C#, Perl, Kotlin, Shell, SQL, and Other; C, C++, Objective-C, Assembly, CUDA, and platform-specific code should usually remain native modules or FFI wrapped by Garnet capabilities. To create one manifested handoff package for local agent/model review, run `python3 scripts/garnet_converter_advisory_bundle.py --language typescript --source path/to/file.ts --output-dir /tmp/garnet-advisory`; it combines feasibility, context, and assist-plan evidence, omits source text by default, and requires `--include-source` before embedding source in the bundle. Before handing that bundle to a model or agent, run `python3 scripts/garnet_converter_advisory_review.py --bundle-dir /tmp/garnet-advisory`; it verifies the bundle manifest, blocks source-included bundles unless explicitly approved, and emits the human-review checklist for lineage, `@sandbox`, `garnet check`, dogfood evidence, and human audit. To package the reviewed, no-source context into a final provider-neutral prompt packet, run `python3 scripts/garnet_converter_advisory_handoff.py --bundle-dir /tmp/garnet-advisory --review-dir /tmp/garnet-advisory-review --output-dir /tmp/garnet-handoff`; it refuses blocked reviews and still does not call a provider or enable conversion.

Garnet Studio exposes the same boundary in the macOS workbench: active deterministic converter lanes still use `Convert`, planned-language risk inventory uses `Assist Plan`, the provider-neutral `Advisory Bundle` action writes a manifested local handoff package under `~/Desktop/dogfood/` without embedding source by default, `Advisory Review` creates that bundle plus a manifested local review report before any model/agent handoff, `Advisory Handoff` packages the reviewed no-source context into the final provider-neutral packet, and `Objective Pulse` runs the repo-native MIT/productization readiness reporter so the app can show the overall percentage separately from tracked-slice completion. The website keeps product-facing copy on `docs/index.html` and detailed readiness caveats on `docs/status.html`.

For repo/site adoption truth, run `python3 scripts/garnet_adoption_surface_status.py`. It ties the public hook, active converter lanes, advisory planning lanes, native-boundary labels, LLM-assist boundaries, verified use cases, and productization gates to current evidence before marketing or README copy moves forward. The durable strategy is documented in [`GARNET_CONVERTER_AND_PLATFORM_STRATEGY.md`](F_Project_Management/GARNET_CONVERTER_AND_PLATFORM_STRATEGY.md), with Windows/Linux handoff and Apple distribution walkthrough packets in [`F_Project_Management/`](F_Project_Management/).

For broader public-readiness accounting, run `python3 scripts/garnet_mit_readiness_status.py` or use Garnet Studio's `Objective Pulse`. It intentionally distinguishes the complete tracked implementation-plan ledger from still-open productization gates such as Developer ID notarization, mobile distribution, promo video, broad converter frontends, LLM assist, proof, and empirical validation.

For the requested 30-second promo/ad lane, run `python3 scripts/garnet_promo_video_status.py`. It defines the storyboard beats, locks the visual identity/source surfaces to real repo assets, verifies the HyperFrames-compatible composition source under `docs/promo/`, and recognizes local Desktop MP4/WebM, automated visual-QA, website-export, and public-site sync evidence when present. `scripts/render_garnet_promo_video.mjs` renders the composition through headless Chrome plus `ffmpeg`; `scripts/qa_garnet_promo_video.mjs` verifies metadata and sample-frame extraction; `scripts/export_garnet_promo_video_site.mjs` packages site-ready media plus an embed snippet; `scripts/sync_garnet_promo_video_site.mjs` copies the verified export into `docs/assets/` and writes site-sync evidence without claiming final human/aesthetic acceptance.

## macOS notarization preflight

Summarize macOS app notarization preflight evidence:

```sh
scripts/preflight_garnet_studio_notarization.sh --copy-to-desktop
python3 scripts/garnet_studio_notarization_status.py --bundle ~/Desktop/dogfood/<preflight-bundle>
```

This reports blocker/warning status only. It does not submit to Apple or claim
Developer ID notarization.
