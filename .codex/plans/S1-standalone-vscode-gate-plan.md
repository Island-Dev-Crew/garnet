# S1 Release-Gate Plan: Standalone VS Code Editor Proof

Contract section: `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` -> `### S1 - LSP MVP` and `## v0.5.0 Release Gate`.

## Current State

At plan start, S1 source and protocol evidence were merged, and PR #197
recorded Mac-local Cursor proof. The release gate still needed standalone
Visual Studio Code diagnostic proof and published Garnet VSIX or release-backed
package evidence before any tag claim.

## Goal

Close the Mac-side standalone editor proof gap without overstating distribution status:

- Use a standalone VS Code build in an isolated throwaway location, not `/usr/local/bin/code` when it points to Cursor.
- Install the existing local `editors/vscode/garnet-0.5.0-lsp-mvp.vsix` into an isolated VS Code user data/extensions dir.
- Package the freshly built `target/release/garnet-lsp` into the VSIX as
  `server/garnet-lsp` so the isolated extension install is self-contained and
  does not rely on a local workspace path setting.
- Capture reproducible evidence for diagnostics, hover, and go-to-definition on a small `.garnet` fixture.
- Seal evidence in a Desktop dogfood bundle with a manifest.
- Update only the current-state/release-gate docs if the proof is real.

## Honest Boundaries

- This does not publish the VSIX to a marketplace.
- This does not prove release-backed package assets.
- This does not tag `v0.5.0`.
- If standalone VS Code download, launch, or GUI proof is blocked, record the exact blocker and keep the release gate open.

## Verification

Local commands/evidence to collect:

```bash
cargo build -p garnet-lsp --release
(cd editors/vscode && npm run package)
<standalone-vscode-cli> --user-data-dir <tmp-user> --extensions-dir <tmp-exts> --install-extension editors/vscode/garnet-0.5.0-lsp-mvp.vsix --force
<standalone-vscode-cli> --user-data-dir <tmp-user> --extensions-dir <tmp-exts> --list-extensions --show-versions
unzip -l editors/vscode/garnet-0.5.0-lsp-mvp.vsix | grep server/garnet-lsp
python3 scripts/smoke_garnet_lsp_protocol.py target/release/garnet-lsp
```

GUI evidence:

- Diagnostic screenshot on an injected syntax error.
- Hover screenshot on `def greet`.
- Go-to-definition before/after evidence from a call site to the function definition.

Status checks after docs update:

```bash
git diff --check
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
python3 scripts/test_garnet_mit_readiness_status.py
```

## PR Shape

- Branch: `codex/s1-standalone-vscode-gate-evidence`.
- PR title: `S1: record standalone VS Code gate evidence`.
- Body must include the evidence bundle path, exact local verification, remote CI status, and explicit deferred claims.
