# S54 Plan — VS Code / OpenVSX / Marketplace path

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S54.
Map: reconciled plan §156 — "VS Code / OpenVSX / Marketplace path".
Branch: `codex/s54-vscode-publish`. Base: `origin/main` @ `6ef4ad4` (S53).

## Landscape
`editors/vscode/package.json` has most marketplace metadata + README/LICENSE;
`vscode-extension.yml` builds VSIX + publishes a GitHub release asset on tag. No
OpenVSX/Marketplace publish (needs ovsx/vsce + OVSX_TOKEN/VSCE_PAT).

## Deliverables
- Add `keywords` to editors/vscode/package.json (discoverability).
- `scripts/garnet_vscode_publish_readiness.py`: assert marketplace-required +
  recommended fields + README/LICENSE present; report the publish path;
  `--gate` fails on regression. `--format md|json`.
- `scripts/test_garnet_vscode_publish_readiness.py`: 5 unit tests.
- Wire test + `--gate` into ci.yml agent-contracts.
- CHANGELOG + contract S54 block.

## Dogfood
- `garnet_vscode_publish_readiness.py --format md` → marketplace-ready (no
  missing required/files); `--gate` exits 0.

## Honest scope (do not soften)
- The OpenVSX/Marketplace PUBLISH needs OVSX_TOKEN/VSCE_PAT = credential/account
  territory → DEFERRED to a human, not done autonomously. S54 makes it
  publish-READY + documents the path; publishes nothing.
- Extension version stays 0.7.0 (v0.8 release-versioning is a release decision).
- No new readiness lane.

## Gates
- readiness gate + tests + ladder (zero Rust changed). Ledger: `s53 → merged(5)`
  advanced this branch; `s54` rides with S55.
