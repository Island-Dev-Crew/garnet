# AGENTS.md — Actor Runtime Contract

## Scope

Owns bounded mailboxes, actor addressing, request/reply behavior, signed hot reload, and state migration behavior.

## Stable Contracts

- Mailboxes must remain bounded and explicit about overflow behavior.
- Hot reload must preserve safety boundaries and signature expectations.
- State migration must fail loudly when compatibility cannot be established.
- Actor examples should remain small, deterministic, and test-backed.

## Required Checks

```sh
cargo test -p garnet-actor-runtime
```
