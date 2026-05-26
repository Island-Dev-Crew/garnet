# Paper VI Experiment 3 Compiler-as-Agent Harness

This directory is the v0.7 S19 harness for Paper VI Experiment 3:
compiler-as-agent time-to-fix. It ships the reproducible shape of the study,
not the study result. Running the stateless/history-aware comparison and
claiming h3a/h3b/h3c is v0.7.1 work after the provider boundary and review
protocol are approved.

## Current Scope

- `codebase_versions/` contains ten small Garnet snapshots of the same toy
  compiler-helper module evolving over time.
- `run_stateless.sh` and `run_history_aware.sh` define the two execution lanes.
- `aggregate.py` and `analyze.py` consume JSONL run records without inventing
  measurements.

The harness is deliberately opt-in. Set `GARNET_EXP3_EXECUTE=1` before invoking
the run scripts; otherwise they print the commands they would run and exit
successfully with a harness-only note.
