# AGENTS.md — Project Template Contract

## Scope

Owns bundled project templates for `garnet new`.

## Stable Contracts

- Template content must remain valid UTF-8 and free of stray `{{name}}` placeholders after rendering.
- Starter projects should be beginner-readable and runnable without network access.
- A starter test is its own program entry: `garnet test` installs the test's
  `@caps(...)` as the entry budget, and the `time` and `mem` classes trap at
  run time (D-02, D-04b). A starter test that reaches host authority — even
  through a helper such as `timestamp()` — declares it (`web-api` declares
  `@caps(time)`, `agent-orchestrator` declares `@caps(mem)`); an undeclared
  starter test fails on the freshly generated project.
- `agent-orchestrator` should demonstrate working, episodic, semantic, and procedural memory concepts where possible.
- Future `--agent-docs` scaffolding should add project-local contracts without making simple CLI/web templates feel heavy by default.

## Required Checks

```sh
cargo test -p garnet-cli new_cmd
cargo test -p garnet-cli --test cli_smoke template
```

`cli_smoke` scaffolds the `web-api` and `agent-orchestrator` templates and runs
`garnet run` and `garnet test` on the generated projects; the agentic dogfood
matrix does the same for all three templates in CI.

Recommended manual smoke:

```sh
garnet new --template cli /tmp/garnet-cli-smoke
garnet new --template web-api /tmp/garnet-web-smoke
garnet new --template agent-orchestrator /tmp/garnet-agent-smoke
```
