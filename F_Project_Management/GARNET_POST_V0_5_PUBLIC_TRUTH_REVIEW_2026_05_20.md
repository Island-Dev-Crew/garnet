# Garnet Post-v0.5 Public Truth Review

Date: 2026-05-20

Purpose: review the launch/blog/community work after v0.5.0 publication and
record the remaining achievable gaps without softening Garnet's calibrated
honesty.

## What Was Already Solid

- `v0.5.0` is published with Linux packages, macOS CLI tarballs, VSIX release
  assets, and `SHA256SUMS`.
- The blog now has a public index, a founder post, a v0.5 release post, and a
  monthly cadence for the next three technical posts.
- GitHub Discussions are live with seed threads for welcome, show-and-tell, and
  package-registry shape.
- The public site links Discussions, the getting-started walkthrough, the
  planned playground stub, promo-video assets, and current status evidence.
- The MIT/productization reporter now reports 69.7% across 18 lanes on the
  current Mac evidence set after S4, while keeping the tracked-slice ledger
  separate.

## Fixes Applied In This Pass

- Removed stale "v0.5.0 release-candidate" and "v0.4.2 latest release" language
  from the README and public install surfaces.
- Added a current `v0.5.0` release-feed entry and a dedicated blog Atom feed.
- Added `docs/stdlib.html` so the site has a first reference page for the
  capability-tagged primitive registry.
- Updated the front-door metric from 67.9% to 69.7% and added the S4 formatter
  baseline to status copy.
- Added `stdlib.html` and `playground.html` to the sitemap.
- Corrected the S4 dogfood contract to use `garnet fmt --stdout`.

## Remaining Achievable Gaps

- Turn `docs/stdlib.html` from a primitive table into fuller examples once
  JSON/HTTP/regex/datetime packages exist.
- Expand `docs/playground.html` from a planned stub into a real static
  playground or WASM-backed demo. Do not claim interactive execution until it
  exists.
- Add a deeper tutorial page after the getting-started happy path: capabilities,
  deterministic build, and signed hot-reload in one small project.
- Keep S3 (`garnet add`) and S6/S7 in v0.5.1 scope unless a separate PR proves
  their dogfood blocks.
- Publish Marketplace/OpenVSX only after account/authorship decisions are
  complete and the release-backed VSIX proof is repeated from the public asset.
- Do not create Discord/social accounts or post externally without maintainer
  login, verification, and final copy approval.

## Codebase Review Lead

The repo is improving visibly, but it is not time for a broad aesthetic
refactor in this public-truth PR. The next code review should be a separate PR
or issue series with findings ordered by user-facing risk:

1. Current-state drift in public docs and FAQ.
2. CLI examples and docs that still reference older v4.2 wording.
3. Test totals and primitive counts that changed since historical handoffs.
4. Formatter boundaries that require a trivia-preserving CST before deeper
   pretty-printing.
5. Large archived documents that are useful for audit but should not be treated
   as current source truth.
