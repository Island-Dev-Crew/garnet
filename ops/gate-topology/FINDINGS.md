# Gate Topology Findings Register

This register enumerates findings at exact Git boundaries. IDs were swept
across every advertised fork branch head and `origin/main`; counts or a stale
"next ID" are not allocation authority.

## U-57 — Acceptance is the terminal content operation

- raised-by: OpenAI Codex, GPT-5-based agent, on Hugh’s MacBook Pro
- confirmed-by: Jon Isaac, Island Development Crew merge-authority seat
- head: `7d9e814c37af0ed29210bc0c2d6ac5916c5d3188`
- companion-parent: `162b96adb0a91c5fdc8c189dc2fcdd22ce996cab`
- reproduced-tree: `c0811960edec0b3f90b9a2a633c136dd4e898ec9`
- command: `git merge-tree --write-tree 162b96adb0a91c5fdc8c189dc2fcdd22ce996cab 7d9e814c37af0ed29210bc0c2d6ac5916c5d3188`
- evidence: `ops/gate-topology/evidence/11-third-merge-integration-red.txt`
- status: `fenced`
- disposition: Acceptance is the last content operation on a candidate. A
  later non-record content merge supersedes the prior acceptance with its
  evidence preserved; it never widens the record class or weakens the
  verifier. For this candidate, merge gate-topology first, then run one
  terminal Phase-0-style freeze, native WV-6 acceptance, and pin rebind in the
  same NUC ceremony. The `410ff11` / `fd96e6d9…/1606` acceptance remains as a
  superseded intermediate. Evidence 10 and Evidence 11 remain active exhibits.
  This is U-29's re-acceptance tax paid one final time under the old regime.

## Collision sweep

- swept-at: `2026-08-11T19:01:33Z`
- source: 461 non-main fork branch heads plus `refs/remotes/origin/main`
- refspec: fork branch heads only; zero `refs/pull/*`. The fetch also
  materialized fork `main`; the first grep mistakenly included it, that result
  was discarded, and the authoritative rerun explicitly excluded
  `refs/remotes/fork/main`.
- result: U-54, U-55, and U-56 are assigned. U-57 through U-61 had no
  assignment; the only U-60 references explicitly said it was unassigned.
  U-57 was therefore the next free ID.
