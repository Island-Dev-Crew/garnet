# {{name}}

An HTTP/1.1 API service generated with `garnet new --template web-api`.

## Run

```sh
garnet run src/main.garnet
# service listens on :8080
```

## Capability model

This service declares `@caps(net, time)`:

- **net** — bind the listener, accept connections, send responses. The
  v3.4 NetDefaults gate denies outbound connections to RFC1918 / loopback
  / link-local by default; lift the gate per-function with
  `@caps(net_internal)` only if the service genuinely needs to reach
  internal addresses.
- **time** — request deadlines, timestamp logging.

The service CANNOT touch the filesystem, spawn subprocesses, or hot-reload
without adding the corresponding cap. `garnet check` will reject the
build if any function transitively invokes a primitive whose required
cap is not in the service's declared set.

## BoundedMail

Each actor handling requests should cap its mailbox explicitly:

```garnet
actor RequestHandler {
  @mailbox(1024)   # BoundedMail; back-pressures at 1024 in-flight
  protocol handle(req: Request) -> Response
  ...
}
```

See v3.4 Security V2 §3 for the BoundedMail rationale.

## Deployment

```sh
# Reproducible, signed build suitable for distribution.
garnet build --deterministic --sign my.key src/main.garnet
```

Consumers verify before running:

```sh
garnet verify src/main.garnet src/main.garnet.manifest.json --signature
```
