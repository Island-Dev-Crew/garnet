# Linux .deb + .rpm packaging (v4.2 Phase 6A)

## Build `.deb`

```sh
cargo install cargo-deb
cd garnet-cli
cargo build --release
cargo deb
# Output: target/debian/garnet_0.4.2-1_amd64.deb
```

## Build `.rpm`

```sh
cargo install cargo-generate-rpm
cd garnet-cli
cargo build --release
cargo generate-rpm
# Output: target/generate-rpm/garnet-0.4.2-1.x86_64.rpm
```

## Install + verify (Debian/Ubuntu)

```sh
sudo apt install ./target/debian/garnet_0.4.2-1_amd64.deb
garnet --version
# Expected: wordmark + "Rust Rigor. Ruby Velocity. One Coherent Language."
```

## Install + verify (Fedora/RHEL)

```sh
sudo dnf install ./target/generate-rpm/garnet-0.4.2-1.x86_64.rpm
garnet --version
```

## Systemd service

The package installs `/usr/lib/systemd/system/garnet-actor.service`
**disabled by default**. Operators who want the actor-runtime to start
at boot run:

```sh
sudo systemctl enable --now garnet-actor
```

The service unit runs an `ExecStartPre=garnet verify ... --signature`
check before starting, so a tampered binary or missing signature
prevents the runtime from coming up. See the service file for the
hardening defaults (NoNewPrivileges, ProtectSystem=strict, restrictive
SystemCallFilter, etc.).

## Repository signing (future)

Post-MIT: package a `.list` file for APT and a `.repo` file for DNF
pointing at `pkg.garnet-lang.org` so consumers can `apt install garnet`
/ `dnf install garnet` without hand-fetching the bundle.
