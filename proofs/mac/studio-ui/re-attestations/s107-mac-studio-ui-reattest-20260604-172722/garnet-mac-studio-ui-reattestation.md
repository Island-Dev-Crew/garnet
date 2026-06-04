# Garnet Mac Studio UI Re-Attestation

- Status: `verified`
- Slice: `S107 Mac Studio UI proof`
- Source PR: `#356`
- Source merge commit: `981c14bce97eb5aa556479282b0cedeca8e159fb`
- Re-attestation base: `00ddfae9e698f0fa8f4d77966a4e7b9c815cff21`
- Prior remedial slice: PR `#362`
- Verified bundle: `proofs/mac/studio-ui/mac-studio-ui-proof-20260604-070007/garnet-mac-studio-ui-proof.json`

## Scope

This remedial record re-attests only the committed Mac Studio UI proof that was
originally bundled into PR #356. It does not re-attest Mac-native domain
execution, and it does not turn PR #356 into a clean one-slice PR after the
fact. PR #362 carries the separate Mac-native domain execution re-attestation.

## Fresh Verification

| Command | Exit | Result |
| --- | ---: | --- |
| `python3 - <<'PY' ... status._verified_mac_studio_ui_proof_under(...) ... PY` | 0 | `mac-studio-ui-proof: verified` |
| `python3 - <<'PY' ... status._committed_mac_studio_ui_proof_evidence() ... PY` | 0 | `committed-mac-studio-ui-evidence: verified` |
| `python3 scripts/garnet_mit_readiness_status.py --check-no-regression` | 0 | `active-partial 91.2%; Mac Studio UI proof verified` |
| `python3 scripts/test_garnet_mit_readiness_status.py` | 0 | `Ran 40 tests; OK` |

## Honest Scope

- Mac Studio UI wrapper proof only.
- No Mac-native domain execution re-attestation in this record.
- No Windows/Linux ownership claim.
- No macOS OS-sandbox, seccomp, Wasmtime fuel, production, v1.0, release tag, or Stage P claim.
