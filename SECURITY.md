# Security Policy

## Supported Versions

| Version | Status          | Security fixes |
|---------|-----------------|----------------|
| 0.8.1   | Current release | ✅ Yes         |
| 0.8.0   | Previous        | ✅ Yes (critical only) |
| 0.5.0   | Previous        | ❌ No          |
| ≤ 0.4.x | Archived        | ❌ No          |

Garnet follows a forward-compatible security support window: the current release + one prior receive security fixes; older releases require upgrade.

## Reporting a Vulnerability

**Do NOT open a public issue for security vulnerabilities.**

Instead, use one of:

1. **GitHub Security Advisory** (preferred): open a private advisory at [github.com/Island-Dev-Crew/garnet/security/advisories/new](https://github.com/Island-Dev-Crew/garnet/security/advisories/new). GitHub notifies the maintainer privately; the disclosure stays invisible to the public until published.
2. **Email**: `jon-isaac@islanddevcrew.com`. **Voice/SMS (security reports only)**: +1 256-652-5534. PGP-encrypt if you have a key reference from a prior handoff; otherwise plaintext is fine — the maintainer will respond with a secure channel. Please reserve the phone line for security disclosure, not general or support questions.

### What to include

- The affected version(s) — `garnet --version` output
- The threat model you're breaking: capability escape? manifest-signature forgery? state-cert type confusion? hot-reload replay? something else entirely?
- A minimal reproducer (a `.garnet` source file + the CLI commands to invoke it)
- The observed vs. expected behavior, and the severity you'd assign (low/medium/high/critical) with rationale
- Any exploit code you've developed (but keep it private — don't publish proof-of-concept until a fix ships)

### Response timeline

- **Within 48 hours**: acknowledgment of receipt + initial severity triage
- **Within 7 days**: preliminary assessment — either a fix is in progress, or we need more information, or we've determined it's not a security issue (with rationale)
- **Within 30 days** (critical) / **90 days** (high) / **180 days** (medium/low): public disclosure after a fix ships. Extended embargo possible by mutual agreement if the fix is genuinely complex.

### Coordinated disclosure

We follow responsible-disclosure norms. The reporter and the project coordinate on:

- The public disclosure date
- The CVE assignment (if applicable)
- The credit line in the advisory + release notes

If the reporter prefers to remain anonymous, that's honored.

### What qualifies as a security issue

**In scope:**

- Capability escape — code with `@caps()` successfully invoking a runtime-gated primitive whose capability it lacks (e.g. `fs::read_file`, which requires `@caps(fs)`); the runtime-gated surface is the one listed in [GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md](C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md)
- Manifest signature forgery — a `garnet verify --signature` accepting a tampered signed manifest
- Hot-reload replay — a ReloadKey-signed reload from a stale sequence number being accepted
- StateCert type confusion — a hot-reload surviving a type mismatch via BLAKE3 fingerprint collision or bypass
- Compiler impersonation — a malicious compiler producing a manifest that verifies against a legitimate release pubkey
- Strategy-miner poisoning — adversarial training-time injection into the knowledge graph that survives to runtime
- Path traversal or escape through an OS-sandbox boundary that is actually applied — today that is the Linux seccomp boundary of the reference harness (`garnet sandbox` generates policy; it does not self-enforce it)
- Remote code execution via any network primitive
- Any confidentiality / integrity / availability breach that bypasses a claim from Papers III/V/VI or the v3.4 Security V2 spec

**Declared, not yet enforced** (reports welcome as roadmap findings, not vulnerabilities): `@bounded` (Wasmtime fuel), memory, time, `@mailbox`, and macOS/Windows OS-sandbox application. The repo's operating brief (`CLAUDE.md`) fixes the line: "`@bounded` (Wasmtime fuel), memory, time, `@mailbox`, and macOS/Windows OS-sandbox application remain declared-not-enforced; only `@caps` + `@max_depth` are enforced (both backends), with seccomp applied on **Linux only**." The enforcement scope table says the same of OS sandboxing — "macOS / Windows OS-sandbox application is **named-deferred**" — and limits the runtime claim to "`@caps` and `@max_depth` trap identically on both backends for the gated surface" ([GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md](C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md)). An `Actor::tell` accepting unbounded messages despite `@mailbox(N)` is therefore expected behavior today, not a bypass; file it as a public issue labeled as a roadmap finding. The same applies to `@sandbox` policy on macOS and Windows, where OS-sandbox application is **named-deferred** in the enforcement scope document: an escape there is a roadmap finding, not a vulnerability, until the boundary is applied.

**Not in scope** (open public issue instead):

- Bugs that crash the `garnet` binary cleanly (no data loss / privilege escalation)
- Incorrect error messages, unhelpful diagnostics, typos in documentation
- Performance issues / memory leaks that don't expose a capability boundary
- Issues in third-party dependencies (file with the upstream project, note here for tracking)
- Social-engineering / physical-access / supply-chain attacks outside the project's TCB

### Proof-of-concept policy

PoCs that don't actually execute — e.g., "I think this is exploitable because..." — are welcome; we'll triage with you. PoCs that execute should be developed privately until the fix ships; please don't publish a working exploit before disclosure.

## Published advisories

Past security advisories are published at [github.com/Island-Dev-Crew/garnet/security/advisories](https://github.com/Island-Dev-Crew/garnet/security/advisories).

Security-specific tests were added across four historical hardening layers (v3.3 Layer 1 through v4.0 Layer 4). No current count of those tests is published here: `docs/truth.json` (`omissions.security_test_count`) records that no trusted derivation exists for the historical "136 security tests" figure, so this document does not restate it. The original threat model is documented in [GARNET_v3_3_SECURITY_THREAT_MODEL.md](F_Project_Management/GARNET_v3_3_SECURITY_THREAT_MODEL.md) — a roadmap of 15 hardening patterns, two of which address Garnet-specific threat classes (strategy-miner adversarial training, `Box<dyn Any>` hot-reload type confusion).

## Cryptographic primitives

- **Ed25519** via `ed25519-dalek 2.2.0` (per `Cargo.lock`) for: manifest signing (ManifestSig, v3.4.1) and signed hot-reload (ReloadKey, v3.5).
- **BLAKE3** via `blake3 1.8.4` (per `Cargo.lock`) for: deterministic manifest hashes, prelude hashes, StateCert type fingerprints.
- **SHA-256** via `sha2 0.10.9` (per `Cargo.lock`) for: HMAC-SHA-256 (CacheHMAC, v3.3 Layer 1).

No in-house cryptography. All primitives are battle-tested libraries with established audit histories.

## Release signing

Every `v*` tag pushed to the GitHub repo triggers `.github/workflows/linux-packages.yml`. On a tag, the workflow builds `.deb` + `.rpm` packages (`build-packages` job) and macOS CLI tarballs (`macos-cli-tarballs` job: `garnet-<version>-{aarch64,x86_64}-apple-darwin.tar.gz`); its `release` job then generates a CycloneDX SBOM (`garnet-sbom-cyclonedx.tgz`), composes one `SHA256SUMS` over the `.deb`, `.rpm`, `.tar.gz`, and `.tgz` assets, and publishes them as GitHub Release assets. When the `GPG_SIGNING_KEY` repository secret is present, the job also signs `SHA256SUMS` and attaches the detached signature `SHA256SUMS.asc`; when it is absent, the job fails closed unless the repository variable `ALLOW_UNSIGNED_RELEASE=true` is set. The `v0.8.1` Release carries `SHA256SUMS.asc` and the SBOM; `v0.8.0` predates signing and carries an unsigned `SHA256SUMS`. The public key is [docs/garnet-release-signing.pub.asc](docs/garnet-release-signing.pub.asc) (fingerprint `04D5 6F91 F038 17DD FFEB  C62A C14D F6E7 1395 6ED1`); the verification procedure is in [docs/release-signing.md](docs/release-signing.md). The VS Code extension `.vsix` assets on the same Release are published by `.github/workflows/vscode-extension.yml` and are not covered by `SHA256SUMS`.

The `https://garnet-lang.org/install.sh` installer (`docs/install.sh`) fetches `SHA256SUMS` from the same GitHub Release and verifies the SHA-256 of each downloaded asset before running the native installer; it does not fetch or verify `SHA256SUMS.asc`.

Current universal-installer integrity is SHA-256 based. Platform signing remains platform-specific: macOS packages should be Developer ID signed and notarized, and Windows MSI packages should be Authenticode signed and timestamped before publication. Do not claim release-signature verification in the installer until a public release key is pinned in the script and the verification path is implemented.

---

*"Be sober, be vigilant." — 1 Peter 5:8*
