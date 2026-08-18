# WV-6 terminal freeze ceremony record

## Seat and transport

- Seat: OpenAI Codex on `NUCBOX_M2PRO_S`.
- Platform: native Microsoft Windows NT `10.0.26200.0`, AMD64, NTFS.
- Checkout: `C:\garnet-terminal-freeze-20260811-019ff342`.
- `core.autocrlf=false`.
- OneDrive paused for the ceremony.
- Stored push identity: `IslandDevCrew`.
- Commit identity: `OpenAI Codex <codex@openai.com>`.
- Fork read and push dry-run completed before ceremony work.

## Third merge

- Merge: `8e88b12eb16c057adac99551c5319289920dc9d3`.
- Tree: `b961e73436bec5c4753bda6a43cd93f20a773b60`.
- First parent: `162b96adb0a91c5fdc8c189dc2fcdd22ce996cab`.
- Second parent: `cedda39cc851383f983711761a9363c8cfa40f83`.
- Merge mode: real `--no-ff` merge; conflict-free.

## Diagnostic replacement gate

Step 2R-A produced zero graph findings at `162b96a`; its only problem was the
required authenticated transport. Step 2R-B's producer census emitted five
trust paths, all inside the gate-topology delta. The exact record is
`01-step-2r-diagnostic.md`.

## F5 fail-closed exhibit

Both reporters exited `1` before rebind and `0` after rebind. The verbatim
stdout captures are `02-f5-red-before.md` and `03-f5-green-after.md`.

## Terminal freeze and native acceptance

The content pair was computed directly from the immutable merge tree through
`garnet_content_provenance.tracked_content_digest`:

- Content SHA-256: `1b45387599223adbe8766ae2e04ddc70e4ecc359712a4867ef1973dac9bd42e4`.
- Path count: `1629`.

It was not compared with a supplied pair. The native acceptance predicates
then passed:

- Core Ring Tier 1: `3/3`.
- MCP raw-byte stdio: `2/2`.
- Sealed baseline: `1/1`.
- Reject-without-seal traps: `6/6`.

The producer-censused four candidate constants and four `PROOF.json` mirrors
were rebound to the merge head/tree and computed pair. The sanctioned WV-6
producer generated the live six-file bundle. Every staged file was SHA-256
hashed before commit. The custody window closed at:

- Acceptance commit: `b87ac5c39bf6d6962ae9b3f715a63af05869067c`.
- Acceptance tree: `99387706a1aad6660bde15aa3f9a8506e1bff698`.

## Supersession with preservation

State: `SUPERSEDED-WITH-PRESERVATION`.

The prior accepted boundary remains:

- Head: `410ff1182cdcefcec9fe046d1346205d8522ec9d`.
- Tree: `57ce26ae1ab8d24609180486bc5fce6179f37957`.
- Pair: `fd96e6d910180f5e33999fbd693ea211e336389a13535930d89b2a870ff54727 / 1606`.
- Prior `WV_ACCEPTANCE.json` SHA-256:
  `56fea133501eab5692a484900d3a15c272c406222b8b6877d0b92b1633fd7fe3`.
- Full preserved bundle:
  `proofs/windows/launch-verification/wv6-terminal-freeze-20260812/superseded-410ff11/`.

All six prior bundle files were copied and hash-compared before the live
producer destination was replaced. The original acceptance commit and its
complete tree remain in append-only Git history.

## Expected pre-approval state

The rolling trust gate is expected to report `REVIEW REQUIRED` at this close-
out because no candidate-binding canonical review record exists and
authenticated transport is required. This is the expected pre-approval state.
It is cured by the carrier's authenticated approval at `#521` and by no action
performed in this ceremony.

No candidate-binding canonical `*.review.json` record was authored. No PR,
GitHub approval, merge to main, tag, token mint, or Jon-only action occurred.
