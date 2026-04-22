<!-- Thanks for contributing to Garnet. Fill in the sections below; delete any that don't apply. -->

## Summary

<!-- 1-3 sentences: what does this PR change, and why? -->

## Related issue(s)

Fixes #<!-- issue number, if applicable -->

## Kind of change

- [ ] Bug fix (non-breaking change that fixes a reported issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Documentation / spec update (no code change)
- [ ] Breaking change (alters public API, language surface, or on-disk format)
- [ ] Internal refactor / infrastructure (no user-visible change)
- [ ] Security fix (uses SECURITY.md disclosure flow, not this template — redirect if so)

## Subsystem(s) touched

<!-- Tick every crate / doc area affected -->

- [ ] `garnet-parser` / `garnet-parser-v0.3`
- [ ] `garnet-interp-v0.3` (including `stdlib_bridge.rs`)
- [ ] `garnet-check-v0.3` (including `caps_graph.rs`)
- [ ] `garnet-memory-v0.3`
- [ ] `garnet-actor-runtime`
- [ ] `garnet-cli` (including `new_cmd`, `convert_cmd`, `manifest`)
- [ ] `garnet-stdlib`
- [ ] `garnet-convert`
- [ ] Mini-Spec or research papers
- [ ] Installer scaffolding (wix/, macos/, linux/, installer/)
- [ ] GHA workflow(s) in `.github/workflows/`

## Test plan

<!-- Every PR that touches code must describe how it was verified. -->

- [ ] `cargo check --workspace --tests` clean (0 errors)
- [ ] `cargo test -p garnet-actor-runtime --release --lib` (expect 17 pass)
- [ ] `cargo test -p garnet-stdlib --release` (expect 74 pass)
- [ ] `cargo test -p garnet-convert --release` (expect 85 pass)
- [ ] New unit tests added for new code: **<list tests here>**
- [ ] `cargo clippy --workspace --all-targets` — no new warnings vs. main

## Rigor checklist

<!-- Garnet's pre-registration discipline. Tick each that applies. -->

- [ ] This change does NOT alter a pre-registered Paper VI threshold post-hoc.
- [ ] If this change adds a new primitive, it has an entry in `garnet-stdlib/src/registry.rs` with the correct `RequiredCaps`.
- [ ] If this change affects the capability inventory, the Mini-Spec + Paper III capability section + `FAQ.md` have been updated.
- [ ] If this change affects the on-disk manifest format, the schema version has been bumped AND old manifests still parse (or the break is documented in the PR description with the migration path).
- [ ] If this change affects `garnet convert`, the lineage JSON schema is preserved (or the break is documented).

## Documentation

- [ ] Public API additions have rustdoc comments
- [ ] User-visible changes are reflected in `README.md` or `FAQ.md` as appropriate
- [ ] If this closes a Known Issue from `GARNET_v4_2_HANDOFF.md`, the handoff has been updated

## Checklist before merge

- [ ] Commit messages are descriptive (not just "fix" or "update")
- [ ] I have read `CONTRIBUTING.md`
- [ ] I have read `CODE_OF_CONDUCT.md`
- [ ] I agree to license my contribution under MIT OR Apache-2.0 (the repo's dual license)
