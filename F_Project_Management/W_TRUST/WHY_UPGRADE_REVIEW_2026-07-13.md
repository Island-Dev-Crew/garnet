# /why upgrade review — 2026-07-13

**Trust-kernel review companion** for the `docs/why.html` upgrade (the page is
in the rolling-review trigger set). Reviewed by: Claude Code (Fable 5) with a
three-agent primary-source verification pass (every published fact fetched at
source on 2026-07-13), executed on Jon's directive to build the upgrade from
the rules in
`F_Project_Management/RESEARCH/WHY_PAGE_RECONCILIATION_2026-07-13.md`.

## What changed on the page

Completes the two placeholder cards the page shipped with (`.cv ins` src said
"verify carrier references before publishing"; `.cv build` said "attribute +
link exact quotes… before naming anyone publicly") and upgrades the threat
card whose src carried the "link primary sources" TODO:

1. **Threat card** — now leads with Shai-Hulud/Megalodon; every figure from a
   fetched source; ten primary-source links in the src line.
2. **Insurance card** — named carriers (Klaimee, Testudo, Armilla, Munich
   Re/HSB), the ISO January-2026 exclusion endorsements, state approvals; six
   source links.
3. **Language-author card** — the two-post Ronacher card, quotes verbatim
   (bracketed typo cleanup), both posts linked.
4. **Vocabulary** — "honest account" → "calibrated account" in the meta
   description and byline.
5. **Untouched:** the two bounded `enforced:` claims (count stays exactly 2 —
   claim fixture green), the fences paragraph, the regulation card (its FDA
   mention was already indirect speech per Flag-2 rules), and everything else.

## Corrections applied vs. the cowork-stream copy (why verification mattered)

The audited draft's own figures failed primary-source verification and were
corrected before publication:

| Cowork/audit copy | Published (sourced) |
|---|---|
| "~172 packages across 403 malicious versions within 48 hours" | **373 malicious package-versions across 169 npm + 2 PyPI packages** (CSA research note; Snyk concurs); first wave **84 versions in ~6 minutes** (StepSecurity). The 48-hour framing is unsourced — dropped. |
| "valid SLSA Build Level 3 attestations" stated flat | Kept the first-documented-worm claim (StepSecurity/Snyk verbatim) **with slsa.dev's own postmortem linked**: attestations were cryptographically valid but the hijacked pipeline no longer met Build L3 isolation — "a signed artifact is not necessarily a trustworthy one." |
| "Its successor, Megalodon" | **"Days later, the Megalodon campaign pushed…"** — OX Security assesses a likely copycat, no shared code/IOCs (The Register quote + OX writeup). Dates verified: Shai-Hulud May 11 → Megalodon May 18, 2026. |
| CISA alert as Megalodon's | Described as **a joint CISA alert** (2026-05-28, also covers the Nx Console extension); alert page fetched, date confirmed. |
| "underwrites point-in-time red-team snapshots" | Only Armilla red-teams. **Published:** underwriting rests on questionnaires, public-data scans, one-time assessments, and external litigation data (Testudo: "no integration with your AI systems") — the no-attestable-instrument thesis stated accurately. |
| Testudo "$10M per insured" flat | "limits **marketed** up to $10M" (its own brokers page: $10M max aggregate, $1M–$10M+ range; its Feb-2026 capacity note said $9.25M). |
| Berkshire/Chubb/Travelers approvals stated flat | Attributed: "per a **Wolfe Research analysis**" (specialty-newsletter sourcing; no regulator primary found). |
| eBuilder cited without link | Linked to the verified **.se** article URL (the .com mirror 404'd to fetchers). |
| "remote command execution" (CVE-2025-49596) | **"remote code execution"** per NVD's wording; CVSS 9.4 is the GHSA CVSS-4.0 assessment. |
| Ronacher card fused two posts | **Two posts, verbatim, both linked** — permission-checks quote (with `a[l]most` bracket) from *Agentic Coding Recommendations* (2025-06-12); `needs { time, rng }` sketch from *A Language For Agents* (2026-02-09). Re-verified directly against lucumr.pocoo.org twice (reconciliation pass + this build). |
| Veracode swap for eBuilder (retracted Report 01) | **Additive**, per 01R: both cited; the two programs (100+ and 150+ models) converge on the same ~45% flaw / >95% syntax flat line. |

## Source ledger (every link on the page was fetched 2026-07-13)

StepSecurity (Mini Shai-Hulud writeup) · CSA research note (373/169+2) ·
slsa.dev postmortem (2026-05-15) · Tenable FAQ (TeamPCP attribution, npm+PyPI)
· SafeDep (Megalodon 5,718/5,561, May 18, six-hour window) · OX Security
(copycat assessment) · CISA alert (2026-05-28) · eBuilder Security 2026 (.se)
· Veracode Spring 2026 GenAI Code Security · Wiz academy (MCP 80% / 38% of
500+) · NVD CVE-2025-49596 · YC/Klaimee · Testudo brokers page · Armilla ·
Munich Re/HSB press release (canonical URL; body 403s to bots — date
corroborated by Reinsurance News, 2026-03-19) · Big "I"/Verisk ISO forms
(CG 40 47 / 40 48 / 35 08, Jan 2026 edition) · insuranceintel (Wolfe Research
analysis) · lucumr.pocoo.org (both Ronacher posts).

## Boundary check

- `enforced:` claim count on the page: **2** (unchanged; claim fixture green).
- No forbidden universal-enforcement phrasing introduced.
- No S114/independence language touched; the fences paragraph is byte-identical.
- The page remains outside the service-worker precache — the correction ships
  immediately to all visitors.
