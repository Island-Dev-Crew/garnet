//! Docs-as-tests: the pure fence extractor (S43).
//!
//! The "evidence not courtesy" discipline says a documented example must
//! be executable, not decorative. `garnet doctest` (see [`crate::cmd::doctest`])
//! reuses `garnet doc`'s `///` extraction, then this module pulls the
//! ` ```garnet ` fenced code blocks out of each doc block so the runner can
//! execute them against the file's own definitions.
//!
//! This module is deliberately pure (string in, [`Fence`]s out) so the
//! fence grammar is unit-tested without any parser or interpreter.
//!
//! ## Fence grammar
//!
//! - An opening fence is a line whose trimmed text is exactly ` ```garnet `.
//! - A closing fence is a line whose trimmed text is exactly ` ``` `.
//! - Fences in any other language (` ```rust `, ` ```text `, a bare ` ``` `
//!   with no language) are ignored entirely.
//! - Inside a fence, a line of the form `# => <value>` records an expected
//!   result: the runner compares it to the displayed value of the fence's
//!   tail expression. `#` is the Garnet line-comment character, so the
//!   marker is also a valid (inert) comment; it is stripped from the code
//!   that is executed. At most one expectation per fence (the last wins).

/// A single ` ```garnet ` fenced code block lifted from a doc comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fence {
    /// The fence body with the `# =>` expectation line (if any) removed.
    pub code: String,
    /// 1-based line number of the opening ` ```garnet ` within the doc block.
    pub start_line: usize,
    /// Expected displayed value of the tail expression, if a `# =>` marker
    /// was present. `None` means "passes iff it runs without error".
    pub expect: Option<String>,
}

/// Parse an expectation marker (`# => <value>`) from a fence line, returning
/// the trimmed expected text. Tolerates optional spaces: `#=>x`, `# => x`.
fn parse_expect(line: &str) -> Option<String> {
    let rest = line.trim_start().strip_prefix('#')?.trim_start();
    let value = rest.strip_prefix("=>")?.trim();
    Some(value.to_string())
}

/// Extract every ` ```garnet ` fenced block from a doc-comment block.
pub fn garnet_fences(doc_block: &str) -> Vec<Fence> {
    let mut fences = Vec::new();
    let mut in_fence = false;
    let mut code: Vec<&str> = Vec::new();
    let mut expect: Option<String> = None;
    let mut start_line = 0usize;

    for (idx, line) in doc_block.lines().enumerate() {
        let trimmed = line.trim();
        if !in_fence {
            if trimmed == "```garnet" {
                in_fence = true;
                start_line = idx + 1;
                code.clear();
                expect = None;
            }
            continue;
        }
        // Inside a fence.
        if trimmed == "```" {
            fences.push(Fence {
                code: code.join("\n"),
                start_line,
                expect: expect.take(),
            });
            in_fence = false;
            continue;
        }
        if let Some(value) = parse_expect(line) {
            // Expectation marker — record it, keep it out of executed code.
            expect = Some(value);
            continue;
        }
        code.push(line);
    }

    // Lenient: an unterminated fence still yields its collected body.
    if in_fence {
        fences.push(Fence {
            code: code.join("\n"),
            start_line,
            expect,
        });
    }

    fences
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_a_single_fence() {
        let doc = "Summary.\n\n```garnet\nlet x = 1\nx\n```\nmore prose";
        let fences = garnet_fences(doc);
        assert_eq!(fences.len(), 1);
        assert_eq!(fences[0].code, "let x = 1\nx");
        assert_eq!(fences[0].expect, None);
        assert_eq!(fences[0].start_line, 3);
    }

    #[test]
    fn records_an_expectation_marker() {
        let doc = "```garnet\nfib(10)\n# => 55\n```";
        let fences = garnet_fences(doc);
        assert_eq!(fences.len(), 1);
        assert_eq!(fences[0].code, "fib(10)");
        assert_eq!(fences[0].expect.as_deref(), Some("55"));
    }

    #[test]
    fn ignores_non_garnet_fences() {
        let doc = "```rust\nlet x = 1;\n```\n```text\nplain\n```";
        assert!(garnet_fences(doc).is_empty());
    }

    #[test]
    fn extracts_multiple_fences() {
        let doc = "```garnet\n1\n```\nbetween\n```garnet\n2\n# => 2\n```";
        let fences = garnet_fences(doc);
        assert_eq!(fences.len(), 2);
        assert_eq!(fences[0].code, "1");
        assert_eq!(fences[0].expect, None);
        assert_eq!(fences[1].code, "2");
        assert_eq!(fences[1].expect.as_deref(), Some("2"));
    }

    #[test]
    fn tolerates_spacing_in_the_marker() {
        assert_eq!(parse_expect("#=>7"), Some("7".to_string()));
        assert_eq!(parse_expect("#   =>   7  "), Some("7".to_string()));
        assert_eq!(parse_expect("# regular comment"), None);
        assert_eq!(parse_expect("let x = 1"), None);
    }

    #[test]
    fn unterminated_fence_still_yields_body() {
        let doc = "```garnet\nlet x = 1\nx";
        let fences = garnet_fences(doc);
        assert_eq!(fences.len(), 1);
        assert_eq!(fences[0].code, "let x = 1\nx");
    }
}
