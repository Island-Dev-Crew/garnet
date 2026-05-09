//! Parser-parity tests for Mini-Spec v1.0 surfaces that are staged before
//! full checker/runtime semantics.

use garnet_parser::ast::{Annotation, ClosureBody, Expr, Item, Stmt, TypeExpr};
use garnet_parser::parse_source;

#[test]
fn parses_top_level_structural_protocol() {
    let src = r#"
        pub protocol Drawable<T> {
            def draw(item: T) -> String
            fn area(borrow self) -> Int
        }
    "#;

    let module = parse_source(src).unwrap();
    match &module.items[0] {
        Item::Protocol(protocol) => {
            assert!(protocol.public);
            assert_eq!(protocol.name, "Drawable");
            assert_eq!(protocol.type_params, vec!["T"]);
            assert_eq!(protocol.items.len(), 2);
        }
        other => panic!("expected top-level protocol, got {other:?}"),
    }
}

#[test]
fn parses_dyn_trait_type_annotation() {
    let src = r#"
        trait Drawable {
            def draw() -> String
        }

        @caps()
        def render(item: dyn Drawable) {
            item.draw()
        }
    "#;

    let module = parse_source(src).unwrap();
    let render = module
        .items
        .iter()
        .find_map(|item| match item {
            Item::Fn(f) if f.name == "render" => Some(f),
            _ => None,
        })
        .expect("render function");

    match render.params[0].ty.as_ref().expect("typed parameter") {
        TypeExpr::Dyn { trait_ty, .. } => match trait_ty.as_ref() {
            TypeExpr::Named { path, .. } => assert_eq!(path, &vec!["Drawable".to_string()]),
            other => panic!("expected named trait type, got {other:?}"),
        },
        other => panic!("expected dyn trait type, got {other:?}"),
    }
}

#[test]
fn parses_protocol_cast_expression() {
    let src = r#"
        protocol Renderable {
            def render() -> String
        }

        struct Widget {
            name: String
        }

        @caps()
        def main() {
            let widget = Widget("panel")
            let renderable = widget as Renderable
            renderable.render()
        }
    "#;

    let module = parse_source(src).unwrap();
    let function = match module.items.last().expect("main function") {
        Item::Fn(f) => f,
        other => panic!("expected function, got {other:?}"),
    };
    let let_stmt = function
        .body
        .stmts
        .iter()
        .find_map(|stmt| match stmt {
            Stmt::Let(decl) if decl.name == "renderable" => Some(decl),
            _ => None,
        })
        .expect("renderable cast binding");
    let Expr::Cast { ty, .. } = &let_stmt.value else {
        panic!("expected renderable binding to parse as a cast expression");
    };
    match ty {
        TypeExpr::Named { path, .. } => assert_eq!(path, &vec!["Renderable".to_string()]),
        other => panic!("expected named protocol cast type, got {other:?}"),
    }
}

#[test]
fn parses_yield_and_next_statements_as_staged_surface() {
    let src = r#"
        @caps()
        def each_item(items) {
            yield 1
            next 2
            3
        }
    "#;

    let module = parse_source(src).unwrap();
    let function = match &module.items[0] {
        Item::Fn(f) => f,
        other => panic!("expected function, got {other:?}"),
    };

    assert!(matches!(function.body.stmts[0], Stmt::Yield { .. }));
    assert!(matches!(function.body.stmts[1], Stmt::Next { .. }));
}

#[test]
fn parses_do_end_block_argument() {
    let src = r#"
        @caps()
        def main() {
            each([1, 2, 3]) do |x|
                yield x + 1
            end
        }
    "#;

    let module = parse_source(src).unwrap();
    let function = match &module.items[0] {
        Item::Fn(f) => f,
        other => panic!("expected function, got {other:?}"),
    };
    let Some(Expr::Call { args, .. }) = function.body.tail_expr.as_deref() else {
        panic!("expected call tail expression");
    };
    assert_eq!(
        args.len(),
        2,
        "do/end block should be a trailing closure arg"
    );

    let Expr::Closure { params, body, .. } = &args[1] else {
        panic!("expected trailing do/end block to parse as a closure arg");
    };
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "x");
    let ClosureBody::Block(block) = body.as_ref() else {
        panic!("expected block-bodied closure");
    };
    assert!(matches!(block.stmts[0], Stmt::Yield { .. }));
}

#[test]
fn preserves_dynamic_and_nonsendable_type_annotations() {
    let src = r#"
        @dynamic
        @nonsendable
        struct RuntimeObject {
            id: Int
        }

        @dynamic
        impl RuntimeObject {
            def label(self) {
                "runtime"
            }
        }
    "#;

    let module = parse_source(src).unwrap();
    match &module.items[0] {
        Item::Struct(s) => {
            assert!(s
                .annotations
                .iter()
                .any(|ann| matches!(ann, Annotation::Dynamic(_))));
            assert!(s
                .annotations
                .iter()
                .any(|ann| matches!(ann, Annotation::NonSendable(_))));
        }
        other => panic!("expected struct, got {other:?}"),
    }

    match &module.items[1] {
        Item::Impl(i) => assert!(i
            .annotations
            .iter()
            .any(|ann| matches!(ann, Annotation::Dynamic(_)))),
        other => panic!("expected impl, got {other:?}"),
    }
}
