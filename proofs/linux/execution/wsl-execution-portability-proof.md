# Garnet S106 wsl Stage V trap proof

_Schema garnet.windows_cross_os_enforcement_proof/v1._

- tier: `execution-portability`
- honesty scope: WSL execution/portability, not enforcement
- git head: `f18da44fa22410f2c0b60779e0f7e0a5405e3265`
- commands: s101_gate=pass, bounded_enforcement=pass, caps_enforcement=pass
- required traps: @max_depth, @caps(env), @caps(proc), @caps(fs), @caps(net), S92 program-entry @caps(proc)
- verdict: ok

Named deferred boundaries:
- WSL is not Linux seccomp enforcement
- WSL is not OS-sandbox enforcement
- Wasmtime fuel / @bounded runtime enforcement remains out of scope
- memory/time/@mailbox runtime ceilings remain out of scope
