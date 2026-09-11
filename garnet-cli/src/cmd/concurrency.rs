//! `garnet concurrency <file.garnet>` — actor concurrency-contract report (S41).
//!
//! Surfaces each actor's message protocols (ask vs tell) and handler count — the
//! concurrency contract codified in
//! `C_Language_Specification/GARNET_CONCURRENCY_CONTRACT.md`.

use crate::{edition_manifest, read_file};
use garnet_check::concurrency_surface;
use std::path::PathBuf;
use std::process::ExitCode;

pub fn run(path: PathBuf) -> ExitCode {
    let src = match read_file(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("garnet concurrency: {e}");
            return ExitCode::from(1);
        }
    };
    let edition = match edition_manifest::resolve_edition_for(&path) {
        Ok(resolved) => {
            if let Some(warning) = resolved.warning {
                eprintln!("{warning}");
            }
            resolved.edition
        }
        Err(message) => {
            eprintln!("garnet concurrency: {message}");
            return ExitCode::from(1);
        }
    };
    let module = match garnet_parser::parse_source_with_edition(&src, edition) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("garnet concurrency: parse error: {e}");
            return ExitCode::from(1);
        }
    };

    let actors = concurrency_surface(&module);
    println!("garnet concurrency for {}", path.display());
    if actors.is_empty() {
        println!("  no actors declared (no concurrency-contract surface).");
    } else {
        for a in &actors {
            println!(
                "  actor {}: {} handler(s), {} protocol(s)",
                a.name,
                a.handlers,
                a.protocols.len()
            );
            for p in &a.protocols {
                let kind = if p.reply {
                    "ask  (request-reply, Result-returning)"
                } else {
                    "tell (fire-and-forget)"
                };
                println!("    - {}/{}: {kind}", p.name, p.arity);
            }
        }
    }
    println!(
        "\nGarnet concurrency model: ACTORS (not async/await — `async` is reserved for a future \
         edition). In this CLI, actors run inside the interpreter: each one has a BOUNDED \
         mailbox of 1024 messages unless `Actor.spawn(capacity)` sets another size, and a \
         `tell` to a full mailbox fails. The @mailbox(N) annotation is range-checked by \
         `garnet check` but does not set the capacity. The thread-per-actor runtime is the \
         separate garnet-actor-runtime crate, which this binary does not include. See \
         C_Language_Specification/GARNET_CONCURRENCY_CONTRACT.md."
    );
    ExitCode::SUCCESS
}
