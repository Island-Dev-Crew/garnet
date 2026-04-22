# Universal shell installer (v4.2 Phase 6A)

`install.sh` in this directory is the script served at
`https://sh.garnet-lang.org`. End users install Garnet with:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.garnet-lang.org | sh
```

## Behavior

1. Print the ASCII wordmark.
2. Detect OS (Linux / Darwin) + arch (x86_64 / aarch64).
3. Pick a package format (`deb` if dpkg present, `rpm` if rpm present,
   `pkg` on macOS, tarball fallback).
4. Download the corresponding asset from
   `https://releases.garnet-lang.org/<version>/garnet-<version>-<triple>.<ext>`.
5. Fetch `SHA256SUMS`, verify the download.
6. Run the native installer (`dpkg -i`, `dnf install -y`,
   `installer -pkg`, or tarball extract).
7. Print `garnet --version` on success.

## Why not trust TLS alone?

The script is served over HTTPS — that's the transport integrity. The
`SHA256SUMS` file provides asset integrity: even a compromised CDN
edge that serves a bad `.deb` cannot fool `sha256sum` unless they ALSO
control the `SHA256SUMS` manifest. Going further, the `SHA256SUMS`
file itself is expected to be signed with minisign by the release
key; verifying that signature is a future extension.

This is the same layering `rustup` and Debian repository tooling use.

## Development

Lint the script before deploying:

```sh
shellcheck installer/sh.garnet-lang.org/install.sh
```

## Deployment

The script lives in-repo for review + change-tracking. Deployment to
`sh.garnet-lang.org` copies the file to the CDN with:

- `Content-Type: text/plain`
- `Cache-Control: max-age=300, must-revalidate` (5 min — balances
  freshness against cache stampedes)
- HSTS header on the host

Version each deployed copy so a bad script can be rolled back in
minutes. Track the deployed SHA-256 of the script itself at
`https://sh.garnet-lang.org/INTEGRITY.txt` for anyone who wants to
verify the installer before piping to shell.
