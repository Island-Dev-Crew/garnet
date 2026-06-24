//! S96 linear/effect safe-mode seed.
//!
//! This pass is deliberately narrow. It does not claim full linear typing or
//! runtime enforcement; it proves one static contract: authority-bearing safe
//! helper functions must expose an explicit ownership-qualified parameter
//! boundary (`own`, `borrow`, `ref`, or `mut`).

use crate::caps_graph::CapsReport;
use crate::capset::CapSet;
use garnet_parser::ast::{FnDef, FnMode, Item, Module, Ownership, TypeExpr};

/// Owner label for an impl block — the last path segment of the target type.
/// Mirrors `capability_surface::type_label` and `caps_graph::type_label` so
/// the effect pass looks up the SAME `Owner::name` transitive-caps key the
/// graph stores for impl methods.
fn type_label(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Named { path, .. } => path.last().cloned().unwrap_or_else(|| "impl".to_string()),
        _ => "impl".to_string(),
    }
}

/// The transitive-caps map key for a function: bare name for free fns,
/// `Owner::name` for impl methods. MUST match `caps_graph::fn_key`.
fn caps_key(owner: Option<&str>, name: &str) -> String {
    match owner {
        Some(owner) => format!("{owner}::{name}"),
        None => name.to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinearParam {
    pub name: String,
    pub ownership: Ownership,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnEffectSummary {
    pub fn_name: String,
    pub safe: bool,
    pub required_caps: CapSet,
    pub linear_params: Vec<LinearParam>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinearEffectViolation {
    pub fn_name: String,
    pub required_caps: CapSet,
}

impl LinearEffectViolation {
    pub fn message(&self) -> String {
        let caps = self.required_caps.names().join(", ");
        format!(
            "linear/effect violation: safe function `{}` performs authority effects [{}] without an explicit ownership-qualified parameter boundary (own/borrow/ref/mut). S96 is a static seed only; no VM or OS sandbox enforcement is claimed.",
            self.fn_name, caps
        )
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LinearEffectReport {
    pub functions: Vec<FnEffectSummary>,
    pub violations: Vec<LinearEffectViolation>,
}

pub fn linear_effect_report(module: &Module, caps_report: &CapsReport) -> LinearEffectReport {
    let mut report = LinearEffectReport::default();
    collect_items(
        &module.items,
        module.safe,
        caps_report,
        &mut report.functions,
        &mut report.violations,
    );
    report
}

fn collect_items(
    items: &[Item],
    module_safe: bool,
    caps_report: &CapsReport,
    functions: &mut Vec<FnEffectSummary>,
    violations: &mut Vec<LinearEffectViolation>,
) {
    for item in items {
        match item {
            Item::Fn(function) => {
                collect_function(
                    function,
                    /*owner=*/ None,
                    module_safe,
                    caps_report,
                    functions,
                    violations,
                );
            }
            Item::Module(module) => {
                collect_items(
                    &module.items,
                    module_safe || module.safe,
                    caps_report,
                    functions,
                    violations,
                );
            }
            Item::Impl(impl_block) => {
                let owner = type_label(&impl_block.target);
                for method in &impl_block.methods {
                    collect_function(
                        method,
                        Some(&owner),
                        module_safe,
                        caps_report,
                        functions,
                        violations,
                    );
                }
            }
            _ => {}
        }
    }
}

fn collect_function(
    function: &FnDef,
    owner: Option<&str>,
    module_safe: bool,
    caps_report: &CapsReport,
    functions: &mut Vec<FnEffectSummary>,
    violations: &mut Vec<LinearEffectViolation>,
) {
    let safe = module_safe || function.mode == FnMode::Safe;
    if !safe {
        return;
    }

    // Look up the transitive caps under the SAME key the caps graph stores —
    // `Owner::name` for impl methods, bare name for free fns. Before the
    // S-slice that type-qualified impl-method graph keys, this bare-name
    // lookup quietly missed impl-method caps.
    let fn_name = caps_key(owner, &function.name);
    let required_caps = caps_report
        .transitive
        .get(&fn_name)
        .copied()
        .unwrap_or_default();
    let linear_params = linear_params(function);
    let has_linear_params = !linear_params.is_empty();
    functions.push(FnEffectSummary {
        fn_name: fn_name.clone(),
        safe,
        required_caps,
        linear_params,
    });

    if function.name != "main" && !required_caps.is_empty() && !has_linear_params {
        violations.push(LinearEffectViolation {
            fn_name,
            required_caps,
        });
    }
}

fn linear_params(function: &FnDef) -> Vec<LinearParam> {
    function
        .params
        .iter()
        .filter_map(|param| {
            param.ownership.map(|ownership| LinearParam {
                name: param.name.clone(),
                ownership,
            })
        })
        .collect()
}
