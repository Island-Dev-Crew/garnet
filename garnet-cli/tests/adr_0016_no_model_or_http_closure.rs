//! ADR 0016 (D-42) — Models stay out of the trust path.
//!
//! Proves:
//! 1. `garnet-suggest-llm` is removed from the workspace (`Cargo.toml`, on-disk
//!    directory, and `Cargo.lock`).
//! 2. No crate in `garnet-cli`'s transitive `Cargo.lock` dependency closure
//!    (tracked with full `(name, version, source)` package identity) links
//!    `garnet-suggest-llm`, an HTTP client/transport, an ML/LLM runtime, or
//!    any unclassified crate outside the reviewed allowlist.
//! 3. The closure checker itself fails closed on transitive forbidden crates,
//!    same-name/same-version packages distinguished only by `source`, and
//!    unclassified crates such as `http_req` (without TLS) or future clients.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN_SUGGEST_CRATES: &[&str] = &["garnet-suggest-llm"];

const FORBIDDEN_HTTP_CLIENT_CRATES: &[&str] = &[
    "attohttpc",
    "awc",
    "curl",
    "curl-sys",
    "cyper",
    "ehttp",
    "h2",
    "h3",
    "http",
    "http-body",
    "http-body-util",
    "http-client",
    "http-req",
    "http-types",
    "hyper",
    "hyper-rustls",
    "hyper-tls",
    "hyper-util",
    "isahc",
    "minreq",
    "native-tls",
    "nyquest",
    "openssl",
    "openssl-sys",
    "oxhttp",
    "reqwest",
    "reqwest-eventsource",
    "reqwest-middleware",
    "rquest",
    "rustls",
    "surf",
    "tiny-http",
    "tinyget",
    "tower-http",
    "ureq",
    "wreq",
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
    "llama-cpp",
    "llama-cpp-2",
    "llama-cpp-sys",
    "llama-cpp-sys-2",
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

/// Reviewed, ADR-0016-compliant transitive `Cargo.lock` dependency closure of
/// `garnet-cli` (strictly sorted, normalized by lowercase hyphen form).
///
/// Any newly introduced dependency (including an HTTP client or ML runtime not
/// present in the explicit denylists above, such as `http_req` with default
/// features disabled) fails this test until explicitly reviewed against
/// ADR 0016.
const ALLOWED_GARNET_CLI_CLOSURE: &[&str] = &[
    "addr2line",
    "adler2",
    "aho-corasick",
    "anes",
    "anstyle",
    "anyhow",
    "arrayref",
    "arrayvec",
    "autocfg",
    "backtrace",
    "backtrace-ext",
    "base64ct",
    "bit-set",
    "bit-vec",
    "bitflags",
    "blake3",
    "block-buffer",
    "borrow-or-share",
    "bumpalo",
    "cast",
    "cc",
    "cfg-if",
    "ciborium",
    "ciborium-io",
    "ciborium-ll",
    "clap",
    "clap-builder",
    "clap-lex",
    "const-oid",
    "constant-time-eq",
    "convert-case",
    "countme",
    "cpufeatures",
    "criterion",
    "criterion-plot",
    "crossbeam-deque",
    "crossbeam-epoch",
    "crossbeam-utils",
    "crossterm",
    "crossterm-winapi",
    "crunchy",
    "crypto-common",
    "curve25519-dalek",
    "curve25519-dalek-derive",
    "der",
    "derive-more",
    "derive-more-impl",
    "digest",
    "document-features",
    "ed25519",
    "ed25519-dalek",
    "either",
    "equivalent",
    "errno",
    "fallible-iterator",
    "fallible-streaming-iterator",
    "fastrand",
    "fd-lock",
    "fiat-crypto",
    "find-msvc-tools",
    "fluent-uri",
    "fnv",
    "foldhash",
    "garnet-check",
    "garnet-cli",
    "garnet-convert",
    "garnet-cst",
    "garnet-interp",
    "garnet-memory",
    "garnet-parser",
    "garnet-prim-macros",
    "garnet-registry-stub",
    "garnet-stdlib",
    "garnet-vm",
    "generic-array",
    "getrandom",
    "gimli",
    "half",
    "hashbrown",
    "hashlink",
    "heck",
    "hermit-abi",
    "hmac",
    "id-arena",
    "indexmap",
    "is-ci",
    "is-terminal",
    "itertools",
    "itoa",
    "js-sys",
    "leb128fmt",
    "libc",
    "libsqlite3-sys",
    "linux-raw-sys",
    "litrs",
    "lock-api",
    "log",
    "memchr",
    "miette",
    "miette-derive",
    "miniz-oxide",
    "mio",
    "nu-ansi-term",
    "num-traits",
    "object",
    "once-cell",
    "oorandom",
    "owo-colors",
    "parking-lot",
    "parking-lot-core",
    "pkcs8",
    "pkg-config",
    "plotters",
    "plotters-backend",
    "plotters-svg",
    "ppv-lite86",
    "prettyplease",
    "proc-macro2",
    "proptest",
    "quick-error",
    "quote",
    "r-efi",
    "rand",
    "rand-chacha",
    "rand-core",
    "rand-xorshift",
    "rayon",
    "rayon-core",
    "redox-syscall",
    "reedline",
    "ref-cast",
    "ref-cast-impl",
    "regex",
    "regex-automata",
    "regex-syntax",
    "rowan",
    "rsqlite-vfs",
    "rusqlite",
    "rustc-demangle",
    "rustc-hash",
    "rustc-version",
    "rustix",
    "rustversion",
    "rusty-fork",
    "same-file",
    "scopeguard",
    "semver",
    "serde",
    "serde-core",
    "serde-derive",
    "serde-json",
    "serde-spanned",
    "sha1",
    "sha2",
    "shlex",
    "signal-hook",
    "signal-hook-mio",
    "signal-hook-registry",
    "signature",
    "smallvec",
    "spki",
    "sqlite-wasm-rs",
    "strip-ansi-escapes",
    "strum",
    "strum-macros",
    "subtle",
    "supports-color",
    "supports-hyperlinks",
    "supports-unicode",
    "syn",
    "tempfile",
    "terminal-size",
    "text-size",
    "textwrap",
    "thiserror",
    "thiserror-impl",
    "tinytemplate",
    "toml",
    "toml-datetime",
    "toml-edit",
    "toml-write",
    "typenum",
    "unarray",
    "unicode-ident",
    "unicode-linebreak",
    "unicode-segmentation",
    "unicode-width",
    "unicode-xid",
    "vcpkg",
    "version-check",
    "vte",
    "wait-timeout",
    "walkdir",
    "wasi",
    "wasip2",
    "wasip3",
    "wasm-bindgen",
    "wasm-bindgen-macro",
    "wasm-bindgen-macro-support",
    "wasm-bindgen-shared",
    "wasm-encoder",
    "wasm-metadata",
    "wasmparser",
    "web-sys",
    "winapi",
    "winapi-i686-pc-windows-gnu",
    "winapi-util",
    "winapi-x86-64-pc-windows-gnu",
    "windows-aarch64-gnullvm",
    "windows-aarch64-msvc",
    "windows-i686-gnu",
    "windows-i686-gnullvm",
    "windows-i686-msvc",
    "windows-link",
    "windows-sys",
    "windows-targets",
    "windows-x86-64-gnu",
    "windows-x86-64-gnullvm",
    "windows-x86-64-msvc",
    "winnow",
    "wit-bindgen",
    "wit-bindgen-core",
    "wit-bindgen-rust",
    "wit-bindgen-rust-macro",
    "wit-component",
    "wit-parser",
    "zerocopy",
    "zerocopy-derive",
    "zeroize",
    "zmij",
];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PackageId {
    name: String,
    version: String,
    source: Option<String>,
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
        let source = pkg.get("source").map(|v| {
            v.as_str()
                .expect("Cargo.lock package source must be string")
                .to_string()
        });
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
        let id = PackageId {
            name,
            version,
            source,
        };
        assert!(
            graph.insert(id.clone(), deps).is_none(),
            "Cargo.lock must not contain duplicate package identity {id:?}"
        );
    }
    graph
}

fn parse_dep_spec(dep_spec: &str) -> (&str, Option<&str>, Option<&str>) {
    // Cargo.lock dependency entries are formatted as:
    //   "name"
    //   "name version"
    //   "name version (source)"
    let (prefix, source) = if let Some((before, after)) = dep_spec.split_once('(') {
        let src = after
            .strip_suffix(')')
            .unwrap_or_else(|| {
                panic!("Cargo.lock dependency `{dep_spec}` has malformed source qualifier")
            })
            .trim();
        assert!(
            !src.is_empty(),
            "Cargo.lock dependency `{dep_spec}` has empty source qualifier"
        );
        (before.trim_end(), Some(src))
    } else {
        (dep_spec.trim(), None)
    };

    let parts: Vec<&str> = prefix.split_whitespace().collect();
    match parts.as_slice() {
        [name] => {
            assert!(
                source.is_none(),
                "Cargo.lock dependency `{dep_spec}` specifies source without version"
            );
            (*name, None, None)
        }
        [name, version] => (*name, Some(*version), source),
        _ => panic!("Cargo.lock dependency `{dep_spec}` has unexpected format"),
    }
}

fn resolve_dep_targets(
    dep_spec: &str,
    by_name: &BTreeMap<String, Vec<PackageId>>,
) -> Vec<PackageId> {
    let (name, version, source) = parse_dep_spec(dep_spec);

    let candidates = by_name.get(name).unwrap_or_else(|| {
        panic!("Cargo.lock dependency `{dep_spec}` references unknown package `{name}`")
    });

    let matched: Vec<PackageId> = candidates
        .iter()
        .filter(|id| version.is_none_or(|ver| id.version == ver))
        .filter(|id| source.is_none_or(|src| id.source.as_deref() == Some(src)))
        .cloned()
        .collect();

    assert!(
        !matched.is_empty(),
        "Cargo.lock dependency `{dep_spec}` matched no entry for `{name}`"
    );
    matched
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

fn forbidden_crate_map() -> BTreeMap<String, &'static str> {
    let mut map: BTreeMap<String, &'static str> = BTreeMap::new();
    for name in FORBIDDEN_SUGGEST_CRATES {
        map.insert(normalize_crate_name(name), "retired suggest-llm crate");
    }
    for name in FORBIDDEN_HTTP_CLIENT_CRATES {
        map.insert(normalize_crate_name(name), "HTTP client/transport");
    }
    for name in FORBIDDEN_ML_RUNTIME_CRATES {
        map.insert(normalize_crate_name(name), "ML/LLM runtime");
    }
    map
}

fn find_forbidden_in_closure(closure: &BTreeSet<String>) -> Vec<(String, &'static str)> {
    let forbidden_map = forbidden_crate_map();
    let mut hits = Vec::new();
    for crate_name in closure {
        let norm = normalize_crate_name(crate_name);
        if let Some(category) = forbidden_map.get(&norm) {
            hits.push((crate_name.clone(), *category));
        }
    }
    hits
}

fn find_unclassified_in_closure(closure: &BTreeSet<String>) -> Vec<String> {
    let allowed: BTreeSet<String> = ALLOWED_GARNET_CLI_CLOSURE
        .iter()
        .map(|name| normalize_crate_name(name))
        .collect();
    closure
        .iter()
        .filter(|name| !allowed.contains(&normalize_crate_name(name)))
        .cloned()
        .collect()
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
fn reviewed_allowlist_is_sorted_unique_and_disjoint_from_denylist() {
    let forbidden = forbidden_crate_map();
    let mut prev: Option<&str> = None;
    for &entry in ALLOWED_GARNET_CLI_CLOSURE {
        assert_eq!(
            entry,
            normalize_crate_name(entry),
            "ALLOWED_GARNET_CLI_CLOSURE entries must use normalized lowercase hyphen form: `{entry}`"
        );
        if let Some(p) = prev {
            assert!(
                p < entry,
                "ALLOWED_GARNET_CLI_CLOSURE must be strictly sorted and unique: `{p}` before `{entry}`"
            );
        }
        assert!(
            !forbidden.contains_key(entry),
            "ALLOWED_GARNET_CLI_CLOSURE must never allow forbidden crate `{entry}`"
        );
        prev = Some(entry);
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

    let unclassified = find_unclassified_in_closure(&closure);
    assert!(
        unclassified.is_empty(),
        "ADR 0016 (D-42) fail-closed review gate: unclassified crate(s) entered garnet-cli's transitive closure: {unclassified:?}. Review against ADR 0016 before updating ALLOWED_GARNET_CLI_CLOSURE."
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

#[test]
fn closure_guard_preserves_source_identity_across_duplicate_name_and_version() {
    // Regression for same-(name, version) packages from different sources:
    // regardless of lockfile entry order, a registry entry without deps must
    // never overwrite a git entry that pulls in a forbidden HTTP client.
    let git_first = r#"
version = 4

[[package]]
name = "garnet-cli"
version = "0.8.4"
dependencies = [
    "middle 1.0.0 (git+https://example.invalid/middle#deadbeef)",
]

[[package]]
name = "middle"
version = "1.0.0"
source = "git+https://example.invalid/middle#deadbeef"
dependencies = [
    "reqwest 0.12.0",
]

[[package]]
name = "middle"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "reqwest"
version = "0.12.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#;

    let registry_first = r#"
version = 4

[[package]]
name = "garnet-cli"
version = "0.8.4"
dependencies = [
    "middle 1.0.0 (git+https://example.invalid/middle#deadbeef)",
]

[[package]]
name = "middle"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "middle"
version = "1.0.0"
source = "git+https://example.invalid/middle#deadbeef"
dependencies = [
    "reqwest 0.12.0",
]

[[package]]
name = "reqwest"
version = "0.12.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#;

    for lock_text in [git_first, registry_first] {
        let closure = transitive_closure(lock_text, "garnet-cli");
        let violations = find_forbidden_in_closure(&closure);
        assert_eq!(
            violations,
            vec![("reqwest".to_string(), "HTTP client/transport")]
        );
    }
}

#[test]
fn closure_guard_blocks_http_req_and_unclassified_novel_crates() {
    // Regression for no-TLS HTTP clients (e.g. `http_req` with default-features=false)
    // and any future unlisted HTTP client or ML runtime: both explicit denylist
    // and fail-closed unclassified-crate review catch them.
    let http_req_lock = r#"
version = 4

[[package]]
name = "garnet-cli"
version = "0.8.4"
dependencies = [
    "http_req 0.14.6",
]

[[package]]
name = "http_req"
version = "0.14.6"
dependencies = [
    "unicase 2.8.0",
]

[[package]]
name = "unicase"
version = "2.8.0"
"#;

    let closure = transitive_closure(http_req_lock, "garnet-cli");
    assert_eq!(
        find_forbidden_in_closure(&closure),
        vec![("http_req".to_string(), "HTTP client/transport")]
    );
    assert_eq!(
        find_unclassified_in_closure(&closure),
        vec!["http_req".to_string(), "unicase".to_string()]
    );

    let novel_client_lock = r#"
version = 4

[[package]]
name = "garnet-cli"
version = "0.8.4"
dependencies = [
    "novel-http-client-xyz 0.1.0",
]

[[package]]
name = "novel-http-client-xyz"
version = "0.1.0"
"#;

    let novel_closure = transitive_closure(novel_client_lock, "garnet-cli");
    assert_eq!(
        find_unclassified_in_closure(&novel_closure),
        vec!["novel-http-client-xyz".to_string()]
    );
}
