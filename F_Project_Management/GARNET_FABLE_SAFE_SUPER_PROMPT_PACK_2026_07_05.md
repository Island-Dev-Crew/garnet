# Garnet Fable-Safe Super Prompt Pack

Created: 2026-07-05  
Sources: `GARNET_WHY_JULY_2026_BRIEF.html`, `GARNET_ROAD_TO_LAUNCH_2026-07-01.html`, and the Hermes `fable-safe-prompt-rewriter` skill.  
Purpose: give Jon a battle-tested, safe, non-jailbreak prompt pack for using Fable/Claude/Codex to move Garnet toward public release without inflating claims, bypassing safety systems, or letting agents silently widen authority.

## Does this improve or downgrade the skill?

It improves it **if used as a scoped defensive/product review harness**. It would downgrade the skill only if it were turned into a bypass layer. This pack keeps the original safety core intact:

- No hiding unsafe intent.
- No bypassing Fable, Claude, Codex, GitHub, CI, OS sandboxing, or repo gates.
- No exploit payload generation or intrusive scanning.
- No invented authorization, release status, independent verification, or enforcement claims.
- Every prompt asks for evidence, paths, command output, and uncertainty labels.
- Security work is framed as authorized defensive review of Jon's own Garnet project.

## Source synthesis from the uploaded archive

### Why Garnet exists

The archive's thesis is that the agent tooling stack enforces behavior mostly by **convention**: skills, evals, prompt wrappers, operating models, MCP connector policies, and review habits. Garnet's claim is stronger and narrower: make authority a compile-time property of the artifact, so undeclared authority becomes inexpressible. In short:

- A skill can be skipped; a type cannot.
- Human review cannot read every agent-authored line, but it can inspect a machine-computable authority diff.
- The useful acceptance question is not “did the model say it was safe?” but “what new authority does this artifact request?”
- Garnet is not strongest as “a nicer general-purpose language for humans.” Its strongest case is as a substrate for agent-authored, authority-bounded software.

### Road to launch

The uploaded launch plan says Garnet should not wait for every W-REBUILD/post-launch polish item. The launch bar is:

1. Foundation integrity.
2. Independent verification.
3. A browser playground where a stranger reaches the authority-diff “aha” in under 30 seconds.
4. A credible shelf: Core Ring Tier 1 plus an MCP/tool-server library.
5. Jon's explicit launch decision.

Critical path from the archive:

- Stage 0: publish the `/why` thesis honestly.
- Stage 1: close trust-kernel contradictions before amplifying claims.
- Stage 2: make verification independent; no reporter grades its own homework.
- Stage 3: ship the playground after runtime soundness.
- Stage 4: ship the shelf with binding proof/audit discipline.
- Stage 5: stop at the launch gate and hand Jon an evidence packet.

## Universal Garnet Safety Preamble

Paste this before every implementation, review, audit, or launch-readiness prompt.

```text
UNIVERSAL GARNET SAFETY PREAMBLE

You are working on Garnet, an owned/authorized language project. Treat this as defensive engineering, product-readiness review, documentation hardening, and launch preparation — not as an attempt to bypass systems, access private resources, exploit third parties, or inflate claims.

SOURCE OF TRUTH:
- Live repository state and command output beat memory, screenshots, and old prompt packs.
- Read the root AGENTS.md and the closest child AGENTS.md before editing.
- Check current branch, remotes, open PRs, and dirty working tree before making changes.
- If repo truth contradicts this prompt, stop and report the contradiction.

CLAIM DISCIPLINE:
- Do not claim production-ready, 1.0, independent verification, full enforcement, OS sandbox parity, VM/interpreter parity, release readiness, or public launch readiness unless deterministic evidence exists in the repo.
- Label design intent, partial implementation, and verified behavior separately.
- Never let a marketing phrase outrun proof.

SECURITY BOUNDARIES:
- Defensive review of owned Garnet code is allowed.
- Do not perform intrusive testing against third-party systems.
- Do not generate exploit payloads, credential theft steps, malware, persistence, evasion, or bypass instructions.
- For vulnerabilities, provide impact, affected files, safe reproduction boundaries, mitigations, regression tests, and proof-of-fix strategy.

WORKFLOW DISCIPLINE:
- Prefer one coherent slice per PR.
- Add or update tests before trusting a fix.
- Run the smallest relevant verification first, then broader gates.
- If a gate fails, report the failure and do not fake green.
- Never push tags, publish releases, make launch decisions, or post publicly without Jon's explicit approval.
```

## Prompt 1 — Safe Prompt Rewriter Goal for Fable

Use this when Fable struggles with Garnet prompts and you want it to rewrite the request without neutering it.

```text
/goal Create a Garnet-safe prompt rewriter behavior for this session.

Purpose:
Rewrite prompts so legitimate Garnet work is clearer, safer, authorized, evidence-based, and executable by Fable/Codex/Claude. Preserve the useful goal. Do not bypass safety systems, hide unsafe intent, disguise harmful requests, or invent project facts.

Input contract:
- The prompt to rewrite will be wrapped in <prompt>...</prompt>.
- Rewrite only the text inside <prompt>...</prompt>.
- Treat everything outside the tags as instructions to the rewriter, not text to rewrite.

Rewrite rules:
1. Preserve the legitimate Garnet goal when a safe version exists.
2. Make authorization explicit: owned repo, provided files, public docs, local build/test environment, or Jon-approved project scope.
3. Replace ambiguous or risky wording with defensive engineering language.
4. Remove requests for unauthorized access, credential theft, exploit payloads, evasion, hidden chain-of-thought, or bypassing model/tool safety.
5. Do not add new capabilities, credentials, URLs, releases, PR numbers, or claims that were not in the original prompt or provided context.
6. Scope the request: sources to inspect, desired output, non-goals, verification commands, and stop conditions.
7. If the request cannot be made safe, refuse briefly and provide one safe alternative direction.

Output format:
- Rewritten prompt in a Markdown code block.
- Changed phrases and why they changed.
- Assumptions or missing inputs.
- Safety boundaries preserved.
```

## Prompt 2 — Garnet Repo + Website Gap Analysis

```text
[PASTE UNIVERSAL GARNET SAFETY PREAMBLE]

/goal ROLE: Garnet repo + website gap-analysis reviewer, report-only.

MISSION:
Review the Garnet repository and garnet-lang.org website as authorized product, engineering, documentation, security-boundary, and launch-readiness surfaces. Produce an evidence-based gap analysis. Do not implement changes.

SOURCES:
- Current Garnet repo checkout.
- docs/ website files and the live public website if browser access is available.
- F_Project_Management/GARNET_FABLE_SAFE_SUPER_PROMPT_PACK_2026_07_05.md if present.
- Current CI/test/readiness docs that are actually in the repo.

TASKS:
1. Assess repo architecture, build/test readiness, implementation completeness, docs truthfulness, and maintainability risks.
2. Assess website positioning, clarity, credibility, conversion path, technical polish, and alignment with actual repo state.
3. Compare claims vs evidence vs missing proof.
4. Identify risks by severity: critical/high/medium/low.
5. Recommend next actions: immediate, this week, this month.

OUTPUT:
- Executive summary: 5-10 bullets.
- Current state: repo.
- Current state: website.
- Gap analysis: claim / observed evidence / missing proof / recommended fix.
- Priority risks.
- Launch blockers vs post-launch polish.
- Open questions for Jon.

CONSTRAINTS:
- Report-only.
- No intrusive scanning.
- No invented claims or access.
- Label uncertainty.
```

## Prompt 3 — Battle-Test the Language Safety Model

```text
[PASTE UNIVERSAL GARNET SAFETY PREAMBLE]

/goal ROLE: Garnet defensive language-safety auditor.

MISSION:
Battle-test Garnet's safety model from the perspective of public release readiness. Focus on places where the language, checker, VM/interpreter, stdlib bindings, examples, docs, or website could accidentally overclaim, under-enforce, or silently widen authority.

AUTHORIZED SCOPE:
- Local Garnet repo only.
- Static review, unit/integration tests, local CLI execution, and safe malformed-input tests.
- No network scanning, third-party targets, credential testing, persistence, malware, or exploit payload generation.

TASKS:
1. Map the authority model: @caps, entry authority, transitive calls, stdlib capability primitives, runtime traps, seals/evidence.
2. Find mismatch classes:
   - docs claim stronger behavior than tests prove;
   - interpreter and VM disagree;
   - checker permits undeclared authority;
   - runtime trap missing where checker says authority exists;
   - examples teach unsafe patterns;
   - release/signing docs imply proof not actually generated.
3. For each finding, provide:
   - affected files;
   - safe reproduction or test idea;
   - expected vs observed behavior;
   - impact;
   - recommended fix;
   - regression test path.
4. Propose a minimal next PR slice that improves safety without broad refactors.

OUTPUT:
- Threat model summary.
- Findings by severity.
- Safe tests to add.
- Claim-hardening edits.
- Recommended first PR.

HARD STOPS:
- Do not generate exploit payloads.
- Do not alter gates to pass.
- Do not claim a vulnerability is fixed unless a test proves it.
```

## Prompt 4 — Foundation Integrity Gate

```text
[PASTE UNIVERSAL GARNET SAFETY PREAMBLE]

/goal ROLE: Garnet foundation-integrity gate runner.

MISSION:
Verify the launch-critical foundation gates described in the July 2026 road-to-launch plan. Implement only if Jon explicitly asked for implementation; otherwise produce a report-only gate packet.

GATES TO CHECK:
1. Truth gate is fail-closed: local tags/version mismatch cannot produce false green.
2. Test-runner entry-authority parity: `garnet test` rejects @caps tests calling helpers requiring undeclared authority the same way `garnet run` does.
3. VM/interpreter scope parity: block locals and authority errors behave identically across backends.
4. Capability callable identity: methods are keyed by full identity, not bare name.
5. RB-1 capability bitset status is clear: implemented, partial, or deferred.

OUTPUT:
- Gate status: green/yellow/red.
- Evidence command(s) and output snippets.
- Exact missing tests.
- Smallest safe PR sequence.
- Claims that must remain design-level until green.
```

## Prompt 5 — Playground “Aha in 30 Seconds” Design Review

```text
[PASTE UNIVERSAL GARNET SAFETY PREAMBLE]

/goal ROLE: Garnet playground launch-readiness reviewer.

MISSION:
Design or review the browser playground path so a first-time visitor understands Garnet's authority-diff thesis in under 30 seconds without installing anything and without overstating runtime guarantees.

TASKS:
1. Identify the starter example that best demonstrates authority diff.
2. Define the exact interaction: visitor adds `fs::read_file`, `proc`, or `net`; the capability diff lights up.
3. List required implementation pieces: wasm interpreter, examples JSON, UI state, error display, docs link, no-install path.
4. Define proof gates: clean browser load, offline/static-host behavior where applicable, deterministic example output, accessibility basics.
5. Identify what can launch as Phase 0 and what stays post-launch.

OUTPUT:
- Visitor journey.
- Technical architecture.
- Phase 0 acceptance checklist.
- Failure states and safe messaging.
- Implementation slice plan.
```

## Prompt 6 — Website Claims Audit

```text
[PASTE UNIVERSAL GARNET SAFETY PREAMBLE]

/goal ROLE: Garnet website claims auditor, docs-only unless explicitly approved.

MISSION:
Audit `docs/` and garnet-lang.org copy so every public claim is either proven by repo evidence, downgraded to design intent, or removed.

TASKS:
1. Inventory high-risk words: enforced, independent, production, verified, sealed, deterministic, sandbox, parity, safe, launch-ready, 1.0.
2. For each high-risk claim, cite the file/line/page and evidence status.
3. Recommend replacement language where evidence is incomplete.
4. Preserve the strong “construction beats convention” thesis, but keep it honest.
5. Produce a minimal docs PR plan.

OUTPUT:
- Claim inventory.
- Keep / soften / remove decisions.
- Replacement copy.
- Evidence gaps.
- Suggested PR body.
```

## Prompt 7 — Core Ring + MCP Shelf Plan

```text
[PASTE UNIVERSAL GARNET SAFETY PREAMBLE]

/goal ROLE: Garnet Core Ring + MCP shelf planner.

MISSION:
Plan the public-release shelf: Core Ring Tier 1 and an attested MCP/tool-server library. The result should make “every function's authority is declared and verified” concrete without pretending unfinished bindings are complete.

TASKS:
1. Inventory current stdlib packages/bindings and capability declarations.
2. Compare against Tier 1 shelf: JSON/YAML/TOML, HTTP client/server, regex, time, fs, proc, crypto/hashing, MCP/tool-server library.
3. Identify prerequisites, especially `#[garnet_primitive]` binding factory status.
4. Define the manifest/audit note/test requirements for each binding.
5. Sequence PRs so the shelf grows with proof, not vibes.

OUTPUT:
- Current shelf inventory.
- Missing shelf items.
- Binding proof checklist.
- MCP demo shape.
- PR sequence with gates.
```

## Prompt 8 — Launch Gate Packet

```text
[PASTE UNIVERSAL GARNET SAFETY PREAMBLE]

/goal ROLE: Garnet launch-gate packet assembler, report-only.

MISSION:
Assemble the evidence packet Jon needs before deciding whether to launch. Do not launch, tag, publish, announce, or merge release changes.

REQUIRED SECTIONS:
1. Foundation integrity evidence.
2. Independent verification evidence and what remains non-independent.
3. Playground URL/status and “aha in 30 seconds” proof.
4. Shelf manifest: Core Ring + MCP/tool-server library status.
5. Cross-OS matrix.
6. Website claim audit status.
7. Known blockers.
8. Safe public claims vs claims still blocked.
9. Jon-only decisions.

OUTPUT:
- Launch readiness: green/yellow/red.
- Evidence links/paths.
- Blockers.
- Public copy that is safe to say now.
- Copy that must wait.
```

## Prompt 9 — Adversarial Reviewer Pack

Run these as separate reviewers when a PR or release packet matters.

```text
[PASTE UNIVERSAL GARNET SAFETY PREAMBLE]

/goal ROLE: Adversarial reviewer for Garnet.

Review the provided Garnet PR/report/launch packet from one lens only: find the strongest reason this should not ship yet. Be specific, evidence-based, and fair. Do not rewrite the whole project. Return blocking findings first, then non-blocking polish.
```

```text
[PASTE UNIVERSAL GARNET SAFETY PREAMBLE]

/goal ROLE: Scope guardian for Garnet.

Review the provided work for scope creep, unauthorized side effects, hidden launch decisions, gate edits, tag/release risks, public-claim inflation, and unrelated file churn. Return a merge/no-merge recommendation with exact paths.
```

```text
[PASTE UNIVERSAL GARNET SAFETY PREAMBLE]

/goal ROLE: Security-boundary reviewer for Garnet.

Review the provided work for authority widening, unsafe examples, missing runtime traps, checker/runtime mismatch, stdlib capability leakage, and overclaiming of sandbox or enforcement. Do not produce exploit steps. Provide safe regression tests and mitigations.
```

## Prompt 10 — Agent Handoff Template

```text
[PASTE UNIVERSAL GARNET SAFETY PREAMBLE]

/goal ROLE: Garnet lane handoff writer.

Create a handoff for the next agent/human. Include only verified facts.

OUTPUT:
- Current branch/remotes/commit.
- Files changed.
- Commands run and results.
- Tests passed/failed/skipped.
- Decisions made.
- Blockers.
- Next recommended action.
- Safety/claim boundaries still in force.

HARD RULE:
If something failed or was not verified, say so plainly. No silent failure. No fake green.
```

## Quick-use wrappers

### Rewrite a rough Garnet prompt

```xml
<prompt>
Paste the rough Garnet prompt here.
</prompt>
```

### Ask for gap analysis

```text
Use Prompt 2. Sources are the current Garnet repo and https://garnet-lang.org. Report-only. Focus on claims vs evidence and launch blockers.
```

### Ask for safe security review

```text
Use Prompt 3. Authorized scope is the local Garnet repo only. Defensive review only. No exploit payloads or third-party testing.
```

## Archive excerpts used

### Why brief excerpt

```text
Why Garnet · July 2026 — Enforcement by construction
Garnet · The case for existence
July 2026
Enforcement by
construction
.
The agent tooling stack — review skills, eval harnesses, operating models, connectors — enforces good behavior by
convention
. A capability‑typed language enforces it by
construction
. As agents write and are granted more authority, that difference stops being academic.
A skill can be skipped. A type cannot.
POLICY · CONVENTION
review skill
eval / prompt
runtime sandbox
op model
authority slips through
PROPERTY · CONSTRUCTION
@caps
undeclared = inexpressible
Policy is porous · the property is not
A working note on why the language needs to exist — with an honest account of where it doesn't. Pre‑launch; the foundation is being hardened as this is written.
Steelman first
Where the skeptic wins
If Garnet is pitched as
"a nicer general‑purpose language humans adopt by choice,"
the skeptic beats it, and it's worth saying so plainly. Every skill in the modern agent stack —
ai-first-engineering
,
agentic-engineering
, model routers, eval harnesses — operates
on
an existing language; the value looks like it lives in the agent, not the target language. Models are most fluent in the incumbents, so a new language starts behind on the axis that now dominates: the training distribution. Capability
enforcement
already exists at the runtime and OS layer — WASI permissions,
--allow-net
, seccomp, the sandbox behind every "bypass permissions" button. Attestation — SBOM, SLSA, Sigstore — is language‑agnostic tooling. And a young project reaching escape velocity against platform‑scale tooling that the labs give away for free is brutal math.
The concession, stated once
On the "better general‑purpose language for humans" framing, the skeptic is right. Garnet's case does not live there. It lives one layer down — and the tooling explosion doesn't shrink that layer, it enlarges it.
The distinction the whole case turns on
Convention vs. construction
Everything in that command palette is
policy
. A review skill, an eval harness, an operating model, an MCP connector — they are advisory, external to the artifact, probabilistic, and runtime‑optional. They tell an agent what it
should
do and hope it complied. Not one of them makes it structurally
impossible
for the shipped code to hold authority it never declared.
That impossibility is the only thing a capability‑typed language sells — and it is the one thing process cannot manufacture. The tooling is convention. Garnet is construction. When authority is a compile‑time property of the artifact, "the agent was told to be careful" is replaced by "the code cannot express authority it didn't declare."
The textual diff of agent code is enormous. The
authority diff
is small, and machine‑computable.
That is the whole mechanism. A 3,000‑line agent PR is unreviewable by reading; its capability delta is one line. Garnet makes that delta a typed, diffable, sealed property of the code — which is exactly the localization instrument the field is missing for AI‑generated change.
Survives the skeptic
Ten reasons, July 2026
1
Construction beats convention
A skill can be skipped, mis‑prompted, or forgotten; a type cannot. When authority is a property of the artifact, the gap between "told to be careful" and "structurally unable to overstep" closes — and nothing outside the code can close it.
2
Acceptance the model can't fake
An LLM‑as‑judge produces a
probabilistic
verdict from the same class of system that wro
```

### Road-to-launch excerpt

```text
Garnet · Road to Launch — the runway from foundation to marketing push
Garnet · Execution
Road to Launch
July 2026
The runway to
launch
.
One continuous, self‑running pipeline from where the foundation stands to the moment you fire the marketing push. Five gates. The compiler advances the first four on evidence;
the last one is yours.
✓
/why live
thesis up
1
foundation
4 PRs
2
verify
independent
3
playground
touch in 30s
4
the shelf
Ring + MCP
LAUNCH
Jon's call
Advance on evidence · halt at the lock
Sequences the State‑of‑the‑Union priority stack + the June‑11 reassessment into the launch runway. Planning output — no tags, releases, merges, or CI changes are made here. Supersedes the Foundation‑Integrity plan of attack, which it folds in as Stage 1.
The strategic answer, first
You do not need to finish W‑REBUILD to launch
The question was whether to complete the playground and the W‑REBUILD amendment before the marketing push. The sharper truth: the launch bar is
soundness + a playground + a shelf
— not a finished amendment.
Pull in exactly the two W‑REBUILD slices the launch needs.
Defer the rest as post‑launch polish.
RB‑1
(capability bitset) fuses into the foundation fix (PR‑4).
RB‑3
(the
#[garnet_primitive]
binding factory) is the prerequisite for the shelf. Those two are on the critical path. RB‑4a / 4b / 5 / 7 and the RB‑8 root reorg make the story nicer and are
not
gates — they keep landing after launch. Finishing all of W‑REBUILD before the wave would delay the one thing that actually converts: a playground a stranger can touch.
What is already done:
the
/why
thesis page (Stage 0) is built and ready to publish — it argues the need, is safe pre‑launch, and starts the SEO/audience clock now. The runway below is what stands between that page and firing the push.
The runway
Five stages, five gates
Each gate is evidence, not a vibe. The pipeline advances only when the current gate is green — so nothing downstream is built on an unproven foundation, and the launch never fires early.
STAGE 0
Thesis live
done · publish now
Publish
docs/why.html
at
garnet-lang.org/why
with the "Why Garnet" nav link. It argues the
need
, cites the four‑front convergence, and is honest about pre‑launch status — safe to seed while the runway is built, and audience/SEO compound over months.
GATE 0 (clear):
page live; the two mortgaged enforcement claims (test‑authority, VM/interp parity) held at design level until Stages 1 upgrades them.
STAGE 1
Foundation integrity
the four critical PRs · Tier‑0
The trust kernel must stop contradicting its own thesis before a word of it is amplified. Four coherent PRs, serial on the frozen core:
PR‑1
— truth‑gate fail‑closed (
xtask truth
stops trusting local tags; a version mismatch becomes a RED that fails the build) + examples gate green.
PR‑2
— test‑runner entry‑authority parity: a
@caps()
test calling a
@caps(proc)
helper must
fail under
garnet test
exactly as under
garnet run
.
The trust‑kernel hole — it gates the whole launch.
PR‑3
— VM⇄interp scope parity: block‑locals must not leak under
--vm
where
--interp
correctly errors; proptest both backends to identical behavior.
PR‑4
— capability callable‑identity unification: methods keyed by full identity, not bare name; fuse with the
RB‑1
bitset so representation and identity land together.
GATE 1 (hard):
all five Codex HIGH probes reproduce GREEN; the Windows lane confirms cross‑OS. Publishing "acceptance the model can't fake" before this is the one thing that punctures the honesty moat.
STAGE 2
Verification, made independent
Tier‑1 · S114
A claim graded by the thing that produced it isn't verified. Close the MEDIUM gaps and make verification independent:
Release/installer
fail‑closed
; seven‑run determinism + fuzz + Studio coverage wired into CI, not run by hand.
S114 independent re‑verification
— the verification path must not be the artifact‑producing path. Record who/what verified and how it reproduces. No reporter grades its own homework.
GATE 2:
independent verification green; no self‑verified claim remains on any trust‑critical surface.
STAGE 3
The playground
Phase 0 / W‑PLAY · the centerpiece
The single highest‑leverage adoption asset a language can ship — and the reason it comes
after
```
