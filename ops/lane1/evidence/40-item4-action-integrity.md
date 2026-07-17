# Lane 1 Item 4 — external action integrity and U-16

Date: 2026-07-17
Authority: public upstream Git refs plus local fail-closed policy gates
State: locally implemented and independently reviewed; cross-OS CI pending.

## Preserved RED

Before the rewrite, the reporter counted 86 external `uses:` occurrences and
all 86 were mutable tag references. The new test module was initially absent,
then the repository reporter returned RED with `mutable_count: 86`.

## Authoritative ref resolution

The upstream action refs were resolved with `git ls-remote` on 2026-07-17.
Annotated tags were peeled and the commit object, not the tag object, is pinned.

| Action / reviewed ref | Exact commit |
|---|---|
| `actions/checkout@v6` | `df4cb1c069e1874edd31b4311f1884172cec0e10` |
| `dtolnay/rust-toolchain@master` | `2c7215f132e9ebf062739d9130488b56d53c060c` |
| `actions/cache@v5` | `caa296126883cff596d87d8935842f9db880ef25` |
| `actions/upload-artifact@v6` | `b7c566a772e6b6bfb58ed0dc250532a479d7789f` |
| `actions/download-artifact@v8` | `3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c` |
| `actions/setup-python@v6` | `ece7cb06caefa5fff74198d8649806c4678c61a1` |
| `softprops/action-gh-release@v2` | `3bb12739c298aeb8a4eeaf626c5b8d85266b0e65` |
| `EmbarkStudios/cargo-deny-action@v2` | `3c6349835b2b7b196a839186cb8b78e02f7b5f25` |
| `actions/setup-node@v4` | `49933ea5288caeca8642d1e84afbd3f7d6820020` |
| `actions/setup-node@v6` | `249970729cb0ef3589644e2896645e5dc5ba9c38` |
| `github/codeql-action/*@v4` | `7188fc363630916deb702c7fdcf4e481b751f97a` |

The peeled tag objects were independently rechecked: cargo-deny tag object
`b66acf...` resolves to the commit above, and CodeQL tag object `eec0bff...`
resolves to the commit above.

`dtolnay/rust-toolchain` documents that its shorthand channel normally comes
from the action `@rev`, and that full-SHA pins must use a commit reachable from
`master`. The workflows therefore use the exact `master` commit above and bind
`with.toolchain: stable` or `with.toolchain: nightly` explicitly. The reporter
rejects a SHA-pinned occurrence that omits that input.

## Fresh local GREEN

```text
test_garnet_workflow_action_integrity_status.py: 13/13
garnet_workflow_action_integrity_status.py --gate:
  occurrence_count: 89
  credited_occurrences: 89
  mutable_count: 0
  manifest_entry_count: 13
  findings: []
test_github_actions_node24_readiness.py: 3/3
test_garnet_msrv_status.py: 25/25
garnet_msrv_status.py --gate: ok true, MSRV 1.95
test_garnet_vscode_release_assets.py: 3/3
```

The count increased from the 86-path RED baseline because Lane 1 added the
cross-OS Python setup and the two pinned actions in the base-controlled
workflow. Every final occurrence is included in the same gate run.

U-16 is implemented in the checked-in ownership/procedural policy: every
`scripts/garnet_github_*` change is Jon-only and requires the same W_TRUST and
cross-OS evidence path as other governance controls.

An independent read-only reviewer approved the final Item 4 bytes, repeated
the upstream ref resolution without authentication, and reproduced the
89-of-89 credited, zero-mutable reporter result. The approval is scoped local
review; Linux and Windows runtime evidence remains pending CI.
