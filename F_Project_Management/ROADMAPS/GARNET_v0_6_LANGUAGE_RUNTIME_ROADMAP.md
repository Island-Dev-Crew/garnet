# Garnet v0.6 Language Runtime Roadmap

Date: 2026-05-20 (post v0.5.0 + v0.5.1 close)
Successor of `F_Project_Management/ROADMAPS/GARNET_v0_5_LANGUAGE_COMPLETION_ROADMAP.md`.

## Thesis

v0.5 shipped the scaffolds. v0.6 makes them load-bearing.

Every v0.5 slice (S1–S10) closed with an honest `deferred` list. The most
load-bearing deferred lines are exactly the ones a user notices the first
time they try to do something real with Garnet:

- They `garnet add ../mylib` and then `garnet run src/main.garnet` doesn't
  actually consume the lib (S3 deferred: resolver).
- They want to install a dep that isn't on disk yet (no registry exists).
- They see the bytecode VM is faster than the interpreter on simple
  programs but their function definitions fall back to tree-walk (S2
  deferred: function-call lowering).
- They open a `.garnet` file in VSCode and get diagnostics + hover +
  go-to-def, but no workspace symbols, no rename, no code actions, no
  semantic tokens (S1 deferred — gates on CST).

v0.6 closes those four lines. That is the entire thesis.

## Confirmed v0.6 slices

| # | Title | Lane | Source-of-truth gap closed |
|---|---|---|---|
| S11 | v0.6 slice contract scaffold | — | Project-management ledger for v0.6 PRs. |
| S12 | Package-manager resolver contract | `pkg_resolver_v0_2` | S3 deferred line #1: interpreter consumes vendored deps at `garnet run` time. |
| S13 | Registry stub v0.1 | `registry_stub_v0_1` | Net-new: `garnet add` becomes an end-to-end loop, not just local-path vendor. |
| S14 | Bytecode VM v0.2 function-call lowering | `vm_function_call_lowering` | S2 deferred: native function calls + ABI v0.2 spec. |
| S15 | Trivia-preserving CST in `garnet-parser-v0.3` | `parser_cst_layer` | Foundational — gates S16, formatter v0.2, richer trust-report. |
| S16 | LSP v0.2 on the CST | `editor_lsp_adoption` (60 % → verified) | S1 deferred: workspace symbols, rename, code actions, semantic tokens, S10 code-action mesh. |

Full contracts: `F_Project_Management/GARNET_v0_6_SLICE_DOGFOOD.md`.

Per-slice plans (after S11 ships and a contract is on disk): each lands
under `.claude/plans/S<N>-plan.md` referencing the contract by section.

## Slice order and dependencies

Strict slice-per-PR discipline (one slice = one PR) survives from v0.5.
Cross-slice constraints:

- **S15 must merge before S16 opens.** S16 consumes the CST API. A
  concurrent agent starting S16 against the trivia-dropping parser would
  rebuild what S15 produces, slower and inconsistently.
- **S12 should merge before S13's end-to-end claim.** S13's crate + spec
  can land independently (it's net-new code), but the "registry-fetched
  dep actually runs" loop only closes once S12 is in.
- **S14 is independent of S12/S13/S15/S16.** It only touches the VM crate
  + the bytecode spec + benches.

A reasonable execution order (current recommendation):

```
S11 → S12 → S14 → S15 → S13 → S16
```

The order respects the dependency edges above and front-loads the slices
with the smallest blast radius (S12 closes a known partial; S14 is
self-contained). S13 lands after S12 so its dogfood block isn't pinned
on a "wait for S12 to merge before this is honest" footnote. S16 lands
last because it consumes the most upstream surface.

Concurrent agents may pick any branch that respects these edges. If two
agents reach for the same lane the colliding PR rebases against
`origin/main`.

## What's explicitly NOT in v0.6

Documented here so v0.7+ planning has a clean handoff. Each item is
already on the lane reporter as `active-partial` / `planned` / `blocked`,
and that signal does not change with v0.6:

- **Remote-source package resolution.** `https://`, `git+ssh://`, scope
  shortnames. v0.6 ships the local-path resolver (S12) and the
  registry-fetched path (S13); the broader remote-source matrix is v0.7+.
- **Transitive dependencies.** Vendoring nested `Garnet.toml` is v0.7+.
- **SemVer matching.** Caret, tilde, equality beyond string compare is
  v0.7+. v0.6 records `version` in the manifest for forward
  compatibility but does not enforce it.
- **Workspace mode.** Multi-crate projects with a root workspace `Garnet.toml`.
- **`garnet verify-deps`.** Lockfile-drift detector.
- **Closures, captured environments, dynamic-receiver method dispatch in
  the VM.** v0.6 lowers parameterized functions returning MVP values;
  closures and dynamic dispatch stay on the tree-walk fallback path.
- **Pattern matching, try/rescue/ensure, struct/enum constructors in the VM.**
- **Cross-version bytecode ABI promise.** v0.6 version-bumps the magic to
  `GARNVM02` and tightens the schema; full ABI stability is v0.7+.
- **Native backend (LLVM/Cranelift codegen).** Tracked in
  `GARNET_NATIVE_BACKEND_PLAN.md`. v0.6 does not touch it.
- **Mechanized proof / external user study.** Tracked in
  `GARNET_FORMAL_PROOF_PLAN.md` and `GARNET_EMPIRICAL_VALIDATION_PLAN.md`.
  v0.6 keeps Paper VI Contribution 1 unchanged (`pending-infra`).
- **Apple Developer ID notarization, signed `.pkg`, Windows / Linux
  runtime proof.** Credential-blocked / infra-blocked; tracked under
  `developer_id_notarization` lane.
- **Broad converter frontends** (JavaScript, TypeScript, Swift, Java, C,
  C++, C#, Perl, Kotlin, Shell, SQL). `broad_converter_frontends` lane
  stays at `planned` 0 %.
- **Mobile distribution.** `mobile_distribution` lane stays at `planned` 0 %.
- **Promo video.** `promo_video` lane stays at `composition-ready` 50 %.

## Target lane delta at v0.6.0 tag

Today (post-v0.5.1):

```
71.9 % / 21 lanes / 12 verified
```

Target at v0.6.0 tag (after S12–S16 merge):

```
≥ 80 % / ≥ 25 lanes / ≥ 17 verified
```

The lane count grows by 4 (S12, S13, S14, S15 each open a new lane; S16
advances an existing lane). The verified count grows by 5 if every v0.6
slice reaches `verified` at the tag (S16 advances `editor_lsp_adoption`
from `source-present 60 %` to `verified 100 %`).

The headline % is a derived signal. The brand-equity signal is the count
of `verified` lanes balanced against the count of `active-partial` /
`planned` / `blocked` lanes — the same calibration v0.5 closed on.

## Honesty anchors (specific to v0.6)

These survive into the v0.6.0 release blog, README, and status outputs
unless a slice explicitly trades them and Jon approves:

- "v0.5 shipped the scaffolds; v0.6 makes them load-bearing — not the
  other way around."
- "Bytecode ABI v0.2 is more stable than v0.1 but not yet a
  cross-version ABI promise."
- "The registry stub serves a static `index.json` — no central registry,
  no auth, no publish flow."
- "Package-manager resolver is local-path-first; remote sources,
  transitive deps, SemVer matching, and workspace mode remain deferred."
- "CST round-trip is source-preserving for canonical examples; recovery
  from malformed input is best-effort."

Carried from v0.5:

- "research-grade prototype (v0.x.x) — not production-complete"
- "production allocator path tracked in MEMORY_CORE_ROADMAP.md"
- "human/aesthetic acceptance remains open"
- Paper VI scorecard: "4 supported, 2 partial (downgraded honestly), 0
  refuted, 1 pending-infra"

## Cadence

No fixed dates. The v0.5 cycle landed eleven slices in a single intense
session because every slice was ≤ a day of focused work. v0.6 slices vary
more widely: S12 and S13 are 1–2 days each; S14 is 2–3 days; S15 is
3–5 days; S16 is 2–3 days after S15.

If an agent reaches a slice that exceeds its branch budget, it splits the
slice into S<N>a + S<N>b under the same plan-file naming, preserving the
state-machine ledger on both halves.
