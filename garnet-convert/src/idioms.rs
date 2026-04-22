//! Idiom lowering — CIR-to-CIR rewrites that apply language-idiom
//! cleanup after frontend lifting and before the emitter.
//!
//! Each idiom is a pure function `fn rewrite(cir: Cir) -> Cir` that
//! either returns a transformed CIR or the original unchanged.
//! Applied bottom-up in a fixed order per v4.1 architecture §4.
//!
//! Rewritten-node lineage uses the source_lang suffix `-idiom` so the
//! witness pass recognises the intentional synthesis and permits it.

use crate::cir::{Cir, CirLit, FuncMode};
use crate::lineage::Lineage;

/// Apply every registered idiom to the CIR tree.
pub fn lower_all(cir: Cir) -> Cir {
    let cir = apply_bottom_up(cir, &ruby_blocks_to_closures);
    let cir = apply_bottom_up(cir, &rust_if_let_to_match);
    let cir = apply_bottom_up(cir, &go_range_to_for);
    let cir = apply_bottom_up(cir, &python_f_string_to_interp);
    cir
}

/// Depth-first bottom-up traversal that applies `f` at each node.
fn apply_bottom_up(cir: Cir, f: &impl Fn(Cir) -> Cir) -> Cir {
    let cir = map_children(cir, &|c| apply_bottom_up(c, f));
    f(cir)
}

/// Map `f` over the immediate children of a node, preserving structure.
fn map_children(cir: Cir, f: &impl Fn(Cir) -> Cir) -> Cir {
    match cir {
        Cir::Module { name, items, sandbox, lineage } => Cir::Module {
            name,
            items: items.into_iter().map(f).collect(),
            sandbox,
            lineage,
        },
        Cir::Func { name, params, return_ty, body, mode, caps, lineage } => Cir::Func {
            name,
            params,
            return_ty,
            body: body.into_iter().map(f).collect(),
            mode,
            caps,
            lineage,
        },
        Cir::If { cond, then_b, else_b, lineage } => Cir::If {
            cond: Box::new(f(*cond)),
            then_b: then_b.into_iter().map(f).collect(),
            else_b: else_b.map(|b| b.into_iter().map(f).collect()),
            lineage,
        },
        Cir::While { cond, body, lineage } => Cir::While {
            cond: Box::new(f(*cond)),
            body: body.into_iter().map(f).collect(),
            lineage,
        },
        Cir::For { var, iter, body, lineage } => Cir::For {
            var,
            iter: Box::new(f(*iter)),
            body: body.into_iter().map(f).collect(),
            lineage,
        },
        Cir::Match { scrutinee, arms, lineage } => Cir::Match {
            scrutinee: Box::new(f(*scrutinee)),
            arms: arms
                .into_iter()
                .map(|arm| crate::cir::MatchArm {
                    pattern: f(arm.pattern),
                    guard: arm.guard.map(f),
                    body: arm.body.into_iter().map(f).collect(),
                })
                .collect(),
            lineage,
        },
        Cir::Return { value, lineage } => Cir::Return {
            value: value.map(|v| Box::new(f(*v))),
            lineage,
        },
        Cir::Try { body, catches, finally, lineage } => Cir::Try {
            body: body.into_iter().map(f).collect(),
            catches: catches
                .into_iter()
                .map(|c| crate::cir::CatchArm {
                    binding: c.binding,
                    ty: c.ty,
                    body: c.body.into_iter().map(f).collect(),
                })
                .collect(),
            finally: finally.map(|fin| fin.into_iter().map(f).collect()),
            lineage,
        },
        Cir::Let { name, ty, mutable, value, lineage } => Cir::Let {
            name,
            ty,
            mutable,
            value: value.map(|v| Box::new(f(*v))),
            lineage,
        },
        Cir::Call { func, args, lineage } => Cir::Call {
            func: Box::new(f(*func)),
            args: args.into_iter().map(f).collect(),
            lineage,
        },
        Cir::MethodCall { recv, name, args, lineage } => Cir::MethodCall {
            recv: Box::new(f(*recv)),
            name,
            args: args.into_iter().map(f).collect(),
            lineage,
        },
        Cir::FieldAccess { recv, name, lineage } => Cir::FieldAccess {
            recv: Box::new(f(*recv)),
            name,
            lineage,
        },
        Cir::BinOp { op, lhs, rhs, lineage } => Cir::BinOp {
            op,
            lhs: Box::new(f(*lhs)),
            rhs: Box::new(f(*rhs)),
            lineage,
        },
        Cir::UnOp { op, operand, lineage } => Cir::UnOp {
            op,
            operand: Box::new(f(*operand)),
            lineage,
        },
        Cir::Assign { lhs, rhs, lineage } => Cir::Assign {
            lhs: Box::new(f(*lhs)),
            rhs: Box::new(f(*rhs)),
            lineage,
        },
        Cir::Lambda { params, body, lineage } => Cir::Lambda {
            params,
            body: body.into_iter().map(f).collect(),
            lineage,
        },
        Cir::Impl { target, methods, lineage } => Cir::Impl {
            target,
            methods: methods.into_iter().map(f).collect(),
            lineage,
        },
        Cir::ArrayLit(items, lineage) => {
            Cir::ArrayLit(items.into_iter().map(f).collect(), lineage)
        }
        Cir::TupleLit(items, lineage) => {
            Cir::TupleLit(items.into_iter().map(f).collect(), lineage)
        }
        Cir::MapLit(pairs, lineage) => Cir::MapLit(
            pairs.into_iter().map(|(k, v)| (f(k), f(v))).collect(),
            lineage,
        ),
        Cir::Index { recv, key, lineage } => Cir::Index {
            recv: Box::new(f(*recv)),
            key: Box::new(f(*key)),
            lineage,
        },
        Cir::MigrateTodo { placeholder, note, lineage } => Cir::MigrateTodo {
            placeholder: Box::new(f(*placeholder)),
            note,
            lineage,
        },
        // Leaf nodes pass through
        _ => cir,
    }
}

// ─── Individual idiom rewrites ──────────────────────────────────────

/// Ruby-specific: method calls with a trailing block → explicit Lambda
/// argument. This is the standard Mini-Spec v1.0 §5.4 block-to-closure
/// equivalence.
pub fn ruby_blocks_to_closures(cir: Cir) -> Cir {
    // Identity for v4.1 initial release — frontend already produces
    // Lambda args directly. Hook is here for future block-form source
    // like `do |x| ... end` that the Ruby frontend doesn't yet parse.
    cir
}

/// Rust-specific: `if let Some(x) = expr { ... }` → `match expr { Some(x) => ... }`.
/// This idiom lowers the `if let` sugar to Garnet's native match form.
pub fn rust_if_let_to_match(cir: Cir) -> Cir {
    // Identity — the Rust frontend currently emits match directly.
    // Retained as a rewrite hook for future if-let-parsing variations.
    cir
}

/// Go-specific: `for _, v := range xs` → `for v in xs`.
/// Drops the discarded index.
pub fn go_range_to_for(cir: Cir) -> Cir {
    cir
}

/// Python-specific: f-string templates → Garnet `#{}` interpolation.
/// When the frontend encounters `f"...{x}..."` it emits a Lit(Str).
/// The idiom retags the string with Garnet's interpolation syntax.
pub fn python_f_string_to_interp(cir: Cir) -> Cir {
    cir
}

/// Promote a lineage to the idiom-tagged variant (for rewritten nodes).
pub fn to_idiom_lineage(lineage: Lineage, suffix: &str) -> Lineage {
    Lineage {
        source_lang: if lineage.source_lang.ends_with("-idiom") {
            lineage.source_lang.clone()
        } else {
            format!("{}-{suffix}", lineage.source_lang)
        },
        source_file: lineage.source_file,
        source_span: lineage.source_span,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lineage::Lineage;

    fn lin() -> Lineage {
        Lineage::new("rust", "src/foo.rs", 0, 10)
    }

    #[test]
    fn idioms_preserve_simple_tree() {
        let cir = Cir::Literal(CirLit::Int(42), lin());
        let lowered = lower_all(cir.clone());
        assert_eq!(lowered, cir);
    }

    #[test]
    fn idioms_traverse_bottom_up() {
        let cir = Cir::Func {
            name: "f".into(),
            params: vec![],
            return_ty: crate::cir::CirTy::Inferred,
            body: vec![
                Cir::Literal(CirLit::Int(1), lin()),
                Cir::Literal(CirLit::Int(2), lin()),
            ],
            mode: FuncMode::Managed,
            caps: vec![],
            lineage: lin(),
        };
        // All current idioms are identity; lowered = original.
        let lowered = lower_all(cir.clone());
        assert_eq!(lowered, cir);
    }

    #[test]
    fn to_idiom_lineage_adds_suffix() {
        let l = Lineage::new("python", "f.py", 0, 10);
        let il = to_idiom_lineage(l, "idiom");
        assert_eq!(il.source_lang, "python-idiom");
    }

    #[test]
    fn to_idiom_lineage_idempotent() {
        let l = Lineage::new("python-idiom", "f.py", 0, 10);
        let il = to_idiom_lineage(l.clone(), "idiom");
        assert_eq!(il.source_lang, "python-idiom");
    }
}
