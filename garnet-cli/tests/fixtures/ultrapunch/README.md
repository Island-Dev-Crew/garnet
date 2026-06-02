# Ultrapunch demo (S103) — capability-bounded acceptance of agent-authored code

The #1 capability: a project autonomously **ACCEPTS** agent-authored code only on
**enforced** evidence, and **REFUSES** a silent authority expansion — the negative
proof is the punch.

Scenario: a capability-bounded report tool declaring `@caps(fs)`. A *simulated*
agent proposes changes; `garnet agent-loop` accepts or refuses each on enforced
evidence (diff-caps + the enforced kernel), then seals the accepted one.

- `baseline.garnet` — the tool (declared surface `{fs}`, bounded recursion).
- `accept_proposal.garnet` — a safe refactor (surface unchanged) → **ACCEPTED + sealed**.
- `reject_widen.garnet` — "adds telemetry", widening `{fs}`→`{fs, net}` → **REFUSED at
  diff-caps** (the punch); never runs, never sealed.
- `reject_overdepth.garnet` — passes diff-caps but unwinds the `@max_depth` bound →
  the enforced kernel **TRAPS** it; never sealed (acceptance rests on the run too).

Reproduce:

```
garnet agent-loop --baseline baseline.garnet --proposal accept_proposal.garnet \
  --record-dir <dir> --attest agent=scripted-agent-v1 --attest model=simulated \
  --gate-version dogfood-gate-v1
```

The accept `--record-dir` holds the **4 trust artifacts**: `capability_manifest.json`
(S36), `diff_caps.txt` (S37), `seal.json` (S38), and `transparency_log.jsonl` (S68,
chain-verifiable with `garnet caps-log --verify`), plus an honest `decision.md`.

**Honest scope:** accepted on capability + depth evidence ONLY — `@caps` and
`@max_depth` are enforced. `@bounded`/memory/time/`@mailbox`/OS-sandbox remain
declared-not-enforced; this is **not** a claim of full boundedness or safety. The
agent is simulated/scripted, not a live LLM (S94). The seal is unsigned unless
cosign is present.

**Two-level symmetry:** this INNER loop (accepting/refusing a simulated agent's
code on diff-caps + the enforced kernel) is the SAME discipline as the OUTER loop by
which this project auto-accepts the agent-authored slices that BUILD Garnet (on the
dogfood gate). Garnet dogfoods the exact acceptance it demonstrates.
