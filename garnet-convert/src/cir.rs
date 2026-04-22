//! Common IR (CIR) — language-independent AST shared across all frontends.
//!
//! Per v4.1 converter architecture §2. Explicitly lossy at structural
//! level; non-lossy at intent level via `Untranslatable` / `MigrateTodo`
//! escape variants.

use crate::lineage::Lineage;

/// Function mode carried by CIR. Frontends set this per source-language
/// rules; emitter maps to `def` (Managed) or `fn` (Safe).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuncMode {
    Managed,
    Safe,
    Unspecified,
}

/// Type disclosure level (per Mini-Spec §11.1 progressive spectrum).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CirTy {
    Inferred,
    Concrete(String),
    Optional(Box<CirTy>),
    Array(Box<CirTy>),
    Map(Box<CirTy>, Box<CirTy>),
    Generic(String),
    Result(Box<CirTy>, Box<CirTy>),
}

/// Ownership annotation for safe-mode parameters (Mini-Spec §5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ownership {
    Borrowed,
    Owned,
    MovedIn,
    Default,
}

/// Function parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: CirTy,
    pub ownership: Ownership,
}

/// Struct field declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDecl {
    pub name: String,
    pub ty: CirTy,
    pub public: bool,
}

/// Enum variant declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantDecl {
    pub name: String,
    pub payload: Vec<CirTy>,
}

/// Match-arm pattern + body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    pub pattern: Cir,
    pub guard: Option<Cir>,
    pub body: Vec<Cir>,
}

/// try/rescue catch arm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatchArm {
    pub binding: Option<String>,
    pub ty: Option<CirTy>,
    pub body: Vec<Cir>,
}

/// CIR literal variants.
#[derive(Debug, Clone, PartialEq)]
pub enum CirLit {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Nil,
    Symbol(String),
}

impl Eq for CirLit {}

/// The Common IR node itself. Language-independent; every node
/// carries a [`Lineage`] back to its source location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cir {
    Module {
        name: String,
        items: Vec<Cir>,
        sandbox: bool,
        lineage: Lineage,
    },
    Func {
        name: String,
        params: Vec<Param>,
        return_ty: CirTy,
        body: Vec<Cir>,
        mode: FuncMode,
        caps: Vec<String>,
        lineage: Lineage,
    },
    If {
        cond: Box<Cir>,
        then_b: Vec<Cir>,
        else_b: Option<Vec<Cir>>,
        lineage: Lineage,
    },
    While {
        cond: Box<Cir>,
        body: Vec<Cir>,
        lineage: Lineage,
    },
    For {
        var: String,
        iter: Box<Cir>,
        body: Vec<Cir>,
        lineage: Lineage,
    },
    Match {
        scrutinee: Box<Cir>,
        arms: Vec<MatchArm>,
        lineage: Lineage,
    },
    Return {
        value: Option<Box<Cir>>,
        lineage: Lineage,
    },
    Try {
        body: Vec<Cir>,
        catches: Vec<CatchArm>,
        finally: Option<Vec<Cir>>,
        lineage: Lineage,
    },
    Let {
        name: String,
        ty: CirTy,
        mutable: bool,
        value: Option<Box<Cir>>,
        lineage: Lineage,
    },

    // Expressions
    Literal(CirLit, Lineage),
    Ident(String, Lineage),
    Call {
        func: Box<Cir>,
        args: Vec<Cir>,
        lineage: Lineage,
    },
    MethodCall {
        recv: Box<Cir>,
        name: String,
        args: Vec<Cir>,
        lineage: Lineage,
    },
    FieldAccess {
        recv: Box<Cir>,
        name: String,
        lineage: Lineage,
    },
    BinOp {
        op: String,
        lhs: Box<Cir>,
        rhs: Box<Cir>,
        lineage: Lineage,
    },
    UnOp {
        op: String,
        operand: Box<Cir>,
        lineage: Lineage,
    },
    Assign {
        lhs: Box<Cir>,
        rhs: Box<Cir>,
        lineage: Lineage,
    },
    Lambda {
        params: Vec<Param>,
        body: Vec<Cir>,
        lineage: Lineage,
    },

    // Structure
    Struct {
        name: String,
        fields: Vec<FieldDecl>,
        lineage: Lineage,
    },
    Enum {
        name: String,
        variants: Vec<VariantDecl>,
        lineage: Lineage,
    },
    Impl {
        target: String,
        methods: Vec<Cir>,
        lineage: Lineage,
    },

    // Collection literals
    ArrayLit(Vec<Cir>, Lineage),
    MapLit(Vec<(Cir, Cir)>, Lineage),
    Index {
        recv: Box<Cir>,
        key: Box<Cir>,
        lineage: Lineage,
    },
    TupleLit(Vec<Cir>, Lineage),

    // Pattern-match patterns
    PatLiteral(CirLit, Lineage),
    PatIdent(String, Lineage),
    PatTuple(Vec<Cir>, Lineage),
    PatEnumVariant {
        path: Vec<String>,
        payload: Vec<Cir>,
        lineage: Lineage,
    },
    PatWildcard(Lineage),

    // Migration-escape nodes — the honest-middle position per prior art
    Untranslatable {
        reason: String,
        lineage: Lineage,
    },
    MigrateTodo {
        placeholder: Box<Cir>,
        note: String,
        lineage: Lineage,
    },
}

impl Cir {
    /// Extract this node's lineage. Used by witness verification.
    pub fn lineage(&self) -> &Lineage {
        match self {
            Cir::Module { lineage, .. }
            | Cir::Func { lineage, .. }
            | Cir::If { lineage, .. }
            | Cir::While { lineage, .. }
            | Cir::For { lineage, .. }
            | Cir::Match { lineage, .. }
            | Cir::Return { lineage, .. }
            | Cir::Try { lineage, .. }
            | Cir::Let { lineage, .. }
            | Cir::Literal(_, lineage)
            | Cir::Ident(_, lineage)
            | Cir::Call { lineage, .. }
            | Cir::MethodCall { lineage, .. }
            | Cir::FieldAccess { lineage, .. }
            | Cir::BinOp { lineage, .. }
            | Cir::UnOp { lineage, .. }
            | Cir::Assign { lineage, .. }
            | Cir::Lambda { lineage, .. }
            | Cir::Struct { lineage, .. }
            | Cir::Enum { lineage, .. }
            | Cir::Impl { lineage, .. }
            | Cir::ArrayLit(_, lineage)
            | Cir::MapLit(_, lineage)
            | Cir::Index { lineage, .. }
            | Cir::TupleLit(_, lineage)
            | Cir::PatLiteral(_, lineage)
            | Cir::PatIdent(_, lineage)
            | Cir::PatTuple(_, lineage)
            | Cir::PatEnumVariant { lineage, .. }
            | Cir::PatWildcard(lineage)
            | Cir::Untranslatable { lineage, .. }
            | Cir::MigrateTodo { lineage, .. } => lineage,
        }
    }

    /// Count total nodes recursively (metrics).
    pub fn node_count(&self) -> usize {
        1 + match self {
            Cir::Module { items, .. } => items.iter().map(Cir::node_count).sum(),
            Cir::Func { body, .. } => body.iter().map(Cir::node_count).sum(),
            Cir::If { cond, then_b, else_b, .. } => {
                cond.node_count()
                    + then_b.iter().map(Cir::node_count).sum::<usize>()
                    + else_b
                        .as_ref()
                        .map(|b| b.iter().map(Cir::node_count).sum::<usize>())
                        .unwrap_or(0)
            }
            Cir::While { cond, body, .. } => {
                cond.node_count() + body.iter().map(Cir::node_count).sum::<usize>()
            }
            Cir::For { iter, body, .. } => {
                iter.node_count() + body.iter().map(Cir::node_count).sum::<usize>()
            }
            Cir::Match { scrutinee, arms, .. } => {
                scrutinee.node_count()
                    + arms
                        .iter()
                        .map(|a| {
                            a.pattern.node_count()
                                + a.body.iter().map(Cir::node_count).sum::<usize>()
                                + a.guard.as_ref().map(Cir::node_count).unwrap_or(0)
                        })
                        .sum::<usize>()
            }
            Cir::Return { value, .. } => value.as_ref().map(|v| v.node_count()).unwrap_or(0),
            Cir::Try { body, catches, finally, .. } => {
                body.iter().map(Cir::node_count).sum::<usize>()
                    + catches
                        .iter()
                        .map(|c| c.body.iter().map(Cir::node_count).sum::<usize>())
                        .sum::<usize>()
                    + finally
                        .as_ref()
                        .map(|f| f.iter().map(Cir::node_count).sum::<usize>())
                        .unwrap_or(0)
            }
            Cir::Let { value, .. } => value.as_ref().map(|v| v.node_count()).unwrap_or(0),
            Cir::Call { func, args, .. } => {
                func.node_count() + args.iter().map(Cir::node_count).sum::<usize>()
            }
            Cir::MethodCall { recv, args, .. } => {
                recv.node_count() + args.iter().map(Cir::node_count).sum::<usize>()
            }
            Cir::FieldAccess { recv, .. } => recv.node_count(),
            Cir::BinOp { lhs, rhs, .. } | Cir::Assign { lhs, rhs, .. } => {
                lhs.node_count() + rhs.node_count()
            }
            Cir::UnOp { operand, .. } => operand.node_count(),
            Cir::Lambda { body, .. } => body.iter().map(Cir::node_count).sum(),
            Cir::Impl { methods, .. } => methods.iter().map(Cir::node_count).sum(),
            Cir::ArrayLit(items, _) | Cir::TupleLit(items, _) | Cir::PatTuple(items, _) => {
                items.iter().map(Cir::node_count).sum()
            }
            Cir::MapLit(pairs, _) => pairs
                .iter()
                .map(|(k, v)| k.node_count() + v.node_count())
                .sum(),
            Cir::Index { recv, key, .. } => recv.node_count() + key.node_count(),
            Cir::PatEnumVariant { payload, .. } => payload.iter().map(Cir::node_count).sum(),
            Cir::MigrateTodo { placeholder, .. } => placeholder.node_count(),
            _ => 0,
        }
    }

    /// True if this node (or any descendant) is a MigrateTodo.
    pub fn has_migrate_todo(&self) -> bool {
        matches!(self, Cir::MigrateTodo { .. })
            || self.children().any(|c| c.has_migrate_todo())
    }

    /// True if this node (or any descendant) is Untranslatable.
    pub fn has_untranslatable(&self) -> bool {
        matches!(self, Cir::Untranslatable { .. })
            || self.children().any(|c| c.has_untranslatable())
    }

    /// Count MigrateTodo descendants.
    pub fn migrate_todo_count(&self) -> usize {
        let here = if matches!(self, Cir::MigrateTodo { .. }) { 1 } else { 0 };
        here + self.children().map(|c| c.migrate_todo_count()).sum::<usize>()
    }

    /// Count Untranslatable descendants.
    pub fn untranslatable_count(&self) -> usize {
        let here = if matches!(self, Cir::Untranslatable { .. }) { 1 } else { 0 };
        here + self.children().map(|c| c.untranslatable_count()).sum::<usize>()
    }

    /// Walk immediate children for traversal. (Box values borrowed.)
    fn children(&self) -> Box<dyn Iterator<Item = &Cir> + '_> {
        match self {
            Cir::Module { items, .. } => Box::new(items.iter()),
            Cir::Func { body, .. } => Box::new(body.iter()),
            Cir::If { cond, then_b, else_b, .. } => Box::new(
                std::iter::once(cond.as_ref())
                    .chain(then_b.iter())
                    .chain(else_b.iter().flat_map(|b| b.iter())),
            ),
            Cir::While { cond, body, .. } => {
                Box::new(std::iter::once(cond.as_ref()).chain(body.iter()))
            }
            Cir::For { iter, body, .. } => {
                Box::new(std::iter::once(iter.as_ref()).chain(body.iter()))
            }
            Cir::Match { scrutinee, arms, .. } => Box::new(
                std::iter::once(scrutinee.as_ref())
                    .chain(arms.iter().flat_map(|a| {
                        std::iter::once(&a.pattern).chain(a.body.iter())
                    })),
            ),
            Cir::Return { value, .. } => Box::new(value.as_deref().into_iter()),
            Cir::Try { body, catches, finally, .. } => Box::new(
                body.iter()
                    .chain(catches.iter().flat_map(|c| c.body.iter()))
                    .chain(finally.iter().flat_map(|f| f.iter())),
            ),
            Cir::Let { value, .. } => Box::new(value.as_deref().into_iter()),
            Cir::Call { func, args, .. } => {
                Box::new(std::iter::once(func.as_ref()).chain(args.iter()))
            }
            Cir::MethodCall { recv, args, .. } => {
                Box::new(std::iter::once(recv.as_ref()).chain(args.iter()))
            }
            Cir::FieldAccess { recv, .. } | Cir::UnOp { operand: recv, .. } => {
                Box::new(std::iter::once(recv.as_ref()))
            }
            Cir::BinOp { lhs, rhs, .. } | Cir::Assign { lhs, rhs, .. } => {
                Box::new(vec![lhs.as_ref(), rhs.as_ref()].into_iter())
            }
            Cir::Lambda { body, .. } => Box::new(body.iter()),
            Cir::Impl { methods, .. } => Box::new(methods.iter()),
            Cir::ArrayLit(items, _) | Cir::TupleLit(items, _) | Cir::PatTuple(items, _) => {
                Box::new(items.iter())
            }
            Cir::MapLit(pairs, _) => {
                Box::new(pairs.iter().flat_map(|(k, v)| std::iter::once(k).chain(std::iter::once(v))))
            }
            Cir::Index { recv, key, .. } => {
                Box::new(vec![recv.as_ref(), key.as_ref()].into_iter())
            }
            Cir::PatEnumVariant { payload, .. } => Box::new(payload.iter()),
            Cir::MigrateTodo { placeholder, .. } => Box::new(std::iter::once(placeholder.as_ref())),
            _ => Box::new(std::iter::empty()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lineage::Lineage;

    fn lin() -> Lineage {
        Lineage {
            source_lang: "test".into(),
            source_file: "test.rs".into(),
            source_span: (0, 0),
        }
    }

    #[test]
    fn literal_node_count_is_one() {
        let n = Cir::Literal(CirLit::Int(42), lin());
        assert_eq!(n.node_count(), 1);
    }

    #[test]
    fn func_counts_body() {
        let func = Cir::Func {
            name: "f".into(),
            params: vec![],
            return_ty: CirTy::Inferred,
            body: vec![
                Cir::Literal(CirLit::Int(1), lin()),
                Cir::Literal(CirLit::Int(2), lin()),
            ],
            mode: FuncMode::Managed,
            caps: vec![],
            lineage: lin(),
        };
        assert_eq!(func.node_count(), 3);
    }

    #[test]
    fn migrate_todo_detected() {
        let n = Cir::MigrateTodo {
            placeholder: Box::new(Cir::Literal(CirLit::Nil, lin())),
            note: "n".into(),
            lineage: lin(),
        };
        assert!(n.has_migrate_todo());
        assert_eq!(n.migrate_todo_count(), 1);
    }

    #[test]
    fn untranslatable_detected() {
        let n = Cir::Untranslatable {
            reason: "unsafe".into(),
            lineage: lin(),
        };
        assert!(n.has_untranslatable());
        assert_eq!(n.untranslatable_count(), 1);
    }

    #[test]
    fn migrate_todo_propagates_up_tree() {
        let func = Cir::Func {
            name: "f".into(),
            params: vec![],
            return_ty: CirTy::Inferred,
            body: vec![Cir::MigrateTodo {
                placeholder: Box::new(Cir::Literal(CirLit::Nil, lin())),
                note: "".into(),
                lineage: lin(),
            }],
            mode: FuncMode::Managed,
            caps: vec![],
            lineage: lin(),
        };
        assert!(func.has_migrate_todo());
    }

    #[test]
    fn lineage_extractable_from_literal() {
        let n = Cir::Literal(CirLit::Bool(true), lin());
        assert_eq!(n.lineage().source_lang, "test");
    }
}
