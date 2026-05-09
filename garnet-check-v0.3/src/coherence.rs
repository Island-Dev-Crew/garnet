//! Conservative trait coherence checks for Mini-Spec §11.5.
//!
//! This is not full specialization or imported-package coherence solving. It
//! enforces useful local guarantees: trait impls must satisfy the orphan rule,
//! exact duplicate impls are rejected, and simple generic blanket impls cannot
//! overlap concrete or renamed blanket impls for the same trait/type pattern.

use std::collections::{BTreeMap, BTreeSet};

use garnet_parser::ast::{Item, Module, ModuleDecl, TypeExpr};

use crate::CheckError;

#[derive(Default)]
struct LocalNames {
    traits: BTreeSet<String>,
    trait_shorts: BTreeSet<String>,
    types: BTreeSet<String>,
    type_shorts: BTreeSet<String>,
}

struct ImplRecord {
    trait_ty: TypeExpr,
    target_ty: TypeExpr,
    type_params: BTreeSet<String>,
    trait_key: String,
    target_key: String,
}

/// Run the conservative trait-coherence pass on a parsed module.
pub fn check_trait_coherence(module: &Module) -> Vec<CheckError> {
    let mut names = LocalNames::default();
    collect_local_names(&module.items, &mut Vec::new(), &mut names);

    let mut seen_impls = BTreeSet::new();
    let mut prior_impls = Vec::new();
    let mut diags = Vec::new();
    check_items(
        &module.items,
        &names,
        &mut seen_impls,
        &mut prior_impls,
        &mut diags,
    );
    diags
}

fn collect_local_names(items: &[Item], module_path: &mut Vec<String>, names: &mut LocalNames) {
    for item in items {
        match item {
            Item::Struct(def) => {
                insert_local_type(&def.name, module_path, names);
            }
            Item::Enum(def) => {
                insert_local_type(&def.name, module_path, names);
            }
            Item::Actor(def) => {
                insert_local_type(&def.name, module_path, names);
            }
            Item::Trait(def) => {
                insert_local_trait(&def.name, module_path, names);
            }
            Item::Protocol(def) => {
                insert_local_trait(&def.name, module_path, names);
            }
            Item::Module(ModuleDecl { name, items, .. }) => {
                module_path.push(name.clone());
                collect_local_names(items, module_path, names);
                module_path.pop();
            }
            _ => {}
        }
    }
}

fn insert_local_type(name: &str, module_path: &[String], names: &mut LocalNames) {
    names.type_shorts.insert(name.to_string());
    names.types.insert(local_name_key(module_path, name));
}

fn insert_local_trait(name: &str, module_path: &[String], names: &mut LocalNames) {
    names.trait_shorts.insert(name.to_string());
    names.traits.insert(local_name_key(module_path, name));
}

fn local_name_key(module_path: &[String], name: &str) -> String {
    if module_path.is_empty() {
        name.to_string()
    } else {
        format!("{}::{name}", module_path.join("::"))
    }
}

fn check_items(
    items: &[Item],
    names: &LocalNames,
    seen_impls: &mut BTreeSet<(String, String)>,
    prior_impls: &mut Vec<ImplRecord>,
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
                let trait_is_local = is_local_trait(trait_ty, names);
                let target_is_local = is_local_type(&impl_block.target, names);

                if !trait_is_local && !target_is_local {
                    diags.push(CheckError::SafeModeViolation(format!(
                        "trait coherence violation: impl `{trait_key}` for `{target_key}` violates the orphan rule; define the trait or the type locally"
                    )));
                }

                let duplicate = !seen_impls.insert((trait_key.clone(), target_key.clone()));
                if duplicate {
                    diags.push(CheckError::SafeModeViolation(format!(
                        "trait coherence violation: duplicate impl `{trait_key}` for `{target_key}`"
                    )));
                } else {
                    let type_params = impl_block
                        .type_params
                        .iter()
                        .cloned()
                        .collect::<BTreeSet<_>>();
                    let record = ImplRecord {
                        trait_ty: trait_ty.clone(),
                        target_ty: impl_block.target.clone(),
                        type_params,
                        trait_key: trait_key.clone(),
                        target_key: target_key.clone(),
                    };

                    for prior in prior_impls.iter() {
                        if impls_overlap(prior, &record) {
                            diags.push(CheckError::SafeModeViolation(format!(
                                "trait coherence violation: overlapping impl `{trait_key}` for `{target_key}` overlaps existing impl `{}` for `{}`; witness type `{}`",
                                prior.trait_key,
                                prior.target_key,
                                target_key
                            )));
                            break;
                        }
                    }

                    prior_impls.push(record);
                }
            }
            Item::Module(ModuleDecl { items, .. }) => {
                check_items(items, names, seen_impls, prior_impls, diags)
            }
            _ => {}
        }
    }
}

fn is_local_trait(ty: &TypeExpr, names: &LocalNames) -> bool {
    type_path(ty).is_some_and(|path| local_path_matches(path, &names.traits, &names.trait_shorts))
}

fn is_local_type(ty: &TypeExpr, names: &LocalNames) -> bool {
    type_path(ty).is_some_and(|path| local_path_matches(path, &names.types, &names.type_shorts))
}

fn local_path_matches(
    path: &[String],
    full_names: &BTreeSet<String>,
    short_names: &BTreeSet<String>,
) -> bool {
    if path.len() == 1 {
        short_names.contains(&path[0])
    } else {
        full_names.contains(&path.join("::"))
    }
}

fn type_path(ty: &TypeExpr) -> Option<&[String]> {
    match ty {
        TypeExpr::Named { path, .. } => Some(path),
        TypeExpr::Dyn { trait_ty, .. } => type_path(trait_ty),
        _ => None,
    }
}

fn impls_overlap(left: &ImplRecord, right: &ImplRecord) -> bool {
    type_patterns_overlap(
        &left.trait_ty,
        &left.type_params,
        &right.trait_ty,
        &right.type_params,
    ) && type_patterns_overlap(
        &left.target_ty,
        &left.type_params,
        &right.target_ty,
        &right.type_params,
    )
}

fn type_patterns_overlap(
    left: &TypeExpr,
    left_params: &BTreeSet<String>,
    right: &TypeExpr,
    right_params: &BTreeSet<String>,
) -> bool {
    let mut left_bindings = BTreeMap::new();
    let mut right_bindings = BTreeMap::new();
    type_patterns_overlap_inner(
        left,
        left_params,
        right,
        right_params,
        &mut left_bindings,
        &mut right_bindings,
    )
}

fn type_patterns_overlap_inner(
    left: &TypeExpr,
    left_params: &BTreeSet<String>,
    right: &TypeExpr,
    right_params: &BTreeSet<String>,
    left_bindings: &mut BTreeMap<String, String>,
    right_bindings: &mut BTreeMap<String, String>,
) -> bool {
    let left_param = type_param_name(left, left_params);
    let right_param = type_param_name(right, right_params);

    match (left_param, right_param) {
        (Some(_), Some(_)) => true,
        (Some(param), None) => bind_param(left_bindings, param, &type_key(right)),
        (None, Some(param)) => bind_param(right_bindings, param, &type_key(left)),
        (None, None) => match (left, right) {
            (
                TypeExpr::Named {
                    path: left_path,
                    args: left_args,
                    ..
                },
                TypeExpr::Named {
                    path: right_path,
                    args: right_args,
                    ..
                },
            ) => {
                left_path == right_path
                    && left_args.len() == right_args.len()
                    && left_args
                        .iter()
                        .zip(right_args)
                        .all(|(left_arg, right_arg)| {
                            type_patterns_overlap_inner(
                                left_arg,
                                left_params,
                                right_arg,
                                right_params,
                                left_bindings,
                                right_bindings,
                            )
                        })
            }
            (
                TypeExpr::Fn {
                    params: left_params_ty,
                    ret: left_ret,
                    ..
                },
                TypeExpr::Fn {
                    params: right_params_ty,
                    ret: right_ret,
                    ..
                },
            ) => {
                left_params_ty.len() == right_params_ty.len()
                    && left_params_ty.iter().zip(right_params_ty).all(
                        |(left_param_ty, right_param_ty)| {
                            type_patterns_overlap_inner(
                                left_param_ty,
                                left_params,
                                right_param_ty,
                                right_params,
                                left_bindings,
                                right_bindings,
                            )
                        },
                    )
                    && type_patterns_overlap_inner(
                        left_ret,
                        left_params,
                        right_ret,
                        right_params,
                        left_bindings,
                        right_bindings,
                    )
            }
            (
                TypeExpr::Tuple {
                    elements: left_elements,
                    ..
                },
                TypeExpr::Tuple {
                    elements: right_elements,
                    ..
                },
            ) => {
                left_elements.len() == right_elements.len()
                    && left_elements.iter().zip(right_elements).all(
                        |(left_element, right_element)| {
                            type_patterns_overlap_inner(
                                left_element,
                                left_params,
                                right_element,
                                right_params,
                                left_bindings,
                                right_bindings,
                            )
                        },
                    )
            }
            (
                TypeExpr::Ref {
                    mutable: left_mutable,
                    inner: left_inner,
                    ..
                },
                TypeExpr::Ref {
                    mutable: right_mutable,
                    inner: right_inner,
                    ..
                },
            ) => {
                left_mutable == right_mutable
                    && type_patterns_overlap_inner(
                        left_inner,
                        left_params,
                        right_inner,
                        right_params,
                        left_bindings,
                        right_bindings,
                    )
            }
            (
                TypeExpr::Dyn {
                    trait_ty: left_trait,
                    ..
                },
                TypeExpr::Dyn {
                    trait_ty: right_trait,
                    ..
                },
            ) => type_patterns_overlap_inner(
                left_trait,
                left_params,
                right_trait,
                right_params,
                left_bindings,
                right_bindings,
            ),
            _ => false,
        },
    }
}

fn type_param_name<'a>(ty: &'a TypeExpr, params: &BTreeSet<String>) -> Option<&'a str> {
    match ty {
        TypeExpr::Named { path, args, .. } if path.len() == 1 && args.is_empty() => {
            params.contains(&path[0]).then_some(path[0].as_str())
        }
        _ => None,
    }
}

fn bind_param(bindings: &mut BTreeMap<String, String>, param: &str, witness: &str) -> bool {
    match bindings.get(param) {
        Some(existing) => existing == witness,
        None => {
            bindings.insert(param.to_string(), witness.to_string());
            true
        }
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
