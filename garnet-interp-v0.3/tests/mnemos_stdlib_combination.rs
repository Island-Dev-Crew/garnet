//! S21 — Mnemos × stdlib combination smoke.
//!
//! Exercises the four cognitively-inspired Mnemos memory kinds
//! (working / episodic / semantic / procedural) composed with the standard
//! library (BLAKE3 content-addressing + RFC 4648 base64), proving the memory
//! core and the stdlib compose across diverse system-programming build options.
//!
//! This is the Rust/system-level companion to S20's modeled-in-Garnet memory
//! recall and S21's qualified stdlib dispatch: here the *real* Mnemos stores
//! carry stdlib-produced payloads end to end.

use garnet_memory::{EpisodeStore, VectorIndex, WorkflowStore, WorkingStore};
use garnet_stdlib::{base64, crypto};

fn hex(digest: &[u8; 32]) -> String {
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// A capability-gated agent pipeline whose every stage's output flows through
/// the stdlib and into a distinct Mnemos store, then is recalled and verified.
#[test]
fn mnemos_four_kinds_compose_with_stdlib() {
    let payload = "ingest:specs=3|transform:rules=2|emit:artifacts=1";

    // ── working memory: stage the content-addressed provenance fingerprint ──
    let working: WorkingStore<String> = WorkingStore::new();
    let provenance = hex(&crypto::blake3_hash(payload.as_bytes()));
    let slot = working.push(provenance.clone());
    assert_eq!(working.len(), 1);
    let recalled = working
        .with(slot, |v| v.clone())
        .expect("working slot present");
    assert_eq!(
        recalled, provenance,
        "working memory must recall the fingerprint"
    );

    // ── episodic memory: record the run as a base64-encoded episode ──
    let episodic: EpisodeStore<String> = EpisodeStore::new();
    let artifact = base64::encode(payload.as_bytes());
    episodic.append(artifact.clone());
    episodic.append(base64::encode(b"emit:artifacts=1"));
    assert_eq!(episodic.len(), 2);
    assert_eq!(episodic.recent(1).len(), 1, "episodic recency window works");
    // The episode's base64 artifact round-trips back to the original bytes.
    assert_eq!(
        base64::decode(&artifact).expect("valid base64"),
        payload.as_bytes(),
        "stdlib base64 round-trips through episodic memory"
    );

    // ── semantic memory: index feature embeddings + retrieve by similarity ──
    let semantic: VectorIndex<String> = VectorIndex::new();
    semantic.insert(vec![1.0, 0.0, 0.0], "ingest".to_string());
    semantic.insert(vec![0.0, 1.0, 0.0], "transform".to_string());
    semantic.insert(vec![0.0, 0.0, 1.0], "emit".to_string());
    assert_eq!(semantic.len(), 3);
    let hits = semantic.search(&[0.0, 1.0, 0.0], 1);
    assert_eq!(hits.len(), 1);
    assert_eq!(
        hits[0].1, "transform",
        "semantic search recalls the nearest stage"
    );

    // ── procedural memory: register + recall a workflow ──
    let procedural: WorkflowStore<String> = WorkflowStore::new();
    procedural.register("build", provenance.clone());
    let wf = procedural.find("build").expect("workflow registered");
    assert_eq!(
        wf.current().cloned(),
        Some(provenance.clone()),
        "procedural memory recalls the registered workflow body"
    );
}

/// Determinism: the same payload yields the same fingerprint + base64 across
/// independent store instances — a stable content identity for the memory core.
#[test]
fn mnemos_stdlib_identity_is_deterministic() {
    let payload = "release:garnet@v0.7.0|artifacts=6";

    let mut fingerprints = Vec::new();
    let mut artifacts = Vec::new();
    for _ in 0..3 {
        let store: WorkingStore<String> = WorkingStore::new();
        let fp = hex(&crypto::blake3_hash(payload.as_bytes()));
        let b64 = base64::encode(payload.as_bytes());
        store.push(fp.clone());
        fingerprints.push(fp);
        artifacts.push(b64);
    }
    assert!(
        fingerprints.windows(2).all(|w| w[0] == w[1]),
        "blake3 provenance is stable across store instances"
    );
    assert!(
        artifacts.windows(2).all(|w| w[0] == w[1]),
        "base64 artifact is stable across store instances"
    );
}
