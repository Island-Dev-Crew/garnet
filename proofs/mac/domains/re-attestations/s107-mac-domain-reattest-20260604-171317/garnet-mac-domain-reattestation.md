# Garnet Mac Domain Re-Attestation

- Status: `verified`
- Slice: `S107 Mac-native domain execution`
- Source PR: `#356`
- Source merge commit: `981c14bce97eb5aa556479282b0cedeca8e159fb`
- Re-attestation base: `b2ba06892b46c2d3c66905af8352bf7d1e429f33`
- Verified bundle: `proofs/mac/domains/mac-domain-proofs-20260604-064412/garnet-mac-domain-proofs.json`

## Scope

This remedial record re-attests only the Mac-native S105 domain execution proof
that was originally bundled into PR #356. It does not re-attest the Studio UI
proof, and it does not turn PR #356 into a clean one-slice PR after the fact.

## Fresh Verification

| Command | Exit | Result |
| --- | ---: | --- |
| `python3 scripts/smoke_garnet_mac_domain_proofs.py --garnet target/release/garnet --verify proofs/mac/domains/mac-domain-proofs-20260604-064412/garnet-mac-domain-proofs.json` | 0 | `mac-domain-proofs: verified` |
| `python3 scripts/test_smoke_garnet_mac_domain_proofs.py` | 0 | `Ran 1 test in 0.370s; OK` |
| `python3 scripts/garnet_mit_readiness_status.py --check-no-regression` | 0 | `active-partial 91.2%; Mac-Codex domain execution proof verified` |

## Honest Scope

- Mac-native domain execution only.
- No Studio UI re-attestation in this record.
- No Windows/Linux completion claim.
- No macOS OS-sandbox, seccomp, Wasmtime fuel, production, v1.0, release tag, or Stage P claim.
