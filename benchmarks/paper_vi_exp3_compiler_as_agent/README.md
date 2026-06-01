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
- S95 adds a 5K-LOC rerun harness:
  - `generate_5k_corpus.py` deterministically generates ten snapshots of at
    least 5,000 LOC each under the requested output directory.
  - `run_5k.py` writes provider-free stateless/history-aware rows for that
    corpus with `pending-provider-rerun`.
  - `aggregate_5k.py` and `analyze_5k.py` preserve the boundary that no new h3a
    measurement is claimed without provider-backed runtime rows.
  - `scripts/garnet_paper_vi_exp3_5k_status.py --gate` is the committed S95
    status proof.

The harness is deliberately opt-in. Set `GARNET_EXP3_EXECUTE=1` before invoking
the run scripts; otherwise they print the commands they would run and exit
successfully with a harness-only note.

The S95 5K lane is also deliberately provider-gated. Provider-free mode proves
corpus scale and rerun plumbing only; the recorded v4.0 6.5% h3a partial stands
until a reviewed provider-backed 5K run exists.
