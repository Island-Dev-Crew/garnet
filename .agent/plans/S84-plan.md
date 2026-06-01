# S84 - Exp 3 Windows WSL/bash path proof

Contract: `F_Project_Management/WINDOWS_AUDIT_S1_S80.md` -> WIN-S71-001.

## Goal

Close the Windows-only Paper VI Exp 3 provider-free harness failure where
`scripts/garnet_paper_vi_exp3_status.py` passes Windows absolute paths to WSL
`bash`, producing `/bin/bash: C:\...\run_stateless.sh: No such file or directory`.

## Scope

- Touch `scripts/garnet_paper_vi_exp3_status.py`.
- Record the Windows proof in `F_Project_Management/WINDOWS_AUDIT_S1_S80.md`.
- Keep the provider-backed H3A re-run pending-infra; no LLM provider call.

## Dogfood

```powershell
python -B scripts\test_garnet_paper_vi_exp3_status.py
python -B scripts\garnet_paper_vi_exp3_status.py --gate --format json
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
```

## Honest Scope

This slice proves the provider-free harness shape runs on Windows. It does not
re-measure Paper VI H3A and does not call an LLM provider.
