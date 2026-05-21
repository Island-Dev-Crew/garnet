# S13 — Registry stub v0.1 — Implementation Plan

Date: 2026-05-21
Contract: `F_Project_Management/GARNET_v0_6_SLICE_DOGFOOD.md` § S13
State: not-started → **planned**
Reviewer: Jon (Island Development Crew)

> PR title: `S13: registry stub v0.1 (index.json + garnet add --registry)`.

## 1. Scope decision: filesystem-backed, no HTTP/tar

The contract sketched `garnet add --registry http://127.0.0.1:8765`. For a
v0.1 **stub**, the substance is the *resolution loop* (index lookup →
content-address verify → vendor), not the transport. HTTP + tarball handling
would add a network client + `tar`/`flate2` to `garnet-cli` — more dependency
and attack surface for the least-interesting part.

So v0.1 is **filesystem-backed**: a registry is a directory with an
`index.json` and `<name>/<version>/` package directories. `garnet add
--registry <dir-or-file-url>` reads the index, copies the versioned package
dir into `.garnet/vendor/<name>/`, and verifies each file's BLAKE3 against
the index. HTTP(S) transport + tarball packaging are deferred to v0.7,
documented in the lane and spec. This reuses the exact vendor + BLAKE3
machinery `garnet add <path>` already has.

## 2. Scope (in)

- **`C_Language_Specification/GARNET_REGISTRY_v0_1.md`** — `index.json` schema,
  package layout, content-address rule, the reserved (not-yet-verified)
  signature field, and the explicit v0.1 non-claims.
- **`garnet-registry-stub/` crate** (new, workspace member):
  - `src/lib.rs` — serde `RegistryIndex` types + `build_index(dir)` (hash every
    `<name>/<version>/` package) + `load_index(dir)` + `resolve(&index, name,
    version)`. Reuses BLAKE3.
  - `src/main.rs` — `garnet-registry-stub build <registry-dir>` writes
    `<registry-dir>/index.json`; `garnet-registry-stub verify <registry-dir>`
    re-hashes and checks the index. Thin CLI.
  - `Cargo.toml` — deps: `serde` + `serde_json` (already vetted in Cargo.lock)
    + `blake3` (already used). Workspace-versioned.
- **`garnet-cli`** — depend on `garnet-registry-stub`; extend
  `src/cmd/add.rs` with `--registry <location>` + `<name>@<version>` parsing.
  Resolution: load index, resolve version, copy package dir into vendor,
  verify BLAKE3 per file, write `Garnet.toml` `[dependencies]` entry
  (`<name> = { registry = "<loc>", version = "<v>", vendor = "..." }`) +
  `Garnet.lock`.
- **Fixture** `examples/registry_stub_fixture/` — a `hello_lib/0.1.0/lib.garnet`
  package + a generated `index.json`.
- **Lane** `registry_stub_v0_1` in `garnet_mit_readiness_status.py` (verified
  when crate + spec exist) + regenerated baseline.
- **Test** `garnet-cli/tests/registry_add.rs` — build a temp registry, run
  `garnet add --registry <dir> hello_lib@0.1.0`, assert vendored + lockfile +
  manifest, and (since S12 is merged) `garnet run` resolves the symbol
  end-to-end. Plus inline unit tests in the stub crate for build/resolve.
- **CHANGELOG** entry + **contract flip** S13 → in-progress.
- `.claude/plans/S13-plan.md` (this file).

## 3. Scope (out)

- **HTTP(S) transport.** `http://`, `https://`. Filesystem + `file://` only.
- **Tarball packaging.** Packages are directories; no `tar`/`gzip`.
- **Publish/auth flow.** No upload, no tokens, no account model.
- **Signature verification.** `index.json` reserves a `signature` field but
  v0.1 does NOT verify it (meshes with the future notarization slice).
- **SemVer ranges.** Exact `<name>@<version>` only; no caret/tilde.
- **Multi-registry resolution.** One registry per `garnet add` invocation.
- **Transitive deps from the registry.** A fetched package's own deps are not
  resolved.

## 4. Concrete tasks (ordered, TDD)

1. Plan (this) → commit.
2. Add `garnet-registry-stub` to workspace members; scaffold crate.
3. Stub lib: index types + `build_index` + `load_index` + `resolve`, with
   inline unit tests (build a temp registry dir, hash, round-trip JSON).
4. Stub bin: `build` + `verify` subcommands.
5. **Red test** `garnet-cli/tests/registry_add.rs` (registry add not wired yet
   → fails).
6. Extend `add.rs` with `--registry`; make the test pass.
7. Fixture + generate its `index.json` via the stub.
8. Lane + baseline + spec + CHANGELOG + contract flip.
9. Full ladder (fmt/clippy/test/deny/readiness/conformance/scripts).
10. Desktop bundle + commit + push + PR + CI + merge.

## 5. Honest doubts and risks

- **New deps (`serde`, `serde_json`) on a workspace crate.** Both are already
  in `Cargo.lock` (from the excluded studio app) and are MIT/Apache; cargo-deny
  should stay green. Verify with `cargo deny check` before pushing.
- **`Garnet.toml` line-based writer.** The S3 writer emits
  `name = { path=..., vendor=... }`. Registry entries need a different shape
  (`registry`, `version`). Extend the writer minimally; keep the existing path
  form working (a regression test guards it).
- **Index path traversal.** A malicious `index.json` `path` like `../../etc`
  must not escape the registry root. Canonicalize + assert the resolved
  package dir is under the registry root before copying. Add a unit test.
- **End-to-end run depends on S12.** S12 is merged, so `garnet run` resolving
  the registry-vendored symbol is honest now (no "after S12" footnote).

## 6. State-machine transitions

| Transition | When | Evidence |
|---|---|---|
| not-started → planned | this file | `.claude/plans/S13-plan.md` |
| planned → in-progress | draft PR | PR URL |
| in-progress → review-ready | CI green + bundle | PR checks |
| review-ready → dogfood-passing | Jon review | review |
| dogfood-passing → merged | squash-merge + lane verified | merge commit |

## 7. What I need from Jon

Nothing blocking. Flagging the filesystem-vs-HTTP scope decision in the PR body
so the HTTP-transport follow-on is clearly owned.
