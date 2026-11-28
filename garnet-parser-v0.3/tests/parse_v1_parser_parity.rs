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
fn parses_paren_free_method_do_end_block_and_chaining() {
    let src = r#"
        @caps()
        def main() {
            [1, 2, 3].map do |x|
                x * 2
            end.filter do |y|
                y > 2
            end
        }
    "#;

    let module = parse_source(src).unwrap();
    let function = match &module.items[0] {
        Item::Fn(f) => f,
        other => panic!("expected function, got {other:?}"),
    };
    let Some(Expr::Method {
        receiver,
        method,
        args,
        ..
    }) = function.body.tail_expr.as_deref()
    else {
        panic!("expected outer Method expr for chained .filter do...end");
    };
    assert_eq!(method, "filter");
    assert_eq!(args.len(), 1);
    assert!(matches!(
        &args[0],
        Expr::Closure {
            is_do_block: true,
            ..
        }
    ));

    let Expr::Method {
        method: inner_method,
        args: inner_args,
        ..
    } = receiver.as_ref()
    else {
        panic!("expected inner Method expr for .map do...end");
    };
    assert_eq!(inner_method, "map");
    assert_eq!(inner_args.len(), 1);
    assert!(matches!(
        &inner_args[0],
        Expr::Closure {
            is_do_block: true,
            ..
        }
    ));
}

#[test]
fn parses_zero_param_paren_free_method_do_end_block() {
    let src = r#"
        @caps()
        def main() {
            runner.run do
                42
            end
        }
    "#;

    let module = parse_source(src).unwrap();
    let Item::Fn(function) = &module.items[0] else {
        panic!("expected function");
    };
    let Some(Expr::Method { method, args, .. }) = function.body.tail_expr.as_deref() else {
        panic!("expected paren-free method call");
    };
    assert_eq!(method, "run");
    assert_eq!(args.len(), 1);
    match &args[0] {
        Expr::Closure {
            params,
            is_do_block,
            ..
        } => {
            assert!(params.is_empty());
            assert!(*is_do_block);
        }
        other => panic!("expected do-block closure arg, got {other:?}"),
    }
}

#[test]
fn plain_field_access_unchanged_without_do_block() {
    let src = "@caps()\ndef main() { obj.field }\n";
    let module = parse_source(src).unwrap();
    let Item::Fn(function) = &module.items[0] else {
        panic!("expected function");
    };
    assert!(matches!(
        function.body.tail_expr.as_deref(),
        Some(Expr::Field { field, .. }) if field == "field"
    ));
}

#[test]
fn parses_ruby_unless_and_until_desugaring() {
    let src = r#"
        @caps()
        def main(flag) {
            let mut n = 0
            until n >= 3 {
                n = n + 1
            }
            unless flag {
                n
            } else {
                0
            }
        }
    "#;
    let module = parse_source(src).unwrap();
    let Item::Fn(function) = &module.items[0] else {
        panic!("expected function");
    };
    assert!(matches!(
        &function.body.stmts[1],
        Stmt::While {
            condition: Expr::Unary {
                op: garnet_parser::ast::UnOp::Not,
                ..
            },
            ..
        }
    ));
    assert!(matches!(
        function.body.tail_expr.as_deref(),
        Some(Expr::If {
            condition,
            elsif_clauses,
            else_block: Some(_),
            ..
        }) if matches!(
            condition.as_ref(),
            Expr::Unary {
                op: garnet_parser::ast::UnOp::Not,
                ..
            }
        ) && elsif_clauses.is_empty()
    ));
}

#[test]
fn parses_go_lexical_defer_in_lifo_order_for_expr_and_block_forms() {
    let src = r#"
        @caps()
        def main() {
            let mut log = []
            defer log.push(1)
            let mid = 10
            defer {
                log.push(2)
            }
            log.push(mid)
            log
        }
    "#;
    let module = parse_source(src).unwrap();
    let Item::Fn(function) = &module.items[0] else {
        panic!("expected function");
    };
    // Prefix before first `defer`: `let mut log = []`
    assert_eq!(function.body.stmts.len(), 1);
    // Outer `defer log.push(1)` becomes outer rescue-free `Expr::Try`
    let Some(Expr::Try {
        body: outer_body,
        rescues: outer_rescues,
        ensure: Some(outer_ensure),
        ..
    }) = function.body.tail_expr.as_deref()
    else {
        panic!("expected outer Try from first defer");
    };
    assert!(outer_rescues.is_empty());
    assert!(outer_ensure.stmts.is_empty());
    assert!(matches!(
        outer_ensure.tail_expr.as_deref(),
        Some(Expr::Method { method, .. }) if method == "push"
    ));

    // Between first and second defer: `let mid = 10`
    assert_eq!(outer_body.stmts.len(), 1);
    let Some(Expr::Try {
        body: inner_body,
        rescues: inner_rescues,
        ensure: Some(inner_ensure),
        ..
    }) = outer_body.tail_expr.as_deref()
    else {
        panic!("expected inner Try from second defer");
    };
    assert!(inner_rescues.is_empty());
    assert!(inner_ensure.stmts.is_empty());
    assert!(matches!(
        inner_ensure.tail_expr.as_deref(),
        Some(Expr::Method { method, .. }) if method == "push"
    ));
    // Guarded tail after second defer: `log.push(mid)` statement + `log` tail expr
    assert_eq!(inner_body.stmts.len(), 1);
    assert!(matches!(
        inner_body.tail_expr.as_deref(),
        Some(Expr::Ident(name, _)) if name == "log"
    ));
}

#[test]
fn parses_defer_at_end_of_block_and_rejects_bare_defer() {
    let src = r#"
        @safe
        @caps()
        fn clean() -> Int {
            let x = 1
            defer cleanup(x)
        }
    "#;
    let module = parse_source(src).unwrap();
    let Item::Fn(function) = &module.items[0] else {
        panic!("expected function");
    };
    assert_eq!(function.body.stmts.len(), 1);
    assert!(matches!(
        function.body.tail_expr.as_deref(),
        Some(Expr::Try {
            body,
            rescues,
            ensure: Some(_),
            ..
        }) if rescues.is_empty() && body.stmts.is_empty() && body.tail_expr.is_none()
    ));

    // Bare `defer` with no block or expression must fail to parse.
    assert!(parse_source("@caps()\ndef bad() { defer }\n").is_err());
    // `defer` is a statement, not an expression.
    assert!(parse_source("@caps()\ndef bad() { let x = defer f() }\n").is_err());
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
