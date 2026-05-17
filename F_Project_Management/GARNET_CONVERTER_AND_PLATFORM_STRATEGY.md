# Garnet Converter and Platform Strategy

Status: active strategy, not an implementation claim.
Date: 2026-05-16.
Owner surface: converter, Studio, public site, Windows/Linux handoff, and future provider-backed assist.

## Executive Position

Garnet should not try to swallow every source language. The stronger architecture is two-way:

- Into Garnet for best-fit imports: high-level product logic, agent orchestration, workflow policy, capability-aware boundaries, memory declarations, and migration planning.
- Beside Garnet for native systems: keep C, C++, Rust, C#, Objective-C, Assembly, CUDA, and platform-specific code in native modules or FFI where low-level fidelity matters, then wrap those calls with Garnet CapCaps, memory declarations, lineage, and sandbox policy.
- Out of Garnet for performance: Garnet lowers out to Wasm and LLVM-style native targets when the compiler backend exists, instead of pretending source-to-source conversion preserves all low-level control.

That keeps adoption palatable without flattening the truth. Users can bring code to Garnet for advisory planning today, use deterministic conversion only where it exists, and keep precision-native code behind explicit boundaries until native backends are proven.

## Best-Fit Imports

These are the areas where conversion into Garnet makes sense:

| Fit | Why Garnet helps | Current lane |
| --- | --- | --- |
| Agent orchestration | Garnet can make capability boundaries, memory state, and run gates visible. | Active through examples, dogfood matrix, and advisory planning. |
| App glue and service workflows | Managed-mode ergonomics are a good match for routing, configuration, policy, and glue code. | Active for Garnet examples; advisory for imported sources. |
| Memory-aware workflows | Garnet can promote caches, history, vector stores, and durable records into explicit memory declarations. | Advisory planning active; full production semantics remain gated by Memory Core proof. |
| Capability-sensitive automation | CapCaps are useful when code needs file, network, process, database, or FFI authority surfaced clearly. | Checker and dogfood gates active. |
| Migration planning | Even when conversion is not available, Garnet can produce a risk inventory and reviewed handoff packet. | Active through context pack, assist plan, advisory bundle, review, and handoff. |

## Bad Direct-Conversion Fits

These should not be promised as clean source-to-source conversion:

| Bad direct-conversion fit | Why direct conversion loses fidelity | Correct Garnet path |
| --- | --- | --- |
| Kernels, drivers, and hard real-time paths | Timing, ABI, volatile memory, interrupt, and scheduler behavior are the program. Rewriting into high-level Garnet would remove the control that makes the source correct. | Native module or FFI with explicit capabilities and audit evidence. |
| Pointer-heavy C/C++ systems | Aliasing, layout, allocator choice, undefined-behavior avoidance, and manual lifetime conventions are often implicit. | Risk inventory first; keep native code native unless a bounded subsystem can be modeled safely. |
| GPU kernels and CUDA | Thread hierarchy, memory spaces, coalescing, and device-specific kernels are not preserved by ordinary source translation. | Native boundary now; possible future specialized backend later. |
| SIMD and hot numeric loops | Performance relies on instruction selection, vector widths, cache layout, and compiler backend behavior. | Native boundary now; future Garnet-to-native lowering once benchmarks exist. |
| Platform ABI glue | Framework calling conventions, Objective-C runtime behavior, COM, JNI, and OS handles often matter more than syntax. | Wrap through FFI/native modules with lineage and CapCaps. |

The difference is not language prestige. It is semantic density. Best-fit imports are mostly about intent, orchestration, policy, and safety boundaries. Bad direct-conversion fits are often about exact layout, timing, ABI, and hardware behavior. Garnet can supervise those systems, but should not pretend it can rewrite them losslessly today.

## Language Menu Taxonomy

### Active conversion

These are deterministic converter lanes today:

- Rust
- Ruby
- Python
- Go

The output is sandboxed Garnet with lineage, metrics, and migrate_todo evidence. It remains a migration assistant, not a full transpiler.

### Advisory planning

These can be selected for risk inventory, context packing, advisory bundle, review, and handoff:

- JavaScript
- TypeScript
- Swift
- Java
- C
- C++
- C#
- Perl
- Kotlin
- Shell
- SQL
- Other

Other is useful only as advisory analysis until deterministic support exists. It should never be described as conversion.

### Native boundary recommended

These should be labeled as native-boundary first:

- C
- C++
- Objective-C
- Assembly
- CUDA
- platform-specific code

Some of these also appear in advisory planning because a risk inventory is useful. That does not mean direct conversion is a good target.

### Backend lowering

Longer term, Garnet needs proven compiler backends so code can move in both directions:

- Wasm for portable plugin and browser/runtime surfaces.
- LLVM-style native targets for performance-sensitive Garnet code.
- Native package toolchains for distribution and platform integration.

This is a future compiler-backend lane. It is not active until build artifacts, tests, dogfood evidence, benchmarks, and release gates exist.

## Converter Pipeline

The correct LLM/advisory pipeline is:

source language classifier -> risk inventory -> Garnet-aware context pack -> advisory plan -> review handoff -> human-approved candidate -> garnet check/test/dogfood

Required invariants:

- Do not execute source during analysis.
- Do not include source text in provider packets by default.
- Preserve source hash and lineage.
- Keep candidate output sandboxed by default.
- Emit migrate_todo evidence for unsupported constructs.
- Run `garnet check`, tests, and dogfood readiness before trust.
- Require human audit before unquarantine.
- Keep deterministic converter output authoritative where deterministic support exists.

## Provider-Backed Assist Options

Provider-backed adapters should come after the local pipeline is stable. The
current feasibility reporter exposes the same ten options as a machine-readable
advisory-only registry with provider-backed conversion disabled, source omitted
by default, privacy review required, and human approval required:

| Option | Best role | Why consider it | Caution |
| --- | --- | --- | --- |
| OpenAI GPT-5.5 class models | Deep migration reasoning, policy synthesis, high-quality review | Strong instruction following and code reasoning for multi-file migrations. | Cost and privacy controls must be explicit. |
| Anthropic Claude Opus/Sonnet class models | Long-context planning, careful rewrite review, human-readable handoffs | Strong at codebase-scale reasoning and conservative critique. | Must keep source inclusion opt-in and reviewed. |
| xAI Grok code models | Fast exploratory code review and alternate migration hypotheses | Useful as a second-opinion reviewer when available. | Treat output as advisory until local gates pass. |
| Kimi/Moonshot Kimi K-series | Large-context source understanding and lower-cost batch analysis | Attractive for repo-scale risk inventory and summaries. | Verify API availability, privacy terms, and model behavior before provider integration. |
| Google Gemini/Gemma | Multimodal docs plus code-context review; local Gemma variants for private runs | Good ecosystem breadth and possible local/private variants. | Avoid mixing marketing-site claims with unproven conversion. |
| DeepSeek coder models | Cost-sensitive code translation drafts and risk extraction | Useful for cheap batch advisory passes. | Needs strict hallucination and security gates. |
| Qwen coder models | Multilingual codebase understanding and local/open-weight paths | Good coverage across languages and deployment options. | Must be benchmarked on Garnet-specific truth. |
| local 1.58-bit models | Private-code advisory summaries on developer machines | Useful bridge for proprietary codebases where source cannot leave local hardware. | Quality may be uneven; use for triage, not authority. |
| Domain-fine-tuned Garnet adapter | Garnet-specific syntax and policy reconstruction | Best long-term quality if enough verified Garnet examples exist. | Only after the language and conformance suite stabilize. |
| Multi-model reviewer quorum | Cross-check migration plans before candidate output | Helps separate consensus risks from one-model style bias. | Expensive and slower; still needs deterministic gates. |

## Platform Productization Path

1. Merge green Studio/advisory slices before starting new distribution claims.
2. Keep macOS Studio local-first until Developer ID signing, notarization, and Gatekeeper evidence are complete.
3. Use a cross-platform shell around the CLI for Windows/Linux Studio. A Tauri/PWA shell is the recommended first MVP because it reuses the docs/PWA and CLI surfaces instead of porting SwiftUI.
4. Keep SwiftUI macOS Studio as the native Apple reference app.
5. Build Windows/Linux handoffs so Codex Desktop on Windows and Claude Code on Windows can divide work cleanly.
6. Add provider adapters only after the advisory pipeline, privacy boundaries, and dogfood gates are stable.

## Current Truth

- Active converter lanes remain Rust, Ruby, Python, and Go.
- Advisory planning can expand now without claiming conversion.
- Native-boundary languages should be labeled honestly.
- Provider-backed LLM conversion remains inactive.
- Wasm/LLVM/native lowering is a planned architecture, not an implemented backend.
- The public site should sell the product on the landing page and move detailed caveats to a status/readiness page.
