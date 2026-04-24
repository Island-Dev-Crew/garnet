//! Compiler-as-Agent knowledge store — Paper VI Contribution 3 (layer 2).
//!
//! `.garnet-cache/knowledge.db` is a SQLite store that holds an AST
//! fingerprint for every compilation context the CLI has seen. Fingerprints
//! are deterministic per AST (no embedding model required): they are 256
//! bits of BLAKE3 hash over a canonical bag-of-features extracted from the
//! AST shape. Similarity is Hamming distance over the 32 fingerprint bytes
//! — a coarse but cheap stand-in for cosine over a real embedding.
//!
//! The schema is intentionally tiny so that knowledge.db remains a
//! human-grokable artifact during MIT review.

use crate::cache::cache_dir_for;
use garnet_parser::ast::*;
use rusqlite::{params, Connection};
use std::path::Path;

const KNOWLEDGE_DB: &str = "knowledge.db";

#[derive(Debug, Clone)]
pub struct ContextRow {
    pub id: i64,
    pub source_hash: String,
    pub fingerprint: [u8; 32],
    pub outcome: String,
    pub ts: i64,
}

/// Open (creating if missing) the knowledge DB under `<base>/.garnet-cache/`.
pub fn open(base: &Path) -> rusqlite::Result<Connection> {
    let dir = cache_dir_for(base);
    std::fs::create_dir_all(&dir).ok();
    let conn = Connection::open(dir.join(KNOWLEDGE_DB))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS compilation_contexts (
            id INTEGER PRIMARY KEY,
            source_hash TEXT NOT NULL,
            ast_fingerprint BLOB NOT NULL,
            outcome TEXT NOT NULL,
            ts INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_outcome
            ON compilation_contexts(outcome);
        CREATE INDEX IF NOT EXISTS idx_source_hash
            ON compilation_contexts(source_hash);",
    )?;
    Ok(conn)
}

/// Insert a new compilation context. `outcome` is the same string the
/// episode log uses ("ok" / "parse_err" / "check_err" / "runtime_err").
pub fn record_context(
    conn: &Connection,
    source_hash: &str,
    fingerprint: &[u8; 32],
    outcome: &str,
    ts: i64,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO compilation_contexts (source_hash, ast_fingerprint, outcome, ts)
         VALUES (?1, ?2, ?3, ?4)",
        params![source_hash, &fingerprint[..], outcome, ts],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Top-k Hamming-similar contexts for `target` (smallest distance first).
pub fn similar_contexts(
    conn: &Connection,
    target: &[u8; 32],
    k: usize,
) -> rusqlite::Result<Vec<(u32, ContextRow)>> {
    let mut stmt = conn.prepare(
        "SELECT id, source_hash, ast_fingerprint, outcome, ts FROM compilation_contexts",
    )?;
    let rows = stmt.query_map([], |r| {
        let fp_bytes: Vec<u8> = r.get(2)?;
        let mut fp = [0u8; 32];
        let len = fp_bytes.len().min(32);
        fp[..len].copy_from_slice(&fp_bytes[..len]);
        Ok(ContextRow {
            id: r.get(0)?,
            source_hash: r.get(1)?,
            fingerprint: fp,
            outcome: r.get(3)?,
            ts: r.get(4)?,
        })
    })?;
    let mut scored: Vec<(u32, ContextRow)> = Vec::new();
    for row in rows {
        let row = row?;
        let dist = hamming_distance(&row.fingerprint, target);
        scored.push((dist, row));
    }
    scored.sort_by_key(|(d, _)| *d);
    scored.truncate(k);
    Ok(scored)
}

pub fn count_contexts(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM compilation_contexts", [], |r| {
        r.get(0)
    })
}

// ── AST fingerprint extraction ──────────────────────────────────────

/// Compute a 256-bit deterministic fingerprint from the module's AST shape.
/// The fingerprint is BLAKE3 of a canonical "kind:count" census of every
/// node type, so two ASTs with the same shape produce the same bytes.
pub fn fingerprint(module: &Module) -> [u8; 32] {
    let mut counts: std::collections::BTreeMap<&'static str, u64> =
        std::collections::BTreeMap::new();
    counts.insert("Module", 1);
    counts.insert("safe", module.safe as u64);
    for item in &module.items {
        count_item(item, &mut counts);
    }
    let mut canonical = String::new();
    for (k, v) in &counts {
        canonical.push_str(k);
        canonical.push('=');
        canonical.push_str(&v.to_string());
        canonical.push(';');
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(canonical.as_bytes());
    let mut out = [0u8; 32];
    out.copy_from_slice(hasher.finalize().as_bytes());
    out
}

fn count_item(item: &Item, c: &mut std::collections::BTreeMap<&'static str, u64>) {
    match item {
        Item::Use(_) => *c.entry("Item::Use").or_default() += 1,
        Item::Module(m) => {
            *c.entry("Item::Module").or_default() += 1;
            for inner in &m.items {
                count_item(inner, c);
            }
        }
        Item::Memory(_) => *c.entry("Item::Memory").or_default() += 1,
        Item::Actor(a) => {
            *c.entry("Item::Actor").or_default() += 1;
            *c.entry("Actor.items").or_default() += a.items.len() as u64;
        }
        Item::Struct(s) => {
            *c.entry("Item::Struct").or_default() += 1;
            *c.entry("Struct.fields").or_default() += s.fields.len() as u64;
        }
        Item::Enum(e) => {
            *c.entry("Item::Enum").or_default() += 1;
            *c.entry("Enum.variants").or_default() += e.variants.len() as u64;
        }
        Item::Trait(_) => *c.entry("Item::Trait").or_default() += 1,
        Item::Impl(_) => *c.entry("Item::Impl").or_default() += 1,
        Item::Fn(f) => {
            *c.entry("Item::Fn").or_default() += 1;
            *c.entry(if matches!(f.mode, FnMode::Safe) {
                "Fn.safe"
            } else {
                "Fn.managed"
            })
            .or_default() += 1;
            *c.entry("Fn.params").or_default() += f.params.len() as u64;
            count_block(&f.body, c);
        }
        Item::Const(_) => *c.entry("Item::Const").or_default() += 1,
        Item::Let(_) => *c.entry("Item::Let").or_default() += 1,
    }
}

fn count_block(b: &Block, c: &mut std::collections::BTreeMap<&'static str, u64>) {
    *c.entry("Block").or_default() += 1;
    *c.entry("Block.stmts").or_default() += b.stmts.len() as u64;
    for s in &b.stmts {
        count_stmt(s, c);
    }
    if let Some(t) = &b.tail_expr {
        count_expr(t, c);
    }
}

fn count_stmt(s: &Stmt, c: &mut std::collections::BTreeMap<&'static str, u64>) {
    match s {
        Stmt::Let(_) => *c.entry("Stmt::Let").or_default() += 1,
        Stmt::Var(_) => *c.entry("Stmt::Var").or_default() += 1,
        Stmt::Const(_) => *c.entry("Stmt::Const").or_default() += 1,
        Stmt::Assign { .. } => *c.entry("Stmt::Assign").or_default() += 1,
        Stmt::While { body, .. } | Stmt::For { body, .. } | Stmt::Loop { body, .. } => {
            *c.entry("Stmt::Loop*").or_default() += 1;
            count_block(body, c);
        }
        Stmt::Break { .. } => *c.entry("Stmt::Break").or_default() += 1,
        Stmt::Continue { .. } => *c.entry("Stmt::Continue").or_default() += 1,
        Stmt::Return { .. } => *c.entry("Stmt::Return").or_default() += 1,
        Stmt::Raise { .. } => *c.entry("Stmt::Raise").or_default() += 1,
        Stmt::Expr(e) => count_expr(e, c),
    }
}

fn count_expr(e: &Expr, c: &mut std::collections::BTreeMap<&'static str, u64>) {
    match e {
        Expr::Int(_, _) => *c.entry("Expr::Int").or_default() += 1,
        Expr::Float(_, _) => *c.entry("Expr::Float").or_default() += 1,
        Expr::Bool(_, _) => *c.entry("Expr::Bool").or_default() += 1,
        Expr::Nil(_) => *c.entry("Expr::Nil").or_default() += 1,
        Expr::Str(_, _) => *c.entry("Expr::Str").or_default() += 1,
        Expr::Symbol(_, _) => *c.entry("Expr::Symbol").or_default() += 1,
        Expr::Ident(_, _) => *c.entry("Expr::Ident").or_default() += 1,
        Expr::Path(_, _) => *c.entry("Expr::Path").or_default() += 1,
        Expr::Binary { lhs, rhs, .. } => {
            *c.entry("Expr::Binary").or_default() += 1;
            count_expr(lhs, c);
            count_expr(rhs, c);
        }
        Expr::Unary { expr, .. } => {
            *c.entry("Expr::Unary").or_default() += 1;
            count_expr(expr, c);
        }
        Expr::Call { args, .. } => {
            *c.entry("Expr::Call").or_default() += 1;
            *c.entry("Call.args").or_default() += args.len() as u64;
        }
        Expr::Method { args, .. } => {
            *c.entry("Expr::Method").or_default() += 1;
            *c.entry("Method.args").or_default() += args.len() as u64;
        }
        Expr::Field { .. } => *c.entry("Expr::Field").or_default() += 1,
        Expr::Index { .. } => *c.entry("Expr::Index").or_default() += 1,
        Expr::If { .. } => *c.entry("Expr::If").or_default() += 1,
        Expr::Match { arms, .. } => {
            *c.entry("Expr::Match").or_default() += 1;
            *c.entry("Match.arms").or_default() += arms.len() as u64;
        }
        Expr::Try { .. } => *c.entry("Expr::Try").or_default() += 1,
        Expr::Closure { .. } => *c.entry("Expr::Closure").or_default() += 1,
        Expr::Spawn { .. } => *c.entry("Expr::Spawn").or_default() += 1,
        Expr::Array { elements, .. } => {
            *c.entry("Expr::Array").or_default() += 1;
            *c.entry("Array.len").or_default() += elements.len() as u64;
        }
        Expr::Map { entries, .. } => {
            *c.entry("Expr::Map").or_default() += 1;
            *c.entry("Map.len").or_default() += entries.len() as u64;
        }
    }
}

fn hamming_distance(a: &[u8; 32], b: &[u8; 32]) -> u32 {
    let mut d = 0u32;
    for i in 0..32 {
        d += (a[i] ^ b[i]).count_ones();
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use garnet_parser::parse_source;

    #[test]
    fn fingerprint_is_deterministic() {
        let m = parse_source("def main() { 1 + 2 }").unwrap();
        let f1 = fingerprint(&m);
        let f2 = fingerprint(&m);
        assert_eq!(f1, f2);
    }

    #[test]
    fn different_shapes_produce_different_fingerprints() {
        let a = parse_source("def main() { 1 }").unwrap();
        let b = parse_source("def main() { [1, 2, 3].map(|x| x * 2) }").unwrap();
        assert_ne!(fingerprint(&a), fingerprint(&b));
    }
}
