# AGENTS.md — CLI Contract

## Scope

Owns the `garnet` binary, subcommand routing, template embedding, deterministic manifests, project scaffolding, formatting/docs commands, and user-facing command text.

## Stable Contracts

- CLI output must be truthful about release readiness and installer availability.
- Templates are embedded with `include_str!`; adding a template file requires adding it to `new_cmd.rs` or it will not ship.
- Public commands should fail clearly with actionable errors.
- Deterministic build/verify behavior must stay reproducible.
- New agent-documentation tooling should start as opt-in or checking behavior before becoming a language requirement.

## Required Checks

```sh
cargo test -p garnet-cli
cargo run -p garnet-cli -- --help
```

For template changes, create each template and run `garnet test` inside it when possible.

## Child Contracts

- `/garnet-cli/templates/AGENTS.md` owns scaffolded project-template expectations.
