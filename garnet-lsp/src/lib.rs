//! Garnet Language Server Protocol MVP v0.2.
//!
//! S16 scope implements document symbols, workspace symbols, rename,
//! code action quick-fixes (rules-based S10), and semantic tokens on top of the CST.

use garnet_check::suggest::Rule;
use garnet_check::{CheckError, CheckReport};
use garnet_cst::{cst_to_ast, identifier_spans, parse_cst, token_infos, SyntaxNode, TokenInfo};
use garnet_parser::ast::{
    ActorDef, Annotation, ConstDecl, Expr, FnDef, FnMode, Item, LetDecl, MemoryDecl, Module,
    ModuleDecl, Ownership, ProtocolDef, TypeExpr,
};
use garnet_parser::error::ParseError;
use garnet_parser::token::{Span, TokenKind};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, CodeActionResponse,
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse,
    GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents, HoverParams,
    InitializeParams, InitializeResult, InitializedParams, Location, MarkupContent, MarkupKind,
    NumberOrString, OneOf, Position, Range, RenameParams, SemanticToken, SemanticTokenType,
    SemanticTokens, SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
    SemanticTokensParams, SemanticTokensResult, SemanticTokensServerCapabilities,
    ServerCapabilities, SymbolInformation, SymbolKind, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextEdit, Url, WorkspaceEdit, WorkspaceSymbolParams,
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

    pub fn insert_range_at_offset(&self, offset: usize) -> Range {
        let position = self.offset_to_position(offset);
        Range {
            start: position,
            end: position,
        }
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

    match parse_lsp_source(source) {
        Ok((module, _cst_root)) => {
            collect_symbols(source, &line_index, &module, &mut symbols);
            diagnostics.extend(check_diagnostics(
                garnet_check::check_module(&module),
                &line_index,
            ));

            // Wire S10 rules-based suggestions into LSP diagnostics as INFORMATION
            let suggestions = garnet_check::suggest::suggest_for_module(&module);
            for sugg in suggestions {
                if let Some(fn_def) = find_function_in_module(&module, &sugg.function) {
                    let range = line_index.span_to_range(fn_def.span);
                    diagnostics.push(Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::INFORMATION),
                        code: Some(NumberOrString::String(sugg.rule.id().to_string())),
                        source: Some("garnet-check".to_string()),
                        message: sugg.message.clone(),
                        ..Diagnostic::default()
                    });
                }
            }
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

fn parse_lsp_source(source: &str) -> Result<(Module, SyntaxNode), ParseError> {
    garnet_parser::parse_source(source)?;
    let parsed = parse_cst(source);
    let module = cst_to_ast(parsed.syntax());
    Ok((module, parsed.root))
}

fn find_function_in_module<'a>(module: &'a Module, name: &str) -> Option<&'a FnDef> {
    find_function_in_items(&module.items, name)
}

fn find_function_in_items<'a>(items: &'a [Item], name: &str) -> Option<&'a FnDef> {
    for item in items {
        match item {
            Item::Fn(fn_def) if fn_def.name == name => return Some(fn_def),
            Item::Fn(_) => {}
            Item::Module(mod_decl) => {
                if let Some(fn_def) = find_function_in_items(&mod_decl.items, name) {
                    return Some(fn_def);
                }
            }
            Item::Impl(impl_block) => {
                if let Some(method) = impl_block.methods.iter().find(|method| method.name == name) {
                    return Some(method);
                }
            }
            _ => {}
        }
    }
    None
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

fn identifier_spans_for_source(source: &str, name: &str) -> Vec<Span> {
    let parsed = parse_cst(source);
    identifier_spans(parsed.syntax(), name)
}

fn rename_text_edits_for_source(source: &str, target_name: &str, new_name: &str) -> Vec<TextEdit> {
    rename_text_edits_for_source_in_span(source, target_name, new_name, None)
}

fn rename_text_edits_for_source_in_span(
    source: &str,
    target_name: &str,
    new_name: &str,
    containing_span: Option<Span>,
) -> Vec<TextEdit> {
    let line_index = LineIndex::new(source);
    identifier_spans_for_source(source, target_name)
        .into_iter()
        .filter(|span| {
            containing_span.is_none_or(|container| {
                span.start >= container.start && span.end() <= container.end()
            })
        })
        .map(|span| TextEdit {
            range: line_index.span_to_range(span),
            new_text: new_name.to_string(),
        })
        .collect()
}

#[cfg(test)]
fn rename_scoped_text_edits(
    source: &str,
    target_name: &str,
    new_name: &str,
    position: Position,
) -> Vec<TextEdit> {
    let line_index = LineIndex::new(source);
    let offset = line_index.position_to_offset(position);
    let containing_span = parameter_function_span_at_offset(source, target_name, offset);
    rename_text_edits_for_source_in_span(source, target_name, new_name, containing_span)
}

fn parameter_function_span_at_offset(
    source: &str,
    target_name: &str,
    offset: usize,
) -> Option<Span> {
    let (module, _) = parse_lsp_source(source).ok()?;
    let function = find_function_containing_offset(&module, offset)?;
    let param = function
        .params
        .iter()
        .find(|param| param.name == target_name)?;
    let in_param_decl = offset >= param.span.start && offset <= param.span.end();
    let in_body = offset >= function.body.span.start && offset <= function.body.span.end();
    (in_param_decl || in_body).then_some(function.span)
}

fn find_function_containing_offset(module: &Module, offset: usize) -> Option<&FnDef> {
    find_function_containing_offset_in_items(&module.items, offset)
}

fn find_function_containing_offset_in_items(items: &[Item], offset: usize) -> Option<&FnDef> {
    for item in items {
        match item {
            Item::Fn(function)
                if offset >= function.span.start && offset <= function.span.end() =>
            {
                return Some(function);
            }
            Item::Module(module) => {
                if let Some(function) =
                    find_function_containing_offset_in_items(&module.items, offset)
                {
                    return Some(function);
                }
            }
            Item::Impl(impl_block) => {
                if let Some(function) = impl_block
                    .methods
                    .iter()
                    .find(|method| offset >= method.span.start && offset <= method.span.end())
                {
                    return Some(function);
                }
            }
            _ => {}
        }
    }
    None
}

fn tokens_for_source(source: &str) -> Vec<TokenInfo> {
    let parsed = parse_cst(source);
    token_infos(parsed.syntax())
}

fn code_actions_for_source(uri: Url, source: &str) -> Vec<CodeAction> {
    let Ok((module, _)) = parse_lsp_source(source) else {
        return Vec::new();
    };
    let line_index = LineIndex::new(source);
    let mut actions = Vec::new();

    for suggestion in garnet_check::suggest::suggest_for_module(&module) {
        let Some(function) = find_function_in_module(&module, &suggestion.function) else {
            continue;
        };
        if let Some(action) = code_action_for_rule(&uri, &line_index, function, suggestion.rule) {
            actions.push(action);
        }
    }

    let mut functions = Vec::new();
    collect_functions(&module.items, &mut functions);
    for function in functions {
        if let Some(action) = add_return_type_action(&uri, source, &line_index, function) {
            actions.push(action);
        }
    }

    actions
}

fn code_action_for_rule(
    uri: &Url,
    line_index: &LineIndex,
    function: &FnDef,
    rule: Rule,
) -> Option<CodeAction> {
    match rule {
        Rule::ManagedFnMissingCaps => Some(CodeAction {
            title: format!("Add `@caps()` to `def {}`", function.name),
            kind: Some(CodeActionKind::QUICKFIX),
            edit: Some(workspace_edit(
                uri,
                vec![TextEdit {
                    range: line_index.insert_range_at_offset(function.span.start),
                    new_text: "@caps()\n".to_string(),
                }],
            )),
            is_preferred: Some(true),
            ..Default::default()
        }),
        Rule::LongParameterList => {
            let params_span = parameter_list_inner_span(function)?;
            let struct_name = format!("{}Params", pascal_case(&function.name));
            Some(CodeAction {
                title: format!(
                    "Refactor long parameter list of `{}` into `{}`",
                    function.name, struct_name
                ),
                kind: Some(CodeActionKind::REFACTOR_REWRITE),
                edit: Some(workspace_edit(
                    uri,
                    vec![
                        TextEdit {
                            range: line_index.insert_range_at_offset(function.span.start),
                            new_text: long_parameter_struct_text(function, &struct_name),
                        },
                        TextEdit {
                            range: line_index.span_to_range(params_span),
                            new_text: format!("options: {struct_name}"),
                        },
                    ],
                )),
                is_preferred: Some(false),
                ..Default::default()
            })
        }
        Rule::EmptyFunctionBody => Some(CodeAction {
            title: format!("Stub empty body of `{}`", function.name),
            kind: Some(CodeActionKind::QUICKFIX),
            edit: Some(workspace_edit(
                uri,
                vec![TextEdit {
                    range: line_index.span_to_range(function.body.span),
                    new_text: "{\n  // intentionally empty\n}".to_string(),
                }],
            )),
            is_preferred: Some(false),
            ..Default::default()
        }),
    }
}

fn workspace_edit(uri: &Url, edits: Vec<TextEdit>) -> WorkspaceEdit {
    let mut changes = HashMap::new();
    changes.insert(uri.clone(), edits);
    WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    }
}

fn parameter_list_inner_span(function: &FnDef) -> Option<Span> {
    let first = function.params.first()?.span;
    let last = function.params.last()?.span;
    Some(Span::new(
        first.start,
        last.end().saturating_sub(first.start),
    ))
}

fn pascal_case(value: &str) -> String {
    let mut out = String::new();
    let mut capitalize_next = true;
    for ch in value.chars() {
        if ch == '_' || ch == '-' {
            capitalize_next = true;
            continue;
        }
        if capitalize_next {
            out.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            out.push(ch);
        }
    }
    if out.is_empty() {
        "Params".to_string()
    } else {
        out
    }
}

fn long_parameter_struct_text(function: &FnDef, struct_name: &str) -> String {
    let fields = function
        .params
        .iter()
        .map(|param| {
            let ty = param
                .ty
                .as_ref()
                .map(format_type)
                .unwrap_or_else(|| "Any".to_string());
            format!("  {}: {ty}", param.name)
        })
        .collect::<Vec<_>>()
        .join(",\n");
    format!("struct {struct_name} {{\n{fields}\n}}\n\n")
}

fn add_return_type_action(
    uri: &Url,
    source: &str,
    line_index: &LineIndex,
    function: &FnDef,
) -> Option<CodeAction> {
    if function.return_ty.is_some() {
        return None;
    }
    let inferred = infer_return_type(function)?;
    let brace_offset = source[function.span.start..function.span.end().min(source.len())]
        .find('{')
        .map(|relative| function.span.start + relative)?;
    Some(CodeAction {
        title: format!("Add return type `{inferred}` to `{}`", function.name),
        kind: Some(CodeActionKind::QUICKFIX),
        edit: Some(workspace_edit(
            uri,
            vec![TextEdit {
                range: line_index.insert_range_at_offset(brace_offset),
                new_text: format!("-> {inferred} "),
            }],
        )),
        is_preferred: Some(false),
        ..Default::default()
    })
}

fn infer_return_type(function: &FnDef) -> Option<&'static str> {
    match function.body.tail_expr.as_deref()? {
        Expr::Int(_, _) => Some("Int"),
        Expr::Float(_, _) => Some("Float"),
        Expr::Bool(_, _) => Some("Bool"),
        Expr::Str(_, _) => Some("String"),
        Expr::Nil(_) => Some("Nil"),
        _ => None,
    }
}

fn collect_functions<'a>(items: &'a [Item], out: &mut Vec<&'a FnDef>) {
    for item in items {
        match item {
            Item::Fn(function) => out.push(function),
            Item::Module(module) => collect_functions(&module.items, out),
            Item::Impl(impl_block) => out.extend(impl_block.methods.iter()),
            _ => {}
        }
    }
}

struct TempSemanticToken {
    line: u32,
    character: u32,
    length: u32,
    token_type: u32,
}

const TOKEN_KEYWORD: u32 = 0;
const TOKEN_FUNCTION: u32 = 1;
const TOKEN_TYPE: u32 = 2;
const TOKEN_PARAMETER: u32 = 3;
const TOKEN_COMMENT: u32 = 4;
const TOKEN_STRING: u32 = 5;
const TOKEN_NUMBER: u32 = 6;
const TOKEN_OPERATOR: u32 = 7;
const TOKEN_CAPABILITY: u32 = 8;
const TOKEN_ATTRIBUTE: u32 = 9;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClassifiedSemanticToken {
    text: String,
    span: Span,
    kind: String,
    token_type: u32,
}

fn semantic_token_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::KEYWORD,
            SemanticTokenType::FUNCTION,
            SemanticTokenType::TYPE,
            SemanticTokenType::PARAMETER,
            SemanticTokenType::COMMENT,
            SemanticTokenType::STRING,
            SemanticTokenType::NUMBER,
            SemanticTokenType::OPERATOR,
            SemanticTokenType::new("capability"),
            SemanticTokenType::new("attribute"),
        ],
        token_modifiers: vec![],
    }
}

fn classify_semantic_tokens(source: &str) -> Vec<ClassifiedSemanticToken> {
    let param_name_spans = parameter_name_spans(source);
    let mut classified = Vec::new();
    let mut previous_kind: Option<TokenKind> = None;
    let mut previous_attribute: Option<String> = None;
    let mut inside_caps_args = false;

    for token in tokens_for_source(source) {
        let token_type = match &token.kind {
            TokenKind::Comment(_) => Some((TOKEN_COMMENT, "comment")),
            TokenKind::Str(_) | TokenKind::RawStr(_) | TokenKind::Symbol(_) => {
                Some((TOKEN_STRING, "string"))
            }
            TokenKind::Int(_) | TokenKind::Float(_) => Some((TOKEN_NUMBER, "number")),
            TokenKind::Ident(name) => {
                if matches!(previous_kind, Some(TokenKind::At)) {
                    previous_attribute = Some(name.clone());
                    Some((TOKEN_ATTRIBUTE, "attribute"))
                } else if inside_caps_args {
                    Some((TOKEN_CAPABILITY, "capability"))
                } else if param_name_spans
                    .iter()
                    .any(|span| span.start == token.span.start && span.len == token.span.len)
                {
                    Some((TOKEN_PARAMETER, "parameter"))
                } else if matches!(previous_kind, Some(TokenKind::KwFn | TokenKind::KwDef)) {
                    Some((TOKEN_FUNCTION, "function"))
                } else if matches!(
                    previous_kind,
                    Some(
                        TokenKind::KwActor
                            | TokenKind::KwStruct
                            | TokenKind::KwEnum
                            | TokenKind::KwTrait
                            | TokenKind::KwProtocol
                    )
                ) {
                    Some((TOKEN_TYPE, "type"))
                } else {
                    None
                }
            }
            kind if is_keyword_kind(kind) => Some((TOKEN_KEYWORD, "keyword")),
            kind if is_operator_kind(kind) => Some((TOKEN_OPERATOR, "operator")),
            TokenKind::LParen if previous_attribute.as_deref() == Some("caps") => {
                inside_caps_args = true;
                None
            }
            TokenKind::RParen => {
                inside_caps_args = false;
                previous_attribute = None;
                None
            }
            _ => None,
        };

        if !matches!(token.kind, TokenKind::Whitespace(_) | TokenKind::Comment(_)) {
            previous_kind = Some(token.kind.clone());
        }

        if let Some((token_type, kind)) = token_type {
            classified.push(ClassifiedSemanticToken {
                text: token.text,
                span: token.span,
                kind: kind.to_string(),
                token_type,
            });
        }
    }

    classified
}

fn parameter_name_spans(source: &str) -> Vec<Span> {
    let Ok((module, _)) = parse_lsp_source(source) else {
        return Vec::new();
    };
    let mut functions = Vec::new();
    collect_functions(&module.items, &mut functions);
    functions
        .into_iter()
        .flat_map(|function| {
            function
                .params
                .iter()
                .filter_map(|param| find_name_span(source, &param.name, param.span))
        })
        .collect()
}

fn encode_semantic_tokens(source: &str, line_index: &LineIndex) -> Vec<SemanticToken> {
    let mut tokens = classify_semantic_tokens(source)
        .into_iter()
        .map(|token| {
            let pos = line_index.offset_to_position(token.span.start);
            TempSemanticToken {
                line: pos.line,
                character: pos.character,
                length: token.span.len as u32,
                token_type: token.token_type,
            }
        })
        .collect::<Vec<_>>();

    tokens.sort_by(|a, b| {
        a.line
            .cmp(&b.line)
            .then_with(|| a.character.cmp(&b.character))
    });

    let mut data = Vec::new();
    let mut prev_line = 0u32;
    let mut prev_char = 0u32;

    for token in tokens {
        let delta_line = token.line - prev_line;
        let delta_start = if delta_line == 0 {
            token.character - prev_char
        } else {
            token.character
        };

        data.push(SemanticToken {
            delta_line,
            delta_start,
            length: token.length,
            token_type: token.token_type,
            token_modifiers_bitset: 0,
        });

        prev_line = token.line;
        prev_char = token.character;
    }

    data
}

fn is_keyword_kind(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::KwModule
            | TokenKind::KwUse
            | TokenKind::KwPub
            | TokenKind::KwDo
            | TokenKind::KwEnd
            | TokenKind::KwDef
            | TokenKind::KwFn
            | TokenKind::KwLet
            | TokenKind::KwVar
            | TokenKind::KwConst
            | TokenKind::KwType
            | TokenKind::KwTrait
            | TokenKind::KwImpl
            | TokenKind::KwStruct
            | TokenKind::KwEnum
            | TokenKind::KwMemory
            | TokenKind::KwWorking
            | TokenKind::KwEpisodic
            | TokenKind::KwSemantic
            | TokenKind::KwProcedural
            | TokenKind::KwActor
            | TokenKind::KwProtocol
            | TokenKind::KwOn
            | TokenKind::KwSpawn
            | TokenKind::KwSend
            | TokenKind::KwIf
            | TokenKind::KwElsif
            | TokenKind::KwElse
            | TokenKind::KwWhile
            | TokenKind::KwFor
            | TokenKind::KwIn
            | TokenKind::KwLoop
            | TokenKind::KwBreak
            | TokenKind::KwContinue
            | TokenKind::KwReturn
            | TokenKind::KwYield
            | TokenKind::KwNext
            | TokenKind::KwMatch
            | TokenKind::KwWhen
            | TokenKind::KwTry
            | TokenKind::KwRescue
            | TokenKind::KwEnsure
            | TokenKind::KwRaise
            | TokenKind::KwOwn
            | TokenKind::KwBorrow
            | TokenKind::KwRef
            | TokenKind::KwMut
            | TokenKind::KwMove
            | TokenKind::KwDyn
            | TokenKind::KwAs
            | TokenKind::KwAnd
            | TokenKind::KwOr
            | TokenKind::KwNot
            | TokenKind::KwTrue
            | TokenKind::KwFalse
            | TokenKind::KwNil
            | TokenKind::KwSelf_
            | TokenKind::KwSuper
    )
}

fn is_operator_kind(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::Eq
            | TokenKind::EqEq
            | TokenKind::BangEq
            | TokenKind::Lt
            | TokenKind::Gt
            | TokenKind::LtEq
            | TokenKind::GtEq
            | TokenKind::Bang
            | TokenKind::Question
            | TokenKind::PipeGt
            | TokenKind::Pipe
            | TokenKind::DotDot
            | TokenKind::DotDotDot
            | TokenKind::FatArrow
            | TokenKind::Arrow
            | TokenKind::PlusEq
            | TokenKind::MinusEq
            | TokenKind::StarEq
            | TokenKind::SlashEq
            | TokenKind::PercentEq
            | TokenKind::Amp
            | TokenKind::At
    )
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
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(
                    tower_lsp::lsp_types::CodeActionProviderCapability::Simple(true),
                ),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: semantic_token_legend(),
                            range: Some(false),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            ..Default::default()
                        },
                    ),
                ),
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

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> LspResult<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;
        let Some(analysis) = self.analyze_uri(&uri).await else {
            return Ok(None);
        };
        let mut symbols = Vec::new();
        for sym in analysis.symbols().values() {
            let kind = match sym.detail.split_whitespace().next() {
                Some("struct") => SymbolKind::STRUCT,
                Some("enum") => SymbolKind::ENUM,
                Some("trait") | Some("protocol") => SymbolKind::INTERFACE,
                Some("actor") => SymbolKind::CLASS,
                Some("memory") => SymbolKind::FIELD,
                Some("module") => SymbolKind::MODULE,
                Some("const") => SymbolKind::CONSTANT,
                Some("let") => SymbolKind::VARIABLE,
                Some("def") | Some("fn") => SymbolKind::FUNCTION,
                _ => SymbolKind::FUNCTION,
            };
            #[allow(deprecated)]
            symbols.push(DocumentSymbol {
                name: sym.name.clone(),
                detail: Some(sym.detail.clone()),
                kind,
                tags: None,
                deprecated: None,
                range: sym.range,
                selection_range: sym.selection_range,
                children: None,
            });
        }
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> LspResult<Option<Vec<SymbolInformation>>> {
        let query = params.query.to_lowercase();
        let documents = self.documents.lock().await;
        let mut result = Vec::new();
        for (uri, source) in documents.iter() {
            let analysis = analyze_source(source);
            for sym in analysis.symbols().values() {
                if sym.name.to_lowercase().contains(&query) {
                    let kind = match sym.detail.split_whitespace().next() {
                        Some("struct") => SymbolKind::STRUCT,
                        Some("enum") => SymbolKind::ENUM,
                        Some("trait") | Some("protocol") => SymbolKind::INTERFACE,
                        Some("actor") => SymbolKind::CLASS,
                        Some("memory") => SymbolKind::FIELD,
                        Some("module") => SymbolKind::MODULE,
                        Some("const") => SymbolKind::CONSTANT,
                        Some("let") => SymbolKind::VARIABLE,
                        Some("def") | Some("fn") => SymbolKind::FUNCTION,
                        _ => SymbolKind::FUNCTION,
                    };
                    #[allow(deprecated)]
                    result.push(SymbolInformation {
                        name: sym.name.clone(),
                        kind,
                        tags: None,
                        deprecated: None,
                        location: Location {
                            uri: uri.clone(),
                            range: sym.selection_range,
                        },
                        container_name: None,
                    });
                }
            }
        }
        Ok(Some(result))
    }

    async fn rename(&self, params: RenameParams) -> LspResult<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        let documents = self.documents.lock().await;
        let Some(active_source) = documents.get(&uri) else {
            return Ok(None);
        };

        let line_index = LineIndex::new(active_source);
        let offset = line_index.position_to_offset(position);
        let Some((start, end)) = ident_bounds_at_offset(active_source, offset) else {
            return Ok(None);
        };
        let target_name = active_source[start..end].to_string();
        let local_function_span =
            parameter_function_span_at_offset(active_source, &target_name, offset);

        let mut changes = HashMap::new();

        for (doc_uri, source) in documents.iter() {
            let edits = if let Some(function_span) = local_function_span {
                if doc_uri == &uri {
                    rename_text_edits_for_source_in_span(
                        source,
                        &target_name,
                        &new_name,
                        Some(function_span),
                    )
                } else {
                    Vec::new()
                }
            } else {
                rename_text_edits_for_source(source, &target_name, &new_name)
            };

            if !edits.is_empty() {
                changes.insert(doc_uri.clone(), edits);
            }
        }

        Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }))
    }

    async fn code_action(&self, params: CodeActionParams) -> LspResult<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let documents = self.documents.lock().await;
        let Some(source) = documents.get(&uri) else {
            return Ok(None);
        };
        let responses = code_actions_for_source(uri, source)
            .into_iter()
            .map(CodeActionOrCommand::CodeAction)
            .collect();

        Ok(Some(responses))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> LspResult<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;
        let documents = self.documents.lock().await;
        let Some(source) = documents.get(&uri) else {
            return Ok(None);
        };
        let line_index = LineIndex::new(source);
        let data = if garnet_parser::parse_source(source).is_ok() {
            encode_semantic_tokens(source, &line_index)
        } else {
            Vec::new()
        };

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data,
        })))
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
    fn rowan_identifier_spans_drive_rename_sites() {
        let source = "def greet(name) {\n  greet(name)\n}\n";
        let parse = garnet_cst::parse_cst(source);
        let spans = garnet_cst::identifier_spans(parse.syntax(), "greet");

        assert_eq!(spans.len(), 2);
        assert_eq!(&source[spans[0].start..spans[0].end()], "greet");
        assert_eq!(&source[spans[1].start..spans[1].end()], "greet");
    }

    #[test]
    fn rename_function_uses_rowan_identifier_tokens_only() {
        let source = "/// greet docs\ndef greet(name) {\n  greet(name)\n}\n";
        let edits = rename_text_edits_for_source(source, "greet", "hello");

        assert_eq!(edits.len(), 2);
        assert!(edits.iter().all(|edit| edit.new_text == "hello"));
        assert_eq!(edits[0].range.start, Position::new(1, 4));
        assert_eq!(edits[1].range.start, Position::new(2, 2));
    }

    #[test]
    fn rename_parameter_stays_inside_declaring_function() {
        let source = "def greet(name) {\n  name\n}\n\ndef other(name) {\n  name\n}\n";
        let edits =
            rename_scoped_text_edits(source, "name", "person", position_for(source, "name) {"));

        assert_eq!(edits.len(), 2);
        assert!(edits.iter().all(|edit| edit.new_text == "person"));
        assert!(edits.iter().all(|edit| edit.range.start.line < 3));
    }

    #[test]
    fn rename_parameter_use_stays_inside_declaring_function() {
        let source = "def greet(name) {\n  name\n}\n\ndef other(name) {\n  name\n}\n";
        let edits = rename_scoped_text_edits(
            source,
            "name",
            "person",
            position_for(source, "name\n}\n\n"),
        );

        assert_eq!(edits.len(), 2);
        assert!(edits.iter().all(|edit| edit.new_text == "person"));
        assert!(edits.iter().all(|edit| edit.range.start.line < 3));
    }

    #[test]
    fn code_action_add_caps_inserts_before_def() {
        let source = "/// entry point\ndef main() {\n}\n";
        let actions = code_actions_for_source(
            Url::parse("file:///tmp/code_action_add_caps.garnet").expect("url"),
            source,
        );
        let action = actions
            .iter()
            .find(|action| action.title.contains("Add `@caps()`"))
            .expect("caps action");
        let edits = action
            .edit
            .as_ref()
            .and_then(|edit| edit.changes.as_ref())
            .and_then(|changes| changes.values().next())
            .expect("edits");

        assert_eq!(edits[0].new_text, "@caps()\n");
        assert_eq!(edits[0].range.start, Position::new(1, 0));
        assert_eq!(edits[0].range.start, edits[0].range.end);
    }

    #[test]
    fn code_action_refactor_long_parameter_list_offers_struct_scaffold() {
        let source = "def build(a, b, c, d) {\n  a\n}\n";
        let actions = code_actions_for_source(
            Url::parse("file:///tmp/code_action_long_params.garnet").expect("url"),
            source,
        );

        assert!(actions
            .iter()
            .any(|action| action.title.contains("Refactor long parameter list")));
    }

    #[test]
    fn code_action_add_return_type_infers_literal_int() {
        let source = "@caps()\ndef answer() {\n  42\n}\n";
        let actions = code_actions_for_source(
            Url::parse("file:///tmp/code_action_return_type.garnet").expect("url"),
            source,
        );

        assert!(actions
            .iter()
            .any(|action| action.title.contains("Add return type `Int`")));
    }

    #[test]
    fn semantic_legend_exposes_s16_categories() {
        let legend = semantic_token_legend();
        let names: Vec<_> = legend
            .token_types
            .iter()
            .map(SemanticTokenType::as_str)
            .collect();

        assert!(names.contains(&"keyword"));
        assert!(names.contains(&"function"));
        assert!(names.contains(&"parameter"));
        assert!(names.contains(&"capability"));
        assert!(names.contains(&"attribute"));
    }

    #[test]
    fn semantic_tokens_classify_caps_as_capability_and_attribute() {
        let source = "@caps(fs)\ndef main(name) {\n  fs::read_file(name)\n}\n";
        let classified = classify_semantic_tokens(source);

        assert!(classified
            .iter()
            .any(|token| token.text == "caps" && token.kind == "attribute"));
        assert!(classified
            .iter()
            .any(|token| token.text == "fs" && token.kind == "capability"));
        assert!(classified
            .iter()
            .any(|token| token.text == "name" && token.kind == "parameter"));
    }

    #[test]
    fn insert_range_is_zero_width() {
        let source = "def main() {}\n";
        let range = LineIndex::new(source).insert_range_at_offset(0);

        assert_eq!(range.start, Position::new(0, 0));
        assert_eq!(range.end, Position::new(0, 0));
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
