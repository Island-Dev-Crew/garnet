//! # Garnet primitive-dispatch macros (RB-3)
//!
//! Two macros that derive the interpreter's native-primitive table from
//! per-adapter declarations, killing the hand-synced-lists drift class
//! between `garnet_stdlib::registry::all_prims()` and the interpreter's
//! `install()`:
//!
//! - [`macro@garnet_primitive`] — one attribute per native adapter:
//!   `#[garnet_primitive("module::name")]` on a
//!   `fn(Vec<Value>) -> Result<Value, RuntimeError>` item. Pure marker +
//!   shape validation; the key must be the primitive's qualified registry
//!   key (or a documented bridge-only key like `memory::working`).
//! - [`macro@garnet_primitive_module`] — on an inline `mod` containing
//!   adapters; collects every `#[garnet_primitive]` item and appends
//!   `pub(crate) fn entries() -> Vec<(&'static str, crate::value::NativeFn)>`
//!   to the module. The interpreter's `install()` chains the per-module
//!   `entries()` and joins them against the registry — binding mode,
//!   arity, and capability metadata all come from the `PrimMeta` row, so
//!   the adapter declares ONLY its key and body.
//!
//! ## Claim boundary
//!
//! The macros generate *registration*, not semantics: adapter bodies (the
//! `Value` conversions and the literal `require_capability` backstops the
//! caps-enforcement gate greps for) remain ordinary source text in
//! `stdlib_bridge.rs`. Generated code references `crate::value::NativeFn`,
//! so the expansion site must be inside `garnet-interp`. Consistency with
//! the registry is enforced behaviorally by the interpreter's
//! registry-join in `install()` plus its trap tests — a key mismatch is a
//! deterministic failure, not silent drift.

#![deny(clippy::unwrap_used, clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

/// Marker + shape validator for one native adapter. Usage:
/// `#[garnet_primitive("time::now_ms")]` on a
/// `fn(Vec<Value>) -> Result<Value, RuntimeError>` item.
#[proc_macro_attribute]
pub fn garnet_primitive(attr: TokenStream, item: TokenStream) -> TokenStream {
    let key = syn::parse_macro_input!(attr as syn::LitStr);
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    if let Err(e) = validate_adapter(&key, &func) {
        return e.to_compile_error().into();
    }
    // Pass the item through unchanged, keeping the attribute's key visible
    // to the enclosing #[garnet_primitive_module] collector via a paired
    // doc-invisible marker is unnecessary: the collector re-reads the
    // original attribute tokens from the module body.
    quote!(#func).into()
}

/// Collector for an inline module of adapters: appends
/// `pub(crate) fn entries() -> Vec<(&'static str, crate::value::NativeFn)>`
/// listing every `#[garnet_primitive("key")]` fn in declaration order.
#[proc_macro_attribute]
pub fn garnet_primitive_module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let module = syn::parse_macro_input!(item as syn::ItemMod);
    match expand_module(module) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn validate_adapter(key: &syn::LitStr, func: &syn::ItemFn) -> syn::Result<()> {
    if key.value().is_empty() {
        return Err(syn::Error::new(
            key.span(),
            "garnet_primitive: key must be the qualified primitive name, e.g. \"time::now_ms\"",
        ));
    }
    if func.sig.inputs.len() != 1 {
        return Err(syn::Error::new(
            func.sig.span(),
            "garnet_primitive: adapter must have the bridge signature \
             fn(Vec<Value>) -> Result<Value, RuntimeError>",
        ));
    }
    Ok(())
}

fn expand_module(mut module: syn::ItemMod) -> syn::Result<proc_macro2::TokenStream> {
    let Some((_brace, items)) = module.content.as_mut() else {
        return Err(syn::Error::new(
            module.span(),
            "garnet_primitive_module: must be applied to an INLINE module \
             (`mod x { ... }`) so the collector can see the adapters",
        ));
    };

    let mut entries: Vec<(String, syn::Ident)> = Vec::new();
    for item in items.iter() {
        if let syn::Item::Fn(func) = item {
            for attr in &func.attrs {
                if attr.path().is_ident("garnet_primitive") {
                    let key: syn::LitStr = attr.parse_args()?;
                    entries.push((key.value(), func.sig.ident.clone()));
                }
            }
        }
    }
    if entries.is_empty() {
        return Err(syn::Error::new(
            module.span(),
            "garnet_primitive_module: no #[garnet_primitive(\"...\")] adapters found",
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    for (key, _) in &entries {
        if !seen.insert(key.clone()) {
            return Err(syn::Error::new(
                module.span(),
                format!("garnet_primitive_module: duplicate primitive key `{key}`"),
            ));
        }
    }

    let keys: Vec<&String> = entries.iter().map(|(k, _)| k).collect();
    let idents: Vec<&syn::Ident> = entries.iter().map(|(_, i)| i).collect();
    let entries_fn: syn::Item = syn::parse_quote! {
        /// Macro-generated (RB-3): every `#[garnet_primitive]` adapter in
        /// this module, in declaration order.
        pub(crate) fn entries() -> Vec<(&'static str, crate::value::NativeFn)> {
            vec![ #( (#keys, #idents as crate::value::NativeFn) ),* ]
        }
    };
    items.push(entries_fn);

    Ok(quote!(#module))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn adapter(sig_inputs: &str) -> syn::ItemFn {
        syn::parse_str(&format!(
            "fn bridge_x({sig_inputs}) -> Result<Value, RuntimeError> {{ unimplemented!() }}"
        ))
        .unwrap()
    }

    #[test]
    fn accepts_the_bridge_signature() {
        let key: syn::LitStr = syn::parse_str("\"time::now_ms\"").unwrap();
        assert!(validate_adapter(&key, &adapter("args: Vec<Value>")).is_ok());
    }

    #[test]
    fn rejects_empty_key_and_wrong_arity() {
        let empty: syn::LitStr = syn::parse_str("\"\"").unwrap();
        assert!(validate_adapter(&empty, &adapter("args: Vec<Value>")).is_err());
        let key: syn::LitStr = syn::parse_str("\"k\"").unwrap();
        assert!(validate_adapter(&key, &adapter("a: Vec<Value>, b: u32")).is_err());
    }

    #[test]
    fn module_collector_emits_entries_and_rejects_duplicates() {
        let m: syn::ItemMod = syn::parse_str(
            r#"mod time_prims {
                #[garnet_primitive("time::now_ms")]
                fn bridge_now(_args: Vec<Value>) -> Result<Value, RuntimeError> { todo!() }
                #[garnet_primitive("time::sleep")]
                fn bridge_sleep(_args: Vec<Value>) -> Result<Value, RuntimeError> { todo!() }
            }"#,
        )
        .unwrap();
        let out = expand_module(m).unwrap().to_string();
        assert!(out.contains("fn entries"));
        assert!(out.contains("time::now_ms"));
        assert!(out.contains("bridge_sleep"));

        let dup: syn::ItemMod = syn::parse_str(
            r#"mod d {
                #[garnet_primitive("k")]
                fn a(_args: Vec<Value>) -> Result<Value, RuntimeError> { todo!() }
                #[garnet_primitive("k")]
                fn b(_args: Vec<Value>) -> Result<Value, RuntimeError> { todo!() }
            }"#,
        )
        .unwrap();
        assert!(expand_module(dup).is_err());
    }

    #[test]
    fn non_inline_module_is_rejected() {
        let m: syn::ItemMod = syn::parse_str("mod external;").unwrap();
        assert!(expand_module(m).is_err());
    }
}
