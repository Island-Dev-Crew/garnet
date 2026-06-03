# Garnet OS-sandbox application — S46 *generated* → *applied + trapped* on real Linux

S46 (`garnet sandbox`) **generates** a seccomp policy from a program's `@caps`
surface but, honestly, **does not enforce it** — `garnet sandbox` prints
`enforced: false (policy generation only)`. This slice closes that gap on **Linux
seccomp**: the generated policy is **applied as a real seccomp filter** and a
policy-violating syscall is **deterministically trapped** on a real kernel.

## What is now proven

On the Mac's **UTM Debian-12 ARM64** guest (Linux 6.1.0-13, `CONFIG_SECCOMP=y`,
libseccomp 2.5.4 — the recorded run is
`tools/seccomp-apply/PROOF_utm_debian12_aarch64.txt`):

- **`@caps(fs)` → `socket(AF_INET)` is DENIED** with `EPERM` ("Operation not
  permitted") — a deterministic trap, **identical across 3 runs**.
- **`@caps(fs, net)` → `socket` is ALLOWED** by the *same* harness — the trap
  **follows the declared capability surface**, it is not a hardcoded deny.
- An allowed baseline syscall (`getpid`) succeeds in both cases.

The mechanism: `tools/seccomp-apply/seccomp_apply.c` reads the syscall allowlist
that `garnet sandbox --format json` emits (default action `SCMP_ACT_ERRNO`), installs
a seccomp filter that allows exactly those syscalls, and probes the boundary.
`tools/seccomp-apply/prove.sh` reproduces it on any Linux host with `cc` +
`libseccomp-dev` + the `garnet` binary.

This moves S46 from **"generated, not enforced"** to **"generated + a reference apply
path proven to deterministically trap on a real Linux kernel."**

## Honest scope (do not soften)

- **Linux seccomp only.** macOS `sandbox-exec` and Windows AppContainer are separate
  and remain **named-deferred** (not proven here).
- It proves the **generated policy is enforceable** (applied + traps), **not** that a
  program is "safe": it enforces the *declared* syscall surface, not program intent.
- The apply path is a **reference C harness**, not yet `garnet`-native — the proof VM
  has **no Rust toolchain**, so Garnet *generates* the policy (on the Mac) and the
  harness *applies* it (on the Linux guest). A `garnet`-native Linux apply path
  (Rust, `cfg(linux)`) and applying the policy to a `garnet`-**spawned subprocess**
  (the S92 `[LINUX-INFRA]` goal — here the filter is applied to the harness's own
  process) are the next increments.
- Proven on **aarch64 / Debian 6.1**; re-run `prove.sh` to record a proof on any
  other kernel/arch.
- `garnet sandbox` still correctly prints `enforced: false` for its *generation*
  step — that flag is honest; enforcement happens in the (separate) apply step here.
  v0.8.1 remains a research-grade prototype; no production/1.0 claim.

See also: `C_Language_Specification/GARNET_SANDBOX_POLICY.md` (S46 generation),
`tools/seccomp-apply/README.md` (the tool).
