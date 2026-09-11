//! The shipped man page (`garnet-cli/man/garnet.1`, installed by the .deb and
//! .rpm at /usr/share/man/man1/garnet.1) documents exactly the subcommands
//! `garnet --help` lists, and its header names no release.

use std::collections::BTreeSet;
use std::process::Command;

const MAN_PAGE: &str = include_str!("../man/garnet.1");

/// Subcommand names from `garnet --help`. Entries are indented exactly four
/// spaces; continuation lines are indented further and skipped.
fn help_subcommands() -> BTreeSet<String> {
    let out = Command::new(env!("CARGO_BIN_EXE_garnet"))
        .arg("--help")
        .output()
        .expect("run garnet --help");
    assert!(
        out.status.success(),
        "garnet --help exited {:?}",
        out.status
    );
    let text = String::from_utf8(out.stdout).expect("help output is UTF-8");
    let mut in_list = false;
    let mut names = BTreeSet::new();
    for line in text.lines() {
        if line == "SUBCOMMANDS:" {
            in_list = true;
            continue;
        }
        if !in_list {
            continue;
        }
        if let Some(rest) = line.strip_prefix("    ") {
            if rest.starts_with(|c: char| c.is_ascii_lowercase()) {
                names.insert(rest.split_whitespace().next().unwrap().to_string());
            }
        }
    }
    names
}

/// Subcommand names tagged in the man page: the first word of each `.B` line
/// that follows a `.TP` inside the SUBCOMMANDS section, with roff's `\-`
/// read as `-`.
fn man_subcommands() -> BTreeSet<String> {
    let mut in_section = false;
    let mut after_tp = false;
    let mut names = BTreeSet::new();
    for line in MAN_PAGE.lines() {
        if let Some(title) = line.strip_prefix(".SH ") {
            in_section = title.trim() == "SUBCOMMANDS";
            after_tp = false;
            continue;
        }
        if !in_section {
            continue;
        }
        if line.starts_with(".TP") {
            after_tp = true;
            continue;
        }
        if after_tp {
            if let Some(rest) = line.strip_prefix(".B ") {
                let tag = rest
                    .split_whitespace()
                    .next()
                    .expect("tag names a subcommand");
                names.insert(tag.replace("\\-", "-"));
            }
            after_tp = false;
        }
    }
    names
}

#[test]
fn man_page_lists_exactly_the_help_subcommands() {
    let help = help_subcommands();
    assert!(
        help.len() >= 20,
        "parsed too few subcommands from --help: {help:?}"
    );
    let man = man_subcommands();
    let missing: Vec<_> = help.difference(&man).collect();
    let extra: Vec<_> = man.difference(&help).collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "man/garnet.1 drifted from `garnet --help`: missing {missing:?}; not in --help {extra:?}"
    );
}

#[test]
fn man_page_header_names_no_release() {
    let th = MAN_PAGE
        .lines()
        .find(|l| l.starts_with(".TH "))
        .expect("man page has a .TH header");
    let has_version = th
        .as_bytes()
        .windows(3)
        .any(|w| w[0].is_ascii_digit() && w[1] == b'.' && w[2].is_ascii_digit());
    assert!(
        !has_version,
        "the .TH header must not carry a release number (`garnet --version` reports it): {th}"
    );
}
