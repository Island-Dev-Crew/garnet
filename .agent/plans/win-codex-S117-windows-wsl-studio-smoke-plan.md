# win-codex S117 Windows/WSL Studio Smoke Plan

Scope: a package-pipeline proof increment, not full S117 release completion.

1. Add a repo-owned Windows/WSL Studio smoke recorder that writes manifest-backed evidence under `proofs/windows/studio/` and `proofs/linux/execution/studio/`.
2. Prove the recorder with focused tests: Windows evidence shape, WSL portability labeling, manifest verification, and forbidden-claim guardrails.
3. Run the real Windows Tauri `--studio-smoke` path and the WSL command-contract replay where local tooling is present.
4. Update readiness/status text only for reproducible committed evidence, preserving Linux desktop launch, signed MSI, winget, Windows ARM64, and production/v1.0 as open.
5. Validate with focused tests, workspace tests, clippy, format/diff checks, dogfood PR body gate, remote CI, and Chrome Work-profile merge.

Honesty boundary: WSL output is portability/command-contract evidence only. It is not Linux seccomp, OS-sandbox enforcement, Wasmtime fuel, Linux desktop GUI launch, or native Linux package proof.
