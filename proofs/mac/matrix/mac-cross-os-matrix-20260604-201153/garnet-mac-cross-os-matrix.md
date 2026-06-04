# Garnet Mac Cross-OS Matrix Row

- Status: `passed`
- Mac rows complete: `true`
- Cross-OS complete: `false`
- Reason: Independent Linux S108 enforcement row is not present; WSL is recorded as execution/portability only and is not Linux seccomp or OS-sandbox enforcement.

## Trap Rows

| Trap | Status | Windows | Mac | WSL |
| --- | --- | --- | --- | --- |
| `max_depth` | `passed` | `true` | `true` | `execution-portability` |
| `caps` | `passed` | `true` | `true` | `execution-portability` |
| `diff_caps_reject` | `passed` | `true` | `true` | `execution-portability` |

## Byte Comparisons

| Artifact | Byte Equal | Delta |
| --- | --- | --- |
| `accept_capability_manifest` | `true` | Must be byte-identical; declared surface is OS-independent. |
| `accept_transparency_log` | `true` | Must be byte-identical for a one-entry local chain over the same accepted proposal. |
| `accept_diff_caps` | `false` | Full text includes absolute OS paths; the path-independent verdict body must match. |
| `accept_seal` | `none` | Full seal JSON differs because the prelude_hash field differs across the older Windows baseline checkout and the current Mac proof; the OS-independent subject, AST, capability manifest, and attestation fields match. |

## Honest Scope

- This is the Mac row for S109 consolidation, not full S109 completion.
- WSL remains execution/portability only, not Linux seccomp or OS-sandbox enforcement.
- No macOS OS-sandbox enforcement, Wasmtime fuel, production, or v1.0 claim is made.
