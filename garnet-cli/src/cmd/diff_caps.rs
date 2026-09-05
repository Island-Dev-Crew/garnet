//! `garnet diff-caps [--machine] <old-path> <new-path>` — S37, the headline
//! novelty.
//!
//! Diffs the declared capability surface (S35) between two revisions and **gates
//! on authority changes**: exits non-zero iff the program GAINED authority (a new
//! aggregate capability or an introduced `@caps(*)` wildcard). Each path is a
//! `.garnet` file or a directory (per-package). "Two revisions" are two source
//! paths the caller supplies (e.g. two git checkouts / worktrees).
//!
//! RB-1 (Directive 15): `--machine` emits the same verdict as deterministic
//! single-line JSON for agent reviewers — the reviewer on the other side of
//! the gate is increasingly an agent. Exit codes are identical in both
//! modes; the human text output is byte-for-byte unchanged when `--machine`
//! is absent. On usage/parse errors (exit 2) NO machine JSON is emitted:
//! stdout stays empty and the human-format error goes to stderr in both
//! modes — an agent consumer must treat exit 2 + empty stdout as fatal.
//!
//! Honest scope (both modes): diff-caps reads the DECLARED surface; it does
//! not prove the absence of undeclared authority (that is the sandbox-policy
//! job, S46). Bound annotations (`@bounded`, `@max_depth`, `@mailbox`) are
//! not part of the declared-caps surface, so no bounds delta is claimed.
//!
//! Crown C B-1: the `scope` string above does NOT cover a third case — a
//! `.garnet` file the walk never opened. Its authority is *declared*, just
//! unread, so no scope caveat about *undeclared* authority applies to it.
//! Both modes therefore disclose what the walk skipped: `--machine` adds
//! `skipped_path_count` + `skipped_paths` (rule names and counts, no paths),
//! and the human mode prints a `walk not total` line when the count is
//! non-zero. `skipped_path_count: 0` is the claim "every directory this walk
//! reached was read or tallied" — a directory symlink met below the supplied
//! root is not followed and is tallied under `symlinked-directory`; a link
//! the walk cannot resolve is an error with no verdict at all (exit 2). The
//! ABSENCE of the field means the verdict came from a pre-cure binary and
//! the walk's coverage is UNKNOWN — a consumer must not read absence as zero.

use crate::cap_manifest::{json_str_array, surface_for_path_with_omissions};
use crate::cmd::verify_gate::ScanOmissions;
use crate::diagnostics::json_escape;
use garnet_check::{diff_caps, CapsDiff};
use std::path::PathBuf;
use std::process::ExitCode;

/// Render the machine verdict as deterministic single-line JSON. Field
/// order is fixed; every list arrives sorted from [`CapsDiff`] and
/// [`ScanOmissions`]. Additive within `garnet.diff-caps.machine/1` — no
/// existing key changes shape, name, or value.
fn machine_json(diff: &CapsDiff, omissions: &ScanOmissions) -> String {
    let (verdict, band, exit_code) = if diff.authority_expanded() {
        ("authority-expanded", "2/5", 1)
    } else {
        ("no-authority-expansion", "5/5", 0)
    };
    let expanded = diff
        .functions_caps_expanded
        .iter()
        .map(|(name, gained)| {
            format!(
                "{{\"name\":\"{}\",\"gained\":{}}}",
                json_escape(name),
                json_str_array(gained)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let skipped = omissions
        .by_rule()
        .iter()
        .map(|(rule, count)| format!("{{\"rule\":\"{}\",\"count\":{count}}}", json_escape(rule)))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"schema\":\"garnet.diff-caps.machine/1\",\
         \"verdict\":\"{verdict}\",\
         \"authority_expanded\":{},\
         \"capability_band\":\"{band}\",\
         \"exit_code\":{exit_code},\
         \"aggregate_gained\":{},\
         \"aggregate_removed\":{},\
         \"wildcard_introduced\":{},\
         \"functions_added\":{},\
         \"functions_removed\":{},\
         \"functions_caps_expanded\":[{expanded}],\
         \"skipped_path_count\":{},\
         \"skipped_paths\":[{skipped}],\
         \"scope\":\"declared-surface-only; does not prove absence of undeclared authority; bound annotations are not part of this surface\"}}",
        diff.authority_expanded(),
        json_str_array(&diff.aggregate_added),
        json_str_array(&diff.aggregate_removed),
        diff.wildcard_introduced,
        json_str_array(&diff.functions_added),
        json_str_array(&diff.functions_removed),
        omissions.total(),
    )
}

pub fn run(old: PathBuf, new: PathBuf, machine: bool) -> ExitCode {
    // The verdict covers BOTH walks, so the omission tallies are merged.
    let mut omissions = ScanOmissions::default();
    let old_surface = match surface_for_path_with_omissions(&old) {
        Ok((s, skipped)) => {
            omissions.merge(&skipped);
            s
        }
        Err(message) => {
            eprintln!("garnet diff-caps: old `{}`: {message}", old.display());
            return ExitCode::from(2);
        }
    };
    let new_surface = match surface_for_path_with_omissions(&new) {
        Ok((s, skipped)) => {
            omissions.merge(&skipped);
            s
        }
        Err(message) => {
            eprintln!("garnet diff-caps: new `{}`: {message}", new.display());
            return ExitCode::from(2);
        }
    };
    let diff = diff_caps(&old_surface, &new_surface);

    if machine {
        println!("{}", machine_json(&diff, &omissions));
        return if diff.authority_expanded() {
            ExitCode::from(1)
        } else {
            ExitCode::SUCCESS
        };
    }

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

    // Byte-stable when the walk WAS total (the overwhelming case, and the one
    // the golden human-output test pins): this line is printed only when
    // something went unread.
    if omissions.total() > 0 {
        let detail = omissions
            .by_rule()
            .iter()
            .map(|(rule, count)| format!("{rule}: {count}"))
            .collect::<Vec<_>>()
            .join(", ");
        let noun = if omissions.total() == 1 {
            "directory"
        } else {
            "directories"
        };
        println!(
            "  ! walk not total: {} {noun} skipped ({detail}); \
             authority declared under a skipped path is NOT in this diff",
            omissions.total()
        );
    }

    if diff.authority_expanded() {
        println!("\ndiff-caps: AUTHORITY EXPANDED — review required (capability band 2/5)");
        ExitCode::from(1)
    } else {
        println!("\ndiff-caps: no authority expansion (capability band 5/5)");
        ExitCode::SUCCESS
    }
}
