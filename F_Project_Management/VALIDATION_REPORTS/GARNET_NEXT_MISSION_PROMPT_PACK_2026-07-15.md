# Garnet next-mission prompt pack — 2026-07-15

Use this pack after reading `Garnet-Launch-Preparation-State-of-the-Union-v1-2026-07-15.html`. Replace bracketed inputs, start every implementation lane from authenticated `origin/main`, keep worktrees disjoint, and require Jon to merge. A green result is evidence, never self-issued authority.

## Model and credit allocation

- **Sol High or Extra High, Standard:** architecture, governance design, security reconciliation, and final synthesis.
- **Sol Ultra, Standard:** only for genuinely parallel audits or the final multi-lane launch review. Do not make it the default.
- **Terra Medium or High, Standard:** implementation, tests, debugging, bounded PRs, and mission heartbeats.
- **Luna Light or Medium, Standard:** evidence extraction, status normalization, repetitive documentation, and reporter regeneration.
- **Fast mode:** reserve for short latency-sensitive interactions. Long builds and tests do not become cheaper when model credits burn faster.
- **Max effort:** reserve for one unsplittable proof or architecture problem; use Ultra when the work truly divides.

## P0 — Reconcile the Mac Air assessment

**Recommended:** Sol Ultra, Standard.

```text
You are the independent reconciliation lead for Garnet. Read the current exact-main State of the Union at [REPORT_PATH], the MacBook Air report at [AIR_REPORT_PATH], CURRENT_STATE.md, the launch and MIT readiness reporters, and the active mission state. Do not assume that either report's percentages or “three thin areas” are current.

Produce one evidence-ranked reconciliation: agreements, contradictions, stale claims, the exact three language gaps (if the Air evidence supports them), launch blockers versus research debt, and a single frozen backlog. Every conclusion must name a file, command, or captured external fact. Preserve separate measures for bounded-mission completion, committed-truth pulse, and launch gates. Do not edit code. End with proposed phases, acceptance commands, dependencies, and items requiring Jon's decision.
```

## P1 — Close governance without an endless hardening spiral

**Recommended:** Sol High/Extra High for design; Terra High for each bounded implementation slice.

```text
Resume Garnet governance from exact authenticated main [EXPECTED_SHA]. Treat GOV-001–008 as preactivation structural controls, not as a completed no-self-grading boundary. Build only the frozen closure sequence: authenticated and paginated GOV-009 transport; fail-closed repository/head/latest-attempt/freshness/outcome/ruleset/no-bypass equality; semantic producer binding; immutable external-action pins; trusted old-base execution; and a separate human 31→32 activation ceremony.

Use ops/mission state and journal heartbeats. One concern per PR, target <=400 changed lines, add negative tests before changing policy, and never activate context 32 before its producer is live and independently observed. Candidate data is inert input; base code supplies policy. Jon alone merges and changes the live ruleset. Stop when the named sequence is proven—do not invent GOV-010+ without a reproduced launch-relevant defect.
```

## P1-V — Adversarial governance verifier

**Recommended:** fresh Sol High/Extra High, Standard, with no author context beyond the PR and contract.

```text
Attempt to make this Garnet governance PR pass while violating its stated invariant. Test missing/duplicate pages, stale or wrong head, reruns, split attempts, disabled workflows, wrong App, mutable actions, skipped/no-op proof steps, candidate-modified validators, API errors, truncated output, and mismatched live ruleset or bypass state. Prefer executed counterexamples over code-reading claims.

Return CONFIRMED, PARTIAL, or REFUTED for every invariant, with exact commands and artifacts. Do not repair the implementation. A green happy path is insufficient; recommend merge only when the strongest counterexample fails closed and the evidence was not authored solely by the candidate.
```

## P2-A — Finish the live W-PLAY browser lane

**Recommended:** Sol Ultra, Standard for decomposition; Terra High for implementation slices.

```text
Execute the bounded W-PLAY browser mission from [EXPECTED_SHA]. Preserve WV-4 as Studio/frontend proof and WV-5 as real Wasm+Node proof; do not reopen their claims. Implement Wasm check_source and capability-surface/diff exports, a reproducible browser package, docs/playground/live.js, the real browser runner, and Playwright proof for successful execution plus fail-closed undeclared authority. Keep browser claims separate from Node claims until the browser proof exists.

Drive the launch reporter from remaining to pass only through committed machine evidence. Use one primary build OS, then replay the frozen package on the second desktop OS; run targeted Linux regression unless browser-specific evidence requires more. Synchronize the public front door and Studio only after the executable path is real. Jon merges every PR.
```

## P2-B — Build the Minimum Shelf and MCP path

**Recommended:** Sol High for contract; Terra High for implementation.

```text
Build Garnet's Minimum Sealed Shelf as a narrow launch product, not a public registry platform. Freeze Core Ring Tier 1 criteria, implement the minimum MCP stdio path (initialize, tools/list, tools/call, deterministic errors, Garnet envelope), and ship one excellent local sealed package demo. Prove both success and reject-without-valid-seal behavior. Add a deterministic shelf reporter; keep network registry, broad ecosystem, and runtime-enforcement claims out of scope.

Use disjoint worktrees and bounded PRs. Every accepted claim must be generated from the implementation or reporter. Finish with a human acceptance checklist and a Studio/front-door integration handoff.
```

## P3 — Close the reconciled language thin areas

**Recommended:** Sol Extra High for semantics; Terra High for test-first vertical slices.

```text
Using the reconciled Air/local gap matrix at [RECONCILIATION_PATH], select exactly one narrow vertical slice from each approved thin area. For every slice, define supported syntax and semantics, explicit non-goals, negative conformance cases, parser/checker/interpreter/runtime coverage, diagnostics, and the public wording unlocked by the result. Do not convert research debt into a launch blocker unless an advertised launch behavior depends on it.

Prefer end-to-end slices over breadth: for example one trait+impl+generic path, one bounded actor-to-runtime path, or one safe-mode precision gap. Keep deferred NLL, full Rust-equivalence, production ARC, and broad soundness claims fenced until their actual gates close.
```

## P4 — MIT-ready archive and final launch audit

**Recommended:** Sol Ultra, Standard; use a fresh verifier lane and the formal security scan only after the candidate freezes.

```text
Audit the frozen Garnet launch candidate at [EXPECTED_SHA] for MIT review and public release. Recompute all truth on that exact clean commit. Reconcile README, /why, status, stdlib, FAQ, mini-spec, Rust baseline, MIT origin summary, conformance matrix, security boundaries, and launch ledger. Archive or clearly supersede stale narrative material without deleting authoritative proof provenance.

Run the formal deep security scan, secret/license/generated-artifact checks, action-pin and workflow-policy validation, cross-platform artifact replay, live-browser proof, shelf proof, installer/signing/notarization decisions, and a fresh independent S114/governance delta. Produce a reviewer packet with reproduction commands and known limits. Do not tag, release, publish, or declare FIRE; present the evidence to Jon for the final human decision.
```

## Compact task-router prompt

```text
Classify this Garnet task before acting: architecture/audit → Sol High+ Standard; parallel final audit → Sol Ultra Standard; implementation/debugging → Terra Medium/High Standard; extraction/report regeneration → Luna Light/Medium Standard. Use the lowest effort that can safely satisfy the gate. State the exact base SHA, authority source, acceptance command, evidence destination, and human-only action. If those are missing, stop and frame them before coding.
```

Official operating references: [Codex models](https://learn.chatgpt.com/docs/models), [speed modes](https://learn.chatgpt.com/docs/agent-configuration/speed), and [pricing](https://learn.chatgpt.com/docs/pricing). The public speed page confirms higher credit consumption for Fast mode but does not currently establish a numeric GPT-5.6 multiplier; do not invent one.
