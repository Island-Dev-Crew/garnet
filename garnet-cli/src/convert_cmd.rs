//! `garnet convert <lang> <file>` subcommand wiring.
//!
//! Phase 5F integration: reads the source file, runs the v4.1 converter
//! pipeline, and writes `<stem>.<lang>.garnet` plus its `.lineage.json`,
//! `.migrate_todo.md` and `.metrics.json`. The language is part of the name, so
//! converting `sample.py` and `sample.rb` side by side keeps both (C1-18).

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
    /// Checked with the Garnet parser after writing; `run` fails otherwise.
    pub output_parses: bool,
}

pub fn run(args: ConvertArgs) -> Result<ConvertOutcome, String> {
    let lang = if args.source_lang.trim().is_empty() {
        args.source_path
            .extension()
            .and_then(|e| e.to_str())
            .and_then(SourceLang::from_extension)
    } else {
        args.source_lang.parse::<SourceLang>().ok()
    }
    .ok_or_else(|| {
        format!(
            "unknown source language: {} (recognised: rust/rs, ruby/rb, python/py, go)",
            args.source_lang
        )
    })?;

    let source = fs::read_to_string(&args.source_path)
        .map_err(|e| format!("read {}: {e}", args.source_path.display()))?;

    let source_loc = source.lines().count();
    let out_dir = args.out_dir.clone().unwrap_or_else(|| {
        args.source_path
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf()
    });
    let basename = args
        .source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("converted")
        .to_string();

    let stem = format!("{basename}.{}", lang.as_str());
    let target_path = out_dir.join(format!("{stem}.garnet"));
    let lineage_path = out_dir.join(format!("{stem}.garnet.lineage.json"));
    let migrate_todo_path = out_dir.join(format!("{stem}.garnet.migrate_todo.md"));
    let metrics_path = out_dir.join(format!("{stem}.garnet.metrics.json"));

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
    let outputs = [
        &target_path,
        &lineage_path,
        &migrate_todo_path,
        &metrics_path,
    ];
    refuse_linked_outputs(&outputs)?;
    fs::write(&target_path, &emitted.garnet).map_err(|e| format!("write garnet: {e}"))?;
    fs::write(&lineage_path, &emitted.lineage_json).map_err(|e| format!("write lineage: {e}"))?;
    fs::write(&migrate_todo_path, &emitted.migrate_todo_md)
        .map_err(|e| format!("write migrate_todo: {e}"))?;
    fs::write(&metrics_path, metrics.to_json()).map_err(|e| format!("write metrics: {e}"))?;

    // The converter already refuses to emit text that does not parse. The
    // "output parses" line is backed by the file as it is on disk after every
    // output was written: it must read back unchanged, and it must parse.
    let written = fs::read_to_string(&target_path)
        .map_err(|e| format!("read back {}: {e}", target_path.display()))?;
    if written != emitted.garnet {
        return Err(format!(
            "{} changed after it was written; another output shares the file",
            target_path.display()
        ));
    }
    garnet_parser::parse_source(&written).map_err(|e| {
        format!(
            "converter bug: {} does not parse as Garnet: {e:?}",
            target_path.display()
        )
    })?;

    let outcome = ConvertOutcome {
        target_path: target_path.clone(),
        lineage_path,
        migrate_todo_path,
        metrics_path,
        total_nodes: metrics.total_cir_nodes,
        migrate_todo_count: metrics.migrate_todo_count,
        untranslatable_count: metrics.untranslatable_count,
        output_parses: true,
    };

    if !args.quiet {
        render_summary(&outcome);
    }

    Ok(outcome)
}

fn render_summary(o: &ConvertOutcome) {
    println!(
        "converted: {} (unreviewed: an @sandbox comment and an empty @caps() mark it)",
        o.target_path.display()
    );
    let mapped = o
        .total_nodes
        .saturating_sub(o.migrate_todo_count + o.untranslatable_count);
    println!(
        "  - {mapped} of {} constructs mapped without a migration to-do",
        o.total_nodes
    );
    println!("  - {} @migrate_todo annotations", o.migrate_todo_count);
    println!("  - {} @untranslatable constructs", o.untranslatable_count);
    println!(
        "  - output parses: {}",
        if o.output_parses { "yes" } else { "no" }
    );
    println!("  - lineage: {}", o.lineage_path.display());
    println!("  - checklist: {}", o.migrate_todo_path.display());
    println!("  - metrics: {}", o.metrics_path.display());
    println!();
    println!("  parsing is not correctness: review the file, resolve each @migrate_todo,");
    println!("  declare the @caps(...) it needs, then run garnet check.");
}

/// Refuse to write through an output path that already exists as anything but a
/// regular file of its own: a symlink, a directory, or a file hard-linked to
/// another output would let one output overwrite another after it was written.
/// The read-back after writing catches any sharing this check cannot see.
fn refuse_linked_outputs(outputs: &[&PathBuf]) -> Result<(), String> {
    #[cfg(unix)]
    let mut seen: Vec<(u64, u64)> = Vec::new();
    for path in outputs {
        let meta = match fs::symlink_metadata(path) {
            Ok(meta) => meta,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(format!("cannot inspect {}: {e}", path.display())),
        };
        if !meta.file_type().is_file() {
            return Err(format!(
                "refusing to write {}: it exists and is not a regular file",
                path.display()
            ));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let id = (meta.dev(), meta.ino());
            if seen.contains(&id) {
                return Err(format!(
                    "refusing to write {}: it is the same file as another output",
                    path.display()
                ));
            }
            seen.push(id);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

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
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let seq = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("{}-{nanos:x}-{seq:x}", std::process::id())
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
    fn explicit_unknown_language_does_not_fallback_to_extension() {
        let path = write_temp("py", "def f(x):\n    return x\n");
        let r = run(ConvertArgs {
            source_lang: "javascript".into(),
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
