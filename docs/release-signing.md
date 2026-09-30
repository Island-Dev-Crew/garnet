# Garnet release signing & verification

How to check that a downloaded Garnet release is intact and was signed by the
release key.

> **Status (2026-09-30).** Every release ships a `SHA256SUMS` manifest (integrity).
> The **`v0.8.1` (re-cut 2026-06-07) and `v0.8.2` Releases are signed**: each carries
> `SHA256SUMS.asc`, a detached GPG signature by the key in the table below. The
> release key is being rotated, and `v0.8.3` and later are signed by the new key.
> Earlier releases (e.g. `v0.8.0`, `v0.5.0`) predate signing and are **unsigned**, not
> tampered. `install.sh` and `install.ps1` check the signature for you when `gpg` is
> installed (section 1). Garnet is a research-grade prototype, not production/1.0.

## Release keys

| Releases | Signing key fingerprint |
|---|---|
| `v0.8.1`, `v0.8.2` | `04D5 6F91 F038 17DD FFEB  C62A C14D F6E7 1395 6ED1` |
| `v0.8.3` and later | the rotated key, added here before `v0.8.3` is tagged |

Every release public key is in [`docs/garnet-release-keys.asc`](garnet-release-keys.asc),
served at `https://garnet-lang.org/garnet-release-keys.asc`. The current key alone is
also published as [`docs/garnet-release-signing.pub.asc`](garnet-release-signing.pub.asc).
The keys file can hold several keys. What decides is the fingerprint for the version
you install, from this table or from the installer's pin.

## 1. What the installers check

When `gpg` is on `PATH`, `install.sh` and `install.ps1` do this before they trust any
checksum:

1. download `SHA256SUMS`, `SHA256SUMS.asc` and the keys file;
2. import the keys into a throwaway keyring, never your own;
3. run `gpg --verify SHA256SUMS.asc SHA256SUMS`, which accepts only a detached
   signature over that exact file;
4. require the signing key's fingerprint to be the one the installer pins for that
   version. A valid signature by any other key in the file is refused;
5. only then look up the asset's line in `SHA256SUMS` and check its SHA-256.

A missing, mismatched or wrong-key signature stops the install. `install.sh` never
falls back to a source build after a signature failure.

- **Without `gpg`**, both installers warn and trust `SHA256SUMS` on integrity only.
  On Windows, [Gpg4win](https://gpg4win.org) provides `gpg`.
- **Releases before `v0.8.1`** were never signed, so the installers warn and check
  integrity only.
- **`GARNET_VERIFY_SIGNATURE=0`** turns the check off, with a warning. It is meant for
  local test assets.
- **`GARNET_SIGNING_KEYS_URL` and `GARNET_SIGNING_KEY_FPR`** point the check at a
  mirror you sign yourself.

**What the check does not cover.** The installer script, its pinned fingerprints and
the keys file all come from `garnet-lang.org`. The check stops a substituted or
altered release asset, `SHA256SUMS` or `SHA256SUMS.asc`. It does not stop someone who
controls `garnet-lang.org` from serving an installer with a different pin. To take
that site out of the chain, verify by hand (sections 2 and 3) against a fingerprint
you got from a source other than `garnet-lang.org`.

## 2. Integrity by hand

Every GitHub Release attaches a `SHA256SUMS` covering each installer artifact. After
downloading the asset(s) and `SHA256SUMS` into one directory:

```sh
sha256sum --check --ignore-missing SHA256SUMS
```

Every line you downloaded must print `OK`.

On Windows, in PowerShell, compare the zip's hash with its line in `SHA256SUMS`:

```powershell
(Get-FileHash .\garnet-<version>-x86_64-pc-windows-msvc.zip -Algorithm SHA256).Hash.ToLower()
Select-String -Path .\SHA256SUMS -Pattern 'x86_64-pc-windows-msvc.zip'
```

The two hashes must be identical. Windows assets are published starting with `v0.8.2`.

## 3. Authenticity by hand

Signed releases attach `SHA256SUMS.asc`. With `SHA256SUMS`, `SHA256SUMS.asc` and
`garnet-release-keys.asc` in one directory:

```sh
# one-time: import the published public keys
gpg --import garnet-release-keys.asc

# verify the signature over the checksum manifest
gpg --verify SHA256SUMS.asc SHA256SUMS
```

`gpg` prints `Good signature from "Garnet Release Signing ..."` and the key it was
made with (`using EDDSA key ...`). That fingerprint must match the table above for
the version you downloaded. A good signature by any other key does not count. Then
check each artifact against `SHA256SUMS` (section 2).

## 4. What this does and does not attest

- **Does:** the artifacts match a checksum manifest signed by the maintainer's key.
- **Does not:** independently attest the *build* (no reproducible-build witness here),
  nor replace the in-language artifact provenance — Garnet's own `garnet build --sign`
  / `garnet verify` Ed25519 manifests and the in-toto `seal` are a separate, additive
  layer for `.garnet` artifacts. A CycloneDX **SBOM** (`garnet-sbom-cyclonedx.tgz`) is
  also attached to signed/unsigned releases alike.
- The VS Code extension `.vsix` assets on the same Release come from a separate
  workflow and are not covered by `SHA256SUMS`.
- Cosign/Sigstore keyless signing and a reproducible-build attestation remain
  **deferred** (pre-1.0 work).

## For maintainers

### Where the signing key lives

The `release` job in `.github/workflows/linux-packages.yml` runs in the `release`
environment. That environment admits only `v*` tag runs, waits for its required
reviewer to approve, and holds the only copy of the key as the environment secrets
`GPG_SIGNING_KEY` and `GPG_PASSPHRASE`. No other workflow or event can read them.
Without the key, a tagged release fails closed, unless the repository variable
`ALLOW_UNSIGNED_RELEASE` is set to `true` to ship unsigned on purpose.

Set the secrets from a terminal no agent session drives. The private key never
leaves your machine except as the encrypted secret:

```sh
gpg --armor --export-secret-keys <FPR> | gh secret set GPG_SIGNING_KEY --env release --repo Island-Dev-Crew/garnet
gh secret set GPG_PASSPHRASE --env release --repo Island-Dev-Crew/garnet
```

### Rotating the key

In one change, before the first tag the new key signs:

1. Add the new public key to `docs/garnet-release-keys.asc`. Keep the old keys:
   older releases still verify against them. Publish only role-address user IDs
   (`hello@`), never a personal address.
2. Publish the new key alone as `docs/garnet-release-signing.pub.asc`.
3. Pin its fingerprint in `docs/install.sh` (and its copy under `installer/`) and in
   `docs/install.ps1` for the versions it signs. Older versions keep their pins.
4. Add its row to the table above.

Then replace the environment secrets with the new key.

### Immutable releases

GitHub's immutable releases stay **off** for `v0.8.3`. Two workflows publish to the
same release: the `release` job in `linux-packages.yml` (packages, `SHA256SUMS`, then
`SHA256SUMS.asc` in a second step) and the `release-vsix` job in
`vscode-extension.yml`. Each publishes a non-draft release, so with immutability on,
whichever upload comes after the first publish would likely be refused. Before
immutability is turned on, the release has to become one draft that a single final
step publishes.
