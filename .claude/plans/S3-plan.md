# S3 — `garnet add` + Manifest Spec v0.1 — Implementation Plan

Date: 2026-05-20 (post-v0.5.0 tag)
Contract: `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` § S3
State: not-started → **planned** → in-progress
Reviewer: Jon (Island Development Crew)

> S3 is a v0.5.1-acceptable slice. PR title: `S3: garnet add + Garnet Manifest v0.1 (vendored deps with content-addressed lockfile)`.

## 1. Scope (in)
- `garnet-cli/src/cmd/add.rs` (~450 LOC, new) — `garnet add [--name <id>] <path>` implementation:
  - Resolves project root via upward search for `Garnet.toml`.
  - Copies source tree into `.garnet/vendor/<name>/`, skipping nested `.garnet/` and `Garnet.lock`.
  - Hashes every regular file with BLAKE3.
  - Upserts `<name> = { path = "...", vendor = "..." }` under `[dependencies]` in `Garnet.toml`.
  - Writes `Garnet.lock` with deterministic ordering (alpha by dep name, lex by file path, lowercase hex).
  - Six inline unit tests covering: vendor copy skipping nested vendor, hash determinism + path-sort, manifest upsert insert + replace, lockfile render/parse round-trip, full end-to-end into a synthetic project.
- `garnet-cli/src/cmd/mod.rs` — register `pub mod add;`.
- `garnet-cli/src/bin/garnet.rs` — dispatch `"add" => cmd::add::run(...)`.
- `C_Language_Specification/GARNET_MANIFEST_v0_1.md` (new) — formal manifest + lockfile spec.
- New "Garnet manifest + vendored deps (`garnet add`)" lane in `scripts/garnet_mit_readiness_status.py` (verified 100%).
- Regenerated readiness baseline.
- `CHANGELOG.md` Added entry under `[Unreleased] — v0.5.1 in flight`.
- `.claude/plans/S3-plan.md` planning doc.

## 2. Scope (out)
- **Resolver contract.** The interpreter does NOT yet load `.garnet/vendor/` deps at parse/check/run time. The vendored bytes sit on disk and `Garnet.lock` records their hashes, but `garnet run` does not consume them. Separate v0.5.x slice.
- **Remote sources.** No `https://`, `git+ssh://`, or registry shortnames (`@scope/name`). Local paths only.
- **Transitive deps.** If `<path>` itself has a `Garnet.toml`, its deps are NOT pulled in.
- **SemVer matching.** A `version` field is recorded for forward compatibility but no caret/tilde/equality matching at add time.
- **Workspace mode.** Multi-crate workspaces out of scope.
- **`garnet verify-deps`.** A separate lockfile-drift detector slice.

## 3. Honest partials (in code, lane evidence, and PR body)
- Top-of-file doc header in `add.rs` lists every "what it does NOT do" line.
- The MIT readiness lane's `deferred` field repeats them.
- Manifest spec `§4. Honest partials (v0.5.1)` lists them.
- CHANGELOG entry says "interpreter does NOT yet resolve `use <dep>::*` at `garnet run` time."

The contract's dogfood line `garnet run src/main.garnet # uses the added lib` would require the resolver work that S3 explicitly defers. The honest framing: this slice lands the **vendor + lockfile** contract; a follow-up slice lands the **resolver** contract.

## 4. Dogfood block (per contract S3, with the resolver step labeled honestly)

```bash
mkdir /tmp/test-add && cd /tmp/test-add
garnet new --template cli demo && cd demo
mkdir ../local-lib && echo 'def hello() { "hi" }' > ../local-lib/lib.garnet
garnet add ../local-lib
grep -q "local-lib" Garnet.lock                  # PASS (S3)
garnet run src/main.garnet                       # PASS (template's own main, NOT consuming the lib yet)
# The contract's "# uses the added lib" expectation is the deferred
# resolver step; v0.5.1 ships the vendor + lockfile contract first.
```

Local verification on this machine:
- `garnet new --template cli demo` succeeded.
- `garnet add ../local-lib`: `vendored 'local-lib' from ../local-lib into .garnet/vendor/local-lib (1 file(s) hashed); Garnet.toml [dependencies] updated; Garnet.lock updated`.
- `grep -c "local-lib" Garnet.lock` → 3 (matches block name + path + vendor lines).
- `cargo test -p garnet-cli --lib cmd::add::tests` → 6/6 pass.
- `cargo clippy -p garnet-cli --all-targets -- -D warnings` → clean.

## 5. State-machine transitions
| Transition | Evidence |
|---|---|
| not-started → planned | this file |
| planned → in-progress | draft PR `S3: garnet add ...` opens |
| in-progress → review-ready | CI green (cargo test workspace includes the 6 new unit tests) |
| review-ready → dogfood-passing | Jon review + CHANGELOG + spec doc |
| dogfood-passing → merged | squash-merge |

## 6. Risks and mitigations
- **Manifest TOML editor is line-based, not a real parser.** Mitigation: behavior is constrained to `[dependencies]` and is covered by two upsert tests (insert + replace) plus the round-trip end-to-end test. A future slice can swap in a real TOML editor (e.g. `toml_edit`) without changing the on-disk contract.
- **Lockfile case** (`Garnet.lock` vs `garnet.lock`). Mitigation: matches the existing project convention (`Garnet.toml`); the contract's bash uses lowercase informally but the codebase is uppercase consistently. Tests assert the canonical case.
- **Hash collision** is BLAKE3-grade; no mitigation needed.
- **Concurrent `garnet add` writers** could race on `Garnet.lock`. Mitigation: single-process model today; users serialize their own invocations. A future slice can add a `.garnet/vendor/.lock` advisory.

## 7. What I need from Jon
None.
