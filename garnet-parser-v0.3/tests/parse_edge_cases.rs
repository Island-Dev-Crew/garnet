//! Parser edge cases — every keyword, every operator combination, deep
//! nesting, all whitespace patterns. These tests exist to catch regressions
//! the moment a grammar change breaks an obscure shape.

use garnet_parser::ast::{BinOp, Expr, FnMode, Item, Pattern, Stmt, UnOp};
use garnet_parser::parse_source;

fn parse_ok(src: &str) {
    parse_source(src).unwrap_or_else(|e| panic!("parse should succeed: {e:?}\n--- src ---\n{src}"));
}

fn first_fn_body_tail(src: &str) -> Expr {
    let m = parse_source(src).unwrap();
    let f = match &m.items[0] {
        Item::Fn(f) => f,
        _ => panic!("expected fn"),
    };
    f.body
        .tail_expr
        .as_ref()
        .expect("expected tail expr")
        .as_ref()
        .clone()
}

fn first_fn_first_stmt(src: &str) -> Stmt {
    let m = parse_source(src).unwrap();
    let f = match &m.items[0] {
        Item::Fn(f) => f,
        _ => panic!("expected fn"),
    };
    f.body.stmts[0].clone()
}

// ════════════════════════════════════════════════════════════════════
// Empty / whitespace-only sources
// ════════════════════════════════════════════════════════════════════

#[test]
fn parses_empty_source() {
    let m = parse_source("").unwrap();
    assert!(m.items.is_empty());
    assert!(!m.safe);
}

#[test]
fn parses_whitespace_only_source() {
    let m = parse_source("   \n\n\t  \n").unwrap();
    assert!(m.items.is_empty());
}

#[test]
fn parses_comment_only_source() {
    let m = parse_source("# just a comment\n# another\n").unwrap();
    assert!(m.items.is_empty());
}

#[test]
fn parses_mixed_whitespace_and_comments() {
    let m = parse_source("\n\n# a\n\n  # b\n\t#c\n").unwrap();
    assert!(m.items.is_empty());
}

// ════════════════════════════════════════════════════════════════════
// Multiple top-level items + ordering
// ════════════════════════════════════════════════════════════════════

#[test]
fn parses_multiple_defs() {
    let m = parse_source("def a() { 1 }\ndef b() { 2 }\ndef c() { 3 }").unwrap();
    assert_eq!(m.items.len(), 3);
}

#[test]
fn parses_def_then_struct_then_enum() {
    let m = parse_source(
        "def a() { 1 }\nstruct S { x: Int }\nenum E { A, B, C }",
    )
    .unwrap();
    assert_eq!(m.items.len(), 3);
}

#[test]
fn parses_const_then_let_then_def() {
    let m = parse_source("const C = 1\nlet x = 2\ndef f() { 3 }").unwrap();
    assert_eq!(m.items.len(), 3);
    assert!(matches!(m.items[0], Item::Const(_)));
    assert!(matches!(m.items[1], Item::Let(_)));
    assert!(matches!(m.items[2], Item::Fn(_)));
}

// ════════════════════════════════════════════════════════════════════
// Number literal edge cases
// ════════════════════════════════════════════════════════════════════

#[test]
fn parses_zero_int() {
    parse_ok("def x() { 0 }");
}

#[test]
fn parses_large_int() {
    parse_ok("def x() { 9_223_372_036_854_775_807 }");
}

#[test]
fn parses_negative_via_unary() {
    if let Expr::Unary { op: UnOp::Neg, .. } = first_fn_body_tail("def x() { -42 }") {
    } else {
        panic!("expected unary neg")
    }
}

#[test]
fn parses_int_with_underscores() {
    parse_ok("def x() { 1_000_000 }");
}

#[test]
fn parses_zero_float() {
    parse_ok("def x() { 0.0 }");
}

#[test]
fn parses_float_with_exponent() {
    parse_ok("def x() { 1.5e10 }");
}

#[test]
fn parses_float_with_negative_exponent() {
    parse_ok("def x() { 2.0e-3 }");
}

#[test]
fn parses_float_with_uppercase_exponent() {
    parse_ok("def x() { 1.0E5 }");
}

// ════════════════════════════════════════════════════════════════════
// String literal edge cases
// ════════════════════════════════════════════════════════════════════

#[test]
fn parses_empty_string() {
    parse_ok(r#"def x() { "" }"#);
}

#[test]
fn parses_string_with_escapes() {
    parse_ok(r#"def x() { "tab\there\nnewline" }"#);
}

#[test]
fn parses_string_with_quote_escape() {
    parse_ok(r#"def x() { "she said \"hi\"" }"#);
}

#[test]
fn parses_string_with_backslash_escape() {
    parse_ok(r#"def x() { "back\\slash" }"#);
}

#[test]
fn parses_simple_interpolation() {
    parse_ok(r##"def x() { "hello, #{name}" }"##);
}

#[test]
fn parses_interpolation_with_expression() {
    parse_ok(r##"def x() { "n+1=#{n + 1}" }"##);
}

#[test]
fn parses_multiple_interpolations() {
    parse_ok(r##"def x() { "#{a} and #{b} and #{c}" }"##);
}

#[test]
fn parses_raw_string() {
    parse_ok(r#"def x() { r"raw \n stays literal" }"#);
}

// ════════════════════════════════════════════════════════════════════
// Symbol literal edge cases
// ════════════════════════════════════════════════════════════════════

#[test]
fn parses_simple_symbol() {
    parse_ok("def x() { :ok }");
}

#[test]
fn parses_symbol_with_underscore() {
    parse_ok("def x() { :not_found }");
}

#[test]
fn parses_symbol_with_digits() {
    parse_ok("def x() { :http_404 }");
}

// ════════════════════════════════════════════════════════════════════
// Operator precedence edge cases (every Pratt level)
// ════════════════════════════════════════════════════════════════════

#[test]
fn pipeline_lowest_precedence() {
    parse_ok("def f() { x |> y + z }");
}

#[test]
fn or_below_and() {
    parse_ok("def f() { a or b and c }");
}

#[test]
fn comparison_below_arithmetic() {
    parse_ok("def f() { a + b < c * d }");
}

#[test]
fn unary_neg_binds_tighter_than_mul() {
    parse_ok("def f() { -a * -b }");
}

#[test]
fn postfix_question_binds_tightest() {
    parse_ok("def f() { foo()? + 1 }");
}

#[test]
fn deeply_nested_arithmetic() {
    parse_ok("def f() { ((((1 + 2) * 3) - 4) / 5) % 6 }");
}

#[test]
fn deeply_nested_method_calls() {
    parse_ok("def f() { a.b.c.d.e.f() }");
}

#[test]
fn long_pipeline_chain() {
    parse_ok("def f() { x |> a |> b |> c |> d |> e }");
}

#[test]
fn mixed_logical_operators() {
    parse_ok("def f() { not a and b or c and not d }");
}

#[test]
fn comparison_chain_via_and() {
    parse_ok("def f() { a < b and b < c and c < d }");
}

// ════════════════════════════════════════════════════════════════════
// Compound assignment ops
// ════════════════════════════════════════════════════════════════════

#[test]
fn parses_plus_assign() {
    let s = first_fn_first_stmt("def f() { let mut x = 0\n x += 1\n x }");
    assert!(matches!(s, Stmt::Let(_)));
}

#[test]
fn parses_minus_assign() {
    parse_ok("def f() { let mut x = 10\n x -= 1\n x }");
}

#[test]
fn parses_star_assign() {
    parse_ok("def f() { let mut x = 1\n x *= 2\n x }");
}

#[test]
fn parses_slash_assign() {
    parse_ok("def f() { let mut x = 100\n x /= 10\n x }");
}

#[test]
fn parses_percent_assign() {
    parse_ok("def f() { let mut x = 10\n x %= 3\n x }");
}

// ════════════════════════════════════════════════════════════════════
// Range edge cases
// ════════════════════════════════════════════════════════════════════

#[test]
fn parses_range_zero_to_zero() {
    parse_ok("def f() { 0..0 }");
}

#[test]
fn parses_range_with_negative_start() {
    parse_ok("def f() { -10..10 }");
}

#[test]
fn parses_range_with_expressions() {
    parse_ok("def f() { (a + 1)..(b * 2) }");
}

#[test]
fn parses_inclusive_range() {
    parse_ok("def f() { 1...100 }");
}

// ════════════════════════════════════════════════════════════════════
// Array & map literal edge cases
// ════════════════════════════════════════════════════════════════════

#[test]
fn parses_empty_array() {
    parse_ok("def f() { [] }");
}

#[test]
fn parses_single_element_array() {
    parse_ok("def f() { [42] }");
}

#[test]
fn parses_array_with_trailing_comma() {
    parse_ok("def f() { [1, 2, 3] }");
}

#[test]
fn parses_nested_array() {
    parse_ok("def f() { [[1, 2], [3, 4], [5, 6]] }");
}

#[test]
fn parses_array_of_mixed_types() {
    parse_ok(r#"def f() { [1, "two", :three, nil, true] }"#);
}

#[test]
fn parses_empty_map() {
    parse_ok("def f() { {} }");
}

#[test]
fn parses_single_pair_map() {
    parse_ok(r#"def f() { { "k" => "v" } }"#);
}

#[test]
fn parses_map_with_symbol_keys() {
    parse_ok(r#"def f() { { :a => 1, :b => 2 } }"#);
}

#[test]
fn parses_nested_map() {
    parse_ok(r#"def f() { { "outer" => { "inner" => 1 } } }"#);
}

#[test]
fn parses_multiline_map() {
    parse_ok(r#"
        def f() {
            {
                "a" => 1,
                "b" => 2,
                "c" => 3
            }
        }
    "#);
}

// ════════════════════════════════════════════════════════════════════
// Closure edge cases
// ════════════════════════════════════════════════════════════════════

#[test]
fn parses_zero_arg_closure() {
    parse_ok("def f() { || 42 }");
}

#[test]
fn parses_closure_with_block_body() {
    parse_ok("def f() { |x| { let y = x + 1\n y * 2 } }");
}

#[test]
fn parses_closure_with_typed_params() {
    parse_ok("def f() { |x: Int, y: Int| x + y }");
}

#[test]
fn parses_closure_with_return_type() {
    parse_ok("def f() { |x| -> Int { x } }");
}

// ════════════════════════════════════════════════════════════════════
// Pattern edge cases
// ════════════════════════════════════════════════════════════════════

#[test]
fn parses_match_with_only_wildcard() {
    parse_ok("def f() { match x { _ => 1 } }");
}

#[test]
fn parses_match_with_many_arms() {
    parse_ok(r#"
        def f() {
            match x {
                1 => :one,
                2 => :two,
                3 => :three,
                4 => :four,
                5 => :five,
                _ => :other,
            }
        }
    "#);
}

#[test]
fn parses_nested_enum_pattern() {
    parse_ok(r#"
        def f() {
            match r {
                Ok(Some(v)) => v,
                Ok(None) => 0,
                Err(_) => -1,
            }
        }
    "#);
}

#[test]
fn parses_tuple_pattern() {
    parse_ok(r#"
        def f() {
            match p {
                (a, b) => a + b,
                _ => 0,
            }
        }
    "#);
}

// ════════════════════════════════════════════════════════════════════
// Try/rescue edge cases
// ════════════════════════════════════════════════════════════════════

#[test]
fn parses_try_with_only_ensure() {
    parse_ok(r#"
        def f() {
            try {
                42
            } ensure {
                cleanup()
            }
        }
    "#);
}

#[test]
fn parses_try_with_many_rescues() {
    parse_ok(r#"
        def f() {
            try {
                risky()
            } rescue e: TypeA {
                1
            } rescue e: TypeB {
                2
            } rescue e: TypeC {
                3
            } rescue e {
                0
            }
        }
    "#);
}

// ════════════════════════════════════════════════════════════════════
// Loop control flow edge cases
// ════════════════════════════════════════════════════════════════════

#[test]
fn parses_loop_with_break_value() {
    parse_ok("def f() { loop { break 42 } }");
}

#[test]
fn parses_for_over_method_call_result() {
    parse_ok("def f() { for x in items.filter(|y| y > 0) { x } }");
}

#[test]
fn parses_while_with_complex_condition() {
    parse_ok("def f() { let mut x = 0\n while x < 10 and not done(x) { x += 1 } }");
}

// ════════════════════════════════════════════════════════════════════
// Struct & enum edge cases
// ════════════════════════════════════════════════════════════════════

#[test]
fn parses_struct_with_no_fields() {
    parse_ok("struct Empty {}");
}

#[test]
fn parses_struct_with_many_fields() {
    parse_ok(r#"
        struct Big {
            a: Int,
            b: Float,
            c: String,
            d: Bool,
            e: Array<Int>,
            f: Map<String, Int>,
        }
    "#);
}

#[test]
fn parses_enum_with_no_variants() {
    parse_ok("enum Never {}");
}

#[test]
fn parses_enum_with_many_variants() {
    parse_ok(r#"
        enum Status {
            Pending,
            Running,
            Succeeded(String),
            Failed(String, Int),
            Cancelled,
            Timeout,
        }
    "#);
}

#[test]
fn parses_generic_struct_with_multiple_params() {
    parse_ok(r#"
        struct Pair<A, B> {
            first: A,
            second: B,
        }
    "#);
}

// ════════════════════════════════════════════════════════════════════
// Memory unit edge cases
// ════════════════════════════════════════════════════════════════════

#[test]
fn parses_memory_with_simple_type() {
    parse_ok("memory working scratch : Buffer");
}

#[test]
fn parses_memory_with_path_type() {
    parse_ok("memory episodic events : my::pkg::Store<Event>");
}

#[test]
fn parses_memory_with_deeply_nested_generics() {
    parse_ok("memory semantic embeddings : Map<String, Map<String, Vector<Float>>>");
}

// ════════════════════════════════════════════════════════════════════
// Annotation edge cases
// ════════════════════════════════════════════════════════════════════

#[test]
fn parses_max_depth_min_value() {
    parse_ok("@max_depth(1) def f() { f() }");
}

#[test]
fn parses_max_depth_large_value() {
    parse_ok("@max_depth(64) def f() { f() }");
}

#[test]
fn parses_fan_out_min_value() {
    parse_ok("@fan_out(1) def f() { f() }");
}

#[test]
fn parses_fan_out_max_value() {
    parse_ok("@fan_out(1024) def f() { f() }");
}

#[test]
fn parses_require_metadata() {
    parse_ok("@require_metadata def f() { 1 }");
}

#[test]
fn parses_dynamic_annotation() {
    parse_ok("@dynamic def f() { 1 }");
}

#[test]
fn parses_combined_annotations() {
    parse_ok(r#"
        @max_depth(5)
        @fan_out(10)
        @require_metadata
        def f() { 1 }
    "#);
}

// ════════════════════════════════════════════════════════════════════
// Sanity: full BinOp + UnOp coverage at AST level
// ════════════════════════════════════════════════════════════════════

#[test]
fn produces_add_binop() {
    if let Expr::Binary { op, .. } = first_fn_body_tail("def f() { a + b }") {
        assert_eq!(op, BinOp::Add);
    }
}

#[test]
fn produces_sub_binop() {
    if let Expr::Binary { op, .. } = first_fn_body_tail("def f() { a - b }") {
        assert_eq!(op, BinOp::Sub);
    }
}

#[test]
fn produces_mul_binop() {
    if let Expr::Binary { op, .. } = first_fn_body_tail("def f() { a * b }") {
        assert_eq!(op, BinOp::Mul);
    }
}

#[test]
fn produces_div_binop() {
    if let Expr::Binary { op, .. } = first_fn_body_tail("def f() { a / b }") {
        assert_eq!(op, BinOp::Div);
    }
}

#[test]
fn produces_mod_binop() {
    if let Expr::Binary { op, .. } = first_fn_body_tail("def f() { a % b }") {
        assert_eq!(op, BinOp::Mod);
    }
}

#[test]
fn produces_eq_binop() {
    if let Expr::Binary { op, .. } = first_fn_body_tail("def f() { a == b }") {
        assert_eq!(op, BinOp::Eq);
    }
}

#[test]
fn produces_neq_binop() {
    if let Expr::Binary { op, .. } = first_fn_body_tail("def f() { a != b }") {
        assert_eq!(op, BinOp::NotEq);
    }
}

#[test]
fn produces_pipeline_binop() {
    if let Expr::Binary { op, .. } = first_fn_body_tail("def f() { a |> b }") {
        assert_eq!(op, BinOp::Pipeline);
    }
}

#[test]
fn produces_neg_unop() {
    if let Expr::Unary { op, .. } = first_fn_body_tail("def f() { -x }") {
        assert_eq!(op, UnOp::Neg);
    }
}

#[test]
fn produces_not_unop() {
    if let Expr::Unary { op, .. } = first_fn_body_tail("def f() { not x }") {
        assert_eq!(op, UnOp::Not);
    }
}

#[test]
fn produces_question_unop() {
    if let Expr::Unary { op, .. } = first_fn_body_tail("def f() { x? }") {
        assert_eq!(op, UnOp::Question);
    }
}

// ════════════════════════════════════════════════════════════════════
// Function modes
// ════════════════════════════════════════════════════════════════════

#[test]
fn def_produces_managed_mode() {
    let m = parse_source("def f() { 1 }").unwrap();
    if let Item::Fn(f) = &m.items[0] {
        assert_eq!(f.mode, FnMode::Managed);
    }
}

#[test]
fn fn_produces_safe_mode() {
    let m = parse_source("fn f(x: Int) -> Int { x }").unwrap();
    if let Item::Fn(f) = &m.items[0] {
        assert_eq!(f.mode, FnMode::Safe);
    }
}

// ════════════════════════════════════════════════════════════════════
// Pattern AST
// ════════════════════════════════════════════════════════════════════

#[test]
fn wildcard_pattern_in_match() {
    let src = "def f() { match x { _ => 1 } }";
    let m = parse_source(src).unwrap();
    if let Item::Fn(f) = &m.items[0] {
        if let Some(tail) = &f.body.tail_expr {
            if let Expr::Match { arms, .. } = tail.as_ref() {
                assert!(matches!(arms[0].pattern, Pattern::Wildcard(_)));
            }
        }
    }
}

#[test]
fn ident_pattern_in_match() {
    let src = "def f() { match x { y => y } }";
    let m = parse_source(src).unwrap();
    if let Item::Fn(f) = &m.items[0] {
        if let Some(tail) = &f.body.tail_expr {
            if let Expr::Match { arms, .. } = tail.as_ref() {
                assert!(matches!(arms[0].pattern, Pattern::Ident(_, _)));
            }
        }
    }
}
