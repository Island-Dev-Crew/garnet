//! S46 — caps-to-sandbox policy generation.
//!
//! Translates a module's declared capability surface (S35
//! `garnet_check::capability_surface`) into three concrete sandbox policy
//! artifacts:
//!
//! - a **seccomp** profile (OCI/Docker-style default-deny syscall allowlist),
//! - a **WASI** capability set (which host facilities a guest may preopen/use),
//! - an **egress** rule (deny-all / loopback-only / allow).
//!
//! ## Honest scope (do not soften)
//!
//! This module **generates policy; it does not enforce it**. Nothing here runs a
//! guest under `wasmtime`, applies the seccomp profile to a live process, or
//! installs an egress firewall. Runtime enforcement requires `wasmtime` (WASI)
//! or a Linux seccomp host — both out of scope for this slice and absent in the
//! build environment. The seccomp profile mirrors the OCI default-deny shape but
//! is not validated against a kernel here; the egress allowlist is a structural
//! placeholder, not a live filter. The mapping is the deliverable: it makes the
//! `@caps` annotations actionable and reviewable.
//!
//! `ffi` capability is a deliberate escape hatch: native calls cannot be
//! constrained by seccomp/WASI, so the policy flags it loudly rather than
//! pretending to contain it. `*` (wildcard) yields a fully permissive policy
//! with a warning (debug-only; CI rejects wildcards upstream).

use crate::diagnostics::json_escape;

/// Always-allowed syscalls: process/memory lifecycle + stdio, needed by any
/// program (including a pure-compute one) to start, write output, and exit.
const BASELINE_SYSCALLS: &[&str] = &[
    "brk",
    "close",
    "exit",
    "exit_group",
    "fstat",
    "futex",
    "getpid",
    "mmap",
    "mprotect",
    "munmap",
    "read",
    "rt_sigaction",
    "rt_sigprocmask",
    "rt_sigreturn",
    "sched_yield",
    "write",
];

/// Syscalls unlocked by the `fs` capability (file access beyond inherited stdio).
const FS_SYSCALLS: &[&str] = &[
    "access",
    "getdents64",
    "lseek",
    "lstat",
    "mkdir",
    "open",
    "openat",
    "readlink",
    "rename",
    "stat",
    "unlink",
];

/// Syscalls unlocked by `net` / `net_internal` (sockets). The loopback-only
/// intent of `net_internal` is expressed by the egress policy, not seccomp
/// (seccomp cannot see destination addresses).
const NET_SYSCALLS: &[&str] = &[
    "accept",
    "accept4",
    "bind",
    "connect",
    "getpeername",
    "getsockname",
    "getsockopt",
    "listen",
    "recvfrom",
    "recvmsg",
    "sendmsg",
    "sendto",
    "setsockopt",
    "socket",
];

/// Syscalls unlocked by the `time` capability.
const TIME_SYSCALLS: &[&str] = &[
    "clock_gettime",
    "clock_nanosleep",
    "gettimeofday",
    "nanosleep",
];

/// Syscalls unlocked by the `proc` capability (process spawn / signals).
const PROC_SYSCALLS: &[&str] = &[
    "clone", "execve", "execveat", "fork", "kill", "vfork", "wait4",
];

/// The egress posture derived from the capability surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EgressMode {
    /// No network capability — all outbound traffic denied.
    DenyAll,
    /// `net_internal` only — loopback / RFC1918 intent.
    LoopbackOnly,
    /// `net` — outbound allowed (subject to the allowlist placeholder).
    Allow,
}

impl EgressMode {
    pub fn wire(self) -> &'static str {
        match self {
            EgressMode::DenyAll => "deny-all",
            EgressMode::LoopbackOnly => "loopback-only",
            EgressMode::Allow => "allow",
        }
    }
}

/// A WASI capability set: which host facilities a guest may use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasiPolicy {
    pub preopens: bool,
    pub sockets: bool,
    pub clocks: bool,
    pub env: bool,
    /// Stdio is always inherited so a guest can produce output.
    pub stdio: bool,
}

/// The full generated sandbox policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxPolicy {
    /// The capability names this policy was derived from (sorted, as given).
    pub caps: Vec<String>,
    /// Seccomp default action when a syscall is not in `seccomp_allow`.
    pub seccomp_default: &'static str,
    /// Sorted, deduplicated allowed syscalls.
    pub seccomp_allow: Vec<String>,
    pub wasi: WasiPolicy,
    pub egress: EgressMode,
    /// Human-facing cautions (ffi escape, wildcard, unknown caps).
    pub warnings: Vec<String>,
}

fn has(caps: &[String], name: &str) -> bool {
    caps.iter().any(|c| c == name)
}

/// Build the sandbox policy from a capability surface aggregate (cap names).
pub fn sandbox_policy(caps: &[String]) -> SandboxPolicy {
    let wildcard = has(caps, "*");
    let mut allow: std::collections::BTreeSet<String> =
        BASELINE_SYSCALLS.iter().map(|s| s.to_string()).collect();
    let mut add = |group: &[&str]| {
        for s in group {
            allow.insert((*s).to_string());
        }
    };

    if wildcard || has(caps, "fs") {
        add(FS_SYSCALLS);
    }
    if wildcard || has(caps, "net") || has(caps, "net_internal") {
        add(NET_SYSCALLS);
    }
    if wildcard || has(caps, "time") {
        add(TIME_SYSCALLS);
    }
    if wildcard || has(caps, "proc") {
        add(PROC_SYSCALLS);
    }

    let egress = if wildcard || has(caps, "net") {
        EgressMode::Allow
    } else if has(caps, "net_internal") {
        EgressMode::LoopbackOnly
    } else {
        EgressMode::DenyAll
    };

    let wasi = WasiPolicy {
        preopens: wildcard || has(caps, "fs"),
        sockets: wildcard || has(caps, "net") || has(caps, "net_internal"),
        clocks: wildcard || has(caps, "time"),
        env: wildcard || has(caps, "env"),
        stdio: true,
    };

    let mut warnings = Vec::new();
    if wildcard {
        warnings.push(
            "wildcard `@caps(*)`: the generated policy is fully permissive (debug only; \
             CI rejects wildcards)"
                .to_string(),
        );
    }
    if has(caps, "ffi") {
        warnings.push(
            "`ffi` capability: native calls cannot be constrained by seccomp or WASI — this \
             policy flags but does not contain FFI"
                .to_string(),
        );
    }
    if has(caps, "proc") {
        warnings.push(
            "`proc` capability: process spawn/exec is allowed; the sandbox cannot bound what \
             child processes do"
                .to_string(),
        );
    }
    // Unknown capabilities the model does not map to syscalls.
    let known = [
        "fs",
        "net",
        "net_internal",
        "time",
        "proc",
        "ffi",
        "env",
        "*",
    ];
    for c in caps {
        if !known.contains(&c.as_str()) {
            warnings.push(format!(
                "unknown capability `{c}`: not mapped to any syscall/WASI facility (no-op in this policy)"
            ));
        }
    }

    SandboxPolicy {
        caps: caps.to_vec(),
        seccomp_default: "SCMP_ACT_ERRNO",
        seccomp_allow: allow.into_iter().collect(),
        wasi,
        egress,
        warnings,
    }
}

impl SandboxPolicy {
    /// Deterministic, hand-rolled JSON (no serde; matches the repo's
    /// determinism stance in `manifest.rs` / `cap_manifest.rs`).
    pub fn to_json(&self) -> String {
        let caps = json_str_array(&self.caps);
        let allow = json_str_array(&self.seccomp_allow);
        let warnings = json_str_array(&self.warnings);
        format!(
            "{{\"schema\":\"garnet.sandbox/v1\",\
             \"caps\":{caps},\
             \"seccomp\":{{\"default_action\":\"{}\",\"allow\":{allow}}},\
             \"wasi\":{{\"preopens\":{},\"sockets\":{},\"clocks\":{},\"env\":{},\"stdio\":{}}},\
             \"egress\":{{\"mode\":\"{}\"}},\
             \"enforced\":false,\
             \"warnings\":{warnings}}}",
            self.seccomp_default,
            self.wasi.preopens,
            self.wasi.sockets,
            self.wasi.clocks,
            self.wasi.env,
            self.wasi.stdio,
            self.egress.wire(),
        )
    }

    /// A human-readable summary.
    pub fn to_human(&self) -> String {
        let mut out = String::new();
        let caps = if self.caps.is_empty() {
            "(none — pure compute)".to_string()
        } else {
            self.caps.join(", ")
        };
        out.push_str(&format!("caps: {caps}\n"));
        out.push_str(&format!(
            "seccomp: default {} + {} allowed syscalls\n",
            self.seccomp_default,
            self.seccomp_allow.len()
        ));
        out.push_str(&format!(
            "wasi: preopens={} sockets={} clocks={} env={} stdio={}\n",
            self.wasi.preopens, self.wasi.sockets, self.wasi.clocks, self.wasi.env, self.wasi.stdio
        ));
        out.push_str(&format!("egress: {}\n", self.egress.wire()));
        out.push_str("enforced: false (policy generation only — see GARNET_SANDBOX_POLICY.md)\n");
        for w in &self.warnings {
            out.push_str(&format!("warning: {w}\n"));
        }
        out
    }
}

/// JSON array of strings, deterministic (input order preserved).
fn json_str_array(items: &[String]) -> String {
    let inner: Vec<String> = items
        .iter()
        .map(|s| format!("\"{}\"", json_escape(s)))
        .collect();
    format!("[{}]", inner.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caps(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn no_caps_is_deny_all_pure_compute() {
        let p = sandbox_policy(&caps(&[]));
        assert_eq!(p.egress, EgressMode::DenyAll);
        assert!(!p.wasi.preopens && !p.wasi.sockets);
        // Baseline only: stdio works, but no fs/net/proc syscalls.
        assert!(p.seccomp_allow.contains(&"write".to_string()));
        assert!(!p.seccomp_allow.contains(&"open".to_string()));
        assert!(!p.seccomp_allow.contains(&"socket".to_string()));
        assert!(p.warnings.is_empty());
    }

    #[test]
    fn fs_unlocks_file_syscalls_only() {
        let p = sandbox_policy(&caps(&["fs"]));
        assert!(p.wasi.preopens);
        assert!(p.seccomp_allow.contains(&"openat".to_string()));
        assert!(!p.seccomp_allow.contains(&"socket".to_string()));
        assert_eq!(p.egress, EgressMode::DenyAll);
    }

    #[test]
    fn net_allows_egress_and_sockets() {
        let p = sandbox_policy(&caps(&["net"]));
        assert_eq!(p.egress, EgressMode::Allow);
        assert!(p.wasi.sockets);
        assert!(p.seccomp_allow.contains(&"connect".to_string()));
    }

    #[test]
    fn net_internal_is_loopback_only() {
        let p = sandbox_policy(&caps(&["net_internal"]));
        assert_eq!(p.egress, EgressMode::LoopbackOnly);
        assert!(p.wasi.sockets);
    }

    #[test]
    fn ffi_and_proc_emit_warnings() {
        let p = sandbox_policy(&caps(&["ffi", "proc"]));
        assert!(p.warnings.iter().any(|w| w.contains("ffi")));
        assert!(p.warnings.iter().any(|w| w.contains("proc")));
    }

    #[test]
    fn wildcard_is_permissive_with_warning() {
        let p = sandbox_policy(&caps(&["*"]));
        assert!(p.seccomp_allow.contains(&"open".to_string()));
        assert!(p.seccomp_allow.contains(&"socket".to_string()));
        assert_eq!(p.egress, EgressMode::Allow);
        assert!(p.warnings.iter().any(|w| w.contains("wildcard")));
    }

    #[test]
    fn json_is_deterministic_and_marks_unenforced() {
        let p = sandbox_policy(&caps(&["fs"]));
        let j = p.to_json();
        assert_eq!(j, sandbox_policy(&caps(&["fs"])).to_json());
        assert!(j.contains("\"enforced\":false"));
        assert!(j.contains("\"schema\":\"garnet.sandbox/v1\""));
        assert!(j.contains("\"mode\":\"deny-all\""));
    }

    #[test]
    fn unknown_cap_is_flagged() {
        let p = sandbox_policy(&caps(&["quantum"]));
        assert!(p.warnings.iter().any(|w| w.contains("quantum")));
    }
}
