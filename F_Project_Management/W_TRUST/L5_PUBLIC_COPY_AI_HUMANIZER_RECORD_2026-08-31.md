# L5 public-copy AI-humanizer record — 2026-08-31

## Scope

This is record-class evidence for the L5 public-copy pass. It is not a trust-kernel review record and carries no acceptance authority.

Both corpora were produced by the same deterministic projection: take the nonblank added lines from a zero-context Git diff of `README.md` and `docs/` against base `f6d3285aa54a4961e38d82d22cfe98ab4c631b22`, and insert one `### <path>` marker before each changed file's additions. The BEFORE projection ends at item-6 commit `b492e92`; the AFTER projection includes the item-7 working-tree rewrite. `F_Project_Management/W_TRUST/` is outside the projection.

The projection intentionally retains HTML and Markdown syntax. The score is therefore an advisory comparison of the same public-copy projection, not a general measure of writing quality.

## Scorer

- Script: `/Users/IDC2.5/.agents/skills/ai-humanizer/scripts/score.js`
- Script SHA-256: `9ec2ed209ded0b9c937b41812d62e3ef3fd4cb2bce4506bc3e2e280661bcc5f9`
- Runtime: Node.js `v22.22.3`
- Host: `Darwin 25.6.0 arm64`

## Evidence files

- `L5_PUBLIC_COPY_AI_HUMANIZER_BEFORE_2026-08-31.txt`
  - SHA-256: `f6d68fe9a2ab003eaf737d1a4359370e59f89832f722d55979577a309cb8e199`
  - Score: `39`
- `L5_PUBLIC_COPY_AI_HUMANIZER_AFTER_2026-08-31.txt`
  - SHA-256: `82702c3caaef2d9c3973a5d11c64037a7d387ad14cf8d5416b130094fafbaf1f`
  - Score: `26`
- The two `*_SCORE_*.json` files preserve the scorer's complete JSON output.
- `L5_PUBLIC_COPY_AI_HUMANIZER_DELTA_2026-08-31.txt` preserves the red-capable delta check output.

## Result

`delta.sh` exited `0`: BEFORE `39`, AFTER `26`, delta `-13`. This result is advisory. No hook or CI policy enforces the score.

The rewrite shortened dash-heavy sentences, removed ornamental quotation, made the origin account more direct, and replaced generic CRA framing with exact reporting-clock language. It preserved the pinned source titles and dates, the W-SHIP construction label, and the bounded enforced-set facts.
