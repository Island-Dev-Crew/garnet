# Garnet S109 Cross-OS Trap Parity Matrix

- Status: `passed`
- Cross-OS complete: `true`

## Trap Rows

| Trap | Status | Windows | Mac | Linux | WSL Treatment |
| --- | --- | --- | --- | --- | --- |
| `max_depth` | `passed` | `true` | `true` | `true` | `execution-portability; excluded=true` |
| `caps` | `passed` | `true` | `true` | `true` | `execution-portability; excluded=true` |
| `diff_caps_reject` | `passed` | `true` | `true` | `true` | `execution-portability; excluded=true` |

## Byte Comparisons

| Artifact | Byte Equal | Delta |
| --- | --- | --- |
| `accept_capability_manifest` | `true` | Must be byte-identical; declared surface is OS-independent. |
| `accept_transparency_log` | `true` | Must be byte-identical for a one-entry local chain over the same accepted proposal. |
| `accept_diff_caps` | `false` | Full text includes absolute OS paths; the path-independent verdict body must match. |
| `accept_seal` | `false` | Full seal JSON differs because the prelude_hash field differs across the older Windows baseline checkout and the current Mac proof; the OS-independent subject, AST, capability manifest, and attestation fields match. |

## Honest Scope

- Full S109 cross-OS trap parity requires committed Windows, Mac, and Linux rows.
- WSL remains execution/portability evidence and is excluded from Linux enforcement.
- Linux seccomp is Linux-only evidence, not Windows/macOS OS-sandbox enforcement.
- No Wasmtime fuel, production, release, tag, S120, or v1.0 claim is made.
