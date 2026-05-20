# Garnet v0.5 draft: six slices merged, tag still gated

Status: draft, not a v0.5.0 release announcement  
Audience: contributors, reviewers, and future release agents  
Source of truth: `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md`

Garnet's v0.5.0 blocking slice work has crossed the important engineering
line: S1, S2, S5, S8, S9, and S10 are merged on `main` with dogfood evidence.
That is real progress. It is also not a reason to loosen the language around
what Garnet is.

The standing description remains:

> "research-grade prototype (v0.x.x) — not production-complete"

The tracked slice work is complete for the v0.5.0 blocking lane, but the release
gate still requires clean-machine reproduction and published-editor evidence
before a tag should exist. In the repo's own words:

> "tracked-slice ledger is complete, but that is not full MIT/productization completion"

## What the six blocking slices added

S1 added the Language Server Protocol MVP: parser/checker diagnostics, hover,
and basic go-to-definition through `garnet-lsp/` plus the VSCode extension
launcher in `editors/vscode/`. The honest status is source-present with protocol
smoke evidence and local standalone VS Code diagnostic proof. Published VSIX
evidence, the full manual hover/go-to-definition screenshot trio, safe-mode
hover, workspace symbols, rename, and CST-grade incremental precision remain
deferred.

S2 added the bytecode VM scaffold. `garnet-vm/` now has a deterministic
serializer, loader, execution surface for the MVP fixture path, 15 native opcode
families, function-level fallback to the tree-walk interpreter, and a bounded
Criterion benchmark harness. This is a scaffold, not a stable bytecode ABI or a
production VM.

S5 added the parser fuzz harness. The `garnet-parser-v0.3/fuzz/` sub-workspace
ships a `parse_input` cargo-fuzz target seeded from canonical Garnet examples
and bounded by a strict `ParseBudget`. Local 60-second smoke evidence and a
scheduled nightly workflow exist. Accumulated long-running fuzz hours,
interpreter/checker fuzzing, differential fuzzing, and OSS-Fuzz integration are
not claimed.

S8 added the signed hot-reload BLAKE3 demo pair. The success example prints
`reloaded successfully`; the mismatch example exits nonzero with
`BLAKE3 fingerprint mismatch`. This closes the Paper VI Contribution 5 surface
gap at the demonstration level. It does not expose managed-mode
`actor.reload_signed` syntax and it does not claim full Ed25519 payload
verification in managed mode.

S9 added cross-machine determinism CI. GitHub Actions now builds the deterministic
manifest on ubuntu-latest and macos-latest with a shared short-lived signing key
and fails if the per-OS SHA-256 manifests diverge. This closes the Paper VI
Contribution 6 verification gap for the current manifest surface. Windows,
Linux aarch64, key rotation, and byte-for-byte native backend determinism remain
future work.

S10 added the compiler-as-agent advisory tier without adding an LLM. The current
engine is deterministic and rules-based: `garnet check --suggest` emits
`compiler suggested:` lines for known local patterns. That establishes the
compiler-suggestion surface while keeping provider-backed suggestions
pending-infra.

## The number moved up, but the caveat stayed

After these merges, `scripts/garnet_mit_readiness_status.py` reports `67.9%`
across 17 productization lanes. That is higher and more granular than the
v0.4.2 productization pulse, but it is still an `active-partial` status. The
blocked and deferred lanes remain visible: Developer ID notarization,
clean-machine Gatekeeper proof, Windows/Linux runtime proof, mobile distribution,
broad converter frontends, LLM assist, proof/empirics, and human review gates.

Paper VI's scorecard also stays explicit:

> Paper VI scorecard: "4 supported, 2 partial (downgraded honestly), 0 refuted, 1 pending-infra"

Two other anchors remain in force:

- "production allocator path tracked in MEMORY_CORE_ROADMAP.md"
- "human/aesthetic acceptance remains open"

Those phrases are not hedges. They are part of the product's credibility.

## Why the tag waits

The v0.5.0 tag should wait until the release gate in
`F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` is reproducible from a clean
machine. The current open item is not code completion; it is release proof:

- install from the public installer path;
- create a fresh CLI project;
- run `garnet test` and `garnet run src/main.garnet`;
- install the Garnet VSIX or published extension artifact;
- confirm diagnostics on an injected syntax error.

Post-merge evidence now proves the public installer source-fallback path on this
Mac with a configured Rust toolchain, and Mac-local Cursor evidence proves the
VSIX can surface diagnostics on an injected syntax error. A clean isolated
standalone VS Code 1.121.0 run now proves the locally packaged VSIX can launch
its bundled `garnet-lsp` and report the injected diagnostic without a workspace
`garnet.lsp.path`. That is useful release-candidate evidence, but it is not the
same as release-backed packages or a published VSIX artifact.

The release path now has a sharper next gate: Garnet can build host-native VSIX
artifacts with bundled `garnet-lsp` binaries, publish those artifacts on `v*`
GitHub Releases, and fail the release smoke if the matching VSIX is absent or
structurally incomplete. The same release path now stages macOS CLI tarballs for
the public installer's Mac fallback path and folds them into the same
`SHA256SUMS` manifest as the Linux packages. Local M5 evidence proves the
tarball shape through the installer in release-only mode from a file-backed
release directory, but that still is not Marketplace/OpenVSX publication, not a
signed/notarized `.pkg`, and not a v0.5.0 tag. Until the release-backed
installer packages and VSIX assets exist on the organization release and the
clean-machine smoke passes, v0.5.0 is merge-ready in substance but not tag-ready
in release discipline. That is the right shape for Garnet: substance first,
public claim second.
