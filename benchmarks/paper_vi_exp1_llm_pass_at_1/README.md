# Paper VI Experiment 1: LLM pass@1 Harness

Status: **harness wired, provider-backed measurement pending**.

This directory contains the S94 source harness for Paper VI Experiment 1. The
experiment asks whether first-attempt LLM code generation does better on Garnet
than on comparable Rust tasks. The full registered study still requires a
larger corpus, model/provider credentials, and reviewed execution.

What S94 does ship:

- a seed-only task manifest that mirrors the registered task shape;
- a provider flag (`none`, `fixture`, `openai`, `anthropic`, `gemini`, `ollama`);
- a provider-free run that records `pending-infra` rows without calling a model;
- a deterministic fixture run that proves JSONL writing and pass/fail
  aggregation without a network call;
- aggregate and analysis scripts that keep measured rows separate from pending
  rows.

What S94 does **not** claim:

- no provider-backed pass@1 result;
- no full 500-task benchmark corpus;
- no statistical significance result;
- no fine-tuned model result.

Run the provider-free proof:

```powershell
python benchmarks/paper_vi_exp1_llm_pass_at_1/run.py --provider none --output target/paper_vi_exp1/none
python benchmarks/paper_vi_exp1_llm_pass_at_1/aggregate.py target/paper_vi_exp1/none/results.jsonl --output target/paper_vi_exp1/none/aggregate.json
python benchmarks/paper_vi_exp1_llm_pass_at_1/analyze.py target/paper_vi_exp1/none/aggregate.json --output target/paper_vi_exp1/none/analysis.md
```

Run the deterministic fixture proof:

```powershell
python benchmarks/paper_vi_exp1_llm_pass_at_1/run.py --provider fixture --output target/paper_vi_exp1/fixture
python benchmarks/paper_vi_exp1_llm_pass_at_1/aggregate.py target/paper_vi_exp1/fixture/results.jsonl --output target/paper_vi_exp1/fixture/aggregate.json
python benchmarks/paper_vi_exp1_llm_pass_at_1/analyze.py target/paper_vi_exp1/fixture/aggregate.json --output target/paper_vi_exp1/fixture/analysis.md
```

Run the S94 repo gate:

```powershell
python scripts/garnet_paper_vi_exp1_status.py --gate --format json
```
