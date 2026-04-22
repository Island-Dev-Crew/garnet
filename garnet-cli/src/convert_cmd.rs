//! `garnet convert <lang> <file>` subcommand wiring.
//!
//! Phase 5F integration: reads the source file, runs the v4.1 converter
//! pipeline, writes `<file>.garnet` + `.lineage.json` + `.migrate_todo.md`.

use garnet_convert::{convert, EmitOpts, SourceLang};
use std::fs;
use std::path::{Path, PathBuf};

pub struct ConvertArgs {
    pub source_lang: String,
    pub source_path: PathBuf,
    pub strict: bool,
    pub fail_on_todo: bool,
    pub fail_on_untranslatable: bool,
    pub out_dir: Option<PathBuf>,
    pub quiet: bool,
}

pub struct ConvertOutcome {
    pub target_path: PathBuf,
    pub lineage_path: PathBuf,
    pub migrate_todo_path: PathBuf,
    pub metrics_path: PathBuf,
    pub total_nodes: usize,
    pub migrate_todo_count: usize,
    pub untranslatable_count: usize,
    pub clean_percent: f64,
}

pub fn run(args: ConvertArgs) -> Result<ConvertOutcome, String> {
    let lang = SourceLang::from_str(&args.source_lang)
        .or_else(|| {
            args.source_path
                .extension()
                .and_then(|e| e.to_str())
                .and_then(SourceLang::from_extension)
        })
        .ok_or_else(|| {
            format!(
                "unknown source language: {} (recognised: rust/rs, ruby/rb, python/py, go)",
                args.source_lang
            )
        })?;

    let source = fs::read_to_string(&args.source_path)
        .map_err(|e| format!("read {}: {e}", args.source_path.display()))?;

    let source_loc = source.lines().count();
    let out_dir = args
        .out_dir
        .clone()
        .unwrap_or_else(|| args.source_path.parent().unwrap_or(Path::new(".")).to_path_buf());
    let basename = args
        .source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("converted")
        .to_string();

    let target_path = out_dir.join(format!("{basename}.garnet"));
    let lineage_path = out_dir.join(format!("{basename}.garnet.lineage.json"));
    let migrate_todo_path = out_dir.join(format!("{basename}.garnet.migrate_todo.md"));
    let metrics_path = out_dir.join(format!("{basename}.garnet.metrics.json"));

    let opts = EmitOpts {
        source_lang: lang.as_str().to_string(),
        source_file: args.source_path.to_string_lossy().into_owned(),
        target_file: target_path.to_string_lossy().into_owned(),
        source_loc,
        strict: args.strict,
        fail_on_todo: args.fail_on_todo,
        fail_on_untranslatable: args.fail_on_untranslatable,
    };

    let (emitted, metrics) = convert(
        &source,
        lang,
        args.source_path.to_str().unwrap_or("?"),
        opts,
    )
    .map_err(|e| e.to_string())?;

    fs::create_dir_all(&out_dir).map_err(|e| format!("create out dir: {e}"))?;
    fs::write(&target_path, &emitted.garnet).map_err(|e| format!("write garnet: {e}"))?;
    fs::write(&lineage_path, &emitted.lineage_json).map_err(|e| format!("write lineage: {e}"))?;
    fs::write(&migrate_todo_path, &emitted.migrate_todo_md)
        .map_err(|e| format!("write migrate_todo: {e}"))?;
    fs::write(&metrics_path, metrics.to_json()).map_err(|e| format!("write metrics: {e}"))?;

    let outcome = ConvertOutcome {
        target_path: target_path.clone(),
        lineage_path,
        migrate_todo_path,
        metrics_path,
        total_nodes: metrics.total_cir_nodes,
        migrate_todo_count: metrics.migrate_todo_count,
        untranslatable_count: metrics.untranslatable_count,
        clean_percent: metrics.clean_translation_percent(),
    };

    if !args.quiet {
        render_summary(&outcome);
    }

    Ok(outcome)
}

fn render_summary(o: &ConvertOutcome) {
    println!("converted: {} (sandboxed)", o.target_path.display());
    println!("  - {} CIR nodes emitted", o.total_nodes);
    println!("  - {} @migrate_todo annotations", o.migrate_todo_count);
    println!("  - {} @untranslatable constructs", o.untranslatable_count);
    println!("  - {:.1}% clean translation", o.clean_percent);
    println!("  - lineage: {}", o.lineage_path.display());
    println!("  - checklist: {}", o.migrate_todo_path.display());
    println!("  - metrics: {}", o.metrics_path.display());
    println!();
    println!("  review the file then change @sandbox to @sandbox(unquarantine)");
    println!("  and add @caps(...) based on your audit.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp(ext: &str, contents: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("garnet-convert-test-{}", rand_suffix()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("input.{ext}"));
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(contents.as_bytes()).unwrap();
        path
    }

    fn rand_suffix() -> String {
        use std::time::SystemTime;
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        format!("{nanos:x}")
    }

    #[test]
    fn convert_rust_file_writes_four_artifacts() {
        let path = write_temp("rs", "fn greet(name: String) -> String { return name; }\n");
        let outcome = run(ConvertArgs {
            source_lang: "rust".into(),
            source_path: path.clone(),
            strict: false,
            fail_on_todo: false,
            fail_on_untranslatable: false,
            out_dir: None,
            quiet: true,
        })
        .unwrap();
        assert!(outcome.target_path.exists());
        assert!(outcome.lineage_path.exists());
        assert!(outcome.migrate_todo_path.exists());
        assert!(outcome.metrics_path.exists());
        let garnet = fs::read_to_string(&outcome.target_path).unwrap();
        assert!(garnet.contains("@sandbox"));
        assert!(garnet.contains("fn greet"));
    }

    #[test]
    fn convert_python_file_infers_language_from_extension() {
        let path = write_temp("py", "def f(x: int) -> int:\n    return x\n");
        let outcome = run(ConvertArgs {
            source_lang: String::new(), // empty; infer from .py
            source_path: path,
            strict: false,
            fail_on_todo: false,
            fail_on_untranslatable: false,
            out_dir: None,
            quiet: true,
        })
        .unwrap();
        assert!(outcome.target_path.exists());
    }

    #[test]
    fn convert_strict_mode_rejects_ruby_method_missing() {
        let path = write_temp("rb", "method_missing x\n");
        let r = run(ConvertArgs {
            source_lang: "ruby".into(),
            source_path: path,
            strict: false,
            fail_on_todo: true,
            fail_on_untranslatable: false,
            out_dir: None,
            quiet: true,
        });
        assert!(r.is_err());
    }

    #[test]
    fn unknown_language_rejected() {
        let path = write_temp("xyz", "garbage\n");
        let r = run(ConvertArgs {
            source_lang: "klingon".into(),
            source_path: path,
            strict: false,
            fail_on_todo: false,
            fail_on_untranslatable: false,
            out_dir: None,
            quiet: true,
        });
        assert!(r.is_err());
    }

    #[test]
    fn out_dir_override() {
        let path = write_temp("rs", "fn f() { return 0; }\n");
        let out_dir = std::env::temp_dir().join(format!("garnet-out-{}", rand_suffix()));
        fs::create_dir_all(&out_dir).unwrap();
        let outcome = run(ConvertArgs {
            source_lang: "rust".into(),
            source_path: path,
            strict: false,
            fail_on_todo: false,
            fail_on_untranslatable: false,
            out_dir: Some(out_dir.clone()),
            quiet: true,
        })
        .unwrap();
        assert!(outcome.target_path.starts_with(&out_dir));
    }
}
