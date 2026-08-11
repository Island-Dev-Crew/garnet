# WV-6 integration re-acceptance — Request 02 addendum
The audit base was corrected from the frozen head to the reviewed head per this ruling: `git diff --name-only 35ddc22809d647ae6637e280db560efa3cc537ed..HEAD` closes at six paths, all under the ruled record/proof allowlist, with zero product paths.

| U-35 boundary | live content digest / path count | drift class |
|---|---|---|
| reviewed tip `35ddc22809d647ae6637e280db560efa3cc537ed` | `fd96e6d910180f5e33999fbd693ea211e336389a13535930d89b2a870ff54727 / 1606` | accepted reviewed baseline |
| reviewer verdict `23bc563fc6a237045912044843078b87688bafcf` | `d7a08be5e22b2033fb65aa1808a3eb9cd745022c909f9151b9d19ec918188c0a / 1607` | reviewer's journal/verdict record drift |
| this one-commit cure successor | `tip-computed / 1608` | cure records plus two byte-identical `proofs/**` relocations; the live digest is intentionally not embedded in its own digest-included request |

Review the exact fetched one-commit successor of `23bc563fc6a237045912044843078b87688bafcf`: confirm the U-35 table and drift-class audit; verify B1's two byte-identical transcript relocations to `proofs/windows/launch-verification/wv6-transcripts/` and green text-byte gate (the verdict's suggested path sat inside a manifest-closed bundle, so the destination was adjusted and the acceptance manifest does not move), F3's full base hash and encoding corrections, F5's discoverable U-52/U-53 registrations, and the unchanged frozen `fd96e6d910180f5e33999fbd693ea211e336389a13535930d89b2a870ff54727 / 1606` pair. The WV-6 verifier is accepted at frozen head `410ff1182cdcefcec9fe046d1346205d8522ec9d` and is `PARTIAL` at the tip by record-class drift only; write any verdict only to `ops/wv6-reaccept/review/02-verdict.md`.
