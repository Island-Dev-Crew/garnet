//! Garnet Language Server Protocol MVP.
//!
//! S1 scope is intentionally narrow: parse/check diagnostics, top-level hover,
//! and top-level go-to-definition. Rename, workspace symbols, incremental CST
//! precision, and richer safe-mode hover remain follow-up work.

use garnet_check::{CheckError, CheckReport};
use garnet_parser::ast::{
    ActorDef, Annotation, ConstDecl, FnDef, FnMode, Item, LetDecl, MemoryDecl, Module, ModuleDecl,
    Ownership, ProtocolDef, TypeExpr,
};
use garnet_parser::error::ParseError;
use garnet_parser::token::Span;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents,
    HoverParams, InitializeParams, InitializeResult, InitializedParams, Location, MarkupContent,
    MarkupKind, OneOf, Position, Range, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, Url,
};
use tower_lsp::{async_trait, Client, LanguageServer};

#[derive(Debug, Clone)]
pub struct LineIndex {
    source: String,
    line_starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (idx, ch) in source.char_indices() {
            if ch == '\n' {
                line_starts.push(idx + ch.len_utf8());
            }
        }
        Self {
            source: source.to_string(),
            line_starts,
        }
    }

    pub fn position_to_offset(&self, position: Position) -> usize {
        let line = position.line as usize;
        let Some(&line_start) = self.line_starts.get(line) else {
            return self.source.len();
        };
        let line_end = self
            .line_starts
            .get(line + 1)
            .copied()
            .unwrap_or(self.source.len());
        let mut utf16_units = 0u32;

        for (relative_idx, ch) in self.source[line_start..line_end].char_indices() {
            if utf16_units >= position.character {
                return line_start + relative_idx;
            }
            utf16_units += ch.len_utf16() as u32;
        }

        line_end
    }

    pub fn offset_to_position(&self, offset: usize) -> Position {
        let offset = offset.min(self.source.len());
        let line = self
            .line_starts
            .partition_point(|line_start| *line_start <= offset)
            .saturating_sub(1);
        let line_start = self.line_starts.get(line).copied().unwrap_or(0);
        let character = self.source[line_start..offset]
            .chars()
            .map(|ch| ch.len_utf16() as u32)
            .sum();

        Position {
            line: line as u32,
            character,
        }
    }

    pub fn span_to_range(&self, span: Span) -> Range {
        let start = self.offset_to_position(span.start);
        let end = self.offset_to_position(span.end().max(span.start + 1));
        Range { start, end }
    }

    pub fn document_range(&self) -> Range {
        Range {
            start: Position::new(0, 0),
            end: self.offset_to_position(self.source.len()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolInfo {
    pub name: String,
    pub detail: String,
    pub hover_markdown: String,
    pub range: Range,
    pub selection_range: Range,
}

#[derive(Debug, Clone)]
pub struct Analysis {
    source: String,
    line_index: LineIndex,
    diagnostics: Vec<Diagnostic>,
    symbols: BTreeMap<String, SymbolInfo>,
}

impl Analysis {
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn symbols(&self) -> &BTreeMap<String, SymbolInfo> {
        &self.symbols
    }

    pub fn hover_at(&self, position: Position) -> Option<String> {
        let word = self.word_at(position)?;
        self.symbols
            .get(&word)
            .map(|symbol| symbol.hover_markdown.clone())
    }

    pub fn definition_at(&self, position: Position, uri: Url) -> Option<Location> {
        let word = self.word_at(position)?;
        self.symbols.get(&word).map(|symbol| Location {
            uri,
            range: symbol.selection_range,
        })
    }

    fn word_at(&self, position: Position) -> Option<String> {
        let offset = self.line_index.position_to_offset(position);
        let (start, end) = ident_bounds_at_offset(&self.source, offset)?;
        Some(self.source[start..end].to_string())
    }
}

pub fn analyze_source(source: &str) -> Analysis {
    let line_index = LineIndex::new(source);
    let mut diagnostics = Vec::new();
    let mut symbols = BTreeMap::new();

    match garnet_parser::parse_source(source) {
        Ok(module) => {
            collect_symbols(source, &line_index, &module, &mut symbols);
            diagnostics.extend(check_diagnostics(
                garnet_check::check_module(&module),
                &line_index,
            ));
        }
        Err(error) => diagnostics.push(parse_diagnostic(error, &line_index)),
    }

    Analysis {
        source: source.to_string(),
        line_index,
        diagnostics,
        symbols,
    }
}

fn parse_diagnostic(error: ParseError, line_index: &LineIndex) -> Diagnostic {
    let span = parse_error_span(&error);
    Diagnostic {
        range: line_index.span_to_range(span),
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("garnet-parser".to_string()),
        message: error.to_string(),
        ..Diagnostic::default()
    }
}

fn parse_error_span(error: &ParseError) -> Span {
    match error {
        ParseError::UnexpectedChar { span, .. }
        | ParseError::UnterminatedString { span }
        | ParseError::InvalidInt { span }
        | ParseError::InvalidFloat { span }
        | ParseError::UnexpectedToken { span, .. }
        | ParseError::UnexpectedEof { span, .. }
        | ParseError::BudgetExceeded { span, .. } => *span,
    }
}

fn check_diagnostics(report: CheckReport, line_index: &LineIndex) -> Vec<Diagnostic> {
    report
        .errors
        .into_iter()
        .map(|error| {
            let severity = match error {
                CheckError::BoundaryNote(_) => DiagnosticSeverity::WARNING,
                _ => DiagnosticSeverity::ERROR,
            };
            Diagnostic {
                range: line_index.document_range(),
                severity: Some(severity),
                source: Some("garnet-check".to_string()),
                message: error.to_string(),
                ..Diagnostic::default()
            }
        })
        .collect()
}

fn collect_symbols(
    source: &str,
    line_index: &LineIndex,
    module: &Module,
    symbols: &mut BTreeMap<String, SymbolInfo>,
) {
    collect_items(source, line_index, &module.items, symbols);
}

fn collect_items(
    source: &str,
    line_index: &LineIndex,
    items: &[Item],
    symbols: &mut BTreeMap<String, SymbolInfo>,
) {
    for item in items {
        if let Some(symbol) = symbol_for_item(source, line_index, item) {
            symbols.insert(symbol.name.clone(), symbol);
        }
        if let Item::Module(module) = item {
            collect_items(source, line_index, &module.items, symbols);
        }
    }
}

fn symbol_for_item(source: &str, line_index: &LineIndex, item: &Item) -> Option<SymbolInfo> {
    match item {
        Item::Fn(function) => Some(function_symbol(source, line_index, function)),
        Item::Struct(value) => Some(named_symbol(
            source,
            line_index,
            &value.name,
            "struct",
            &format!("struct {}", value.name),
            value.span,
        )),
        Item::Enum(value) => Some(named_symbol(
            source,
            line_index,
            &value.name,
            "enum",
            &format!("enum {}", value.name),
            value.span,
        )),
        Item::Trait(value) => Some(named_symbol(
            source,
            line_index,
            &value.name,
            "trait",
            &format!("trait {}", value.name),
            value.span,
        )),
        Item::Protocol(value) => Some(protocol_symbol(source, line_index, value)),
        Item::Actor(value) => Some(actor_symbol(source, line_index, value)),
        Item::Memory(value) => Some(memory_symbol(source, line_index, value)),
        Item::Module(value) => Some(module_symbol(source, line_index, value)),
        Item::Const(value) => Some(const_symbol(source, line_index, value)),
        Item::Let(value) => Some(let_symbol(source, line_index, value)),
        Item::Impl(_) | Item::Use(_) => None,
    }
}

fn function_symbol(source: &str, line_index: &LineIndex, function: &FnDef) -> SymbolInfo {
    let keyword = match function.mode {
        FnMode::Managed => "def",
        FnMode::Safe => "fn",
    };
    let signature = format_function_signature(function);
    named_symbol(
        source,
        line_index,
        &function.name,
        keyword,
        &signature,
        item_span_with_annotations(&function.annotations, function.span),
    )
}

fn protocol_symbol(source: &str, line_index: &LineIndex, value: &ProtocolDef) -> SymbolInfo {
    named_symbol(
        source,
        line_index,
        &value.name,
        "protocol",
        &format!("protocol {} ({} items)", value.name, value.items.len()),
        value.span,
    )
}

fn actor_symbol(source: &str, line_index: &LineIndex, value: &ActorDef) -> SymbolInfo {
    named_symbol(
        source,
        line_index,
        &value.name,
        "actor",
        &format!("actor {} ({} items)", value.name, value.items.len()),
        value.span,
    )
}

fn memory_symbol(source: &str, line_index: &LineIndex, value: &MemoryDecl) -> SymbolInfo {
    named_symbol(
        source,
        line_index,
        &value.name,
        "memory",
        &format!(
            "memory {:?} {}: {}",
            value.kind,
            value.name,
            format_type(&value.store)
        ),
        value.span,
    )
}

fn module_symbol(source: &str, line_index: &LineIndex, value: &ModuleDecl) -> SymbolInfo {
    named_symbol(
        source,
        line_index,
        &value.name,
        "module",
        &format!("module {}", value.name),
        value.span,
    )
}

fn const_symbol(source: &str, line_index: &LineIndex, value: &ConstDecl) -> SymbolInfo {
    named_symbol(
        source,
        line_index,
        &value.name,
        "const",
        &format!("const {}", value.name),
        value.span,
    )
}

fn let_symbol(source: &str, line_index: &LineIndex, value: &LetDecl) -> SymbolInfo {
    named_symbol(
        source,
        line_index,
        &value.name,
        "let",
        &format!("let {}", value.name),
        value.span,
    )
}

fn named_symbol(
    source: &str,
    line_index: &LineIndex,
    name: &str,
    kind: &str,
    detail: &str,
    span: Span,
) -> SymbolInfo {
    let name_span = find_name_span(source, name, span).unwrap_or(span);
    let docs = extract_doc_comments_before(source, span.start);
    let hover_markdown = if docs.is_empty() {
        format!("```garnet\n{detail}\n```\n\n_{kind}_")
    } else {
        format!("```garnet\n{detail}\n```\n\n{docs}")
    };

    SymbolInfo {
        name: name.to_string(),
        detail: detail.to_string(),
        hover_markdown,
        range: line_index.span_to_range(span),
        selection_range: line_index.span_to_range(name_span),
    }
}

fn item_span_with_annotations(annotations: &[Annotation], fallback: Span) -> Span {
    annotations
        .iter()
        .filter_map(annotation_span)
        .min_by_key(|span| span.start)
        .unwrap_or(fallback)
}

fn annotation_span(annotation: &Annotation) -> Option<Span> {
    match annotation {
        Annotation::MaxDepth(_, span)
        | Annotation::FanOut(_, span)
        | Annotation::RequireMetadata(span)
        | Annotation::Safe(span)
        | Annotation::Dynamic(span)
        | Annotation::Caps(_, span)
        | Annotation::Mailbox(_, span)
        | Annotation::NonSendable(span) => Some(*span),
    }
}

fn find_name_span(source: &str, name: &str, item_span: Span) -> Option<Span> {
    let search_start = item_span.start.min(source.len());
    let search_end = item_span.end().min(source.len());
    let within_item = &source[search_start..search_end];
    let relative = within_item.find(name)?;
    Some(Span::new(search_start + relative, name.len()))
}

fn format_function_signature(function: &FnDef) -> String {
    let keyword = match function.mode {
        FnMode::Managed => "def",
        FnMode::Safe => "fn",
    };
    let type_params = if function.type_params.is_empty() {
        String::new()
    } else {
        format!("<{}>", function.type_params.join(", "))
    };
    let params = function
        .params
        .iter()
        .map(|param| {
            let ownership = param
                .ownership
                .map(format_ownership)
                .map(|value| format!("{value} "))
                .unwrap_or_default();
            let ty = param
                .ty
                .as_ref()
                .map(|ty| format!(": {}", format_type(ty)))
                .unwrap_or_default();
            format!("{ownership}{}{ty}", param.name)
        })
        .collect::<Vec<_>>()
        .join(", ");
    let ret = function
        .return_ty
        .as_ref()
        .map(|ty| format!(" -> {}", format_type(ty)))
        .unwrap_or_default();

    format!("{keyword} {}{type_params}({params}){ret}", function.name)
}

fn format_ownership(ownership: Ownership) -> &'static str {
    match ownership {
        Ownership::Own => "own",
        Ownership::Borrow => "borrow",
        Ownership::Ref => "ref",
        Ownership::Mut => "mut",
    }
}

fn format_type(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Named { path, args, .. } => {
            let mut out = path.join("::");
            if !args.is_empty() {
                out.push('<');
                out.push_str(&args.iter().map(format_type).collect::<Vec<_>>().join(", "));
                out.push('>');
            }
            out
        }
        TypeExpr::Fn { params, ret, .. } => format!(
            "({}) -> {}",
            params
                .iter()
                .map(format_type)
                .collect::<Vec<_>>()
                .join(", "),
            format_type(ret)
        ),
        TypeExpr::Tuple { elements, .. } => format!(
            "({})",
            elements
                .iter()
                .map(format_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeExpr::Ref { mutable, inner, .. } => {
            if *mutable {
                format!("&mut {}", format_type(inner))
            } else {
                format!("&{}", format_type(inner))
            }
        }
        TypeExpr::Dyn { trait_ty, .. } => format!("dyn {}", format_type(trait_ty)),
    }
}

pub fn extract_doc_comments_before(source: &str, byte_offset: usize) -> String {
    let cutoff = byte_offset.min(source.len());
    let head = &source[..cutoff];
    let mut collected: Vec<&str> = Vec::new();

    for line in head.lines().rev() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("///") {
            collected.push(rest.strip_prefix(' ').unwrap_or(rest));
        } else {
            break;
        }
    }

    collected.reverse();
    collected.join("\n")
}

fn ident_bounds_at_offset(source: &str, offset: usize) -> Option<(usize, usize)> {
    let offset = offset.min(source.len());
    let mut start = offset;
    while start > 0 {
        let (prev_start, ch) = source[..start].char_indices().next_back()?;
        if is_ident_char(ch) {
            start = prev_start;
        } else {
            break;
        }
    }

    let mut end = offset;
    while end < source.len() {
        let ch = source[end..].chars().next()?;
        if is_ident_char(ch) {
            end += ch.len_utf8();
        } else {
            break;
        }
    }

    if start == end {
        None
    } else {
        Some((start, end))
    }
}

fn is_ident_char(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

#[derive(Debug)]
pub struct Backend {
    client: Client,
    documents: Arc<Mutex<HashMap<Url, String>>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn analyze_uri(&self, uri: &Url) -> Option<Analysis> {
        let documents = self.documents.lock().await;
        documents.get(uri).map(|source| analyze_source(source))
    }

    async fn publish_diagnostics(&self, uri: Url, source: &str) {
        let analysis = analyze_source(source);
        self.client
            .publish_diagnostics(uri, analysis.diagnostics().to_vec(), None)
            .await;
    }
}

#[async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> LspResult<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(tower_lsp::lsp_types::HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
            server_info: Some(tower_lsp::lsp_types::ServerInfo {
                name: "garnet-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {}

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        self.documents
            .lock()
            .await
            .insert(uri.clone(), text.clone());
        self.publish_diagnostics(uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let Some(change) = params.content_changes.into_iter().last() else {
            return;
        };
        self.documents
            .lock()
            .await
            .insert(uri.clone(), change.text.clone());
        self.publish_diagnostics(uri, &change.text).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(analysis) = self.analyze_uri(&uri).await {
            self.client
                .publish_diagnostics(uri, analysis.diagnostics().to_vec(), None)
                .await;
        }
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let Some(analysis) = self.analyze_uri(&uri).await else {
            return Ok(None);
        };
        Ok(analysis.hover_at(position).map(|value| Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value,
            }),
            range: None,
        }))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> LspResult<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let Some(analysis) = self.analyze_uri(&uri).await else {
            return Ok(None);
        };
        Ok(analysis
            .definition_at(position, uri)
            .map(GotoDefinitionResponse::Scalar))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn position_for(source: &str, needle: &str) -> Position {
        let offset = source.find(needle).expect("needle present");
        LineIndex::new(source).offset_to_position(offset)
    }

    #[test]
    fn parser_diagnostic_uses_source_span() {
        let source = "def broken( {\n";
        let analysis = analyze_source(source);

        assert_eq!(analysis.diagnostics().len(), 1);
        assert_eq!(
            analysis.diagnostics()[0].source.as_deref(),
            Some("garnet-parser")
        );
        assert!(analysis.diagnostics()[0].message.contains("expected"));
    }

    #[test]
    fn checker_diagnostic_is_reported() {
        let source = "@caps()\ndef main() {\n  fs::read_file(\"x\")\n}\n";
        let analysis = analyze_source(source);

        assert!(analysis
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.message.contains("does not declare `fs`")));
    }

    #[test]
    fn hover_finds_function_signature_and_docs() {
        let source = "/// Friendly greeting\ndef greet(name) {\n  name\n}\n";
        let analysis = analyze_source(source);
        let hover = analysis
            .hover_at(position_for(source, "greet(name)"))
            .expect("hover");

        assert!(hover.contains("def greet(name)"));
        assert!(hover.contains("Friendly greeting"));
    }

    #[test]
    fn definition_from_call_site_finds_top_level_function() {
        let source = "def greet(name) {\n  name\n}\n\n@caps()\ndef main() {\n  greet(\"Ada\")\n}\n";
        let analysis = analyze_source(source);
        let location = analysis
            .definition_at(
                position_for(source, "greet(\"Ada\")"),
                Url::parse("file:///tmp/example.garnet").expect("url"),
            )
            .expect("definition");

        assert_eq!(location.range.start, Position::new(0, 4));
    }

    #[test]
    fn indexes_multiple_top_level_symbol_kinds() {
        let source = "memory episodic log: EpisodeStore<Event>\nstruct User { name: String }\n";
        let analysis = analyze_source(source);

        assert!(analysis.symbols().contains_key("log"));
        assert!(analysis.symbols().contains_key("User"));
    }
}
