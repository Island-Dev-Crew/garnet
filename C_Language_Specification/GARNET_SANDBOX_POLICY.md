# Garnet Sandbox Policy (S46)

`garnet sandbox <file>` translates a module's declared capability surface
(`@caps(...)`, S35) into three concrete sandbox policy artifacts. It is the
bridge from *declared* capability to *enforceable* configuration.

## Honest scope — generation, not enforcement

**This slice generates policy; it does not enforce it.** Nothing in `garnet
sandbox` runs a guest under `wasmtime`, applies a seccomp profile to a live
process, or installs an egress firewall. Every emitted policy is marked
`"enforced": false`. Runtime enforcement requires:

- a `wasmtime` (or other WASI) host to honor the WASI capability set, and
- a Linux kernel + a seccomp loader to honor the syscall profile, and
- a network layer to honor the egress rule.

These are **out of scope for S46** (and `wasmtime`/`wasm-tools` are absent from
the current build environment; seccomp is Linux-only). The seccomp profile
mirrors the OCI/Docker default-deny shape but is **not** validated against a live
kernel here; the egress allowlist is a **structural placeholder**, not a live
filter. The deliverable is the *mapping*: it makes `@caps` annotations
actionable, reviewable, and diff-able alongside the capability manifest (S36)
and capability-surface diff gate (S37).

## The mapping

The policy is derived from the **aggregate** capability surface (the union of
every function's `@caps`). Each capability unlocks a syscall group, a WASI
facility, and/or an egress posture.

| Capability     | seccomp syscalls (added)                              | WASI            | Egress         |
|----------------|------------------------------------------------------|-----------------|----------------|
| *(none)*       | baseline only (stdio + process/memory lifecycle)     | stdio           | deny-all       |
| `fs`           | open, openat, stat, lstat, lseek, getdents64, mkdir, unlink, rename, access, readlink | preopens | deny-all |
| `net`          | socket, connect, bind, listen, accept(4), send*, recv*, get/setsockopt, get{peer,sock}name | sockets | allow |
| `net_internal` | (same socket syscalls as `net`)                      | sockets         | loopback-only  |
| `time`         | clock_gettime, clock_nanosleep, gettimeofday, nanosleep | clocks       | —              |
| `proc`         | fork, vfork, clone, execve, execveat, wait4, kill    | —               | — (+warning)   |
| `env`          | —                                                    | env             | —              |
| `ffi`          | — (**escape hatch** — see below)                     | —               | — (+warning)   |
| `*` (wildcard) | all groups                                           | all             | allow (+warning) |

The **baseline** syscalls (always allowed, even for pure compute) are: `brk`,
`close`, `exit`, `exit_group`, `fstat`, `futex`, `getpid`, `mmap`, `mprotect`,
`munmap`, `read`, `rt_sigaction`, `rt_sigprocmask`, `rt_sigreturn`,
`sched_yield`, `write`. They let any program start, write output, and exit; they
do not grant file, network, time, or process authority.

The seccomp `default_action` is `SCMP_ACT_ERRNO` (deny-by-errno); only the
allowed syscalls carry `SCMP_ACT_ALLOW`.

## Warnings

The generator flags cases where the sandbox cannot honestly contain the program:

- **`ffi`** — native calls bypass both seccomp syscall filtering (the FFI shim
  issues syscalls the static caps don't predict) and WASI. The policy *flags*
  FFI; it does not contain it.
- **`proc`** — process spawn/exec is allowed, but the sandbox cannot bound what
  the child does.
- **`*` (wildcard)** — fully permissive; debug-only (CI rejects wildcards
  upstream).
- **unknown capability** — a cap name the model does not map is a no-op in the
  policy and is reported.

## Output

`--format human` prints a summary; `--format json` emits a deterministic,
hand-rolled document (`"schema": "garnet.sandbox/v1"`) suitable for review,
diffing, or feeding a future enforcement step. The JSON always carries
`"enforced": false`.
