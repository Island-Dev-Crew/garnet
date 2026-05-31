# Garnet Deep Gap Analysis, Source-of-Truth PRD, and Six-Agent Plan

Date: 2026-05-26
Status: execution-ready planning artifact
Scope: Garnet website, repository, documentation corpus, library story,
positioning, promo-video lane, and agent execution plan.

## Executive Read

Garnet's public website is ahead of the repo in readability. The site is clean,
coherent, and enjoyable to read. The repository is much stronger than a normal
prototype repo in contracts, tests, and evidence, but it still asks a new reader
to cross too many historical layers before they understand what is current.

The strategic fix is not to flatten the thesis into "Rust rigor, Ruby velocity."
That line is useful as a bridge, but it invites direct comparison to Swift,
Kotlin, Crystal, Gleam, Mojo, and Zig. The stronger position is:

> Garnet is an evidence-native language workbench for agentic systems: code
> declares authority, memory kind, trust boundaries, and provenance as first
> order program structure.

Use Rust/Ruby/Swift as comparators inside the explanation, not as the headline
identity.

## Source-of-Truth State

Current machine-readable reporters run from this checkout:

- `scripts/garnet_adoption_surface_status.py`
  - Headline: "Rust rigor, Ruby velocity, agent-native dogfood evidence."
  - Active converter lanes: Rust, Ruby, Python, Go.
  - Advisory-only lanes: JavaScript, TypeScript, Swift, Java, C, C++, C#,
    Perl, Kotlin, Shell, SQL, Other.
  - Native-boundary-first lanes: C, C++, Objective-C, Assembly, CUDA,
    platform-specific code.
- `scripts/garnet_mit_readiness_status.py`
  - Overall status: active-partial.
  - Objective completion observed after promo evidence sync: 80.6%.
  - Tracked implementation plan remains complete at 87/87 slices.
- `scripts/garnet_promo_video_status.py`
  - Current reporter status after this pass: public-site-embedded, 95.0%.
  - Desktop render, visual-QA, website-export, and public-site sync evidence
    are present; human/aesthetic acceptance remains the open promo gate.

That creates the first urgent consistency rule:

> Site, README, and deck copy must cite the reporter outputs, not hand-maintained
> numbers, unless the hand-maintained copy is regenerated in the same pass.

## Comparative Website and Repo Analysis

Rust sets the best mature-language pattern: its front page states one crisp
promise, then immediately separates performance, reliability, productivity,
domain use cases, production proof, learning, contribution, governance, and
tooling. The repo mirrors that with compiler, library, tests, governance, and
contributor docs clearly visible.

Ruby sets the trust pattern for human warmth: the website foregrounds beauty,
approachability, documentation, community, tools, RubyGems, RDoc/RI, RBS,
TypeProf, and the lineage of the language. Garnet can learn from Ruby's humane
voice, but must avoid Ruby-style dynamic-language expectations unless managed
mode boundaries are explicit.

Zig sets the "serious systems language, no overclaiming" pattern. Its docs put
the language reference, standard library, testing namespace, build system,
C interop, WebAssembly, targets, and style guide in one densely navigable place.
Garnet's current site has the same seriousness, but the stdlib and language
reference need more navigable depth.

Gleam sets the "friendly typed language plus package ecosystem" pattern. It has
a clear documentation page, package index, and repo identity. Garnet lacks the
equivalent package discovery story because its registry stub and Layer-2 package
plan are not yet public ecosystem surfaces.

## Gaps That Matter Most

| Gap | Severity | Why it matters | Patch direction |
| --- | --- | --- | --- |
| Source-of-truth drift | P0 | Drift was present at audit start: `README.md` and `docs/status.html` showed 78.0%, while the reporter had moved beyond that and the promo evidence bundle was missing locally. | Patched this pass to 80.6% MIT/productization and 95.0% promo public-site-embedded; next step is a generated sync script that fails when values drift. |
| Research-folder readability | P1 | `A_Research_Papers/` mixes paper numbers, addenda, DOCX/PDF/Markdown, spacing, capitalization, and version schemes. | Add an index now; run a dedicated rename/link-migration phase later. |
| Library story is honest but thin | P1 | `garnet-stdlib` has 24 registry primitives, 74 unit tests, CapCaps metadata, and docs. That is a real foundation, not a mature library ecosystem. | Execute S17 stdlib layer policy, grow to about 50 primitives, then S18 official packages. |
| Positioning still leads with comparators | P1 | "Rust rigor, Ruby velocity" is clear but easy to reduce to "Swift/Kotlin/Crystal/Gleam/Mojo already bridge modes." | Promote "evidence-native agentic systems language" and use Rust/Ruby/Swift as proof lineage. |
| Promo lane truth split | P1 | The existing clip is technically present but the reporter needs evidence; the composition itself is too static for a premium landing-page slot. | Improve the HyperFrames composition, render, QA, export, sync, then rerun reporter. |
| Repository onboarding depth | P2 | README is accurate but long; `CURRENT_STATE.md` is massive; new readers need a one-page "what is current" map. | Add a public `docs/current-truth.html` or repo `SOURCE_OF_TRUTH.md` generated from reporters. |
| Library examples | P2 | The stdlib page is a table, not a story of what users can build. | Add example-backed recipes: files, network policy, crypto manifest, memory recall, actor mailbox. |
| Playground stub | P2 | `docs/playground.html` is honest but feels less complete than the rest of the site. | Build static examples first; WASM execution only when proven. |

## Library Story Assessment

Current score: 42/100 for mature-language library expectations, 76/100 for
prototype honesty and safety discipline.

What is real:

- 24 capability-tagged primitive entries.
- 74 stdlib unit tests listed by `cargo test -p garnet-stdlib -- --list`.
- Pure helpers for strings, arrays, and crypto.
- Authority-gated time, filesystem, and network primitives.
- NetDefaults guardrails, sandbox profiles, rate-limit helpers, and capability
  metadata.
- Interpreter bridge and CapCaps checker integration.

What is still missing for maximum value:

- Layer model implemented, not just planned.
- `@stability(...)` on primitives/packages.
- JSON, regex, datetime, UUID, config, structured logging, CLI args, test
  helpers, HTTP client/server, and async primitives.
- Package docs that look like `std` docs, not just registry metadata.
- A package index story comparable to crates.io/RubyGems/Hex.
- Dogfood examples that prove each library lane solves a real user problem.

First-order principle:

> A Garnet library is not "good" because it wraps a useful API. It is good when
> capability authority, memory behavior, stability tier, provenance, sandbox
> status, and test evidence are visible at the same time.

## Positioning Recommendation

Replace only the top-line hook and first explanatory frame; keep most existing
copy as supporting material.

Copy replacement confidence:

- Replace headline/tagline direction: 72%.
- Keep and add alongside existing dual-mode explanation: 88%.
- Rewrite all website copy from scratch: 18%.

Recommended hierarchy:

1. Headline:
   "The evidence-native language workbench for agentic systems."
2. Subhead:
   "Garnet makes authority, memory, mode boundaries, and provenance visible in
   code, so long-horizon agents and humans can build, audit, and evolve the
   same system."
3. Comparator support:
   "Managed mode keeps orchestration fluent. Safe mode gives ownership-sensitive
   paths Rust-like discipline. Swift proves ARC and actor isolation can be
   mainstream. Garnet's unique move is making the boundary, memory, capability,
   and evidence surfaces first-class."
4. Proof strip:
   `@caps`, `memory episodic`, `@safe fn`, ModeAuditLog, signed manifests,
   sandboxed converter output, dogfood readiness.

Avoid leading with:

- "Rust + Ruby" as the identity.
- "LLM-native" as the only novelty.
- "production-ready" language claims.
- broad conversion promises.

## Five Major Problems Garnet Can Solve

1. Agent systems lose track of authority.
   - Garnet solution: `@caps(...)`, transitive CapCaps checks, sandbox defaults,
     and native-boundary wrappers make OS authority visible and auditable.
   - Buildable demo: a file/network automation planner that refuses undeclared
     file or network operations and emits an authority report.

2. Long-horizon agents forget what kind of memory they are using.
   - Garnet solution: `memory working|episodic|semantic|procedural` turns memory
     role into program structure instead of comments or naming conventions.
   - Buildable demo: a support-agent triage program with separate working
     scratch, episodic incident history, semantic policy index, and procedural
     playbook store.

3. Teams split velocity code and safety code across painful language seams.
   - Garnet solution: managed `def` and safe `@safe fn` can live in one source
     file with audited mode-boundary bridging.
   - Buildable demo: an ingestion pipeline where managed orchestration calls a
     safe parser/validator and records every trust crossing.

4. AI-assisted migration creates unaudited, over-trusted code.
   - Garnet solution: converter output starts sandboxed, lineage-backed, and
     human-audit-gated; advisory plans are separated from deterministic
     conversion.
   - Buildable demo: TypeScript advisory handoff packet -> sandboxed Garnet
     candidate -> `garnet check` -> migrate_todo -> human unquarantine path.

5. Language projects overclaim readiness and lose reviewer trust.
   - Garnet solution: dogfood-readiness gates, signed manifests, current-state
     reporters, conformance matrix, and public status split make evidence a
     product feature.
   - Buildable demo: `garnet evidence report` for a project that composes tests,
     caps, dependencies, deterministic build manifest, and docs-contract health.

## C++ Interview Design Filter

The Bjarne Stroustrup interview reinforces five tests Garnet should pass:

- Problem first. Garnet should begin every surface with the problem it solves
  for agentic systems, not with feature inventory.
- High and low levels together. Garnet's equivalent is orchestration fluency
  plus safe/native boundaries, not pretending one mode handles every concern.
- Do not add gun decks without foundation. The library story must expand only
  behind CapCaps, stability, tests, docs, and dogfood.
- Standardization/community before dialects. Garnet needs governance and RFC
  shape before external ecosystem claims become real.
- Abstraction can reduce overhead when compiled away or made explicit. Garnet
  should prove which abstractions are zero/negative overhead only after the VM
  and native backend evidence exist.

## PRD: Source-of-Truth and Library Value Push

### Goal

Make Garnet's site and repository tell one consistent truth, improve the
documentation reading path, and turn the library story from "24 primitives" into
an evidence-backed ecosystem ladder.

### Non-Goals

- Do not rename the entire research corpus in the same PR as copy fixes.
- Do not claim provider-backed LLM conversion.
- Do not claim notarized macOS distribution or mobile distribution.
- Do not claim a mature stdlib before S17/S18 land.

### Requirements

- Add generated or checked current-truth data for readiness, promo, converter,
  and adoption surfaces.
- Add a readable index to research papers and later create a rename migration.
- Update website and README numbers from reporters.
- Reframe top-line positioning around evidence-native agentic systems.
- Improve promo creative and preserve render/QA/export evidence.
- Create library roadmap pages that connect current primitives to S17/S18.

### Acceptance Criteria

- `python3 scripts/garnet_adoption_surface_status.py` and website copy agree.
- `python3 scripts/garnet_mit_readiness_status.py` and README/status/index agree
  on the objective percent.
- `python3 scripts/garnet_promo_video_status.py` reaches public-site-embedded
  only after Desktop evidence exists.
- `A_Research_Papers/README.md` gives a clear reading order.
- `docs/stdlib.html` links current registry, next layer policy, and examples.
- Promo MP4/WebM passes automated QA and has representative frames.
- The final deck imports into Gamma/Canva/PowerPoint-like tools without external
  assets.

## Six-Agent Implementation Plan

Use `python3 scripts/garnet_phase_id.py` before any agent claims a phase id.
Every agent reads root `AGENTS.md` plus the nearest local contract before edits.

| Agent | Ownership | First tasks | Verification |
| --- | --- | --- | --- |
| Mac Opus | Positioning and copy architecture | Rewrite hero hook, README first 60 lines, FAQ positioning answer, status caveat language. | `python3 scripts/garnet_adoption_surface_status.py`; link scan; visual review. |
| Mac Codex | Website/source-truth sync | Add reporter-driven sync/check script; update `docs/index.html`, `docs/status.html`, service worker references if assets change. | `scripts/smoke_garnet_pages_pwa.sh --strict`; `scripts/smoke_garnet_web_pwa.sh --strict`. |
| Windows Opus | Stdlib layer policy | Execute S17 spec: `GARNET_STDLIB_LAYER_POLICY.md`, `@stability` design, primitive layer taxonomy. | `cargo test -p garnet-stdlib -p garnet-check --no-fail-fast`; new layer gate. |
| Windows Codex | Stdlib examples and docs | Add recipe examples, stdlib page expansion, template docs for capability-safe library use. | `cargo test -p garnet-cli --test examples`; docs smoke. |
| Mac Opus 2 | Research corpus normalization plan | Produce rename/link migration PR plan for `A_Research_Papers/`, `D_Executive_and_Presentation/`, historical docs. | `rg` before/after dead-link report; no mass rename without map. |
| Mac Codex 2 | Promo/deck production | Improve HyperFrames/Remotion lane, render/QA/export/sync evidence, final HTML deck. | promo render/QA/export/sync scripts; browser screenshots; deck overflow check. |

## Suggested First PR Sequence

1. Truth sync and positioning patch.
2. Research-folder index and normalization plan.
3. Promo creative/evidence patch.
4. Stdlib layer policy and gate.
5. Stdlib examples/docs.
6. Layer-2 official package bootstrap.

## Confidence Scores

- Current repo is serious enough to present as an evidence-rich prototype: 86%.
- Current site is polished enough to keep as the front door: 82%.
- Current repo docs are easy enough for a first-time reader: 49%.
- Current library story is enough for real user adoption: 42%.
- Positioning should shift away from comparator-led headline: 72%.
- Most existing site copy should be preserved with additions: 88%.
- Promo creative needs material improvement before hero use: 84%.
