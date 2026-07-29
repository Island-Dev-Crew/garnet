# Lane 2C Journal

## 2026-07-28 - implementer session

Cold booted from a fresh `origin` clone with `core.autocrlf=false`; verified
main and the stated parent lineage; all five truth-floor gates passed. The NUC
measurement ran on WSL2 ext4 after stopping OneDrive and Ubuntu cron and
verifying no concurrent build, sync, cron, or spawned agent.

The untouched base reproduced quadratic teardown in working, episodic, and
semantic stores across 256, 512, and 1,024 roots. Product commit
`0649d796ac6b78b968d868398b517974838112f3` adds exact incoming managed-ARC
edge accounting and an O(1) isolated-root candidate rejection. The same count
harness now reports linear curves. Focused cycle tests and the full memory
crate passed. The lane is parked for independent review; no verdict,
acceptance, PR merge, tag, or release action was recorded.

## 2026-07-29 - harness-placement amendment

The preceding entry's statement that the lane was parked for review is
superseded. A fresh MSRV gate found the active
`ops/lane2c/probe/Cargo.toml` as an undeclared nineteenth manifest. The
implementer stopped without changing the gate, the root workspace, or any
trust-kernel path. The stop was accepted as correct.

The superseded probe manifest had an empty `[workspace]` stanza. Cargo
therefore built it as a separate nested workspace and wrote its lockfile
outside the repository; root `Cargo.lock` remained at SHA-256
`01b8986b1cee0ef6a53ac439bd018b54fc1dca825a8f845a259ed8001e6715fa`.

The harness now ships as
`garnet-memory-v0.3/examples/lane2c_teardown_probe.rs` at product head
`5cd113617acd35307bb028463833a8da2bbd6ad2`. The active ops manifest,
transient lockfile, and probe source were removed. Proposed U-46 is recorded
in `ops/lane2c/DOCTRINE.md`: build manifests never live under `ops/**`;
harnesses live inside the crate under test; ops holds outputs, records,
verifiers, and replay scripts.

All prior profile outputs were replaced. The shipped example produced fresh
base and product curves for three cases at 256, 512, and 1,024 roots during
the quiet window `2026-07-29T08:34:09.8071444Z` through
`2026-07-29T08:35:39.5943570Z`; the ignored stress set passed 4/4 in that same
window. OneDrive, its sync helper, and Ubuntu cron were restored afterward.
Independent review remains pending; no verdict, acceptance, merge, tag, or
release action was recorded.
