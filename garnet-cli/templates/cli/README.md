# {{name}}

A Garnet CLI application generated with `garnet new --template cli`.

## Run

```sh
garnet run src/main.garnet
```

## Test

```sh
garnet test
```

## Build (deterministic + signed)

```sh
# One-time: generate a signing key.
garnet keygen my.key

# Build and sign.
garnet build --deterministic --sign my.key src/main.garnet

# Anyone with the resulting manifest.json can verify the build:
garnet verify src/main.garnet src/main.garnet.manifest.json --signature
```

## Capability model

Every function in this project declares `@caps(...)` listing the OS
authority it exercises. The compiler enforces the declaration
transitively: if `main()` calls `read_config()` which calls
`fs::read_file(...)`, then `main()` must declare `@caps(fs)`. See
Paper III §6 for the full capability inventory and Paper VI
Contribution 1 for the "semantic beacon" rationale.

## Project layout

```
{{name}}/
  Garnet.toml              # project manifest
  src/main.garnet          # entry point with @caps(...)
  tests/test_main.garnet   # test functions named test_*
  .gitignore               # excludes .garnet-cache/ + signing keys
  README.md                # this file
```

## License

Choose a license appropriate for your use case. Common choices:
[MIT](https://opensource.org/licenses/MIT) or
[Apache-2.0](https://opensource.org/licenses/Apache-2.0). Garnet
itself ships dual-licensed MIT OR Apache-2.0.
