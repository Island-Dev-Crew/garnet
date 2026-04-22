//! Witness verification — walks the CIR tree and rejects any node
//! whose lineage is `Lineage::unknown()` (the hallucination-defense
//! rule from v4.1 converter architecture §5).

use crate::cir::Cir;
use crate::error::ConvertError;

/// Verify every node in the CIR has a real lineage (not
/// `Lineage::unknown()`). Returns `Ok(())` on success or the first
/// untagged node's path on failure.
pub fn verify(cir: &Cir) -> Result<(), ConvertError> {
    let mut index = 0;
    verify_recursive(cir, &mut index)
}

fn verify_recursive(cir: &Cir, index: &mut usize) -> Result<(), ConvertError> {
    *index += 1;
    let lin = cir.lineage();
    if lin.source_lang == "unknown" && lin.source_file.is_empty() {
        return Err(ConvertError::MissingLineage {
            node_kind: kind_name(cir).to_string(),
            at: *index,
        });
    }
    // Idiom-synthesized nodes are tagged with source_lang ending in "-idiom"
    // which is permitted; truly untagged nodes are rejected.
    match cir {
        Cir::Module { items, .. } => {
            for item in items {
                verify_recursive(item, index)?;
            }
        }
        Cir::Func { body, .. } => {
            for stmt in body {
                verify_recursive(stmt, index)?;
            }
        }
        Cir::If { cond, then_b, else_b, .. } => {
            verify_recursive(cond, index)?;
            for s in then_b {
                verify_recursive(s, index)?;
            }
            if let Some(b) = else_b {
                for s in b {
                    verify_recursive(s, index)?;
                }
            }
        }
        Cir::While { cond, body, .. } => {
            verify_recursive(cond, index)?;
            for s in body {
                verify_recursive(s, index)?;
            }
        }
        Cir::For { iter, body, .. } => {
            verify_recursive(iter, index)?;
            for s in body {
                verify_recursive(s, index)?;
            }
        }
        Cir::Match { scrutinee, arms, .. } => {
            verify_recursive(scrutinee, index)?;
            for arm in arms {
                verify_recursive(&arm.pattern, index)?;
                if let Some(g) = &arm.guard {
                    verify_recursive(g, index)?;
                }
                for s in &arm.body {
                    verify_recursive(s, index)?;
                }
            }
        }
        Cir::Return { value, .. } => {
            if let Some(v) = value {
                verify_recursive(v, index)?;
            }
        }
        Cir::Try { body, catches, finally, .. } => {
            for s in body {
                verify_recursive(s, index)?;
            }
            for c in catches {
                for s in &c.body {
                    verify_recursive(s, index)?;
                }
            }
            if let Some(f) = finally {
                for s in f {
                    verify_recursive(s, index)?;
                }
            }
        }
        Cir::Let { value, .. } => {
            if let Some(v) = value {
                verify_recursive(v, index)?;
            }
        }
        Cir::Call { func, args, .. } => {
            verify_recursive(func, index)?;
            for a in args {
                verify_recursive(a, index)?;
            }
        }
        Cir::MethodCall { recv, args, .. } => {
            verify_recursive(recv, index)?;
            for a in args {
                verify_recursive(a, index)?;
            }
        }
        Cir::FieldAccess { recv, .. } | Cir::UnOp { operand: recv, .. } => {
            verify_recursive(recv, index)?;
        }
        Cir::BinOp { lhs, rhs, .. } | Cir::Assign { lhs, rhs, .. } => {
            verify_recursive(lhs, index)?;
            verify_recursive(rhs, index)?;
        }
        Cir::Lambda { body, .. } => {
            for s in body {
                verify_recursive(s, index)?;
            }
        }
        Cir::Impl { methods, .. } => {
            for m in methods {
                verify_recursive(m, index)?;
            }
        }
        Cir::ArrayLit(items, _) | Cir::TupleLit(items, _) | Cir::PatTuple(items, _) => {
            for i in items {
                verify_recursive(i, index)?;
            }
        }
        Cir::MapLit(pairs, _) => {
            for (k, v) in pairs {
                verify_recursive(k, index)?;
                verify_recursive(v, index)?;
            }
        }
        Cir::Index { recv, key, .. } => {
            verify_recursive(recv, index)?;
            verify_recursive(key, index)?;
        }
        Cir::PatEnumVariant { payload, .. } => {
            for p in payload {
                verify_recursive(p, index)?;
            }
        }
        Cir::MigrateTodo { placeholder, .. } => {
            verify_recursive(placeholder, index)?;
        }
        _ => {}
    }
    Ok(())
}

fn kind_name(cir: &Cir) -> &'static str {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cir::{CirLit, FuncMode};
    use crate::lineage::Lineage;

    fn lin() -> Lineage {
        Lineage::new("rust", "a.rs", 0, 10)
    }

    #[test]
    fn tagged_tree_verifies() {
        let cir = Cir::Func {
            name: "f".into(),
            params: vec![],
            return_ty: crate::cir::CirTy::Inferred,
            body: vec![Cir::Literal(CirLit::Int(1), lin())],
            mode: FuncMode::Managed,
            caps: vec![],
            lineage: lin(),
        };
        assert!(verify(&cir).is_ok());
    }

    #[test]
    fn untagged_leaf_rejected() {
        let cir = Cir::Func {
            name: "f".into(),
            params: vec![],
            return_ty: crate::cir::CirTy::Inferred,
            body: vec![Cir::Literal(CirLit::Int(1), Lineage::unknown())],
            mode: FuncMode::Managed,
            caps: vec![],
            lineage: lin(),
        };
        match verify(&cir) {
            Err(ConvertError::MissingLineage { .. }) => {}
            other => panic!("expected MissingLineage, got {other:?}"),
        }
    }

    #[test]
    fn idiom_synthesized_nodes_permitted() {
        // source_lang "rust-idiom" is not "unknown", so should pass
        let idiom_lin = Lineage::new("rust-idiom", "a.rs", 0, 10);
        let cir = Cir::Func {
            name: "f".into(),
            params: vec![],
            return_ty: crate::cir::CirTy::Inferred,
            body: vec![Cir::Literal(CirLit::Int(1), idiom_lin)],
            mode: FuncMode::Managed,
            caps: vec![],
            lineage: lin(),
        };
        assert!(verify(&cir).is_ok());
    }
}
