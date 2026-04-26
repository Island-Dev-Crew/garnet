# Universal shell installer (v4.2 Phase 6A)

`install.sh` in this directory is the canonical source for the script served at
`https://garnet-lang.org/install.sh`. `docs/install.sh` is the GitHub Pages copy
and should remain byte-for-byte identical. End users install Garnet with:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://garnet-lang.org/install.sh | sh
```

## Behavior

1. Print the ASCII wordmark.
2. Detect OS (Linux / Darwin) + arch (x86_64 / aarch64).
3. Pick a package format (`deb` if dpkg present, `rpm` if rpm present,
   `pkg` on macOS, tarball fallback).
4. Prefer the corresponding asset from the GitHub Release at
   `https://github.com/Island-Dev-Crew/garnet/releases/download/v<version>/`.
5. Fetch `SHA256SUMS`, verify the download.
6. Run the native installer (`dpkg -i`, `dnf install -y`,
   `installer -pkg`, or tarball extract).
7. If release assets are unavailable in auto mode, fall back to cloning the
   repository and running `cargo install --path garnet-cli --locked`.
8. Print `garnet --version` on success.

Set `GARNET_INSTALL_MODE=release` to require release assets, or
`GARNET_INSTALL_MODE=source` to skip package lookup and build from source.

## Why not trust TLS alone?

The script is served over HTTPS — that's the transport integrity. The
`SHA256SUMS` file provides asset integrity: even a compromised CDN
edge that serves a bad `.deb` cannot fool `sha256sum` unless they ALSO
control the `SHA256SUMS` manifest. Going further, the `SHA256SUMS`
file itself can later be signed with minisign once the public key and
verification path are pinned in the installer.

This is the same layering `rustup` and Debian repository tooling use.

## Development

Lint the script before deploying:

```sh
shellcheck installer/sh.garnet-lang.org/install.sh docs/install.sh
cmp -s installer/sh.garnet-lang.org/install.sh docs/install.sh
```

## Deployment

The script lives in-repo for review + change-tracking. GitHub Pages serves the
copy at `docs/install.sh` from `https://garnet-lang.org/install.sh`; deployment
should preserve:

- `Content-Type: text/plain`
- `Cache-Control: max-age=300, must-revalidate` (5 min — balances
  freshness against cache stampedes)
- HSTS header on the host

Version each deployed copy so a bad script can be rolled back in minutes. A
future `INTEGRITY.txt` can publish the deployed SHA-256 for users who want to
verify the installer before piping to shell.
