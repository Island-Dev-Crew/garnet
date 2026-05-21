# S12 — Package-manager resolver contract — Implementation Plan

Date: 2026-05-21
Contract: `F_Project_Management/GARNET_v0_6_SLICE_DOGFOOD.md` § S12
State: not-started → **planned** (this plan file commits the transition)
Reviewer: Jon (Island Development Crew)

> S12 closes the deferred line #1 of S3 ("interpreter does NOT yet load
> `.garnet/vendor/` deps at `garnet run` time"). PR title:
> `S12: package-manager resolver contract (garnet run loads vendored deps)`.

---

## 1. Why S12 is the right next slice

The v0.6 slice order recorded in the roadmap is
`S11 → S12 → S14 → S15 → S13 → S16`. S11 just merged
([PR #215](https://github.com/Island-Dev-Crew/garnet/pull/215)) so the v0.6
contract is on disk. S12 is next because:

- It closes a known partial that anyone reading S3's lane already sees as
  `Resolver contract: interpreter does NOT yet load .garnet/vendor/ deps
  at garnet run time (separate slice)`.
- The implementation surface is small (one new module, one CLI hook, one
  integration test, one lane).
- It is the prerequisite for S13 (registry stub) to be end-to-end honest.

## 2. Scope (in)

- **Test first (RED).** New `garnet-cli/tests/run_resolver.rs` — workspace
  integration test that:
  - Builds a temp project (`Garnet.toml` + `src/main.garnet` +
    `.garnet/vendor/local_lib/lib.garnet` + `Garnet.lock`).
  - Runs the cli binary as a subprocess: `garnet run src/main.garnet`.
  - Asserts stdout contains the string the vendored lib emits (e.g.
    `hi from local-lib`).
  - Initially RED — `Item::Use` is a no-op and no pre-load exists.
- **Manifest reader.** Add `pub fn read_dependency_table(project_root: &Path)
  -> io::Result<Vec<DependencyEntry>>` to `garnet-cli/src/cmd/add.rs`
  (it's the manifest module today; the reader can live alongside the
  writer). `DependencyEntry { name: String, vendor_rel: PathBuf }`.
- **Pre-loader.** Add `fn preload_dependencies(interp: &mut Interpreter,
  project_root: &Path) -> Result<(), String>` to
  `garnet-cli/src/cmd/run.rs`. Iterates deps; for each, walks vendor dir;
  for each `.garnet` file, calls `interp.load_source(...)`. Errors are
  surfaced as a single line on stderr ("dep <name>: parse error in
  <file>: <e>") but **continue the run** (best-effort; the user may want
  to ship a half-broken vendor and let main fail loudly).
- **Hook into run.** Both `run_interpreter` and `run_vm` call
  `preload_dependencies` before `interp.load_source(src)`. Best-effort
  project-root lookup; if no `Garnet.toml` in the file's parent chain,
  skip pre-load (preserves `garnet run /tmp/anything.garnet`).
- **Example fixture.** `examples/pkg_resolver_demo_lib.garnet`
  (`def hello() { "hi from local-lib" }`) + a project layout note in
  the test (the example sits at the top-level for `garnet check` /
  `garnet run` to pick up directly).
- **Readiness lane.** New `ObjectiveLane(id="pkg_resolver_v0_2", ...)` in
  `scripts/garnet_mit_readiness_status.py`. `verified` when
  `garnet-cli/tests/run_resolver.rs` exists AND the pre-load function
  exists in `cmd/run.rs`. Deferred list captures remote sources,
  transitive deps, SemVer, workspace mode, qualified-path resolution,
  name-collision handling, `garnet verify-deps`.
- **Regenerate baseline.**
  `python3 scripts/garnet_mit_readiness_status.py --format json >
  F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json`.
- **CHANGELOG.md.** New `S12` entry under `[Unreleased] — v0.6.0 in flight`.
- **Update S3 lane evidence.** The existing `garnet_add_manifest` lane's
  deferred list still names the resolver as separate. Update its
  `deferred` to point at the new `pkg_resolver_v0_2` lane and drop the
  "(separate slice)" qualifier from the line, since S12 closes it.
- **Update v0.6 contract.** Flip S12 from `not-started` to
  `review-ready` (and after merge, `merged`) in
  `GARNET_v0_6_SLICE_DOGFOOD.md` § S12.
- `.claude/plans/S12-plan.md` (this file).

## 3. Scope (out)

- **Qualified-path resolution.** `local_lib::hello()` (with the prefix in
  the call site) does **not** resolve in S12 — only `use local_lib::*`
  unqualified resolution is in. The S3 dogfood literal `garnet run
  src/main.garnet # uses the added lib` is satisfied by `use ::*` plus
  a direct call, which matches the S12 dogfood block in the v0.6 contract.
- **Transitive deps.** If a vendored dep itself has a `Garnet.toml`, its
  deps are not pulled in.
- **SemVer matching.** Pure path-based vendor resolution; no version
  arbitration.
- **Workspace mode.** Multi-crate project workspaces.
- **Remote sources.** `https://`, `git+ssh://`, `@scope/name`.
- **Name-collision handling.** If two deps both export `hello`, the
  last-loaded wins. Documented as a deferred line; a future slice can
  add a strict mode.
- **Lockfile verification at run time.** S12 reads vendor bytes
  directly; it does NOT verify them against `Garnet.lock` BLAKE3 hashes
  before loading (that's a `garnet verify-deps` slice).
- **Module-scoped definitions in the dep.** The dep can declare
  `module Foo { ... }` but those scoped names aren't reachable via
  `use local_lib::Foo::bar` — only top-level items work in v0.6.
- **VM resolver.** The pre-load goes through the tree-walk interpreter;
  the bytecode VM path uses `run_source_with_options` which doesn't
  share the pre-loaded env. **For S12, pre-load only applies to the
  `--interp` path.** The VM path skips pre-load with a one-line stderr
  note. S14 will harmonize this when the VM gets its own load_source
  equivalent.
- **No new dependencies.** S12 reuses existing crates.

## 4. Concrete tasks (ordered, TDD style)

1. **Plan file.** This document → commit on draft branch.
2. **Red test.** Write `garnet-cli/tests/run_resolver.rs`. Run
   `cargo test -p garnet-cli --test run_resolver`. Expect FAIL with
   stdout missing `"hi from local-lib"`.
3. **Manifest reader.** Add `read_dependency_table` to
   `garnet-cli/src/cmd/add.rs` (refactor: the existing
   `update_manifest` already understands the `[dependencies]` table
   line-by-line; the reader is the symmetric inverse). Inline unit
   test in the existing `tests` module.
4. **Pre-loader.** Implement `preload_dependencies` in
   `garnet-cli/src/cmd/run.rs`. Call it from `run_interpreter` (NOT
   `run_vm` yet; document the VM gap).
5. **Re-run the red test.** Expect PASS.
6. **Lane + baseline.** Add `pkg_resolver_v0_2` lane to
   `scripts/garnet_mit_readiness_status.py`. Update `garnet_add_manifest`
   lane's deferred line to credit S12. Regenerate baseline.
7. **CHANGELOG.** Add `S12` entry under `[Unreleased] — v0.6.0 in flight`.
8. **Contract update.** Flip S12 status in
   `GARNET_v0_6_SLICE_DOGFOOD.md` § Slice Contracts to
   `review-ready` (this PR opens).
9. **Full local ladder.**
   ```
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace --no-fail-fast
   cargo deny check
   python3 scripts/garnet_mit_readiness_status.py --check-no-regression
   python3 scripts/garnet_conformance_matrix_check.py
   python3 -m unittest discover scripts/ -p 'test_*.py'  # 2 pre-existing failures expected
   ```
10. **Desktop bundle.**
    `/Users/IDC2.5/Desktop/dogfood/garnet-s12-resolver-<UTCstamp>/`.
11. **Commit, push to fork, open PR against `Island-Dev-Crew/garnet:main`.**
12. **CI green → IslandDevCrew merge → mark task #3 completed → move
    to S14.**

## 5. Honest doubts and risks

- **VM path is split.** Pre-load only wires to `--interp`. The VM still
  uses `run_source_with_options` from `garnet-vm` and has no shared
  env. Flag in the PR body; commit to S14 closing this.
- **Best-effort error handling for vendor parse errors.** A broken dep
  emits a stderr line and the run continues. The alternative — fail
  fast — is friendlier for users but louder for tooling that scans
  many projects. Pick best-effort with a warning prefix
  (`garnet run: dep <name>: ...`); revisit if a downstream slice wants
  strict mode.
- **Iteration order across vendor files.** Multiple files in
  `.garnet/vendor/<dep>/` get loaded in `walkdir` order, which is
  filesystem-dependent and non-deterministic on some platforms.
  Mitigate with `walkdir + sort by relative path` so the load order is
  deterministic across machines (matches the lockfile's lex-sort).
- **`load_source` panic on duplicate definitions.** The interpreter's
  `define` shadows by name; pre-loading two deps that both define
  `hello` silently picks the last. Document as a deferred line; an
  ergonomic future slice can `define_namespaced` instead.
- **`@caps()` on dep `main`.** Vendored libs that have a `main` would
  shadow user `main`. Fix: skip top-level `main` from pre-load (a dep
  shouldn't run its own entry point when consumed). Documented as a
  small explicit rule in `preload_dependencies`.
- **Manifest parser fragility.** `cmd/add.rs::update_manifest` is
  line-based and minimal. The new reader inherits that minimalism —
  only `[dependencies]` block, only `name = { path = "...",
  vendor = "..." }` shape. Future slice can swap in `toml_edit` without
  breaking the on-disk format.

## 6. State-machine transitions

| Transition | When | Evidence |
|---|---|---|
| not-started → planned | this file | `.claude/plans/S12-plan.md` |
| planned → in-progress | draft PR opens | PR URL |
| in-progress → review-ready | CI green + dogfood headings + bundle | PR check status |
| review-ready → dogfood-passing | Jon review | PR review |
| dogfood-passing → merged | squash-merge + lane flips verified | merge commit + reporter output |

## 7. What I need from Jon

Nothing blocking. Draft PR opens automatically; flagging the VM split in
the PR body so the S14 owner sees it immediately.
