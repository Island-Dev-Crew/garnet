//! S96 linear/effect safe-mode seed.
//!
//! This pass is deliberately narrow. It does not claim full linear typing or
//! runtime enforcement; it proves one static contract: authority-bearing safe
//! helper functions must expose an explicit ownership-qualified parameter
//! boundary (`own`, `borrow`, `ref`, or `mut`).

use crate::caps_graph::{CapsReport, CapsSet};
use garnet_parser::ast::{FnDef, FnMode, Item, Module, Ownership};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinearParam {
    pub name: String,
    pub ownership: Ownership,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnEffectSummary {
    pub fn_name: String,
    pub safe: bool,
    pub required_caps: CapsSet,
    pub linear_params: Vec<LinearParam>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinearEffectViolation {
    pub fn_name: String,
    pub required_caps: CapsSet,
}

impl LinearEffectViolation {
    pub fn message(&self) -> String {
        let caps = self
            .required_caps
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
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
                collect_function(function, module_safe, caps_report, functions, violations);
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
                for method in &impl_block.methods {
                    collect_function(method, module_safe, caps_report, functions, violations);
                }
            }
            _ => {}
        }
    }
}

fn collect_function(
    function: &FnDef,
    module_safe: bool,
    caps_report: &CapsReport,
    functions: &mut Vec<FnEffectSummary>,
    violations: &mut Vec<LinearEffectViolation>,
) {
    let safe = module_safe || function.mode == FnMode::Safe;
    if !safe {
        return;
    }

    let required_caps = caps_report
        .transitive
        .get(&function.name)
        .cloned()
        .unwrap_or_default();
    let linear_params = linear_params(function);
    functions.push(FnEffectSummary {
        fn_name: function.name.clone(),
        safe,
        required_caps: required_caps.clone(),
        linear_params: linear_params.clone(),
    });

    if function.name != "main" && !required_caps.is_empty() && linear_params.is_empty() {
        violations.push(LinearEffectViolation {
            fn_name: function.name.clone(),
            required_caps: sorted_caps(required_caps),
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

fn sorted_caps(caps: BTreeSet<String>) -> BTreeSet<String> {
    caps
}
