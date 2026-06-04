# S108 Linux Cross-OS Enforcement Proof

- Schema: `garnet.linux_cross_os_enforcement_proof.v1`
- Status: `passed`
- Tier: `linux-enforcement-proof`
- Cross-OS role: `S108 Linux row for S109 consolidation`
- Environment: `utm-debian-12-arm64`
- Kernel: `Linux debian 6.1.0-13-arm64 #1 SMP Debian 6.1.55-1 (2023-09-29) aarch64 GNU/Linux`
- Git head: `55f0353a558672da62008bf7c40d45b4ee5cb9ee`
- Seccomp attempted: `true`
- Seccomp status: `proven`
- Denied socket trapped: `true`
- Declared net socket policy-driven: `true`
- Deterministic denied runs: `3`

## Stage-V Traps

- `max_depth`: `passed`
- `caps`: `passed`
- `s92_program_entry_proc`: `passed`

## Honest Scope

- This is the independent Linux S108 enforcement row for S109 consolidation.
- Linux seccomp is Linux-only evidence, not Windows/macOS OS-sandbox enforcement.
- This is not full S109 completion; S109 still needs a separate consolidation gate update.
- No Wasmtime fuel, production, or v1.0 claim is made.
