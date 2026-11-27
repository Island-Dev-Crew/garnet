//! ADR 0016 (D-42) — Models stay out of the trust path.
//!
//! Proves:
//! 1. `garnet-suggest-llm` is removed from the workspace (`Cargo.toml`, on-disk
//!    directory, and `Cargo.lock`).
//! 2. No crate in `garnet-cli`'s transitive `Cargo.lock` dependency closure
//!    links `garnet-suggest-llm`, an HTTP client/transport, or an ML/LLM runtime.
//! 3. The closure checker itself fails closed when a forbidden HTTP client, ML
//!    runtime, or `garnet-suggest-llm` appears transitively in a lockfile graph.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN_SUGGEST_CRATES: &[&str] = &["garnet-suggest-llm"];

const FORBIDDEN_HTTP_CLIENT_CRATES: &[&str] = &[
    "attohttpc",
    "awc",
    "curl",
    "curl-sys",
    "h2",
    "h3",
    "http",
    "http-body",
    "http-body-util",
    "http-client",
    "http-types",
    "hyper",
    "hyper-rustls",
    "hyper-tls",
    "hyper-util",
    "isahc",
    "minreq",
    "native-tls",
    "openssl",
    "openssl-sys",
    "reqwest",
    "reqwest-eventsource",
    "reqwest-middleware",
    "rustls",
    "surf",
    "tiny_http",
    "tower-http",
    "ureq",
];

const FORBIDDEN_ML_RUNTIME_CRATES: &[&str] = &[
    "anthropic",
    "anthropic-sdk",
    "async-openai",
    "burn",
    "burn-core",
    "candle-core",
    "candle-nn",
    "candle-transformers",
    "ggml",
    "ggml-sys",
    "hf-hub",
    "kalosm",
    "langchain-rust",
    "linfa",
    "llama-cpp-2",
    "llama-cpp-sys",
    "llama-cpp-sys-2",
    "llama_cpp",
    "mistralrs",
    "mistralrs-core",
    "ndarray",
    "ollama-rs",
    "onnxruntime",
    "onnxruntime-sys",
    "ort",
    "ort-sys",
    "rig-core",
    "safetensors",
    "smartcore",
    "tch",
    "tensorflow",
    "tflite",
    "tokenizers",
    "torch-sys",
    "tract-core",
    "tract-onnx",
];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PackageId {
    name: String,
    version: String,
}

fn normalize_crate_name(name: &str) -> String {
    name.replace('_', "-").to_ascii_lowercase()
}

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("garnet-cli manifest directory must have workspace parent")
        .to_path_buf()
}

fn parse_lock_packages(lock_text: &str) -> BTreeMap<PackageId, Vec<String>> {
    let value: toml::Value = toml::from_str(lock_text).expect("Cargo.lock must be valid TOML");
    let packages = value
        .get("package")
        .and_then(toml::Value::as_array)
        .expect("Cargo.lock must contain [[package]] array");

    let mut graph: BTreeMap<PackageId, Vec<String>> = BTreeMap::new();
    for pkg in packages {
        let name = pkg
            .get("name")
            .and_then(toml::Value::as_str)
            .expect("Cargo.lock package must have string name")
            .to_string();
        let version = pkg
            .get("version")
            .and_then(toml::Value::as_str)
            .expect("Cargo.lock package must have string version")
            .to_string();
        let deps = pkg
            .get("dependencies")
            .and_then(toml::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .map(|item| {
                        item.as_str()
                            .expect("Cargo.lock dependency entry must be string")
                            .to_string()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        graph.insert(PackageId { name, version }, deps);
    }
    graph
}

fn resolve_dep_targets(
    dep_spec: &str,
    by_name: &BTreeMap<String, Vec<PackageId>>,
) -> Vec<PackageId> {
    // Cargo.lock dependency entries are formatted as:
    //   "name"
    //   "name version"
    //   "name version (source)"
    let mut parts = dep_spec.split_whitespace();
    let name = parts
        .next()
        .expect("Cargo.lock dependency entry must not be empty");
    let version = parts.next();

    let candidates = by_name.get(name).unwrap_or_else(|| {
        panic!("Cargo.lock dependency `{dep_spec}` references unknown package `{name}`")
    });

    if let Some(ver) = version {
        let matched: Vec<PackageId> = candidates
            .iter()
            .filter(|id| id.version == ver)
            .cloned()
            .collect();
        assert!(
            !matched.is_empty(),
            "Cargo.lock dependency `{dep_spec}` matched no version of `{name}`"
        );
        matched
    } else {
        candidates.clone()
    }
}

fn transitive_closure(lock_text: &str, root_crate: &str) -> BTreeSet<String> {
    let graph = parse_lock_packages(lock_text);
    let mut by_name: BTreeMap<String, Vec<PackageId>> = BTreeMap::new();
    for id in graph.keys() {
        by_name.entry(id.name.clone()).or_default().push(id.clone());
    }

    let roots = by_name
        .get(root_crate)
        .unwrap_or_else(|| panic!("Cargo.lock must contain root crate `{root_crate}`"));
    assert!(
        !roots.is_empty(),
        "Cargo.lock must contain at least one `{root_crate}` entry"
    );

    let mut visited: BTreeSet<PackageId> = BTreeSet::new();
    let mut queue: VecDeque<PackageId> = VecDeque::new();
    for root in roots {
        if visited.insert(root.clone()) {
            queue.push_back(root.clone());
        }
    }

    while let Some(current) = queue.pop_front() {
        if let Some(deps) = graph.get(&current) {
            for dep in deps {
                for target in resolve_dep_targets(dep, &by_name) {
                    if visited.insert(target.clone()) {
                        queue.push_back(target);
                    }
                }
            }
        }
    }

    visited.into_iter().map(|id| id.name).collect()
}

fn find_forbidden_in_closure(closure: &BTreeSet<String>) -> Vec<(String, &'static str)> {
    let mut forbidden_map: BTreeMap<String, &'static str> = BTreeMap::new();
    for name in FORBIDDEN_SUGGEST_CRATES {
        forbidden_map.insert(normalize_crate_name(name), "retired suggest-llm crate");
    }
    for name in FORBIDDEN_HTTP_CLIENT_CRATES {
        forbidden_map.insert(normalize_crate_name(name), "HTTP client/transport");
    }
    for name in FORBIDDEN_ML_RUNTIME_CRATES {
        forbidden_map.insert(normalize_crate_name(name), "ML/LLM runtime");
    }

    let mut hits = Vec::new();
    for crate_name in closure {
        let norm = normalize_crate_name(crate_name);
        if let Some(category) = forbidden_map.get(&norm) {
            hits.push((crate_name.clone(), *category));
        }
    }
    hits
}

fn check_workspace_excludes_suggest_llm(root: &Path) {
    assert!(
        !root.join("garnet-suggest-llm").exists(),
        "ADR 0016 (D-42): garnet-suggest-llm directory must be removed"
    );

    let cargo_toml_text =
        fs::read_to_string(root.join("Cargo.toml")).expect("read root Cargo.toml");
    let cargo_toml: toml::Value =
        toml::from_str(&cargo_toml_text).expect("root Cargo.toml must be valid TOML");
    let members = cargo_toml
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(toml::Value::as_array)
        .expect("root Cargo.toml must declare [workspace].members");
    for member in members {
        let member_str = member
            .as_str()
            .expect("workspace member entry must be string");
        assert_ne!(
            normalize_crate_name(member_str),
            "garnet-suggest-llm",
            "ADR 0016 (D-42): garnet-suggest-llm must not be in [workspace].members"
        );
    }
}

#[test]
fn garnet_cli_closure_links_no_http_client_ml_runtime_or_suggest_llm() {
    let root = workspace_root();
    check_workspace_excludes_suggest_llm(&root);

    let lock_text = fs::read_to_string(root.join("Cargo.lock")).expect("read root Cargo.lock");
    let all_packages = parse_lock_packages(&lock_text);
    assert!(
        all_packages
            .keys()
            .all(|id| normalize_crate_name(&id.name) != "garnet-suggest-llm"),
        "ADR 0016 (D-42): garnet-suggest-llm must not remain in Cargo.lock"
    );

    let closure = transitive_closure(&lock_text, "garnet-cli");
    for expected_core in [
        "garnet-cli",
        "garnet-parser",
        "garnet-check",
        "garnet-interp",
        "garnet-vm",
        "garnet-stdlib",
        "garnet-memory",
    ] {
        assert!(
            closure.contains(expected_core),
            "closure walk must reach core workspace crate `{expected_core}`"
        );
    }

    let violations = find_forbidden_in_closure(&closure);
    assert!(
        violations.is_empty(),
        "ADR 0016 (D-42) violation: garnet-cli transitive dependency closure links forbidden crates: {violations:?}"
    );
}

#[test]
fn closure_guard_fails_closed_on_transitive_http_ml_or_suggest_llm() {
    let synthetic_lock = r#"
version = 4

[[package]]
name = "garnet-cli"
version = "0.8.4"
dependencies = [
    "mid-layer 0.1.0",
]

[[package]]
name = "mid-layer"
version = "0.1.0"
dependencies = [
    "reqwest 0.12.0",
    "candle-core 0.9.0",
    "garnet_suggest_llm 0.1.0",
]

[[package]]
name = "reqwest"
version = "0.12.0"

[[package]]
name = "candle-core"
version = "0.9.0"

[[package]]
name = "garnet_suggest_llm"
version = "0.1.0"
"#;

    let closure = transitive_closure(synthetic_lock, "garnet-cli");
    let violations = find_forbidden_in_closure(&closure);
    assert_eq!(
        violations,
        vec![
            ("candle-core".to_string(), "ML/LLM runtime"),
            (
                "garnet_suggest_llm".to_string(),
                "retired suggest-llm crate"
            ),
            ("reqwest".to_string(), "HTTP client/transport"),
        ]
    );
}
