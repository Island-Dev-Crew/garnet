# Linux .deb and .rpm packaging

Release packages are built by `.github/workflows/linux-packages.yml` on every
`v*` tag. To build them locally from the workspace root:

## Build `.deb`

```sh
cargo install cargo-deb
cargo build --release -p garnet-cli
(cd garnet-cli && cargo deb --no-build)
# Output: target/debian/garnet_<version>-1_<arch>.deb
```

## Build `.rpm`

```sh
cargo install cargo-generate-rpm
cargo build --release -p garnet-cli
cargo generate-rpm -p garnet-cli
# Output: target/generate-rpm/garnet-<version>-1.<arch>.rpm
```

## Install and check (Debian/Ubuntu)

```sh
sudo apt install ./target/debian/garnet_<version>-1_<arch>.deb
garnet --version
man garnet
```

## Install and check (Fedora/RHEL)

```sh
sudo dnf install ./target/generate-rpm/garnet-<version>-1.<arch>.rpm
garnet --version
```

## Systemd service

The packages install `/usr/lib/systemd/system/garnet-actor.service`,
disabled. It runs one Garnet program, `/etc/garnet/entry.garnet`, and its
`ExecStartPre` refuses to start unless that file matches the signed
deterministic manifest beside it (`garnet build --deterministic --sign`
writes `entry.garnet.manifest.json`). The check proves the signature is valid;
it does not pin which key signed it, so keep `/etc/garnet` writable by root
only. The unit's comments give the setup steps. The service runs as a
transient user (`DynamicUser=yes`) and keeps its machine key and run cache in
`/var/lib/garnet`.

Under systemd 252 (Debian bookworm) the unit runs a signed entry program to a
clean exit and refuses to start one edited after signing.

## Package repositories

There is no APT or DNF repository yet; install the release packages directly.
