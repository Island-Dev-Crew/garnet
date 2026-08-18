# WV-6 terminal freeze 2 ceremony record

## Seat and candidate

- Seat: OpenAI Codex on the native-Windows NUC.
- Checkout: `C:\gtf2-01a08f2c`.
- `core.autocrlf=false`; OneDrive remained stopped.
- Commit identity: `OpenAI Codex <codex@openai.com>`.
- Fourth merge: `4a6d1aed9c81a624efa2335b28de12b4bdb82c8f`.
- Fourth-merge tree: `a4829ce899c7525260c222ed16c14137b228c647`.
- First parent: `efac4cb17b48b830c5e30e5ab08ad4d55111d2d0`.
- Second parent: `dda543ee25f68eb798d2c5e02980b3f0023c6a1c`.

## F5 red-before and green-after

The Shelf and WV-6 reporters both exited `1` at the pristine fourth merge and
both exited `0` at the acceptance commit. The verbatim outputs, external raw
capture paths, and four hashes for each half are recorded in
`01-f5-red-before.md` and `02-f5-green-after.md`.

## Terminal freeze 2 and determinism

The content pair was computed at the immutable fourth-merge tree through the
repository provenance producer:

- Content SHA-256: `8ea996129074e2e70c5ad2f9abd6082c85f7021e943dc017d3a9ba2859ed3ac7`.
- Path count: `1634`.

The committed producer reproduced that same pair, reviewed head
`4a6d1aed9c81a624efa2335b28de12b4bdb82c8f`, and reviewed tree
`a4829ce899c7525260c222ed16c14137b228c647` before the acceptance commit.
The determinism check passed.

The producer-owned four reporter pins and four `PROOF.json` mirrors were
rebound. No producer logic or other trust-adjacent code was changed. The
producer emitted a fresh six-file live bundle into the vacant destination.
Every staged file was SHA-256 hashed before the single acceptance commit:

- Acceptance commit: `6e94374556d4d94148c27d2d2edaa3aa839cab6a`.
- Acceptance tree: `01ff8096fc09fbd2c226a89a83aa5a030115903d`.
- Acceptance parent: `4a6d1aed9c81a624efa2335b28de12b4bdb82c8f`.

## Supersession with preservation

State: `SUPERSEDED-WITH-PRESERVATION`.

The superseded accepted boundary is preserved as follows:

- Reviewed head: `8e88b12eb16c057adac99551c5319289920dc9d3`.
- Reviewed tree: `b961e73436bec5c4753bda6a43cd93f20a773b60`.
- Pair: `1b45387599223adbe8766ae2e04ddc70e4ecc359712a4867ef1973dac9bd42e4 / 1629`.
- Preserved `WV_ACCEPTANCE.json` SHA-256:
  `3007885df2315a785b0a4345ce94a4a890f039440ecffd97da4d91794a307f58`.
- Complete preserved bundle:
  `proofs/windows/launch-verification/wv6-terminal-freeze-20260814/superseded-8e88b12eb16c057adac99551c5319289920dc9d3/`.

The full pair chain is:

1. `2cb25d0b47f55c9dd987bf69fc8a97dade5a4767ef0beda0abc8616808dddc0c / 1605`.
2. `fd96e6d910180f5e33999fbd693ea211e336389a13535930d89b2a870ff54727 / 1606`.
3. `1b45387599223adbe8766ae2e04ddc70e4ecc359712a4867ef1973dac9bd42e4 / 1629`.
4. `8ea996129074e2e70c5ad2f9abd6082c85f7021e943dc017d3a9ba2859ed3ac7 / 1634`.

All predecessor records and their Git history remain intact.

## Expected pre-approval state

No candidate-binding canonical `*.review.json` record was authored. The
rolling trust gate is expected to remain `REVIEW REQUIRED` for the absent
candidate-binding record and authenticated transport. That state is cured at
`#521`, not by this ceremony.

No PR, GitHub approval, merge to main, tag, token mint, or Jon-only action was
performed.
