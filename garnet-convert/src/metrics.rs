//! Conversion metrics — JSON-serializable summary shipped alongside
//! each converted file. Aggregated metrics feed Paper VI §C1 and Paper
//! III §7.

use crate::cir::Cir;
use blake3::Hasher;

#[derive(Debug, Clone)]
pub struct ConvertMetrics {
    pub source_lang: String,
    pub source_file: String,
    pub target_file: String,
    pub source_loc: usize,
    pub target_loc: usize,
    pub migrate_todo_count: usize,
    pub untranslatable_count: usize,
    pub sandbox_status: SandboxStatus,
    pub converter_version: &'static str,
    pub witness_hash: String,
    pub total_cir_nodes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxStatus {
    Quarantined,
    Unquarantined, // only set by manual user edit; never by converter
}

impl ConvertMetrics {
    /// Expressiveness ratio — target LOC / source LOC. Around 1.0 is
    /// natural for converter output since it isn't optimized for
    /// minimalism; humans polishing typically drop 10-20% further.
    pub fn expressiveness_ratio(&self) -> f64 {
        if self.source_loc == 0 {
            return 1.0;
        }
        self.target_loc as f64 / self.source_loc as f64
    }

    /// Percentage of the conversion that produced clean Garnet
    /// (no MigrateTodo, no Untranslatable).
    pub fn clean_translation_percent(&self) -> f64 {
        if self.total_cir_nodes == 0 {
            return 100.0;
        }
        let problematic = self.migrate_todo_count + self.untranslatable_count;
        let clean = self.total_cir_nodes.saturating_sub(problematic);
        (clean as f64 / self.total_cir_nodes as f64) * 100.0
    }

    /// Render as a single JSON object (no serde dep).
    pub fn to_json(&self) -> String {
        format!(
            r#"{{
  "source_lang": "{}",
  "source_file": "{}",
  "target_file": "{}",
  "source_loc": {},
  "target_loc": {},
  "expressiveness_ratio": {:.4},
  "clean_translation_percent": {:.2},
  "migrate_todo_count": {},
  "untranslatable_count": {},
  "total_cir_nodes": {},
  "sandbox_status": "{}",
  "converter_version": "{}",
  "witness_hash": "{}"
}}"#,
            escape(&self.source_lang),
            escape(&self.source_file),
            escape(&self.target_file),
            self.source_loc,
            self.target_loc,
            self.expressiveness_ratio(),
            self.clean_translation_percent(),
            self.migrate_todo_count,
            self.untranslatable_count,
            self.total_cir_nodes,
            match self.sandbox_status {
                SandboxStatus::Quarantined => "quarantined",
                SandboxStatus::Unquarantined => "unquarantined",
            },
            self.converter_version,
            self.witness_hash,
        )
    }

    /// Build metrics from a CIR root + file metadata.
    pub fn from_cir(
        source_lang: &str,
        source_file: &str,
        target_file: &str,
        source_loc: usize,
        target_source: &str,
        cir: &Cir,
    ) -> Self {
        let target_loc = target_source.lines().count();
        let witness_hash = compute_witness_hash(cir);
        Self {
            source_lang: source_lang.to_string(),
            source_file: source_file.to_string(),
            target_file: target_file.to_string(),
            source_loc,
            target_loc,
            migrate_todo_count: cir.migrate_todo_count(),
            untranslatable_count: cir.untranslatable_count(),
            total_cir_nodes: cir.node_count(),
            sandbox_status: SandboxStatus::Quarantined,
            converter_version: env!("CARGO_PKG_VERSION"),
            witness_hash,
        }
    }
}

fn compute_witness_hash(cir: &Cir) -> String {
    let mut h = Hasher::new();
    h.update(b"garnet-convert-witness-v1");
    hash_cir(&mut h, cir);
    let bytes = *h.finalize().as_bytes();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hash_cir(h: &mut Hasher, cir: &Cir) {
    let kind = discriminant_name(cir);
    h.update(kind.as_bytes());
    let lin = cir.lineage();
    h.update(lin.source_lang.as_bytes());
    h.update(lin.source_file.as_bytes());
    h.update(&(lin.source_span.0 as u64).to_le_bytes());
    h.update(&(lin.source_span.1 as u64).to_le_bytes());
}

fn discriminant_name(cir: &Cir) -> &'static str {
    match cir {
        Cir::Module { .. } => "Module",
        Cir::Func { .. } => "Func",
        Cir::If { .. } => "If",
        Cir::While { .. } => "While",
        Cir::For { .. } => "For",
        Cir::Match { .. } => "Match",
        Cir::Return { .. } => "Return",
        Cir::Try { .. } => "Try",
        Cir::Let { .. } => "Let",
        Cir::Literal(..) => "Literal",
        Cir::Ident(..) => "Ident",
        Cir::Call { .. } => "Call",
        Cir::MethodCall { .. } => "MethodCall",
        Cir::FieldAccess { .. } => "FieldAccess",
        Cir::BinOp { .. } => "BinOp",
        Cir::UnOp { .. } => "UnOp",
        Cir::Assign { .. } => "Assign",
        Cir::Lambda { .. } => "Lambda",
        Cir::Struct { .. } => "Struct",
        Cir::Enum { .. } => "Enum",
        Cir::Impl { .. } => "Impl",
        Cir::ArrayLit(..) => "ArrayLit",
        Cir::MapLit(..) => "MapLit",
        Cir::Index { .. } => "Index",
        Cir::TupleLit(..) => "TupleLit",
        Cir::PatLiteral(..) => "PatLiteral",
        Cir::PatIdent(..) => "PatIdent",
        Cir::PatTuple(..) => "PatTuple",
        Cir::PatEnumVariant { .. } => "PatEnumVariant",
        Cir::PatWildcard(..) => "PatWildcard",
        Cir::Untranslatable { .. } => "Untranslatable",
        Cir::MigrateTodo { .. } => "MigrateTodo",
    }
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cir::{CirLit, FuncMode};
    use crate::lineage::Lineage;

    fn lin() -> Lineage {
        Lineage::new("rust", "src/foo.rs", 0, 10)
    }

    #[test]
    fn expressiveness_ratio_basic() {
        let cir = Cir::Literal(CirLit::Int(1), lin());
        let m = ConvertMetrics::from_cir("rust", "src/foo.rs", "src/foo.garnet", 100, &"a\nb\nc\n".repeat(30), &cir);
        // 30 × 3 lines = 90 target, 100 source
        assert!(m.expressiveness_ratio() < 1.0);
    }

    #[test]
    fn clean_percent_is_100_when_no_todos() {
        let cir = Cir::Func {
            name: "f".into(),
            params: vec![],
            return_ty: crate::cir::CirTy::Inferred,
            body: vec![Cir::Literal(CirLit::Int(1), lin())],
            mode: FuncMode::Managed,
            caps: vec![],
            lineage: lin(),
        };
        let m = ConvertMetrics::from_cir("rust", "a.rs", "a.garnet", 10, "x", &cir);
        assert_eq!(m.clean_translation_percent(), 100.0);
    }

    #[test]
    fn clean_percent_drops_with_todos() {
        let cir = Cir::Func {
            name: "f".into(),
            params: vec![],
            return_ty: crate::cir::CirTy::Inferred,
            body: vec![
                Cir::Literal(CirLit::Int(1), lin()),
                Cir::MigrateTodo {
                    placeholder: Box::new(Cir::Literal(CirLit::Nil, lin())),
                    note: "n".into(),
                    lineage: lin(),
                },
            ],
            mode: FuncMode::Managed,
            caps: vec![],
            lineage: lin(),
        };
        let m = ConvertMetrics::from_cir("rust", "a.rs", "a.garnet", 10, "x", &cir);
        // Total: Func(1) + Literal(1) + MigrateTodo(1) + placeholder Literal(1) = 4 nodes
        // MigrateTodo count: 1
        // Clean: 75%
        assert_eq!(m.total_cir_nodes, 4);
        assert_eq!(m.migrate_todo_count, 1);
        assert_eq!(m.clean_translation_percent(), 75.0);
    }

    #[test]
    fn json_is_well_formed_single_object() {
        let cir = Cir::Literal(CirLit::Nil, lin());
        let m = ConvertMetrics::from_cir("rust", "a.rs", "a.garnet", 10, "x", &cir);
        let j = m.to_json();
        assert!(j.starts_with('{'));
        assert!(j.trim_end().ends_with('}'));
        assert!(j.contains("\"source_lang\": \"rust\""));
        assert!(j.contains("\"sandbox_status\": \"quarantined\""));
    }

    #[test]
    fn sandbox_status_default_quarantined() {
        let cir = Cir::Literal(CirLit::Nil, lin());
        let m = ConvertMetrics::from_cir("rust", "a.rs", "a.garnet", 10, "x", &cir);
        assert_eq!(m.sandbox_status, SandboxStatus::Quarantined);
    }

    #[test]
    fn witness_hash_is_deterministic() {
        let cir = Cir::Literal(CirLit::Int(42), lin());
        let m1 = ConvertMetrics::from_cir("rust", "a.rs", "a.garnet", 10, "x", &cir);
        let m2 = ConvertMetrics::from_cir("rust", "a.rs", "a.garnet", 10, "x", &cir);
        assert_eq!(m1.witness_hash, m2.witness_hash);
        assert_eq!(m1.witness_hash.len(), 64);
    }

    #[test]
    fn witness_hash_changes_with_cir_change() {
        let cir1 = Cir::Literal(CirLit::Int(42), lin());
        let cir2 = Cir::Literal(CirLit::Int(43), Lineage::new("rust", "src/foo.rs", 5, 15));
        let m1 = ConvertMetrics::from_cir("rust", "a.rs", "a.garnet", 10, "x", &cir1);
        let m2 = ConvertMetrics::from_cir("rust", "a.rs", "a.garnet", 10, "x", &cir2);
        assert_ne!(m1.witness_hash, m2.witness_hash);
    }
}
