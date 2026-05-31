# Garnet Agent Handoff — 2026-05-21

**Audience:** the next assistant or contributor opening this repo cold after the
v0.6 entry session (S11–S14 merged).
**Status:** v0.6 is in flight. Three of the four v0.6 streams Jon named
(package manager, bytecode VM, registry stub) are merged. The fourth
(LSP / language server) is the only one left and is gated on a
trivia-preserving CST.

---

## 0. Read this first

1. `CLAUDE.md` — the harness contract + boot sequence.
2. `F_Project_Management/GARNET_v0_6_SLICE_DOGFOOD.md` — **the v0.6 contract**
   (successor of the v0.5 one). State machine, verification primitives,
   cross-slice gates, contracts for S11–S16, v0.6.0 release gate, PR body
   template, honesty anchors. Flip slice rows in the same commit as the work.
3. `F_Project_Management/ROADMAPS/GARNET_v0_6_LANGUAGE_RUNTIME_ROADMAP.md` —
   v0.6 thesis ("v0.5 shipped scaffolds; v0.6 makes them load-bearing"),
   slice order, deferred-to-v0.7 list, target lane delta.
4. `CHANGELOG.md` — `[Unreleased] — v0.6.0 in flight` names every merged slice.
5. `F_Project_Management/GARNET_AGENT_HANDOFF_2026_05_20.md` — the prior
   (v0.5.x close) handoff.
6. Memory files: `~/.claude/projects/-Users-IDC2-5-Desktop-Garnet/memory/`
   — `garnet-slice-discipline.md`, `garnet-paper-vi-anchors.md`,
   `garnet-toolchain-paths.md` (cargo is NOT on PATH — see §7).

---

## 1. Current repo truth

- **`origin/main` tip:** `b508069` (S13). Run
  `git log --oneline --decorate --max-count=8 origin/main` after `git fetch`.
- **MIT readiness:** **75.4 % / 24 lanes / 15 verified**
  (`python3 scripts/garnet_mit_readiness_status.py | head -8`).
- **Open PRs upstream:** none from this session at close. Verify with
  `gh pr list --repo Island-Dev-Crew/garnet --state open`.
- **Two pre-existing `scripts/` unittest failures** on clean `main`
  (`test_repo_and_site_point_to_the_adoption_surface_reporter`,
  `test_docs_converter_section_matches_current_truth`) — website-content
  drift, NOT introduced by any v0.6 slice. A dedicated "website / adoption
  surface parity" slice owns them. They do not block the dogfood gate (which
  greps the PR body, not the test suite) but they do mean
  `python3 -m unittest discover scripts/` reports `failures=2`.

```sh
# Boot verification (run before any edit):
cd /Users/IDC2.5/Desktop/Garnet
git fetch --prune origin
git fetch --prune fork
git status --short --branch
git log --oneline --decorate --max-count=8 origin/main
gh pr list --repo Island-Dev-Crew/garnet --state open --json number,title,headRefName,isDraft,mergeStateStatus,url --limit 50
export PATH="/Users/IDC2.5/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"  # see §7
python3 scripts/garnet_mit_readiness_status.py | head -8
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
```

---

## 2. What this session landed

| # | PR | Slice | Lane |
|---|---|---|---|
| — | [#214](https://github.com/Island-Dev-Crew/garnet/pull/214) | 2026-05-20 handoff doc | — |
| S11 | [#215](https://github.com/Island-Dev-Crew/garnet/pull/215) | v0.6 slice contract scaffold + readiness skill v0.5 refresh + CHANGELOG conflict fix | — |
| S12 | [#216](https://github.com/Island-Dev-Crew/garnet/pull/216) | Package-manager resolver — `garnet run` loads vendored `use <dep>::*` | `pkg_resolver_v0_2` |
| S14 | [#218](https://github.com/Island-Dev-Crew/garnet/pull/218) | Bytecode VM v0.2 — explicit call-frame stack + ABI `GARNVM02` + `--dump-lowering` | `vm_function_call_lowering` |
| S13 | [#219](https://github.com/Island-Dev-Crew/garnet/pull/219) | Registry stub v0.1 — `index.json` + `garnet add --registry` | `registry_stub_v0_1` |

Per-slice plans live under `.claude/plans/S<N>-plan.md`. Per-slice Desktop
dogfood bundles under `/Users/IDC2.5/Desktop/dogfood/garnet-<slug>-<UTCstamp>/`
with sealed `MANIFEST.sha256`.

Executed order was `S11 → S12 → S14 → S13` (the roadmap suggested
`S11 → S12 → S14 → S15 → S13 → S16`; S13 was pulled ahead of S15 because Jon
prioritized closing the registry-stub ask, and S13 is end-to-end-honest now
that S12 is in).

---

## 3. Honest partials per shipped slice (the receipts the next slice cashes)

- **S12 resolver:** `--interp` only (the `--vm` path does NOT pre-load vendored
  deps); `use local_lib::*` unqualified resolution only (no `local_lib::hello()`
  qualified paths); no transitive deps / SemVer / workspace mode / remote
  sources; no run-time lockfile BLAKE3 verification; name collisions
  last-wins; a vendored dep's own `def main` is stripped by a line-based
  scanner (AST-based is future).
- **S14 VM v0.2:** tail-call optimization deferred (one heap frame per call);
  closures / captured envs / dynamic-receiver dispatch / pattern matching /
  try-rescue / struct-enum constructors still fall back; `and`/`or`
  short-circuit native lowering DROPPED (Ruby-style operand-returning
  semantics need value-preserving conditional-jump + `Dup` opcodes — its own
  slice); `--vm` path doesn't pre-load vendored deps; `GARNVM02` is tightened,
  not a frozen cross-version ABI; no production native-compiler proof.
- **S13 registry:** filesystem / `file://` only (no HTTP(S)); packages are
  directories (no tarballs); no auth / accounts / publish flow; index
  `signature` field reserved but unread; exact `<name>@<version>` only (no
  SemVer ranges); one registry per invocation; no transitive resolution.

Each is captured verbatim in the matching `garnet_mit_readiness_status.py`
lane's `deferred` list and in the slice's CHANGELOG entry.

---

## 4. The remaining stream: LSP / language server (S15 → S16)

This is the only v0.6 stream not yet started. It is a two-slice arc.

### S15 — Trivia-preserving CST in `garnet-parser-v0.3` (do first)

**Why:** today the parser drops trivia (whitespace, comments). That single gap
blocks LSP precision (hover ranges, semantic tokens), an AST-driven formatter
(S4 is whitespace-only because of this), and richer trust-report spans.

**Design notes for whoever picks it up:**
- A green/red tree (à la `rowan`) is the standard approach; the S15 contract
  notes "hand-rolled if it can be kept ≤ 600 LOC." `rowan` is a clean,
  widely-used crate but adds a dependency — run `cargo deny check` and check
  the license before committing to it.
- Keep the **AST as the semantic reference**; the CST is the trivia-faithful
  spine that LSP/formatter consume. Do NOT route semantics through the green
  tree.
- The headline test is a **source round-trip**: every `examples/{mvp_,det_}*.garnet`
  must be byte-identical after `parse → cst → emit_source`. Add
  `garnet-parser-v0.3/tests/cst_round_trip.rs`.
- Lane: add `parser_cst_layer` to `garnet_mit_readiness_status.py`, regenerate
  the baseline.
- Watch: the parser has a fuzz sub-workspace (`garnet-parser-v0.3/fuzz/`, its
  own `[workspace]`). CST changes must keep `cargo +nightly fuzz run
  parse_input` clean.

### S16 — LSP v0.2 on the CST (after S15 merges)

- Workspace symbols, rename, code-actions, semantic tokens — all consuming the
  S15 CST. Wire S10's `garnet-check-v0.3/src/suggest.rs` advisories in as LSP
  code-actions.
- VSCode extension (`editors/vscode/`) declares the new capabilities; bump its
  package version.
- Advances the existing `editor_lsp_adoption` lane from `source-present 60 %`
  toward `verified 100 %`.
- `garnet-lsp/` is currently a thin `tower-lsp` MVP (`src/lib.rs`, `src/main.rs`)
  — diagnostics + hover + go-to-def only.

**Do not open S16 against the trivia-dropping parser.** S15 must merge first or
S16 rebuilds what S15 produces.

---

## 5. Other open lanes (not the LSP stream)

From the readiness reporter, still `active-partial` / `planned` / `blocked`:
`developer_id_notarization` (credential-blocked), `macos_studio_dmg` (75 %),
`windows_linux_distribution` (50 %), `promo_video` (50 %), `llm_assist`
(40 %), `broad_converter_frontends` (planned), `proof_empirics` (45 %),
`mobile_distribution` (0 %). Plus the website/adoption-surface parity fix for
the two failing `scripts/` tests (§1).

---

## 6. Slice discipline that worked (inherit this exactly)

- One slice per PR. Title `S<N>: <short>`. Branch from fresh `origin/main`:
  `git switch -c codex/s<N>-<slug> origin/main`.
- Plan first: `.claude/plans/S<N>-plan.md` referencing the contract section.
- **Red test before behavior change.** (S12, S14, S13 each had a failing test
  first — for S14 the red was a process abort on deep recursion, verified via
  a probe binary since the abort kills the test harness.)
- Dogfood bundle required at
  `/Users/IDC2.5/Desktop/dogfood/garnet-<slug>-<UTCstamp>/`: report, diff,
  verification logs, sealed `MANIFEST.sha256` (`shasum -a 256 -c` must be all
  OK).
- **PR body MUST use the dogfood-readiness headings** — the gate
  (`scripts/check_dogfood_pr_body.py`, run by
  `.github/workflows/dogfood-readiness.yml`) greps for `## Dogfood Readiness`
  + `### Current truth` / `### Local verification` / `### Remote verification`
  / `### Desktop dogfood bundle` / `### Deferred / out of scope`, each with ≥1
  `- [x]` item. **Anything under `F_Project_Management/`, `C_Language_*`,
  `examples/`, `garnet-parser-v0.3/`, `.github/workflows/`, etc. is
  "readiness-sensitive" and triggers this gate** — even doc-only PRs.
- **The gate does NOT re-run on `pull_request: edited`.** If you fix the PR
  body after CI ran, you must push a new commit (or
  `git commit --amend --no-edit` + `git push --force-with-lease`) to trigger a
  fresh `synchronize` event with the updated body. (Hit this on S13.)
- New crate? It needs an `AGENTS.md`, registered in BOTH
  `scripts/check-agent-contracts.py` (`CONTRACT_RULES` with required phrases)
  AND the root `AGENTS.md` index. (Hit this on S13's `garnet-registry-stub`.)
- Adding a lane → regenerate
  `F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json` via
  `python3 scripts/garnet_mit_readiness_status.py --format json > <baseline>`,
  in the same PR.
- Fork → PR → IslandDevCrew merges:
  ```sh
  git push -u fork codex/<slug>
  gh pr create --repo Island-Dev-Crew/garnet --base main \
    --head Navigata1:codex/<slug> --no-maintainer-edit \
    --title "S<N>: <short>" --body-file /tmp/sN-pr-body.md
  # after CI green:
  gh auth switch --user IslandDevCrew
  gh pr merge <n> --repo Island-Dev-Crew/garnet --squash --delete-branch
  gh auth switch --user Navigata1
  git checkout main && git fetch origin --quiet && git merge --ff-only origin/main
  ```

---

## 7. Machine quirk: cargo is NOT on PATH

`cargo`/`rustc` are not on `PATH` and there is no `~/.cargo/bin/cargo` shim.
The toolchain lives at
`/Users/IDC2.5/.rustup/toolchains/stable-aarch64-apple-darwin/bin/`. Prefix
cargo commands with:

```sh
export PATH="/Users/IDC2.5/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
```

(`cargo-deny` is the only thing in `~/.cargo/bin/`.) Saved as the
`garnet-toolchain-paths` memory.

---

## 8. Standard verification ladder (before any commit)

```sh
export PATH="/Users/IDC2.5/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
cargo deny check
python3 scripts/check-agent-contracts.py
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
python3 scripts/garnet_conformance_matrix_check.py
python3 -m unittest discover scripts/ -p 'test_*.py'   # expect failures=2 (pre-existing)
```

---

## 9. Things NOT to do (preserved)

- Do not soften the honesty anchors (`garnet-paper-vi-anchors.md` + the v0.6
  contract's § Honesty Anchors). If a slice would weaken one, say so in the PR
  body so Jon decides.
- Do not rename Mnemos / Memory Core.
- Do not add dependencies without `cargo deny check` clean.
- Do not force-push to `origin/main`. (Force-push to your own feature branch to
  re-trigger CI is fine — see §6.)
- Do not treat the dogfood bundle path as decoration — CI greps the PR body for
  `### Desktop dogfood bundle`.

— end handoff —
