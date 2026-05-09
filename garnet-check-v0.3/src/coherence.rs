//! Conservative trait coherence checks for Mini-Spec §11.5.
//!
//! This is not full generic overlap solving. It enforces the first useful
//! local guarantees: trait impls must satisfy the orphan rule, and exact
//! duplicate impls for the same trait/type pair are rejected.

use std::collections::BTreeSet;

use garnet_parser::ast::{Item, Module, ModuleDecl, TypeExpr};

use crate::CheckError;

#[derive(Default)]
struct LocalNames {
    traits: BTreeSet<String>,
    types: BTreeSet<String>,
}

/// Run the conservative trait-coherence pass on a parsed module.
pub fn check_trait_coherence(module: &Module) -> Vec<CheckError> {
    let mut names = LocalNames::default();
    collect_local_names(&module.items, &mut names);

    let mut seen_impls = BTreeSet::new();
    let mut diags = Vec::new();
    check_items(&module.items, &names, &mut seen_impls, &mut diags);
    diags
}

fn collect_local_names(items: &[Item], names: &mut LocalNames) {
    for item in items {
        match item {
            Item::Struct(def) => {
                names.types.insert(def.name.clone());
            }
            Item::Enum(def) => {
                names.types.insert(def.name.clone());
            }
            Item::Actor(def) => {
                names.types.insert(def.name.clone());
            }
            Item::Trait(def) => {
                names.traits.insert(def.name.clone());
            }
            Item::Protocol(def) => {
                names.traits.insert(def.name.clone());
            }
            Item::Module(ModuleDecl { items, .. }) => collect_local_names(items, names),
            _ => {}
        }
    }
}

fn check_items(
    items: &[Item],
    names: &LocalNames,
    seen_impls: &mut BTreeSet<(String, String)>,
    diags: &mut Vec<CheckError>,
) {
    for item in items {
        match item {
            Item::Impl(impl_block) => {
                let Some(trait_ty) = &impl_block.trait_ty else {
                    continue;
                };
                let trait_key = type_key(trait_ty);
                let target_key = type_key(&impl_block.target);
                let trait_head = type_head(trait_ty);
                let target_head = type_head(&impl_block.target);

                let trait_is_local = trait_head
                    .as_ref()
                    .is_some_and(|name| names.traits.contains(name));
                let target_is_local = target_head
                    .as_ref()
                    .is_some_and(|name| names.types.contains(name));

                if !trait_is_local && !target_is_local {
                    diags.push(CheckError::SafeModeViolation(format!(
                        "trait coherence violation: impl `{trait_key}` for `{target_key}` violates the orphan rule; define the trait or the type locally"
                    )));
                }

                if !seen_impls.insert((trait_key.clone(), target_key.clone())) {
                    diags.push(CheckError::SafeModeViolation(format!(
                        "trait coherence violation: duplicate impl `{trait_key}` for `{target_key}`"
                    )));
                }
            }
            Item::Module(ModuleDecl { items, .. }) => check_items(items, names, seen_impls, diags),
            _ => {}
        }
    }
}

fn type_head(ty: &TypeExpr) -> Option<String> {
    match ty {
        TypeExpr::Named { path, .. } => path.last().cloned(),
        TypeExpr::Dyn { trait_ty, .. } => type_head(trait_ty),
        _ => None,
    }
}

fn type_key(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Named { path, args, .. } if args.is_empty() => path.join("::"),
        TypeExpr::Named { path, args, .. } => {
            let args = args.iter().map(type_key).collect::<Vec<_>>().join(", ");
            format!("{}<{args}>", path.join("::"))
        }
        TypeExpr::Fn { params, ret, .. } => {
            let params = params.iter().map(type_key).collect::<Vec<_>>().join(", ");
            format!("({params}) -> {}", type_key(ret))
        }
        TypeExpr::Tuple { elements, .. } => {
            let elements = elements.iter().map(type_key).collect::<Vec<_>>().join(", ");
            format!("({elements})")
        }
        TypeExpr::Ref { mutable, inner, .. } => {
            if *mutable {
                format!("&mut {}", type_key(inner))
            } else {
                format!("&{}", type_key(inner))
            }
        }
        TypeExpr::Dyn { trait_ty, .. } => format!("dyn {}", type_key(trait_ty)),
    }
}
