//! Smoke test for the typed-node surface S16 consumes: cast the root, walk
//! items, and read names/params off a `FnDef`.

use garnet_cst::{parse_cst, CstNodeExt, FnDef, Root};

#[test]
fn typed_nodes_expose_fn_name_and_params() {
    let src = "def greet(name, greeting) {\n  greeting\n}\nstruct Config { host: String }\n";
    let parse = parse_cst(src);
    assert!(parse.ok(), "valid program should parse without errors");

    let root = Root::cast(parse.syntax().clone()).expect("root node casts to Root");
    let items: Vec<_> = root.items().collect();
    assert!(items.len() >= 2, "expected at least a fn and a struct item");

    let fndef = items
        .iter()
        .cloned()
        .find_map(FnDef::cast)
        .expect("corpus has a FnDef");
    assert_eq!(
        fndef.name().and_then(|n| n.ident()).as_deref(),
        Some("greet")
    );

    let params: Vec<String> = fndef
        .param_list()
        .expect("fn has a parameter list")
        .params()
        .filter_map(|p| p.name().and_then(|n| n.ident()))
        .collect();
    assert_eq!(params, vec!["name".to_string(), "greeting".to_string()]);
}
