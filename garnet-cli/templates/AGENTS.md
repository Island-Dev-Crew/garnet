# AGENTS.md — Project Template Contract

## Scope

Owns bundled project templates for `garnet new`.

## Stable Contracts

- Template content must remain valid UTF-8 and free of stray `{{name}}` placeholders after rendering.
- Starter projects should be beginner-readable and runnable without network access.
- `agent-orchestrator` should demonstrate working, episodic, semantic, and procedural memory concepts where possible.
- Future `--agent-docs` scaffolding should add project-local contracts without making simple CLI/web templates feel heavy by default.

## Required Checks

```sh
cargo test -p garnet-cli new_cmd
```

Recommended manual smoke:

```sh
garnet new --template cli /tmp/garnet-cli-smoke
garnet new --template web-api /tmp/garnet-web-smoke
garnet new --template agent-orchestrator /tmp/garnet-agent-smoke
```
