# AGENTS.md — Project Management and Episodic Memory

## Scope

This folder owns handoffs, verification logs, release-stage notes, empirical run records, and project-state snapshots.

## Stable Contracts

- Treat these files primarily as episodic memory: what happened, when, under which assumptions, and with which evidence.
- Do not treat a dated handoff as the only source of current implementation truth if a crate contract or spec supersedes it.
- Promote durable rules into the nearest `AGENTS.md`, README, or language spec.
- Preserve verification evidence: commands run, platform, commit, pass/fail state, and known gaps.

## Update Rules

When a phase completes, record the evidence here, then update the owning semantic/procedural docs if the phase changed stable behavior.

When an agent resumes from a handoff, it should read the relevant handoff plus the closest current `AGENTS.md` before acting.
