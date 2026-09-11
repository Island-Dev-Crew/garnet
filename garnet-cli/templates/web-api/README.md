# {{name}}

An HTTP/1.1 API service scaffold generated with `garnet new --template web-api`.
It does not listen on a port yet: `serve_forever` in `src/main.garnet` is a
placeholder for the listener you write.

## Run

```sh
garnet run src/main.garnet
# prints a startup line and the placeholder message, then exits
```

## Capability model

`main` declares `@caps(net, time)`, the budget for the service you build:

- **net** — network access. `@caps(net_internal)` does not change what
  `net::tcp_connect` may reach at run time, and `net::tcp_listen` is not
  bridged into the runtime yet.
- **time** — timestamps and deadlines (`wall_clock_ms`).

`garnet check` reports an annotated function whose named, acyclic calls reach
a primitive needing a capability that function does not declare; calls
through function values, closures or cycles are not traced. At run time,
the file-system, process, environment and outbound-network primitives also
trap unless `main` declares their capability; the `time` primitives are
checked by `garnet check` only. The exact scope is
`C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md` in the
Garnet repository.

## Bounded mailboxes

Each managed actor has a bounded mailbox: 1024 messages unless you pass
another size to `spawn`. A `tell` to a full mailbox fails.

```garnet
let handler = RequestHandler.spawn(256)
```

The `@mailbox(N)` annotation is range-checked by `garnet check` but does not
set the capacity.

## Deployment

```sh
# Reproducible, signed build suitable for distribution.
garnet build --deterministic --sign my.key src/main.garnet
```

Consumers verify before running:

```sh
garnet verify src/main.garnet src/main.garnet.manifest.json --signature
```
