//! Typed wrappers over `SyntaxNode` for the major productions.
//!
//! This is the ergonomic surface S16 (LSP precision) consumes: each wrapper is
//! a thin newtype around a `SyntaxNode`, and `cast` checks the node kind.
//! Accessors locate children by `SyntaxKind`. The set here is intentionally
//! small (the productions LSP symbol/rename features touch first); more
//! wrappers can be added additively without changing the surface.

use crate::syntax_kind::{SyntaxKind, SyntaxNode};
use crate::CstNode;

/// Cast + small conveniences shared by every typed node.
pub trait CstNodeExt: CstNode + Sized {
    /// The `SyntaxKind` this wrapper represents.
    const KIND: SyntaxKind;

    /// Wrap `node` iff its kind is `KIND`.
    fn cast(node: SyntaxNode) -> Option<Self>;

    /// The node's full source text (trivia included).
    fn source_text(&self) -> String {
        self.syntax().text().to_string()
    }
}

macro_rules! typed_node {
    ($(#[$m:meta])* $name:ident => $kind:ident) => {
        $(#[$m])*
        #[derive(Debug, Clone)]
        pub struct $name(SyntaxNode);

        impl CstNode for $name {
            fn syntax(&self) -> &SyntaxNode {
                &self.0
            }
        }

        impl CstNodeExt for $name {
            const KIND: SyntaxKind = SyntaxKind::$kind;
            fn cast(node: SyntaxNode) -> Option<Self> {
                if node.kind() == SyntaxKind::$kind {
                    Some($name(node))
                } else {
                    None
                }
            }
        }
    };
}

typed_node!(
    /// The CST root (one per source file).
    Root => Root
);
typed_node!(
    /// A managed-mode `def` function.
    FnDef => FnDef
);
typed_node!(
    /// A `struct` definition.
    StructDef => StructDef
);
typed_node!(
    /// An `enum` definition.
    EnumDef => EnumDef
);
typed_node!(
    /// A declaration name.
    Name => Name
);
typed_node!(
    /// A parenthesized parameter list.
    ParamList => ParamList
);
typed_node!(
    /// A single parameter.
    Param => Param
);

/// `SyntaxKind`s that introduce a named, top-level-ish declaration. Used by
/// `Root::items` and by LSP document-symbol collection.
pub(crate) fn is_item_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::FnDef
            | SyntaxKind::SafeFnDef
            | SyntaxKind::StructDef
            | SyntaxKind::EnumDef
            | SyntaxKind::TraitDef
            | SyntaxKind::StructProtocolDef
            | SyntaxKind::ImplBlock
            | SyntaxKind::ActorDef
            | SyntaxKind::MemoryDecl
            | SyntaxKind::Module
            | SyntaxKind::UseDecl
            | SyntaxKind::ConstDef
            | SyntaxKind::LetStmt
    )
}

impl Root {
    /// The top-level item nodes, in source order.
    pub fn items(&self) -> impl Iterator<Item = SyntaxNode> + '_ {
        self.0.children().filter(|n| is_item_kind(n.kind()))
    }
}

impl FnDef {
    /// The function's declared name, if present.
    pub fn name(&self) -> Option<Name> {
        self.0.children().find_map(Name::cast)
    }

    /// The function's parameter list, if present.
    pub fn param_list(&self) -> Option<ParamList> {
        self.0.children().find_map(ParamList::cast)
    }
}

impl StructDef {
    /// The struct's declared name, if present.
    pub fn name(&self) -> Option<Name> {
        self.0.children().find_map(Name::cast)
    }
}

impl EnumDef {
    /// The enum's declared name, if present.
    pub fn name(&self) -> Option<Name> {
        self.0.children().find_map(Name::cast)
    }
}

impl ParamList {
    /// The parameters, in order.
    pub fn params(&self) -> impl Iterator<Item = Param> + '_ {
        self.0.children().filter_map(Param::cast)
    }
}

impl Param {
    /// The parameter's name, if present.
    pub fn name(&self) -> Option<Name> {
        self.0.children().find_map(Name::cast)
    }
}

impl Name {
    /// The identifier text (the first non-trivia token of the name).
    pub fn ident(&self) -> Option<String> {
        self.0
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .find(|t| !t.kind().is_trivia())
            .map(|t| t.text().to_string())
    }
}
