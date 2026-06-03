# Garnet S110 Ultrapunch Reproduction

- platform: `linux`
- evidence tier: `portability-repro`
- status: `passed`
- accept artifacts retained: capability_manifest.json, diff_caps.txt, seal.json, transparency_log.jsonl, decision.md
- transparency log verified: yes
- widening refused and never sealed: yes
- over-depth refused and never sealed: yes

Honest scope: accepted on capability + depth evidence only. WSL/Linux rows from this recorder are portability-repro evidence unless paired with a separate real-kernel enforcement proof. This is not seccomp, OS-sandbox, Wasmtime fuel, production, or v1.0 proof.
