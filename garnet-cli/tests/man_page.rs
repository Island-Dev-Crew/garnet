//! The shipped man page (`garnet-cli/man/garnet.1`, installed by the .deb and
//! .rpm at /usr/share/man/man1/garnet.1) documents exactly the subcommand
//! entries `garnet --help` lists, and its header names no release.

use std::process::Command;

const MAN_PAGE: &str = include_str!("../man/garnet.1");

/// Subcommand entries from `garnet --help`, sorted, one per entry (so the
/// two `verify` forms count twice). Entries are indented exactly four
/// spaces; continuation lines are indented further and skipped.
fn help_subcommands() -> Vec<String> {
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
    let mut names = Vec::new();
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
                names.push(rest.split_whitespace().next().unwrap().to_string());
            }
        }
    }
    names.sort();
    names
}

/// Subcommand entries tagged in the man page, sorted: the first word of each
/// `.B` line that follows a `.TP` inside the SUBCOMMANDS section, with
/// roff's `\-` read as `-`.
fn man_subcommands() -> Vec<String> {
    let mut in_section = false;
    let mut after_tp = false;
    let mut names = Vec::new();
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
                names.push(tag.replace("\\-", "-"));
            }
            after_tp = false;
        }
    }
    names.sort();
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
    assert_eq!(
        man, help,
        "man/garnet.1 drifted from `garnet --help` (entries are compared one for one, so both \
         `verify` forms must appear in each)"
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
