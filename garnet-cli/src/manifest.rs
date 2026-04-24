//! Deterministic build manifest — Paper VI Contribution 7.
//!
//! `garnet build --deterministic <file>` produces a manifest JSON sidecar
//! capturing every input that affects the compiled output: source-content
//! hash, stable AST hash, parser/interp versions, target triple, the
//! deterministic-flag set. `garnet verify <file> <manifest>` re-derives the
//! manifest from the source and exits non-zero on any mismatch.
//!
//! All hashes are BLAKE3. The AST hash is computed over a stable, sorted
//! pretty-print of the AST so that reordering token spans (which are
//! source-position artifacts, not semantic content) does not change the
//! hash.

use garnet_parser::ast::*;
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// On-disk manifest schema. Field order is fixed by `Manifest::serialize` —
/// we deliberately do NOT use `serde` here because `serde_json` does not
/// guarantee map iteration order across versions and we want byte-identical
/// output across runs and rust toolchain bumps.
///
/// `signer_pubkey` + `signature` are the v3.4.1 ManifestSig fields (Security
/// V2 §4). Both are hex-encoded; empty-string means "unsigned". The
/// signature covers the canonical JSON of every OTHER field; signing is
/// opt-in via `garnet build --deterministic --sign <keyfile>`, so older
/// tooling emits manifests with empty signer/signature and newer tooling
/// populates both when asked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub schema: String,
    pub source_hash: String,
    pub ast_hash: String,
    pub parser_version: String,
    pub interp_version: String,
    pub prelude_hash: String,
    pub target_triple: String,
    pub deterministic_flags: Vec<String>,
    /// Hex-encoded Ed25519 public key of the signer (32 bytes → 64 chars).
    /// Empty when the manifest is unsigned.
    pub signer_pubkey: String,
    /// Hex-encoded Ed25519 signature (64 bytes → 128 chars) over the
    /// canonical JSON of the manifest's OTHER fields. Empty when unsigned.
    pub signature: String,
}

impl Manifest {
    /// Schema identifier baked into every manifest. Bump when the schema
    /// changes; old verifiers reject manifests they don't recognise.
    pub const SCHEMA: &'static str = "garnet-manifest-v1";

    /// Build a manifest by hashing the given source + AST + environment.
    ///
    /// `prelude_hash` captures the actual prelude.rs contents via the
    /// `PRELUDE_SOURCE` constant exposed by `garnet-interp`, prepended with
    /// the `PRELUDE_VERSION` tag. Any edit to the prelude (adding a prim,
    /// changing an arity, renaming a builtin, or even changing a comment)
    /// changes the hash — so manifests cannot silently go stale when the
    /// prelude evolves. Fixes the v3.2 gap where `prelude_hash` was a hash
    /// of a hardcoded version string.
    pub fn build(source: &str, module: &Module) -> Manifest {
        Manifest {
            schema: Self::SCHEMA.to_string(),
            source_hash: hash_str(source),
            ast_hash: hash_str(&stable_ast_repr(module)),
            parser_version: env!("CARGO_PKG_VERSION").to_string(),
            interp_version: env!("CARGO_PKG_VERSION").to_string(),
            prelude_hash: hash_prelude(),
            target_triple: target_triple(),
            deterministic_flags: vec![
                "lto=on".to_string(),
                "codegen-units=1".to_string(),
                "strip=symbols".to_string(),
            ],
            signer_pubkey: String::new(),
            signature: String::new(),
        }
    }

    /// True iff this manifest carries a non-empty signer + signature pair.
    pub fn is_signed(&self) -> bool {
        !self.signer_pubkey.is_empty() && !self.signature.is_empty()
    }

    /// Canonical byte-string over which the Ed25519 signature is computed.
    /// This is deliberately the canonical JSON of every field EXCEPT
    /// `signer_pubkey` and `signature` — signing only ever covers the
    /// unsigned part, which gives us a stable invariant: a signer cannot
    /// accidentally sign their own signature.
    pub fn canonical_signing_payload(&self) -> String {
        let mut unsigned = self.clone();
        unsigned.signer_pubkey.clear();
        unsigned.signature.clear();
        unsigned.to_canonical_json_unsigned()
    }

    /// Sign this manifest in place using the provided Ed25519 signing key.
    /// Populates both `signer_pubkey` and `signature` with hex-encoded
    /// values; any prior signature is replaced. Returns the canonical JSON
    /// payload that was signed, for out-of-band verification.
    pub fn sign(&mut self, signing_key: &ed25519_dalek::SigningKey) -> String {
        use ed25519_dalek::Signer;
        let payload = self.canonical_signing_payload();
        let sig = signing_key.sign(payload.as_bytes());
        let verifying_key = signing_key.verifying_key();
        self.signer_pubkey = hex_encode(&verifying_key.to_bytes());
        self.signature = hex_encode(&sig.to_bytes());
        payload
    }

    /// Verify the manifest's embedded signature against its declared
    /// pubkey. Returns `Ok(())` iff the signature is valid; `Err(reason)`
    /// otherwise. An unsigned manifest fails verification with a specific
    /// "not signed" error — callers who accept unsigned manifests should
    /// check `is_signed()` first and skip `verify_signature()`.
    pub fn verify_signature(&self) -> Result<(), String> {
        if !self.is_signed() {
            return Err("manifest is not signed".to_string());
        }
        let pubkey_bytes: [u8; 32] =
            hex_decode_32(&self.signer_pubkey).map_err(|e| format!("bad signer_pubkey: {e}"))?;
        let sig_bytes: [u8; 64] =
            hex_decode_64(&self.signature).map_err(|e| format!("bad signature: {e}"))?;
        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&pubkey_bytes)
            .map_err(|e| format!("invalid Ed25519 pubkey: {e}"))?;
        let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);
        let payload = self.canonical_signing_payload();
        use ed25519_dalek::Verifier;
        verifying_key
            .verify(payload.as_bytes(), &signature)
            .map_err(|e| format!("signature verification failed: {e}"))
    }

    /// Internal: canonical JSON without signer_pubkey + signature fields.
    /// Used by `canonical_signing_payload`.
    fn to_canonical_json_unsigned(&self) -> String {
        let mut out = String::from("{\n");
        let mut entries: BTreeMap<&'static str, String> = BTreeMap::new();
        entries.insert("schema", quoted(&self.schema));
        entries.insert("source_hash", quoted(&self.source_hash));
        entries.insert("ast_hash", quoted(&self.ast_hash));
        entries.insert("parser_version", quoted(&self.parser_version));
        entries.insert("interp_version", quoted(&self.interp_version));
        entries.insert("prelude_hash", quoted(&self.prelude_hash));
        entries.insert("target_triple", quoted(&self.target_triple));
        entries.insert(
            "deterministic_flags",
            format!(
                "[{}]",
                self.deterministic_flags
                    .iter()
                    .map(|f| quoted(f))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
        let mut first = true;
        for (k, v) in &entries {
            if !first {
                out.push_str(",\n");
            }
            first = false;
            let _ = writeln!(out, "  {}: {}", quoted(k), v);
        }
        out.push_str("\n}\n");
        out
    }

    /// Serialise to canonical JSON. `BTreeMap`-backed key ordering plus
    /// deliberate field-by-field emission guarantees byte-identical output
    /// across runs. Includes `signer_pubkey` and `signature` fields (which
    /// carry empty strings for unsigned manifests — cf. `is_signed`).
    pub fn to_canonical_json(&self) -> String {
        let mut out = String::from("{\n");
        let mut entries: BTreeMap<&'static str, String> = BTreeMap::new();
        entries.insert("schema", quoted(&self.schema));
        entries.insert("source_hash", quoted(&self.source_hash));
        entries.insert("ast_hash", quoted(&self.ast_hash));
        entries.insert("parser_version", quoted(&self.parser_version));
        entries.insert("interp_version", quoted(&self.interp_version));
        entries.insert("prelude_hash", quoted(&self.prelude_hash));
        entries.insert("target_triple", quoted(&self.target_triple));
        entries.insert(
            "deterministic_flags",
            format!(
                "[{}]",
                self.deterministic_flags
                    .iter()
                    .map(|f| quoted(f))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
        entries.insert("signer_pubkey", quoted(&self.signer_pubkey));
        entries.insert("signature", quoted(&self.signature));
        let mut first = true;
        for (k, v) in &entries {
            if !first {
                out.push_str(",\n");
            }
            first = false;
            let _ = writeln!(out, "  {}: {}", quoted(k), v);
        }
        out.push_str("\n}\n");
        out
    }

    /// Parse a manifest JSON back into a struct. Lenient — looks for the
    /// expected keys and fails if any are missing. We do NOT use `serde_json`
    /// because the canonical form is small and we control both ends.
    pub fn from_canonical_json(json: &str) -> Result<Manifest, String> {
        let mut schema = None;
        let mut source_hash = None;
        let mut ast_hash = None;
        let mut parser_version = None;
        let mut interp_version = None;
        let mut prelude_hash = None;
        let mut target_triple = None;
        let mut deterministic_flags = Vec::new();
        let mut signer_pubkey = String::new();
        let mut signature = String::new();
        for line in json.lines() {
            let line = line.trim().trim_end_matches(',');
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().trim_matches('"');
                let value = value.trim();
                match key {
                    "schema" => schema = Some(unquote(value)),
                    "source_hash" => source_hash = Some(unquote(value)),
                    "ast_hash" => ast_hash = Some(unquote(value)),
                    "parser_version" => parser_version = Some(unquote(value)),
                    "interp_version" => interp_version = Some(unquote(value)),
                    "prelude_hash" => prelude_hash = Some(unquote(value)),
                    "target_triple" => target_triple = Some(unquote(value)),
                    "signer_pubkey" => signer_pubkey = unquote(value),
                    "signature" => signature = unquote(value),
                    "deterministic_flags" => {
                        let inner = value.trim_start_matches('[').trim_end_matches(']');
                        deterministic_flags = inner
                            .split(',')
                            .filter_map(|p| {
                                let p = p.trim();
                                if p.is_empty() {
                                    None
                                } else {
                                    Some(unquote(p))
                                }
                            })
                            .collect();
                    }
                    _ => {}
                }
            }
        }
        Ok(Manifest {
            schema: schema.ok_or("missing schema")?,
            source_hash: source_hash.ok_or("missing source_hash")?,
            ast_hash: ast_hash.ok_or("missing ast_hash")?,
            parser_version: parser_version.ok_or("missing parser_version")?,
            interp_version: interp_version.ok_or("missing interp_version")?,
            prelude_hash: prelude_hash.ok_or("missing prelude_hash")?,
            target_triple: target_triple.ok_or("missing target_triple")?,
            deterministic_flags,
            signer_pubkey,
            signature,
        })
    }
}

// ── Ed25519 signing key persistence helpers ─────────────────────────────

/// Generate a fresh Ed25519 signing key using the OS RNG. Returns
/// `(signing_key, verifying_key_hex)` — the verifying-key hex is the value
/// that downstream consumers need to recognize the signer (it gets written
/// into the manifest's `signer_pubkey` field when the key is used to sign).
pub fn generate_signing_key() -> (ed25519_dalek::SigningKey, String) {
    use rand_core::OsRng;
    let signing_key = ed25519_dalek::SigningKey::generate(&mut OsRng);
    let pubkey_hex = hex_encode(&signing_key.verifying_key().to_bytes());
    (signing_key, pubkey_hex)
}

/// Serialize a signing key to a stable ASCII-hex form suitable for writing
/// to a keyfile on disk. The file ships 64 hex chars (32 bytes) + a newline.
/// File permissions are the caller's responsibility — the CLI should chmod
/// 0600 the keyfile on UNIX and set the equivalent ACL on Windows.
pub fn signing_key_to_hex(key: &ed25519_dalek::SigningKey) -> String {
    hex_encode(&key.to_bytes())
}

/// Parse a signing key from its hex form. Rejects keys of wrong length with
/// a descriptive error (64 hex chars = 32 bytes required).
pub fn signing_key_from_hex(hex: &str) -> Result<ed25519_dalek::SigningKey, String> {
    let bytes = hex_decode_32(hex.trim())?;
    Ok(ed25519_dalek::SigningKey::from_bytes(&bytes))
}

// ── Hex encode / decode ───────────────────────────────────────────────

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(out, "{b:02x}");
    }
    out
}

fn hex_decode_32(hex: &str) -> Result<[u8; 32], String> {
    let v = hex_decode(hex)?;
    if v.len() != 32 {
        return Err(format!("expected 32 bytes (64 hex chars), got {}", v.len()));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&v);
    Ok(arr)
}

fn hex_decode_64(hex: &str) -> Result<[u8; 64], String> {
    let v = hex_decode(hex)?;
    if v.len() != 64 {
        return Err(format!(
            "expected 64 bytes (128 hex chars), got {}",
            v.len()
        ));
    }
    let mut arr = [0u8; 64];
    arr.copy_from_slice(&v);
    Ok(arr)
}

fn hex_decode(hex: &str) -> Result<Vec<u8>, String> {
    if !hex.len().is_multiple_of(2) {
        return Err(format!("hex length must be even, got {}", hex.len()));
    }
    let mut out = Vec::with_capacity(hex.len() / 2);
    let chars: Vec<char> = hex.chars().collect();
    for pair in chars.chunks(2) {
        let hi = hex_nibble(pair[0])?;
        let lo = hex_nibble(pair[1])?;
        out.push((hi << 4) | lo);
    }
    Ok(out)
}

fn hex_nibble(c: char) -> Result<u8, String> {
    match c {
        '0'..='9' => Ok(c as u8 - b'0'),
        'a'..='f' => Ok(c as u8 - b'a' + 10),
        'A'..='F' => Ok(c as u8 - b'A' + 10),
        _ => Err(format!("non-hex character '{c}'")),
    }
}

fn quoted(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t");
    format!("\"{escaped}\"")
}

fn unquote(s: &str) -> String {
    let s = s.trim_matches('"');
    s.replace("\\\\", "\\")
        .replace("\\\"", "\"")
        .replace("\\n", "\n")
        .replace("\\t", "\t")
}

fn hash_str(s: &str) -> String {
    blake3::hash(s.as_bytes()).to_hex().to_string()
}

/// Hash the prelude's actual content (prepended with the version tag for
/// change-isolation between semantic-contract bumps and content edits).
/// Reads from `garnet_interp::PRELUDE_SOURCE` which is populated via
/// `include_str!` at compile time, so the hash is deterministic and
/// reflects any change to the prelude source file.
fn hash_prelude() -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(garnet_interp::PRELUDE_VERSION.as_bytes());
    hasher.update(b"\n");
    hasher.update(garnet_interp::PRELUDE_SOURCE.as_bytes());
    hasher.finalize().to_hex().to_string()
}

fn target_triple() -> String {
    // Compile-time target triple, baked in via the `RUSTC_TARGET` env we
    // set up during build. Falls back to a portable identifier if not
    // available so the manifest is still deterministic.
    option_env!("TARGET")
        .unwrap_or("unknown-target")
        .to_string()
}

// ── Stable AST → string projection ──────────────────────────────────

/// Produce a canonical, deterministic string projection of a Module that
/// excludes `Span` information (which is source-position only). Two
/// programs that parse to AST-equivalent shapes produce identical strings.
pub fn stable_ast_repr(module: &Module) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Module(safe={})", module.safe);
    for item in &module.items {
        write_item(&mut out, item, 0);
    }
    out
}

fn indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

fn write_item(out: &mut String, item: &Item, depth: usize) {
    indent(out, depth);
    match item {
        Item::Use(decl) => {
            let _ = writeln!(out, "Use({})", decl.path.join("::"));
        }
        Item::Module(m) => {
            let _ = writeln!(out, "Module({}, safe={})", m.name, m.safe);
            for inner in &m.items {
                write_item(out, inner, depth + 1);
            }
        }
        Item::Memory(m) => {
            let _ = writeln!(out, "Memory({:?}, {})", m.kind, m.name);
        }
        Item::Actor(a) => {
            let _ = writeln!(out, "Actor({}, items={})", a.name, a.items.len());
        }
        Item::Struct(s) => {
            let _ = writeln!(out, "Struct({}, fields={})", s.name, s.fields.len());
            for fd in &s.fields {
                indent(out, depth + 1);
                let _ = writeln!(out, "Field({})", fd.name);
            }
        }
        Item::Enum(e) => {
            let _ = writeln!(out, "Enum({}, variants={})", e.name, e.variants.len());
            for v in &e.variants {
                indent(out, depth + 1);
                let _ = writeln!(out, "Variant({}, fields={})", v.name, v.fields.len());
            }
        }
        Item::Trait(t) => {
            let _ = writeln!(out, "Trait({}, items={})", t.name, t.items.len());
        }
        Item::Impl(i) => {
            let _ = writeln!(out, "Impl(methods={})", i.methods.len());
        }
        Item::Fn(f) => {
            let _ = writeln!(
                out,
                "Fn({}, mode={:?}, params={}, has_ret={})",
                f.name,
                f.mode,
                f.params.len(),
                f.return_ty.is_some()
            );
            write_block(out, &f.body, depth + 1);
        }
        Item::Const(c) => {
            let _ = writeln!(out, "Const({}, public={})", c.name, c.public);
        }
        Item::Let(l) => {
            let _ = writeln!(out, "Let({}, mut={})", l.name, l.mutable);
            indent(out, depth + 1);
            write_expr(out, &l.value);
            out.push('\n');
        }
    }
}

fn write_block(out: &mut String, block: &Block, depth: usize) {
    indent(out, depth);
    let _ = writeln!(
        out,
        "Block(stmts={}, has_tail={})",
        block.stmts.len(),
        block.tail_expr.is_some()
    );
    for s in &block.stmts {
        indent(out, depth + 1);
        write_stmt(out, s);
        out.push('\n');
    }
    if let Some(tail) = &block.tail_expr {
        indent(out, depth + 1);
        out.push_str("Tail: ");
        write_expr(out, tail);
        out.push('\n');
    }
}

fn write_stmt(out: &mut String, stmt: &Stmt) {
    match stmt {
        Stmt::Let(d) => {
            let _ = write!(out, "Let({}, mut={}, ", d.name, d.mutable);
            write_expr(out, &d.value);
            out.push(')');
        }
        Stmt::Var(d) => {
            let _ = write!(out, "Var({}, ", d.name);
            write_expr(out, &d.value);
            out.push(')');
        }
        Stmt::Const(d) => {
            let _ = write!(out, "Const({}, ", d.name);
            write_expr(out, &d.value);
            out.push(')');
        }
        Stmt::Assign { op, value, .. } => {
            let _ = write!(out, "Assign({:?}, ", op);
            write_expr(out, value);
            out.push(')');
        }
        Stmt::While { .. } => out.push_str("While(...)"),
        Stmt::For { var, .. } => {
            let _ = write!(out, "For({}, ...)", var);
        }
        Stmt::Loop { .. } => out.push_str("Loop(...)"),
        Stmt::Break { .. } => out.push_str("Break"),
        Stmt::Continue { .. } => out.push_str("Continue"),
        Stmt::Return { .. } => out.push_str("Return"),
        Stmt::Raise { value, .. } => {
            out.push_str("Raise(");
            write_expr(out, value);
            out.push(')');
        }
        Stmt::Expr(e) => {
            out.push_str("Expr(");
            write_expr(out, e);
            out.push(')');
        }
    }
}

fn write_expr(out: &mut String, expr: &Expr) {
    match expr {
        Expr::Int(v, _) => {
            let _ = write!(out, "Int({})", v);
        }
        Expr::Float(v, _) => {
            let _ = write!(out, "Float({})", v);
        }
        Expr::Bool(b, _) => {
            let _ = write!(out, "Bool({})", b);
        }
        Expr::Nil(_) => out.push_str("Nil"),
        Expr::Str(_, _) => out.push_str("Str(...)"),
        Expr::Symbol(s, _) => {
            let _ = write!(out, "Sym({})", s);
        }
        Expr::Ident(n, _) => {
            let _ = write!(out, "Id({})", n);
        }
        Expr::Path(segs, _) => {
            let _ = write!(out, "Path({})", segs.join("::"));
        }
        Expr::Binary { op, lhs, rhs, .. } => {
            let _ = write!(out, "Bin({:?}, ", op);
            write_expr(out, lhs);
            out.push_str(", ");
            write_expr(out, rhs);
            out.push(')');
        }
        Expr::Unary { op, expr, .. } => {
            let _ = write!(out, "Un({:?}, ", op);
            write_expr(out, expr);
            out.push(')');
        }
        Expr::Call { callee, args, .. } => {
            out.push_str("Call(");
            write_expr(out, callee);
            let _ = write!(out, ", argc={})", args.len());
        }
        Expr::Method {
            receiver,
            method,
            args,
            ..
        } => {
            out.push_str("Method(");
            write_expr(out, receiver);
            let _ = write!(out, ", .{}, argc={})", method, args.len());
        }
        Expr::Field {
            receiver, field, ..
        } => {
            out.push_str("Field(");
            write_expr(out, receiver);
            let _ = write!(out, ", .{})", field);
        }
        Expr::Index {
            receiver, index, ..
        } => {
            out.push_str("Index(");
            write_expr(out, receiver);
            out.push_str(", ");
            write_expr(out, index);
            out.push(')');
        }
        Expr::If { .. } => out.push_str("If(...)"),
        Expr::Match { arms, .. } => {
            let _ = write!(out, "Match(arms={})", arms.len());
        }
        Expr::Try { .. } => out.push_str("Try(...)"),
        Expr::Closure { params, .. } => {
            let _ = write!(out, "Closure(params={})", params.len());
        }
        Expr::Spawn { .. } => out.push_str("Spawn(...)"),
        Expr::Array { elements, .. } => {
            let _ = write!(out, "Array(len={})", elements.len());
        }
        Expr::Map { entries, .. } => {
            let _ = write!(out, "Map(len={})", entries.len());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use garnet_parser::parse_source;

    #[test]
    fn manifest_serialization_is_byte_stable() {
        let src = "def main() { 1 + 2 }";
        let m = parse_source(src).unwrap();
        let manifest = Manifest::build(src, &m);
        let json1 = manifest.to_canonical_json();
        let json2 = manifest.to_canonical_json();
        let json3 = Manifest::build(src, &m).to_canonical_json();
        assert_eq!(json1, json2);
        assert_eq!(json2, json3);
    }

    #[test]
    fn manifest_roundtrip_through_canonical_json() {
        let src = "def add(x, y) { x + y }";
        let m = parse_source(src).unwrap();
        let manifest = Manifest::build(src, &m);
        let json = manifest.to_canonical_json();
        let parsed = Manifest::from_canonical_json(&json).unwrap();
        assert_eq!(manifest, parsed);
    }

    #[test]
    fn different_sources_have_different_hashes() {
        let m1 = parse_source("def main() { 1 }").unwrap();
        let m2 = parse_source("def main() { 2 }").unwrap();
        let h1 = Manifest::build("def main() { 1 }", &m1);
        let h2 = Manifest::build("def main() { 2 }", &m2);
        assert_ne!(h1.source_hash, h2.source_hash);
        assert_ne!(h1.ast_hash, h2.ast_hash);
    }

    #[test]
    fn whitespace_change_changes_source_hash_but_not_ast() {
        let src1 = "def main() { 1 }";
        let src2 = "def main() {\n  1\n}";
        let m1 = parse_source(src1).unwrap();
        let m2 = parse_source(src2).unwrap();
        let mfa = Manifest::build(src1, &m1);
        let mfb = Manifest::build(src2, &m2);
        assert_ne!(
            mfa.source_hash, mfb.source_hash,
            "source hashes must differ"
        );
        assert_eq!(
            mfa.ast_hash, mfb.ast_hash,
            "AST hashes must match — same shape"
        );
    }

    #[test]
    fn prelude_hash_depends_on_actual_prelude_content_not_just_version() {
        // Regression: v3.2 manifest.rs computed prelude_hash as
        // hash_str("garnet-prelude-v0.3.2") — a literal version string —
        // which meant any change to the prelude without a version bump
        // would silently produce bit-stable manifests. v3.3 fixes this by
        // hashing the actual prelude source via include_str!.
        let only_version = hash_str(garnet_interp::PRELUDE_VERSION);
        let real_prelude = hash_prelude();
        assert_ne!(
            only_version, real_prelude,
            "prelude_hash must incorporate actual prelude content, not just the version tag"
        );
        // And it must match a fresh recomputation (determinism check).
        assert_eq!(
            real_prelude,
            hash_prelude(),
            "prelude hash must be stable across calls"
        );
    }

    #[test]
    fn prelude_hash_is_non_trivial_length() {
        // BLAKE3 hex output is 64 chars. Guards against a regression where
        // prelude_hash silently becomes empty or truncated.
        let m = parse_source("def main() { 1 }").unwrap();
        let mf = Manifest::build("def main() { 1 }", &m);
        assert_eq!(mf.prelude_hash.len(), 64, "BLAKE3 hex is 64 chars");
        assert!(
            mf.prelude_hash.chars().all(|c| c.is_ascii_hexdigit()),
            "prelude_hash must be hex"
        );
    }

    // ── ManifestSig (v3.4.1 Day 3 — Ed25519 signed manifests) ──

    #[test]
    fn unsigned_manifest_reports_unsigned() {
        let src = "def main() { 1 }";
        let m = parse_source(src).unwrap();
        let mf = Manifest::build(src, &m);
        assert!(!mf.is_signed());
        assert!(mf.signer_pubkey.is_empty());
        assert!(mf.signature.is_empty());
    }

    #[test]
    fn unsigned_manifest_fails_signature_verification() {
        let src = "def main() { 1 }";
        let m = parse_source(src).unwrap();
        let mf = Manifest::build(src, &m);
        match mf.verify_signature() {
            Err(msg) => assert!(msg.contains("not signed")),
            Ok(_) => panic!("unsigned manifest should not verify"),
        }
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let src = "def main() { 1 + 2 }";
        let m = parse_source(src).unwrap();
        let mut mf = Manifest::build(src, &m);

        let (signing_key, pubkey_hex) = generate_signing_key();
        let payload = mf.sign(&signing_key);

        assert!(mf.is_signed());
        assert_eq!(mf.signer_pubkey, pubkey_hex);
        assert_eq!(
            mf.signature.len(),
            128,
            "Ed25519 signature should be 64 bytes = 128 hex chars"
        );
        assert!(mf.verify_signature().is_ok());
        // Payload returned by sign() matches the recomputed signing payload.
        assert_eq!(payload, mf.canonical_signing_payload());
    }

    #[test]
    fn signature_does_not_cover_itself() {
        // The signing payload must exclude `signer_pubkey` and `signature`,
        // otherwise signing would be self-referential. Check the invariant.
        let src = "def main() { 0 }";
        let m = parse_source(src).unwrap();
        let mut mf = Manifest::build(src, &m);
        let (signing_key, _) = generate_signing_key();
        let payload_before = mf.canonical_signing_payload();
        mf.sign(&signing_key);
        let payload_after = mf.canonical_signing_payload();
        assert_eq!(
            payload_before, payload_after,
            "signing must not change the signing payload"
        );
    }

    #[test]
    fn signature_rejects_tampered_source_hash() {
        let src = "def main() { 10 }";
        let m = parse_source(src).unwrap();
        let mut mf = Manifest::build(src, &m);
        let (signing_key, _) = generate_signing_key();
        mf.sign(&signing_key);
        assert!(mf.verify_signature().is_ok());

        // Tamper with source_hash.
        mf.source_hash = hash_str("// tampered content");
        match mf.verify_signature() {
            Err(msg) => assert!(
                msg.contains("signature verification failed"),
                "expected verify error, got {msg}"
            ),
            Ok(_) => panic!("tampered manifest should not verify"),
        }
    }

    #[test]
    fn signature_rejects_wrong_pubkey() {
        let src = "def main() { 0 }";
        let m = parse_source(src).unwrap();
        let mut mf = Manifest::build(src, &m);
        let (signing_key_a, _) = generate_signing_key();
        let (_signing_key_b, pubkey_b) = generate_signing_key();
        mf.sign(&signing_key_a);
        // Replace pubkey with a different key's pubkey — verification must fail.
        mf.signer_pubkey = pubkey_b;
        match mf.verify_signature() {
            Err(_) => {}
            Ok(_) => panic!("wrong pubkey should fail verification"),
        }
    }

    #[test]
    fn signed_manifest_survives_json_roundtrip() {
        let src = "def f(x, y) { x * y }";
        let m = parse_source(src).unwrap();
        let mut mf = Manifest::build(src, &m);
        let (signing_key, _) = generate_signing_key();
        mf.sign(&signing_key);
        assert!(mf.verify_signature().is_ok());

        let json = mf.to_canonical_json();
        let parsed = Manifest::from_canonical_json(&json).unwrap();
        assert_eq!(mf, parsed);
        assert!(parsed.verify_signature().is_ok());
    }

    #[test]
    fn unsigned_manifest_survives_json_roundtrip() {
        // The signer_pubkey and signature fields must round-trip as empty
        // strings for unsigned manifests.
        let src = "def main() { 7 }";
        let m = parse_source(src).unwrap();
        let mf = Manifest::build(src, &m);
        let json = mf.to_canonical_json();
        let parsed = Manifest::from_canonical_json(&json).unwrap();
        assert_eq!(mf, parsed);
        assert!(!parsed.is_signed());
    }

    #[test]
    fn signing_key_hex_roundtrip() {
        let (key_a, pubkey_hex_a) = generate_signing_key();
        let hex = signing_key_to_hex(&key_a);
        let key_b = signing_key_from_hex(&hex).unwrap();
        // Verifying keys must match after roundtrip.
        let pubkey_hex_b = hex_encode(&key_b.verifying_key().to_bytes());
        assert_eq!(pubkey_hex_a, pubkey_hex_b);
    }

    #[test]
    fn signing_key_from_hex_rejects_bad_length() {
        match signing_key_from_hex("deadbeef") {
            Err(msg) => assert!(msg.contains("expected 32 bytes"), "got {msg}"),
            Ok(_) => panic!("bad-length hex must be rejected"),
        }
    }

    #[test]
    fn signing_key_from_hex_rejects_non_hex() {
        let bad = "g".repeat(64); // 'g' is not a hex digit
        match signing_key_from_hex(&bad) {
            Err(msg) => assert!(
                msg.contains("non-hex") || msg.contains("expected"),
                "got {msg}"
            ),
            Ok(_) => panic!("non-hex must be rejected"),
        }
    }

    #[test]
    fn two_signers_on_same_manifest_differ_in_pubkey_and_signature() {
        let src = "def main() { 42 }";
        let m = parse_source(src).unwrap();
        let mut mf_a = Manifest::build(src, &m);
        let mut mf_b = Manifest::build(src, &m);
        let (key_a, _) = generate_signing_key();
        let (key_b, _) = generate_signing_key();
        mf_a.sign(&key_a);
        mf_b.sign(&key_b);
        assert_ne!(mf_a.signer_pubkey, mf_b.signer_pubkey);
        assert_ne!(mf_a.signature, mf_b.signature);
        assert!(mf_a.verify_signature().is_ok());
        assert!(mf_b.verify_signature().is_ok());
    }
}
