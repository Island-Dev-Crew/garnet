# Security Policy

## Supported Versions

| Version | Status          | Security fixes |
|---------|-----------------|----------------|
| 0.4.2   | Current release | ✅ Yes         |
| 0.4.1   | Previous        | ✅ Yes (critical only) |
| 0.4.0   | Previous        | ❌ No          |
| ≤ 0.3.x | Archived        | ❌ No          |

Garnet follows a forward-compatible security support window: the current release + one prior receive security fixes; older releases require upgrade.

## Reporting a Vulnerability

**Do NOT open a public issue for security vulnerabilities.**

Instead, use one of:

1. **GitHub Security Advisory** (preferred): open a private advisory at [github.com/IslandDevCrew/garnet/security/advisories/new](https://github.com/IslandDevCrew/garnet/security/advisories/new). GitHub notifies the maintainer privately; the disclosure stays invisible to the public until published.
2. **Email**: `security@garnet-lang.org` (once the domain is registered) or `jon@island-dev-crew.example`. PGP-encrypt if you have a key reference from a prior handoff; otherwise plaintext is fine — the maintainer will respond with a secure channel.

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

- Capability escape — code with `@caps()` successfully invoking a primitive requiring `@caps(fs)`
- Manifest signature forgery — a `garnet verify --signature` accepting a tampered signed manifest
- Hot-reload replay — a ReloadKey-signed reload from a stale sequence number being accepted
- StateCert type confusion — a hot-reload surviving a type mismatch via BLAKE3 fingerprint collision or bypass
- Compiler impersonation — a malicious compiler producing a manifest that verifies against a legitimate release pubkey
- Strategy-miner poisoning — adversarial training-time injection into the knowledge graph that survives to runtime
- BoundedMail bypass — an `Actor::tell` accepting unbounded messages despite `@mailbox(N)` annotation
- Path traversal / sandbox escape in `@sandbox` code
- Remote code execution via any network primitive
- Any confidentiality / integrity / availability breach that bypasses a claim from Papers III/V/VI or the v3.4 Security V2 spec

**Not in scope** (open public issue instead):

- Bugs that crash the `garnet` binary cleanly (no data loss / privilege escalation)
- Incorrect error messages, unhelpful diagnostics, typos in documentation
- Performance issues / memory leaks that don't expose a capability boundary
- Issues in third-party dependencies (file with the upstream project, note here for tracking)
- Social-engineering / physical-access / supply-chain attacks outside the project's TCB

### Proof-of-concept policy

PoCs that don't actually execute — e.g., "I think this is exploitable because..." — are welcome; we'll triage with you. PoCs that execute should be developed privately until the fix ships; please don't publish a working exploit before disclosure.

## Published advisories

Past security advisories are published at [github.com/IslandDevCrew/garnet/security/advisories](https://github.com/IslandDevCrew/garnet/security/advisories).

v4.2 ships with 136 security-specific tests across 4 hardening layers (v3.3 Layer 1 through v4.0 Layer 4). The threat model is documented in `Garnet_Final/F_Project_Management/GARNET_v3_3_SECURITY_THREAT_MODEL.md` — 15 hardening patterns, two of which are novel Garnet-specific classes (strategy-miner adversarial training, `Box<dyn Any>` hot-reload type confusion) with no prior art elsewhere.

## Cryptographic primitives

- **Ed25519** via `ed25519-dalek 2.1` for: manifest signing (ManifestSig, v3.4.1) and signed hot-reload (ReloadKey, v3.5).
- **BLAKE3** via `blake3 1.5` for: deterministic manifest hashes, prelude hashes, StateCert type fingerprints.
- **SHA-256** via `sha2` for: HMAC-SHA-256 (CacheHMAC, v3.3 Layer 1).

No in-house cryptography. All primitives are battle-tested libraries with established audit histories.

## Release signing

Every `v*` tag pushed to the GitHub repo triggers `.github/workflows/linux-packages.yml`, which builds `.deb` + `.rpm` + `SHA256SUMS` and publishes them as a GitHub Release asset. The `sh.garnet-lang.org/install.sh` installer fetches `SHA256SUMS` from `releases.garnet-lang.org` and verifies every downloaded asset before running the native installer.

Binaries themselves are Ed25519-signed by the project's release key (pubkey pinned in the install script). Signature verification is mandatory when installing via the universal installer.

---

*"Be sober, be vigilant." — 1 Peter 5:8*
