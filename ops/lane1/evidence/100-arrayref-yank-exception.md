# Evidence 100 — arrayref 0.3.9 yanked exception

- Observation window: `2026-08-20T09:04:29.616Z` through
  `2026-08-20T09:04:29.998Z` on the native Windows NUC seat.
- Boundary before this record: ruleset-pin refresh
  `7b01726fd1ab9ca9194c9203b9d6d418d9d922b1`.
- Registry source: `https://crates.io/api/v1/crates/arrayref` returned HTTP
  200. Versions `0.3.5`, `0.3.6`, `0.3.7`, `0.3.8`, and `0.3.9` each report
  `yanked: true` and `yank_message: null`; `0.3.4` reports `yanked: false`
  and is the registry's newest non-yanked version.
- Dependency source:
  `https://crates.io/api/v1/crates/blake3/1.8.4/dependencies` returned HTTP
  200 and reports a non-optional normal dependency on `arrayref ^0.3.5`.
- Locked bytes: `Cargo.lock` binds `arrayref 0.3.9` with checksum
  `76a2e8124351fda1ef8aaaa3bbd7ebbcb486bbcd4225aca0aa0d84bb2db8fecb`
  through `blake3 1.8.4`. The sole non-yanked `0.3.4` cannot satisfy
  BLAKE3's requirement.
- Advisory census: RustSec advisory-db
  `2f08fbb85332687b721f2f22706d07448369451b` has no `crates/arrayref`
  directory and a case-insensitive word census over `crates/` returned zero
  matches. That database snapshot predates the registry yanks; it establishes
  only that no captured RustSec advisory names `arrayref`, not why the versions
  were yanked.

## Tool and schema custody

- The NUC had no pre-existing `cargo deny` subcommand. Garnet CI pins
  `EmbarkStudios/cargo-deny-action@3c6349835b2b7b196a839186cb8b78e02f7b5f25`,
  whose Dockerfile selects cargo-deny `0.20.2`.
- The official Windows `0.20.2` release archive reproduced its published
  SHA-256:
  `975a22143262fd27476d19ee00c7af67978426e40e1dee94eed6bbade1cf87dc`.
  The extracted `cargo-deny.exe` reports `cargo-deny 0.20.2`.
- Cargo-deny `0.20.2` documents exact yanked-package exceptions as a
  `PackageSpec` with optional reason in `[advisories].ignore`. Global
  `yanked = "deny"` remains unchanged.

## Red reproduction

With a newly created task-specific `CARGO_HOME`, the unmodified policy ran:

```text
error[yanked]: detected yanked crate (try `cargo update -p arrayref`)
arrayref 0.3.9 registry+https://github.com/rust-lang/crates.io-index
advisories FAILED, bans ok, licenses ok, sources ok
```

The command exited 1. An earlier replay against the seat's existing registry
cache exited 0; that cache-specific result was not used as current-registry
evidence.

After applying the exact-package exception, the same current registry cache
produced:

```text
advisories ok
advisories ok, bans ok, licenses ok, sources ok
```

Both commands exited 0. The advisory-only run's stderr was empty; the full
run retained only the policy's pre-existing warning-class license and
duplicate-version diagnostics. Green capture SHA-256 values:

- Advisory-only stdout:
  `799fc05e6513bf365cc7565ea51cc5cb9ca6690b1cd1fe7777339be83437131b`
- Advisory-only stderr:
  `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- Full-check stdout:
  `f1a0fca39d4280363937aabd77783990ea6480bd9ca257816de3b68fc8efa845`
- Full-check stderr:
  `cc5c3f753395c77346f344e16befc66c5be9138768df2192f0dcbe01b5456162`

## Narrow exception and removal lane

The policy continues to deny every other yanked package and ignores exactly
`arrayref@0.3.9`. The reason carried in `deny.toml` records the undocumented
mass-yank of `0.3.5` through `0.3.9`, the captured absence of an arrayref
RustSec advisory, the incompatible sole non-yanked `0.3.4`, and the reviewed
lockfile checksum.

Follow-up lane: remove this exact exception when BLAKE3's dependency moves to
a satisfiable non-yanked arrayref release or when the upstream registry reverses
the yank. The reviewing seat observed that reversal upstream on `2026-08-20`,
so the exception is presently inert and is retained only as a timestamped,
narrow, non-widening allowance. Do not broaden the exception and do not rewrite
`Cargo.lock` merely to suppress registry-state evidence.

## Native verification

- `cargo build --workspace --all-targets --all-features`: exit 0.
- Minimum-shelf native acceptance commands: 3/3, 2/2, 1/1, and 6/6.
- Full Python discovery at the pre-freeze A2 worktree: 1,160 tests; the ten
  established native-Windows failure/error names were reproduced, with one
  additional `WV-6` status failure because this digest-domain change had not
  yet received its terminal rebind. That additional name is the Part B
  red-before transition, not a product-test regression; the accepted tip must
  clear it and return to the ten-name platform set.
- `Cargo.lock` remained byte-identical at SHA-256
  `01b8986b1cee0ef6a53ac439bd018b54fc1dca825a8f845a259ed8001e6715fa`.

## Recomputable capture hashes

All raw captures are held outside the repository under the ceremony capture
root. SHA-256 values:

- `crates-io-arrayref.json`:
  `2f879f3ffd2c6967624e2e0f35df42bf5356a353b1a7e6cebdb1898eb6f557f2`
- `crates-io-arrayref.headers.txt`:
  `9badf8a7226b2f4511e8f8445ec4d9b1d9e38f2b63a019e5a6dbe76cc08ad335`
- `crates-io-blake3-1.8.4-dependencies.json`:
  `ad9defc0a6b938a32b2e42013c746ab846d666eddd07d814ee271bb4ae6c1d3a`
- `crates-io-blake3-1.8.4-dependencies.headers.txt`:
  `7150f8a5c08887ad1aa1526b970832cce20b088069c57bab1419eebaebbd7149`
- Fresh-home red stdout:
  `da37e522d5b3103a1c4aba99887f6c3cc89aee5534b41819544124a22f55dfd3`
- Fresh-home red stderr:
  `178c8434701b2910bcb688b008165789031f0aebf4161297aaa0cf23a74008e4`
- Fresh-home execution metadata:
  `eea9f15dfb78d0eb5c757e8154c59a004da1cefd8b47f30d20f2efd3a5b096df`
