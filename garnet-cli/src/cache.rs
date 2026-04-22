//! Compiler-as-Agent episode log — Paper VI Contribution 3 (layer 1).
//!
//! Every CLI invocation appends one NDJSON record to `.garnet-cache/episodes.log`
//! capturing what was run, on what file, with what outcome, and how long it
//! took. The log is the substrate the higher cache layers (knowledge.db,
//! strategies.db) consume in Phase 7.
//!
//! **v3.3 CacheHMAC hardening.** Every record carries an HMAC-equivalent
//! BLAKE3 keyed MAC over its canonical serialization, computed using the
//! per-machine key from `machine_key.rs`. Records whose MAC doesn't
//! verify against this machine's key are silently skipped on read — so
//! a committed `.garnet-cache/` from another machine, or a file mutated
//! by a co-tenant in a shared tmp/CI dir, cannot influence compilation
//! decisions. The read path returns the SkipCount so callers can warn
//! loudly when any unverified entries are found.

use crate::machine_key;
use std::fmt::Write as _;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const CACHE_DIR: &str = ".garnet-cache";
const EPISODES_FILE: &str = "episodes.log";

/// One record per CLI invocation. Schema is stable; new fields go at the
/// end and are optional in the parser.
///
/// **v3.3 CacheHMAC:** `hmac` field carries the hex-encoded BLAKE3 keyed
/// MAC over the canonical serialization of all other fields. Computed
/// at write time via the process's machine key. Verified at read time
/// — records with missing or invalid MAC are skipped (see `read_all_in`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Episode {
    pub ts: u64,
    pub cmd: String,
    pub file: String,
    pub source_hash: String,
    pub outcome: String,        // "ok" | "parse_err" | "check_err" | "runtime_err"
    pub error_kind: Option<String>,
    pub duration_ms: u64,
    pub parser_version: String,
    pub exit_code: i32,
    /// Hex-encoded BLAKE3-keyed MAC over canonical serialization of the
    /// other fields. `None` until computed by `sign_with_key`.
    pub hmac: Option<String>,
}

impl Episode {
    pub fn now(
        cmd: impl Into<String>,
        file: impl Into<String>,
        source_hash: impl Into<String>,
        outcome: impl Into<String>,
        error_kind: Option<String>,
        duration_ms: u64,
        exit_code: i32,
    ) -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Episode {
            ts,
            cmd: cmd.into(),
            file: file.into(),
            source_hash: source_hash.into(),
            outcome: outcome.into(),
            error_kind,
            duration_ms,
            parser_version: env!("CARGO_PKG_VERSION").to_string(),
            exit_code,
            hmac: None,
        }
    }

    /// Canonical length-prefixed serialization of every field EXCEPT
    /// `hmac`. Length-prefixing avoids ambiguity with embedded nulls
    /// or commas in string fields — two distinct episodes cannot
    /// collide to the same canonical bytes.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        push_field(&mut out, &self.ts.to_le_bytes());
        push_field(&mut out, self.cmd.as_bytes());
        push_field(&mut out, self.file.as_bytes());
        push_field(&mut out, self.source_hash.as_bytes());
        push_field(&mut out, self.outcome.as_bytes());
        // error_kind is Option<String>: tag presence with a length
        // prefix of 0 for None so Some("") != None canonically.
        match &self.error_kind {
            Some(ek) => {
                out.push(1);
                push_field(&mut out, ek.as_bytes());
            }
            None => out.push(0),
        }
        push_field(&mut out, &self.duration_ms.to_le_bytes());
        push_field(&mut out, self.parser_version.as_bytes());
        push_field(&mut out, &self.exit_code.to_le_bytes());
        out
    }

    /// Compute the HMAC-equivalent MAC for this episode using the
    /// per-machine key and store it in `self.hmac`.
    pub fn sign(&mut self) {
        let mac = machine_key::mac(&self.canonical_bytes());
        self.hmac = Some(machine_key::mac_to_hex(&mac));
    }

    /// Same, with an explicit key (used by tests).
    pub fn sign_with_key(&mut self, key: &[u8; 32]) {
        let mac = machine_key::mac_with_key(key, &self.canonical_bytes());
        self.hmac = Some(machine_key::mac_to_hex(&mac));
    }

    /// Verify this episode's stored MAC matches a freshly-computed MAC
    /// over its canonical bytes using the given key. Returns `false`
    /// for any failure mode: missing MAC, malformed hex, byte mismatch.
    pub fn verify_with_key(&self, key: &[u8; 32]) -> bool {
        let Some(hex) = &self.hmac else { return false };
        let Some(recorded) = machine_key::mac_from_hex(hex) else {
            return false;
        };
        let fresh = machine_key::mac_with_key(key, &self.canonical_bytes());
        constant_time_eq(&recorded, &fresh)
    }

    /// Convenience: verify using this machine's key.
    pub fn verify(&self) -> bool {
        self.verify_with_key(machine_key::machine_key())
    }

    pub fn to_ndjson_line(&self) -> String {
        let mut s = String::from("{");
        let _ = write!(s, "\"ts\":{}", self.ts);
        let _ = write!(s, ",\"cmd\":{}", json_str(&self.cmd));
        let _ = write!(s, ",\"file\":{}", json_str(&self.file));
        let _ = write!(s, ",\"source_hash\":{}", json_str(&self.source_hash));
        let _ = write!(s, ",\"outcome\":{}", json_str(&self.outcome));
        if let Some(ek) = &self.error_kind {
            let _ = write!(s, ",\"error_kind\":{}", json_str(ek));
        }
        let _ = write!(s, ",\"duration_ms\":{}", self.duration_ms);
        let _ = write!(s, ",\"parser_version\":{}", json_str(&self.parser_version));
        let _ = write!(s, ",\"exit_code\":{}", self.exit_code);
        if let Some(hmac) = &self.hmac {
            let _ = write!(s, ",\"hmac\":{}", json_str(hmac));
        }
        s.push('}');
        s.push('\n');
        s
    }

    /// Permissive line parser: looks for the seven required fields.
    /// Returns `None` for malformed records (forward compatibility).
    ///
    /// The `hmac` field is optional in the parser (records written before
    /// CacheHMAC will have no MAC) but unverified records are SKIPPED
    /// by `read_all_in_verified` in v3.3 — only legitimate v3.3-written
    /// records survive the read filter.
    pub fn from_ndjson_line(line: &str) -> Option<Episode> {
        let mut ts = None;
        let mut cmd = None;
        let mut file = None;
        let mut source_hash = None;
        let mut outcome = None;
        let mut error_kind = None;
        let mut duration_ms = None;
        let mut parser_version = None;
        let mut exit_code = None;
        let mut hmac = None;
        for field in split_top_level(line.trim().trim_start_matches('{').trim_end_matches('}')) {
            if let Some((key, value)) = field.split_once(':') {
                let key = key.trim().trim_matches('"');
                let value = value.trim();
                match key {
                    "ts" => ts = value.parse().ok(),
                    "cmd" => cmd = Some(unjson_str(value)),
                    "file" => file = Some(unjson_str(value)),
                    "source_hash" => source_hash = Some(unjson_str(value)),
                    "outcome" => outcome = Some(unjson_str(value)),
                    "error_kind" => error_kind = Some(unjson_str(value)),
                    "duration_ms" => duration_ms = value.parse().ok(),
                    "parser_version" => parser_version = Some(unjson_str(value)),
                    "exit_code" => exit_code = value.parse().ok(),
                    "hmac" => hmac = Some(unjson_str(value)),
                    _ => {}
                }
            }
        }
        Some(Episode {
            ts: ts?,
            cmd: cmd?,
            file: file?,
            source_hash: source_hash?,
            outcome: outcome?,
            error_kind,
            duration_ms: duration_ms?,
            parser_version: parser_version?,
            exit_code: exit_code?,
            hmac,
        })
    }
}

fn push_field(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
}

/// Constant-time byte comparison for MAC verification. Avoids early-
/// exit timing side channels that would leak MAC prefix information to
/// an attacker probing the cache. 32 bytes, no short-circuit.
fn constant_time_eq(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

/// Resolve the cache directory: prefer `<cwd>/.garnet-cache`, but fall back
/// to a temp-dir subfolder if the cwd isn't writable. `cache_dir_for` lets
/// tests override the location.
pub fn cache_dir() -> PathBuf {
    PathBuf::from(CACHE_DIR)
}

pub fn cache_dir_for(base: &Path) -> PathBuf {
    base.join(CACHE_DIR)
}

/// Append one episode to `<cache_dir>/episodes.log`, creating the directory
/// if missing. Errors are non-fatal: we suppress them and return `false` so
/// the CLI never fails because of cache I/O.
pub fn record_episode(ep: &Episode) -> bool {
    record_episode_in(&cache_dir(), ep)
}

pub fn record_episode_in(dir: &Path, ep: &Episode) -> bool {
    record_episode_in_with_key(dir, ep, machine_key::machine_key())
}

/// Sign and append an episode with an explicit key. The caller's Episode
/// is cloned so the stored MAC doesn't leak back into their local state.
pub fn record_episode_in_with_key(dir: &Path, ep: &Episode, key: &[u8; 32]) -> bool {
    if std::fs::create_dir_all(dir).is_err() {
        return false;
    }
    let path = dir.join(EPISODES_FILE);
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) else {
        return false;
    };
    let mut signed = ep.clone();
    signed.sign_with_key(key);
    file.write_all(signed.to_ndjson_line().as_bytes()).is_ok()
}

/// Read all episodes whose `source_hash` matches `target`. Most-recent
/// last. Unverified entries are silently skipped — see `recall_audit`
/// for a variant that reports the skip count.
pub fn recall(target: &str) -> Vec<Episode> {
    recall_in(&cache_dir(), target)
}

pub fn recall_in(dir: &Path, target: &str) -> Vec<Episode> {
    recall_in_with_key(dir, target, machine_key::machine_key()).episodes
}

/// Outcome of a verified read: the verified episodes and how many
/// unverified records were skipped. Callers that care about cache
/// integrity (e.g., the strategy miner) should warn when `skipped > 0`.
pub struct ReadResult {
    pub episodes: Vec<Episode>,
    /// Records that parsed but whose HMAC failed to verify (wrong key,
    /// tampered bytes, or pre-CacheHMAC records without a MAC).
    pub skipped: usize,
}

pub fn recall_in_with_key(dir: &Path, target: &str, key: &[u8; 32]) -> ReadResult {
    let path = dir.join(EPISODES_FILE);
    let Ok(file) = File::open(&path) else {
        return ReadResult {
            episodes: Vec::new(),
            skipped: 0,
        };
    };
    let mut out = Vec::new();
    let mut skipped = 0;
    for line in BufReader::new(file).lines().map_while(Result::ok) {
        if let Some(ep) = Episode::from_ndjson_line(&line) {
            if !ep.verify_with_key(key) {
                skipped += 1;
                continue;
            }
            if ep.source_hash == target {
                out.push(ep);
            }
        }
    }
    ReadResult {
        episodes: out,
        skipped,
    }
}

/// Read all episodes from the given cache directory (no source-hash
/// filter). v3.3: skips records whose HMAC doesn't verify against this
/// machine's key — the returned list contains only locally-trusted
/// records. For full visibility (including foreign records), use
/// `read_all_in_audit`.
pub fn read_all_in(dir: &Path) -> Vec<Episode> {
    read_all_in_with_key(dir, machine_key::machine_key()).episodes
}

pub fn read_all_in_with_key(dir: &Path, key: &[u8; 32]) -> ReadResult {
    let path = dir.join(EPISODES_FILE);
    let Ok(file) = File::open(&path) else {
        return ReadResult {
            episodes: Vec::new(),
            skipped: 0,
        };
    };
    let mut out = Vec::new();
    let mut skipped = 0;
    for line in BufReader::new(file).lines().map_while(Result::ok) {
        if let Some(ep) = Episode::from_ndjson_line(&line) {
            if ep.verify_with_key(key) {
                out.push(ep);
            } else {
                skipped += 1;
            }
        }
    }
    ReadResult {
        episodes: out,
        skipped,
    }
}

/// Compute the BLAKE3 source hash used as the join key in episode and
/// knowledge tables. Re-uses `manifest`'s hashing convention.
pub fn source_hash(src: &str) -> String {
    blake3::hash(src.as_bytes()).to_hex().to_string()
}

// ── Tiny JSON helpers (we control both ends; serde would be overkill) ──

fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn unjson_str(value: &str) -> String {
    let v = value.trim();
    let v = v.strip_prefix('"').unwrap_or(v);
    let v = v.strip_suffix('"').unwrap_or(v);
    let mut out = String::with_capacity(v.len());
    let mut chars = v.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Split a comma-separated key:value list at the top level (ignores commas
/// inside string literals). The schema is flat so we don't need full JSON.
fn split_top_level(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut in_str = false;
    let mut prev = '\0';
    for c in body.chars() {
        if c == '"' && prev != '\\' {
            in_str = !in_str;
        }
        if c == ',' && !in_str {
            out.push(std::mem::take(&mut current));
        } else {
            current.push(c);
        }
        prev = c;
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn episode_roundtrip_through_ndjson() {
        let ep = Episode::now("parse", "/tmp/foo.garnet", "abc123", "ok", None, 12, 0);
        let line = ep.to_ndjson_line();
        let parsed = Episode::from_ndjson_line(&line).unwrap();
        assert_eq!(parsed, ep);
    }

    #[test]
    fn episode_with_error_kind_roundtrips() {
        let ep = Episode::now(
            "check",
            "/tmp/bad.garnet",
            "deadbeef",
            "check_err",
            Some("safe_violation".to_string()),
            55,
            1,
        );
        let line = ep.to_ndjson_line();
        let parsed = Episode::from_ndjson_line(&line).unwrap();
        assert_eq!(parsed, ep);
    }
}
