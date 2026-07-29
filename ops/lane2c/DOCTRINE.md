# Lane 2C Doctrine

## U-46 - Measurement and replay scaffolding had no declared home

Status: proposed register entry; lane-local placement rule effective now.
This file does not modify the frozen backlog and does not record acceptance.

The Lane 2C fence limited implementation to memory-crate paths and described
`ops/lane2c/` as the evidence namespace. Placing the first probe there was
therefore within a reasonable reading of the fence, but an active
`ops/lane2c/probe/Cargo.toml` was visible to repository-wide manifest gates.
The fence did not state that consequence.

### Rule

- Build manifests never live under `ops/**`.
- Measurement and replay harnesses live inside the crate under test as a bench
  or example.
- `ops/**` may hold outputs, records, deterministic verifiers, and replay
  scripts. It may not hold an active build manifest.
- If a future probe cannot be expressed inside the crate, an inert
  `Cargo.toml.txt` plus a replay script that materializes a temporary workspace
  is the only lane-local fallback. The reason for the fallback must be
  recorded.
- Neither the MSRV gate nor the root workspace membership/exclusion list is a
  cure for harness placement.

Lane 2C applies the preferred path:
`garnet-memory-v0.3/examples/lane2c_teardown_probe.rs`. There are zero new
manifests and zero new dependencies.

## Account of the superseded probe

The transient probe manifest did contain its own empty `[workspace]` stanza.
That stanza made the probe the root of a separate nested workspace, which is
why Cargo built it instead of rejecting it as an unlisted package inside the
repository workspace. Cargo wrote the transient lockfile under the external
probe workspace; the repository root lockfile remained unchanged.

Root `Cargo.lock` SHA-256 before and after:
`01b8986b1cee0ef6a53ac439bd018b54fc1dca825a8f845a259ed8001e6715fa`.

The active `ops/**` manifest and its transient lockfile/source were removed.
All recorded operation counts were then regenerated from the shipped crate
example; none of the superseded probe's counts are carried into the record.
