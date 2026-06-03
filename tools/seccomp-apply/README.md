# Garnet seccomp-apply — turning S46's *generated* policy into an *applied + trapped* one

S46 (`garnet sandbox`) **generates** a seccomp policy from a program's `@caps`
surface but explicitly **does not enforce it** ("policy generation only"). This tool
closes that gap on **Linux**: it takes the syscall allowlist Garnet generates and
**applies it as a real seccomp filter**, then demonstrates a deterministic **trap**.

- `seccomp_apply.c` — the apply harness. Reads a newline-separated syscall allowlist
  (from `garnet sandbox --format json`), installs a seccomp filter with default
  action `ERRNO(EPERM)` (mirroring Garnet's `SCMP_ACT_ERRNO`) allowing exactly those
  syscalls, then probes: `getpid()` (allowed) succeeds; `socket(AF_INET)` is **denied**
  under `@caps(fs)` and **allowed** under `@caps(fs, net)` — proving the trap follows
  the declared capability surface, not a hardcoded rule.
- `prove.sh` — reproduces the whole proof on a Linux host (needs `cc`,
  `libseccomp-dev`, and the `garnet` binary): generates the policy, builds the
  harness, and asserts the deny + the policy-driven allow.
- `PROOF_utm_debian12_aarch64.txt` — the **recorded proof** on a real kernel: the
  Mac's UTM Debian-12 ARM64 guest (Linux 6.1, `CONFIG_SECCOMP=y`, libseccomp 2.5.4).
  `@caps(fs)` traps `socket` (EPERM) deterministically across 3 runs; `@caps(fs,net)`
  allows it.

## Build + run

```sh
# On a Linux host (the garnet binary may be built elsewhere and the policy transferred):
cc -O2 -o seccomp_apply tools/seccomp-apply/seccomp_apply.c -lseccomp
garnet sandbox --format json prog.garnet \
  | python3 -c "import json,sys;print('\n'.join(json.load(sys.stdin)['seccomp']['allow']))" > allow.txt
./seccomp_apply allow.txt denied        # @caps(fs): socket -> BLOCKED (trap)
# or: bash tools/seccomp-apply/prove.sh  (full Linux host with garnet built)
```

## Honest scope (do not soften)

- **Linux seccomp only.** macOS `sandbox-exec` and Windows AppContainer are separate
  and remain **named-deferred**.
- This proves the **generated policy is enforceable** (applied + traps) on a real
  Linux kernel — **not** that a program is "safe". It enforces the *declared* syscall
  surface; it does not reason about program intent.
- The apply path is a **reference harness** (C), not yet `garnet`-native: the proof
  VM has no Rust toolchain, so Garnet *generates* the policy (on the Mac) and the
  harness *applies* it (on the Linux guest). A `garnet`-native Linux apply path
  (Rust, `cfg(linux)`) is a follow-up.
- The filter is applied to the **harness's own process** (a self-sandbox probe).
  Applying the generated policy to a `garnet`-**spawned subprocess** (the S92
  `[LINUX-INFRA]` goal) is the next increment.
- Proven on **aarch64 / Debian 6.1**; all 27/41 syscall names resolved there via
  libseccomp. Other kernels/arches must re-run `prove.sh` to record their own proof.
