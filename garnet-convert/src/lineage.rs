//! Lineage tagging — every CIR node carries a pointer back to the
//! source AST node that produced it. The witness pass verifies that
//! every emitted node traces to the source; untagged nodes are
//! rejected as potential LLM hallucinations.

use std::collections::BTreeMap;
use std::fmt;

/// A source-location pointer on every CIR node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lineage {
    pub source_lang: String,
    pub source_file: String,
    pub source_span: (usize, usize),
}

impl Lineage {
    pub fn new(source_lang: &str, source_file: &str, start: usize, end: usize) -> Self {
        Self {
            source_lang: source_lang.to_string(),
            source_file: source_file.to_string(),
            source_span: (start, end),
        }
    }

    pub fn unknown() -> Self {
        Self {
            source_lang: "unknown".into(),
            source_file: "".into(),
            source_span: (0, 0),
        }
    }
}

impl fmt::Display for Lineage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}-{}",
            self.source_lang, self.source_file, self.source_span.0, self.source_span.1
        )
    }
}

/// A witness record — one entry per emitted CIR node. Assembled by
/// the emitter and serialized as `source.garnet.lineage.json`.
#[derive(Debug, Clone)]
pub struct WitnessEntry {
    pub cir_node_id: usize,
    pub cir_node_kind: String,
    pub source_span: (usize, usize),
    pub emit_line: usize,
}

#[derive(Debug, Default)]
pub struct LineageMap {
    pub source_lang: String,
    pub source_file: String,
    pub entries: Vec<WitnessEntry>,
}

impl LineageMap {
    pub fn new(source_lang: &str, source_file: &str) -> Self {
        Self {
            source_lang: source_lang.to_string(),
            source_file: source_file.to_string(),
            entries: Vec::new(),
        }
    }

    pub fn push(&mut self, entry: WitnessEntry) {
        self.entries.push(entry);
    }

    /// Render as a reproducible JSON string (no serde dep). Used for
    /// `source.garnet.lineage.json`.
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        out.push_str("{\n");
        out.push_str(&format!(
            "  \"source_lang\": \"{}\",\n  \"source_file\": \"{}\",\n  \"entries\": [\n",
            json_escape(&self.source_lang),
            json_escape(&self.source_file)
        ));
        for (i, e) in self.entries.iter().enumerate() {
            out.push_str(&format!(
                "    {{ \"id\": {}, \"kind\": \"{}\", \"source_span\": [{}, {}], \"emit_line\": {} }}",
                e.cir_node_id,
                json_escape(&e.cir_node_kind),
                e.source_span.0,
                e.source_span.1,
                e.emit_line,
            ));
            if i + 1 < self.entries.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  ]\n}\n");
        out
    }

    /// Group entries by emit_line for a source-lineage view.
    pub fn by_line(&self) -> BTreeMap<usize, Vec<&WitnessEntry>> {
        let mut m: BTreeMap<usize, Vec<&WitnessEntry>> = BTreeMap::new();
        for e in &self.entries {
            m.entry(e.emit_line).or_default().push(e);
        }
        m
    }
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lineage_display() {
        let l = Lineage::new("rust", "src/foo.rs", 10, 20);
        assert_eq!(l.to_string(), "rust:src/foo.rs:10-20");
    }

    #[test]
    fn lineage_map_serializes_to_valid_json() {
        let mut m = LineageMap::new("rust", "src/foo.rs");
        m.push(WitnessEntry {
            cir_node_id: 1,
            cir_node_kind: "Func".into(),
            source_span: (0, 50),
            emit_line: 5,
        });
        m.push(WitnessEntry {
            cir_node_id: 2,
            cir_node_kind: "Literal".into(),
            source_span: (20, 22),
            emit_line: 7,
        });
        let j = m.to_json();
        assert!(j.contains("\"source_lang\": \"rust\""));
        assert!(j.contains("\"id\": 1"));
        assert!(j.contains("\"id\": 2"));
        assert!(j.contains("\"kind\": \"Func\""));
        // Two entries, one comma separator between them
        assert_eq!(j.matches("\"id\":").count(), 2);
    }

    #[test]
    fn json_escape_handles_quotes() {
        assert_eq!(json_escape(r#"he said "hi""#), r#"he said \"hi\""#);
        assert_eq!(json_escape("a\nb"), "a\\nb");
        assert_eq!(json_escape("c\\d"), "c\\\\d");
    }

    #[test]
    fn by_line_groups_correctly() {
        let mut m = LineageMap::new("rust", "f.rs");
        m.push(WitnessEntry {
            cir_node_id: 1,
            cir_node_kind: "A".into(),
            source_span: (0, 5),
            emit_line: 3,
        });
        m.push(WitnessEntry {
            cir_node_id: 2,
            cir_node_kind: "B".into(),
            source_span: (10, 15),
            emit_line: 3,
        });
        m.push(WitnessEntry {
            cir_node_id: 3,
            cir_node_kind: "C".into(),
            source_span: (20, 25),
            emit_line: 5,
        });
        let grouped = m.by_line();
        assert_eq!(grouped.get(&3).unwrap().len(), 2);
        assert_eq!(grouped.get(&5).unwrap().len(), 1);
    }
}
