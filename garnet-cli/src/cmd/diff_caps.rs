//! `garnet diff-caps <old-path> <new-path>` — S37, the headline novelty.
//!
//! Diffs the declared capability surface (S35) between two revisions and **gates
//! on authority changes**: exits non-zero iff the program GAINED authority (a new
//! aggregate capability or an introduced `@caps(*)` wildcard). Each path is a
//! `.garnet` file or a directory (per-package). "Two revisions" are two source
//! paths the caller supplies (e.g. two git checkouts / worktrees).
//!
//! Honest scope: diff-caps reads the DECLARED surface; it does not prove the
//! absence of undeclared authority (that is the sandbox-policy job, S46).

use crate::cap_manifest::surface_for_path;
use garnet_check::diff_caps;
use std::path::PathBuf;
use std::process::ExitCode;

pub fn run(old: PathBuf, new: PathBuf) -> ExitCode {
    let old_surface = match surface_for_path(&old) {
        Ok(s) => s,
        Err(message) => {
            eprintln!("garnet diff-caps: old `{}`: {message}", old.display());
            return ExitCode::from(2);
        }
    };
    let new_surface = match surface_for_path(&new) {
        Ok(s) => s,
        Err(message) => {
            eprintln!("garnet diff-caps: new `{}`: {message}", new.display());
            return ExitCode::from(2);
        }
    };
    let diff = diff_caps(&old_surface, &new_surface);

    println!("garnet diff-caps: {} -> {}", old.display(), new.display());
    if diff.is_empty() {
        println!("  no capability changes.");
    } else {
        if !diff.aggregate_added.is_empty() {
            println!("  + caps GAINED:  {}", diff.aggregate_added.join(", "));
        }
        if !diff.aggregate_removed.is_empty() {
            println!("  - caps removed: {}", diff.aggregate_removed.join(", "));
        }
        if diff.wildcard_introduced {
            println!("  ! wildcard @caps(*) introduced");
        }
        if !diff.functions_added.is_empty() {
            println!("  + functions:    {}", diff.functions_added.join(", "));
        }
        if !diff.functions_removed.is_empty() {
            println!("  - functions:    {}", diff.functions_removed.join(", "));
        }
        for (name, gained) in &diff.functions_caps_expanded {
            println!("  ~ {name} gained: {}", gained.join(", "));
        }
    }

    if diff.authority_expanded() {
        println!("\ndiff-caps: AUTHORITY EXPANDED — review required (capability band 2/5)");
        ExitCode::from(1)
    } else {
        println!("\ndiff-caps: no authority expansion (capability band 5/5)");
        ExitCode::SUCCESS
    }
}
