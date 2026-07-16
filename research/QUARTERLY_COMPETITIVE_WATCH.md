# Quarterly competitive watch

Schema: `garnet.quarterly_competitive_watch/v1`

Cadence: quarterly

First report: 2026 Q3

First due: 2026-09-30

Report directory: `research/competitive-watch/`

Status: standing slice activated; the first report is planned and has not run.

## Purpose and claim boundary

The watch tests whether Garnet’s integration and timing claims remain plausible.
It does not prove that no competitor exists. **A search miss is not evidence of
absence.** Every report must distinguish observed results, search coverage,
unavailable sources, and inference.

## Required categories

### Agent-native languages

Record languages or language layers that explicitly target agent-authored,
agent-operated, or agent-reviewed software. Search at least project repositories,
official documentation, release notes, and primary maintainer announcements.

### Agent sandbox and runtime systems

Record runtimes, sandboxes, policy engines, and tool-execution systems that bound
agent authority or resource use. Separate process/OS enforcement from reference
semantics and marketing claims.

### Attestation, provenance, and evidence tooling

Record artifact attestation, software provenance, capability-manifest,
transparency, SBOM, and acceptance-gate tools. Distinguish emitted evidence from
verified consumption and failure behavior.

### Agent governance, standards, and regulation

Record primary standards-body, government, and regulator changes affecting
agent identity, authorization, risk bounding, software evidence, or autonomous
operation. Secondary commentary may route discovery but cannot be the only
authority for a claim.

## Source-query requirements

Each category must record:

1. Exact query strings, search date, search engine or API, domain filters, and
   coverage window.
2. Direct primary-source URLs, publisher/maintainer identity, publication or
   release date, access date, and the claim each source supports.
3. False positives, inaccessible sources, language/geography limitations, and
   known blind spots.
4. At least one repository/release-note query and one broader discovery query.
5. A bounded comparison against Garnet’s current evidence, never against a
   planned feature as though it shipped.

## Report contract

A completed report is named `YYYY-QN.md` and contains these exact fields:

- `Status: completed`
- `Coverage window:`
- `Search date:`
- `Source queries:`
- `Primary sources:`
- `Coverage limitations:`
- `A search miss is not evidence of absence.`

It also contains one `##` section for every required category above. A
placeholder, plan, or missed report does not count as completed evidence.

Run:

```sh
python3 -I scripts/garnet_quarterly_competitive_watch_status.py --gate
```

The reporter stays green before a due date with `state=planned`, changes to
`state=active` after a valid report, and fails closed with `state=overdue` when a
required quarter passes without a valid report.
