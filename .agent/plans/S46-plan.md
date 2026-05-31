# S46 Plan — caps-to-sandbox policy (WASI / seccomp / egress)

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S46.
Map: reconciled plan §147 — "caps-to-sandbox policy (declared @caps →
enforceable WASI/seccomp/egress policy)."
Branch: `codex/s46-sandbox`. Base: `origin/main` @ `62de5af` (S45).

## Goal
Make `@caps` actionable: derive a concrete, reviewable sandbox configuration
from the declared capability surface.

## Environment reality → honest-partial
`wasmtime`/`wasm-tools` ABSENT; seccomp is Linux-only (host macOS). So S46 ships
**policy generation**, deterministic + tested, and explicitly does NOT enforce.

## Deliverables
- `garnet-cli/src/sandbox.rs` (pure): `sandbox_policy(caps: &[String]) ->
  SandboxPolicy` — baseline syscalls + cap-gated groups (fs/net/time/proc),
  `WasiPolicy`, `EgressMode` (deny-all/loopback-only/allow), warnings
  (ffi/proc/wildcard/unknown). Deterministic `to_json` (`garnet.sandbox/v1`,
  `"enforced":false`) + `to_human`. 8 unit tests.
- `garnet-cli/src/cmd/sandbox.rs` + dispatch + help: `garnet sandbox <file>
  [--format human|json]`, reads `cap_manifest::surface_for_path`.
- `garnet-cli/tests/sandbox_cmd.rs` — 4 integration tests.
- `C_Language_Specification/GARNET_SANDBOX_POLICY.md` — mapping table + the
  honest enforcement boundary.

## Dogfood
- `garnet sandbox` on `@caps(fs)` → preopens, no sockets, egress deny-all;
  `@caps(net)` → sockets + egress allow; no caps → pure-compute deny-all;
  `@caps(ffi)` → escape-hatch warning. JSON carries `"enforced":false`.

## End-state / gates
- Full ladder green; CHANGELOG + contract S46 block + sandbox-policy doc.
  Ledger: `s45 → merged(5)` advanced this branch; `s46` advance rides with S47.

## Honest scope (do not soften)
- **Generation only.** No runtime enforcement performed or claimed.
- seccomp shape mirrors OCI default-deny but is not kernel-validated; egress
  allowlist is a structural placeholder.
- **No new readiness lane** — generation-only ≠ enforcement readiness.
