# Garnet Launch-Convergence Design

**Date:** 2026-07-09  
**Status:** Approved direction; design ready for Jon review  
**Baseline:** `origin/main` at `9c9ca9e3538e4dd226e9cea356cf7ecd1ba92297`  
**Release posture:** research-grade v0.x; not production or 1.0  
**Launch authority:** Jon only

## 1. Objective

Move Garnet from its current post-foundation state to the smallest honest public
launch bar:

1. a real browser-executed Garnet playground that reveals an authority diff in
   under 30 seconds;
2. a deliberately small, usable, sealed launch shelf with an MCP tool-server
   library and flagship demo;
3. one synchronized account of current truth across the repository, Garnet
   Studio, `garnet-lang.org`, and the launch packet; and
4. a hard stop at the launch lock for Jon's tag, release, and public-posting
   decisions.

After Jon confirms that the launch occurred, the first post-launch deliverable
is a State of the Union HTML report generated from the launched commit. That
report becomes the ordered backlog for any remaining engineering capacity.

## 2. Current Baseline

The design starts from live repository and public-surface recon, not the July 1
planning snapshot.

- W-REBUILD RB-0 through RB-7 is complete.
- The S114 independent re-verification found additional HIGH issues; the fixes,
  residual deny-by-default closure, and anti-rot gates are merged. Jon still owns
  the final acceptance/relabel decision.
- macOS Studio reached the M8 completion dossier.
- Native ARM64 Linux reached L4 through PRs #449-#452.
- The tracked implementation reporter reads 87/87, while the wider
  MIT/productization reporter reads 92.8%; these measure different things.
- The public playground is an honest static gallery, not live execution.
- `https://garnet-lang.org/why` currently returns 404.
- The public site and the running Tauri Studio contain stale Linux/readiness
  wording and some claims broader than the deterministic traps support.
- `docs/truth.json` still records a workspace-test measurement from
  `c4b9e28-dirty`, so matching generated markers do not by themselves mean the
  measurement is current.
- The Core Ring launch shelf and MCP tool-server library are not complete.
- PR #453 is a green, mergeable prompt-pack PR. It is planning material, not
  launch-gate evidence.

The untracked planning files in the shared checkout are proposal input until a
focused PR makes selected content durable repository truth.

## 3. Scope

### 3.1 Launch-critical scope

- Truth and contract convergence.
- Browser WASM execution for pure Garnet programs.
- Browser `check` and `diff-caps` over editable source.
- A minimum sealed launch shelf.
- A real MCP tool-server library plus one flagship demo.
- Public `/why`, playground, status, and landing-page convergence.
- Cross-OS and browser evidence sufficient for the launch packet.
- Post-launch State of the Union generation.

### 3.2 Explicitly post-launch

- Production allocator-integrated ARC and user-payload finalizers.
- Full Core Ring Tier 1 breadth beyond the launch set.
- RB-8 crate de-suffixing/root flattening.
- Apple Developer ID notarization and clean-machine Gatekeeper proof.
- Signed Windows distribution, winget, and Windows ARM64.
- Provider-backed LLM conversion.
- LLVM/native backend implementation.
- Full mechanized proof and external empirical studies.
- Marketplace publication, community-account creation, and broad localization.

These items remain real work. They do not block the selected launch bar.

## 4. Approaches Considered

### A. Evidence-first launch convergence — selected

Complete the live playground, minimum shelf, MCP demo, and truth synchronization;
then stop for Jon. This produces the fastest launch that still demonstrates
Garnet's distinctive utility rather than only describing it.

### B. Full productization before launch — rejected for this runway

Wait for production ARC, notarization, broad distribution, the complete Core
Ring, native backends, and wider research validation. This reduces deferred work
but postpones contact with users and mixes externally blocked tasks with the
engineering critical path.

### C. Site-first preview launch — rejected as the main launch

Publish `/why` and visual improvements before live execution and the shelf. This
starts audience building sooner but repeats the current problem: a strong thesis
without a stranger-accessible proof.

The `/why` page may go live during convergence as a pre-launch thesis page, but
it is not the Garnet launch.

## 5. Dependency Model

```text
Truth Lock
    |
    +--> W-PLAY: wasm target --> browser adapter --> live playground --> 30s proof
    |
    +--> Launch Shelf: inventory --> packages --> MCP library --> flagship demo
    |
    +--> Front Door: /why + claim cleanup + status sync + promo review
                 \             |             /
                  +------ Launch Lock ------+
                              |
                         JON: FIRE/HOLD
                              |
                    Post-launch State of Union
```

Truth Lock is first. W-PLAY and the Launch Shelf may then run in parallel only
when they use separate worktrees and disjoint files. Front-door implementation
can proceed alongside them after the claims ledger is frozen, but final copy
waits for the proving artifacts.

## 6. Workstream A — Truth Lock

### Purpose

Create one durable launch ledger and eliminate contradictions before new public
claims are added.

### Deliverables

1. Track the canonical launch-readiness ledger under project management.
2. Refresh machine truth from the current commit, including a fresh workspace
   test measurement rather than carrying the `c4b9e28-dirty` count.
3. Update readiness reporters for native Linux L1-L4 and current Studio proof.
4. Reconcile `CURRENT_STATE.md`, `CHANGELOG.md`, owning `AGENTS.md` contracts,
   and the public status model.
5. Correct the stale integer-overflow contract: RFC-0002 is implemented for
   checked `+`, `-`, `*`, and unary negation; the explicit wrapping escape hatch
   remains deferred.
6. Record that the panic firewall exists and that `trybuild` remains open.
7. Classify PR #453 as optional planning documentation and resolve it through
   normal review without counting it toward a launch gate.

### Acceptance

- Every PASS in the launch ledger names a command, test, artifact, and commit.
- Reporter outputs, generated truth, and public status agree on completed Linux
  evidence and remaining distribution boundaries.
- No untracked local file is treated as shared truth.
- No claim uses `enforced`, `independent`, `signed`, `sandboxed`, or
  `production` beyond its proving boundary.

## 7. Workstream B — W-PLAY

### Purpose

Turn Garnet's central acceptance argument into something a first-time visitor
can operate in a browser without installing a toolchain.

### Architecture

1. Make the interpreter's browser-facing dependency graph compile for
   `wasm32-unknown-unknown`:
   - enable the narrowly required browser randomness support;
   - remove or feature-gate terminal/backtrace-only `miette` features;
   - keep CLI-only REPL dependencies outside the interpreter.
2. Add a dedicated `garnet-wasm` workspace crate (`cdylib` + `rlib`) with no
   CLI, terminal, filesystem, process, network, or environment dependency. It
   owns the `wasm-bindgen` boundary and exports stable JSON operations:
   - `check(source) -> diagnostics`;
   - `run_pure(source) -> value/diagnostic`;
   - `diff_caps(baseline, proposal) -> machine verdict`.
3. Do not expose browser filesystem, process, network, or environment authority.
   Host-authority examples demonstrate static capability deltas and rejection;
   they are not executed in the browser.
4. Replace the static playground's recorded-output center with a real editor,
   run/check controls, and an authority-diff panel. Preserve the static examples
   as selectable starters and as a fallback when WASM fails to initialize.

### Flagship interaction

The default baseline is a pure tool function. The visitor adds a call that
requires `fs`, `net`, or `proc`. The capability panel changes from no gain to an
authority-expansion verdict and explains that the proposal cannot pass the
acceptance gate unchanged.

### Failure behavior

- WASM initialization failure leaves the static gallery visible and reports the
  failure honestly.
- Parse/check errors never crash the page.
- `run_pure` rejects host-authority programs rather than emulating authority.
- A capability gain is never rendered as accepted because of a UI error.

### Acceptance

- A clean browser can load the playground with no local Garnet installation.
- A pure starter program executes in the browser.
- Editing the proposal produces a real `diff-caps`-equivalent machine verdict.
- Playwright proves the authority-diff aha path in under 30 seconds without
  hidden setup steps.
- Desktop and mobile layouts have no overlap or horizontal overflow.

## 8. Workstream C — Minimum Sealed Launch Shelf

### Purpose

Launch with enough usable surface to build one real tool, without pretending the
entire future package ecosystem is complete.

### Distribution boundary

The launch shelf uses the existing deterministic, filesystem-backed registry
contract and `garnet add --registry <path>`. The source bundle carries the
registry seed and its verified `index.json`; the launch packet may package that
directory as a downloadable artifact after Jon's release decision. The site
must call this a repo-bundled filesystem registry. HTTP registry transport,
authentication, and publishing remain post-launch.

The existing `http-client` seed returns request descriptors only. It is not a
working HTTP client and is not counted as launch-shelf utility.

### Launch set

1. **Serialization package:** JSON plus bounded TOML/YAML support with explicit
   errors and deterministic examples.
2. **MCP tool-server library:** stdio framing, initialization, tool listing,
   tool invocation, structured errors, and a Garnet capability-envelope helper.
3. **Flagship MCP package/demo:** an installable example that uses the library,
   exposes a pure baseline tool, and carries the bounded authority-widening
   rejection scenario.

Existing regex, time, filesystem, process, and crypto primitives are documented
as built-in shelf foundations and are not reimplemented merely to create package
count. Real HTTP client/server transport remains a post-launch shelf expansion.

### MCP honesty boundary

The existing `.mcpcaps` surface is self-declared and reviewable, not MCP-host
enforced. The launch demo earns a stronger statement only where the tool itself
runs as Garnet code under `@caps` enforcement. The library must not imply that
an arbitrary external MCP host enforces Garnet's declarations.

### Per-item evidence

Each launch-shelf item must include:

- runnable documentation and a negative-path test;
- compiler-derived capability manifest;
- dependency/FFI audit note;
- deterministic seal or explicit unsigned-attestation wording;
- dogfood bundle and manifest verification;
- stability classification.

### Flagship demo

Build an attested local MCP tool server with a pure baseline tool and a bounded
filesystem proposal. The demo must show:

1. the baseline checks, runs, and seals;
2. the proposal's authority gain is visible;
3. the unchanged acceptance policy rejects and does not seal the widened
   proposal; and
4. the evidence identifies which layer enforced the decision.

### Acceptance

- A clean source checkout can build and run the server and client smoke.
- The baseline/proposal sequence is deterministic.
- The widening case exits non-zero before seal creation.
- All shelf claims remain within the exact package and host boundaries tested.

## 9. Workstream D — Front Door

### Purpose

Make `garnet-lang.org` lead with Garnet's own category and connect the thesis to
the live proof.

### Information architecture

- H1 remains `Garnet`.
- Supporting thesis becomes `Enforcement by construction`.
- Rust/Ruby remain explanatory comparators deeper in the page, not the primary
  identity.
- `/why` explains the convention-versus-construction distinction with sourced
  external facts or removes unsourced specifics.
- The primary action opens the live playground.
- The secondary action opens the flagship MCP demo and its evidence.
- The status page owns caveats and platform matrices; the landing page links to
  them rather than repeating stale inventories.

### Required corrections

- Remove universal wording such as `no ambient authority, ever`.
- Distinguish compiler-derived manifests from external signatures.
- Distinguish GPG-signed release checksums from unsigned per-program seals.
- Replace `Playground (planned)` with the measured current state.
- Reflect native Linux proof while preserving unsigned/notarization boundaries.
- Update or remove stale dates and test counts.

### Visual and browser acceptance

- Preserve the canonical logo and core brand assets.
- Use real Studio, playground, diff-caps, and evidence surfaces as primary media.
- Verify desktop and mobile widths in the browser.
- Check links, media, console errors, text overlap, and horizontal overflow.
- Human/aesthetic acceptance of the existing promo remains a named Jon gate.

## 10. Workstream E — Launch Lock

### Launch packet

The lead lane maintains one packet containing:

- launch commit and tree state;
- Gate 1/2 foundation and independent-review status;
- live playground URL and 30-second evidence;
- launch-shelf manifest and seals;
- flagship MCP demo reproduction commands;
- macOS, Windows, Linux, and browser matrix;
- public-claim allowlist and named deferred boundaries;
- known defects and recovery/fallback paths;
- promo human-review status; and
- exact release/tag/public-posting actions reserved to Jon.

### Final ladder

For every implementation PR:

1. focused red/green tests;
2. `cargo test --workspace --no-fail-fast`;
3. `cargo clippy --workspace --all-targets -- -D warnings`;
4. `cargo fmt --all -- --check` and `cargo doc` where applicable;
5. agent-contract checks for contract changes;
6. Garnet dogfood-readiness 5/5;
7. browser/Playwright rungs for playground or site changes;
8. sealed, manifest-verified evidence bundle;
9. PR body validation;
10. full remote CI green before the established merge flow.

No PR may modify the gate it merges under without Jon's explicit human-merge
approval. A diff-capability widening blocks merge.

### Hard stop

When the packet is complete, all autonomous work stops. Jon chooses FIRE or
HOLD. No agent pushes a tag, creates a release, publishes to a marketplace,
creates community accounts, or posts the launch wave.

## 11. Post-Launch State of the Union

### Trigger

Generate this artifact only after Jon records that the launch occurred and
names the launched commit/tag. If the decision is HOLD, update the launch packet
instead; do not label any report post-launch.

### Artifact

Create:

`F_Project_Management/GARNET_POST_LAUNCH_STATE_OF_UNION_<YYYY_MM_DD>.html`

The HTML is self-contained, responsive, printable, and source-backed. It must
include:

1. launched version, commit, assets, and verification links;
2. what users can actually do today;
3. adoption and playground telemetry only if real, consent-compatible data
   exists;
4. platform/distribution status;
5. trust claims and exact enforcement boundaries;
6. package/shelf inventory;
7. known defects and support burden;
8. launch response and community signals, without invented metrics;
9. research/proof status;
10. prioritized remaining work with effort, dependency, risk, and evidence gate.

### Post-launch execution order

Unless fresh evidence changes the order:

1. launch defects and onboarding friction;
2. shelf expansion driven by real usage;
3. distribution signing/notarization;
4. production Memory Core allocator/finalizer integration;
5. native/backend and performance work;
6. provider-backed assist under an explicit privacy/security contract;
7. RB-8 and repository-shape cleanup;
8. broader proof, empirical studies, localization, and community infrastructure.

The `garnet-memory-core-implementer` owns only an approved, bounded Memory Core
slice after the State of the Union confirms it is the next highest-value lane.
It does not compete with launch-critical interpreter, WASM, shelf, or site work.

## 12. Ownership and Concurrency

- **Launch lead:** one primary implementation lane sequences Truth Lock,
  integration, and the launch packet.
- **W-PLAY implementer:** owns WASM adapter and playground runtime files.
- **Shelf implementer:** owns package/library/demo files and does not edit WASM
  or site files.
- **Front-door implementer:** owns `docs/` after the claim ledger is frozen.
- **Readiness reviewer:** read-only skeptical review before publication and
  again after remote CI.
- **Target-system verifiers:** Windows and Linux lanes produce native evidence;
  they do not author correctness-critical shared-kernel fixes.
- **Memory Core implementer:** post-launch bounded work only unless a launch
  defect directly proves Memory Core is on the critical path.

At most one lane edits any shared crate or public truth reporter at a time.
Parallel lanes use separate worktrees and one coherent PR per slice.

## 13. Completion Definition

The convergence program is complete when:

- Truth Lock is merged and current.
- The live playground executes pure Garnet and exposes an authority diff in the
  measured 30-second path.
- The minimum shelf and MCP flagship demo are built, sealed, and reproducible.
- The public site and Studio tell the same bounded story as the repo.
- The cross-OS/browser evidence matrix and launch packet are complete.
- The pipeline has stopped at Jon's launch lock.

The project is not thereby `production`, `1.0`, fully sandboxed on every OS,
fully productized, or finished forever. Launch completion and post-launch
engineering completion are deliberately separate states.
