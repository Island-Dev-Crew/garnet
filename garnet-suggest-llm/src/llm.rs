use garnet_check::suggest::{self, Suggestion};
use garnet_parser::ast::Module;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DEFAULT_TOKEN_BUDGET: u32 = 50_000;
pub const REPRO_LOG_RELATIVE_PATH: &str = ".garnet-cache/llm-suggest-log.jsonl";
pub const NON_DETERMINISTIC_STABILITY: &str = "@stability(non-deterministic)";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Provider {
    Anthropic,
    OpenAi,
    Ollama,
}

impl Provider {
    pub fn as_str(self) -> &'static str {
        match self {
            Provider::Anthropic => "anthropic",
            Provider::OpenAi => "openai",
            Provider::Ollama => "ollama",
        }
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmClientDescriptor {
    pub provider: Provider,
    pub model: String,
    pub temperature: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TokenBudget {
    Limited(u32),
    Unbounded,
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self::Limited(DEFAULT_TOKEN_BUDGET)
    }
}

impl TokenBudget {
    pub fn request_limit(self) -> Option<u32> {
        match self {
            TokenBudget::Limited(limit) => Some(limit),
            TokenBudget::Unbounded => None,
        }
    }

    pub fn warning(self) -> Option<&'static str> {
        match self {
            TokenBudget::Limited(_) => None,
            TokenBudget::Unbounded => Some(
                "unbounded LLM budget requested; provider-side context and billing limits still apply",
            ),
        }
    }
}

/// Provider-facing completion trait.
///
/// Authority: `@caps(net)`. Implementations may contact a remote provider or a
/// local model server. The trait itself performs no ambient I/O.
pub trait LlmClient {
    fn descriptor(&self) -> LlmClientDescriptor;
    fn complete(&self, prompt: &str, budget: TokenBudget) -> Result<LlmCompletion, LlmError>;
}

/// HTTP-like transport used by provider clients.
///
/// Authority: `@caps(net)`. Future command-backed transports must additionally
/// declare `@caps(proc)` at their Garnet-facing boundary.
pub trait LlmTransport: Clone {
    fn send(&self, request: LlmHttpRequest) -> Result<LlmHttpResponse, LlmError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmHttpRequest {
    pub provider: Provider,
    pub endpoint: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmHttpResponse {
    pub status: u16,
    pub body: String,
}

impl LlmHttpResponse {
    pub fn ok(body: impl Into<String>) -> Self {
        Self {
            status: 200,
            body: body.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmCompletion {
    pub text: String,
    pub raw_response: String,
}

#[derive(Debug, Clone)]
pub struct AnthropicClient<T: LlmTransport> {
    api_key: String,
    model: String,
    endpoint: String,
    temperature: f32,
    transport: T,
}

impl<T: LlmTransport> AnthropicClient<T> {
    pub fn new(
        api_key: impl Into<String>,
        model: impl Into<String>,
        temperature: f32,
        transport: T,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            endpoint: "https://api.anthropic.com/v1/messages".to_string(),
            temperature,
            transport,
        }
    }

    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    pub fn request(&self, prompt: &str, budget: TokenBudget) -> Result<LlmHttpRequest, LlmError> {
        let mut body = json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": self.temperature,
        });
        let max_tokens = budget.request_limit().unwrap_or(DEFAULT_TOKEN_BUDGET);
        body["max_tokens"] = json!(max_tokens);
        Ok(LlmHttpRequest {
            provider: Provider::Anthropic,
            endpoint: self.endpoint.clone(),
            headers: vec![
                ("content-type".to_string(), "application/json".to_string()),
                ("x-api-key".to_string(), self.api_key.clone()),
                ("anthropic-version".to_string(), "2023-06-01".to_string()),
            ],
            body: serde_json::to_string(&body)?,
        })
    }
}

impl<T: LlmTransport> LlmClient for AnthropicClient<T> {
    fn descriptor(&self) -> LlmClientDescriptor {
        LlmClientDescriptor {
            provider: Provider::Anthropic,
            model: self.model.clone(),
            temperature: self.temperature,
        }
    }

    fn complete(&self, prompt: &str, budget: TokenBudget) -> Result<LlmCompletion, LlmError> {
        let request = self.request(prompt, budget)?;
        let response = self.transport.send(request)?;
        ensure_success(Provider::Anthropic, &response)?;
        Ok(LlmCompletion {
            text: parse_anthropic_text(&response.body)?,
            raw_response: response.body,
        })
    }
}

#[derive(Debug, Clone)]
pub struct OpenAiClient<T: LlmTransport> {
    api_key: String,
    model: String,
    endpoint: String,
    temperature: f32,
    transport: T,
}

impl<T: LlmTransport> OpenAiClient<T> {
    pub fn new(
        api_key: impl Into<String>,
        model: impl Into<String>,
        temperature: f32,
        transport: T,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            temperature,
            transport,
        }
    }

    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    pub fn request(&self, prompt: &str, budget: TokenBudget) -> Result<LlmHttpRequest, LlmError> {
        let mut body = json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": self.temperature,
        });
        if let Some(limit) = budget.request_limit() {
            body["max_completion_tokens"] = json!(limit);
        }
        Ok(LlmHttpRequest {
            provider: Provider::OpenAi,
            endpoint: self.endpoint.clone(),
            headers: vec![
                ("content-type".to_string(), "application/json".to_string()),
                (
                    "authorization".to_string(),
                    format!("Bearer {}", self.api_key),
                ),
            ],
            body: serde_json::to_string(&body)?,
        })
    }
}

impl<T: LlmTransport> LlmClient for OpenAiClient<T> {
    fn descriptor(&self) -> LlmClientDescriptor {
        LlmClientDescriptor {
            provider: Provider::OpenAi,
            model: self.model.clone(),
            temperature: self.temperature,
        }
    }

    fn complete(&self, prompt: &str, budget: TokenBudget) -> Result<LlmCompletion, LlmError> {
        let request = self.request(prompt, budget)?;
        let response = self.transport.send(request)?;
        ensure_success(Provider::OpenAi, &response)?;
        Ok(LlmCompletion {
            text: parse_openai_text(&response.body)?,
            raw_response: response.body,
        })
    }
}

#[derive(Debug, Clone)]
pub struct OllamaClient<T: LlmTransport> {
    model: String,
    endpoint: String,
    temperature: f32,
    transport: T,
}

impl<T: LlmTransport> OllamaClient<T> {
    pub fn new(model: impl Into<String>, temperature: f32, transport: T) -> Self {
        Self {
            model: model.into(),
            endpoint: "http://localhost:11434/api/generate".to_string(),
            temperature,
            transport,
        }
    }

    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    pub fn request(&self, prompt: &str, budget: TokenBudget) -> Result<LlmHttpRequest, LlmError> {
        let mut options = json!({
            "temperature": self.temperature,
        });
        if let Some(limit) = budget.request_limit() {
            options["num_predict"] = json!(limit);
        }
        let body = json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
            "options": options,
        });
        Ok(LlmHttpRequest {
            provider: Provider::Ollama,
            endpoint: self.endpoint.clone(),
            headers: vec![("content-type".to_string(), "application/json".to_string())],
            body: serde_json::to_string(&body)?,
        })
    }
}

impl<T: LlmTransport> LlmClient for OllamaClient<T> {
    fn descriptor(&self) -> LlmClientDescriptor {
        LlmClientDescriptor {
            provider: Provider::Ollama,
            model: self.model.clone(),
            temperature: self.temperature,
        }
    }

    fn complete(&self, prompt: &str, budget: TokenBudget) -> Result<LlmCompletion, LlmError> {
        let request = self.request(prompt, budget)?;
        let response = self.transport.send(request)?;
        ensure_success(Provider::Ollama, &response)?;
        Ok(LlmCompletion {
            text: parse_ollama_text(&response.body)?,
            raw_response: response.body,
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilationHistory {
    pub episodes: Vec<CompilationEpisode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilationEpisode {
    pub raw: String,
}

pub fn read_compilation_history(path: impl AsRef<Path>) -> Result<CompilationHistory, LlmError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(CompilationHistory::default());
    }
    let text = fs::read_to_string(path).map_err(|source| LlmError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(CompilationHistory {
        episodes: text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| CompilationEpisode {
                raw: line.trim().to_string(),
            })
            .collect(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmSuggestion {
    pub function: Option<String>,
    pub message: String,
    pub stability: String,
}

impl LlmSuggestion {
    pub fn new(function: Option<String>, message: impl Into<String>) -> Self {
        let mut message = message.into().trim().to_string();
        if !message.contains(NON_DETERMINISTIC_STABILITY) {
            message = format!("{NON_DETERMINISTIC_STABILITY} {message}");
        }
        Self {
            function,
            message,
            stability: NON_DETERMINISTIC_STABILITY.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmSuggestionReport {
    pub deterministic: Vec<Suggestion>,
    pub llm: Vec<LlmSuggestion>,
    pub prompt_hash: String,
    pub log_path: Option<PathBuf>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmSuggestOptions {
    pub token_budget: TokenBudget,
    pub log_root: Option<PathBuf>,
}

impl Default for LlmSuggestOptions {
    fn default() -> Self {
        Self {
            token_budget: TokenBudget::default(),
            log_root: Some(PathBuf::from(".")),
        }
    }
}

impl LlmSuggestOptions {
    pub fn without_log(mut self) -> Self {
        self.log_root = None;
        self
    }

    pub fn with_log_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.log_root = Some(root.into());
        self
    }

    pub fn with_token_budget(mut self, budget: TokenBudget) -> Self {
        self.token_budget = budget;
        self
    }
}

pub fn suggest_for_source_with_llm(
    source: &str,
    history: Option<&CompilationHistory>,
    client: &dyn LlmClient,
    options: LlmSuggestOptions,
) -> Result<LlmSuggestionReport, LlmError> {
    let module =
        garnet_parser::parse_source(source).map_err(|err| LlmError::Parse(err.to_string()))?;
    suggest_for_module_with_source_and_llm(&module, source, history, client, options)
}

/// Additive LLM advisory pass over an already parsed module.
///
/// Prefer `suggest_for_source_with_llm` when the original source is available;
/// this compatibility entry point uses an AST-debug prompt because
/// `garnet_parser::ast::Module` does not retain full source text.
pub fn suggest_for_module_with_llm(
    module: &Module,
    history: Option<&CompilationHistory>,
    client: &dyn LlmClient,
) -> Result<LlmSuggestionReport, LlmError> {
    let source = format!("{module:#?}");
    let options = LlmSuggestOptions::default();
    suggest_for_module_with_source_and_llm(module, &source, history, client, options)
}

pub fn suggest_for_module_with_source_and_llm(
    module: &Module,
    source: &str,
    history: Option<&CompilationHistory>,
    client: &dyn LlmClient,
    options: LlmSuggestOptions,
) -> Result<LlmSuggestionReport, LlmError> {
    let deterministic = suggest::suggest_for_module(module);
    let prompt = build_prompt(source, &deterministic, history);
    let prompt_hash = prompt_hash(&prompt);
    let mut warnings = Vec::new();
    if let Some(warning) = options.token_budget.warning() {
        warnings.push(warning.to_string());
    }
    let completion = client.complete(&prompt, options.token_budget)?;
    let llm = parse_llm_suggestions(&completion.text);
    let descriptor = client.descriptor();
    let log_path = if let Some(root) = &options.log_root {
        let entry = ReproLogEntry::from_parts(
            prompt_hash.clone(),
            descriptor,
            completion.raw_response,
            llm.clone(),
            options.token_budget,
            warnings.clone(),
        );
        Some(append_repro_log(root, &entry)?)
    } else {
        None
    };
    Ok(LlmSuggestionReport {
        deterministic,
        llm,
        prompt_hash,
        log_path,
        warnings,
    })
}

pub fn build_prompt(
    source: &str,
    deterministic: &[Suggestion],
    history: Option<&CompilationHistory>,
) -> String {
    let mut prompt = String::new();
    prompt.push_str("You are Garnet's compiler-as-agent advisory tier.\n");
    prompt.push_str("Rules:\n");
    prompt.push_str("- Deterministic analyzer findings below are ground truth.\n");
    prompt.push_str("- Do not repeat them unless adding genuinely new context.\n");
    prompt.push_str("- Return JSON: [{\"function\":\"name-or-null\",\"message\":\"advice\"}].\n");
    prompt.push_str("- Every suggestion is advisory and non-deterministic.\n\n");

    prompt.push_str("Deterministic findings:\n");
    if deterministic.is_empty() {
        prompt.push_str("- none\n");
    } else {
        for item in deterministic {
            prompt.push_str(&format!(
                "- [{}] function `{}`: {}\n",
                item.rule.id(),
                item.function,
                item.message
            ));
        }
    }

    prompt.push_str("\nCompilation history from .garnet-cache/episodes.log:\n");
    match history {
        Some(history) if !history.episodes.is_empty() => {
            for (idx, episode) in history.episodes.iter().enumerate() {
                prompt.push_str(&format!("{}. {}\n", idx + 1, episode.raw));
            }
        }
        _ => prompt.push_str("- none\n"),
    }

    prompt.push_str("\nSource:\n```garnet\n");
    prompt.push_str(source);
    if !source.ends_with('\n') {
        prompt.push('\n');
    }
    prompt.push_str("```\n");
    prompt
}

pub fn parse_llm_suggestions(text: &str) -> Vec<LlmSuggestion> {
    if let Ok(Value::Array(items)) = serde_json::from_str::<Value>(text) {
        let suggestions: Vec<LlmSuggestion> = items
            .into_iter()
            .filter_map(|item| {
                let message = item.get("message")?.as_str()?.trim();
                if message.is_empty() {
                    return None;
                }
                let function = item
                    .get("function")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty() && *value != "null")
                    .map(ToOwned::to_owned);
                Some(LlmSuggestion::new(function, message))
            })
            .collect();
        if !suggestions.is_empty() {
            return suggestions;
        }
    }

    let suggestions: Vec<LlmSuggestion> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.trim_start_matches("- ").trim())
        .filter(|line| !line.is_empty())
        .map(|line| LlmSuggestion::new(None, line))
        .collect();
    if suggestions.is_empty() {
        Vec::new()
    } else {
        suggestions
    }
}

pub fn prompt_hash(prompt: &str) -> String {
    format!("blake3:{}", blake3::hash(prompt.as_bytes()).to_hex())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReproLogEntry {
    pub prompt_hash: String,
    pub provider: Provider,
    pub model: String,
    pub temperature: f32,
    pub response: String,
    pub suggestions_emitted: Vec<LlmSuggestion>,
    pub timestamp: String,
    pub token_budget: TokenBudget,
    pub warnings: Vec<String>,
}

impl ReproLogEntry {
    fn from_parts(
        prompt_hash: String,
        descriptor: LlmClientDescriptor,
        response: String,
        suggestions_emitted: Vec<LlmSuggestion>,
        token_budget: TokenBudget,
        warnings: Vec<String>,
    ) -> Self {
        Self {
            prompt_hash,
            provider: descriptor.provider,
            model: descriptor.model,
            temperature: descriptor.temperature,
            response,
            suggestions_emitted,
            timestamp: timestamp(),
            token_budget,
            warnings,
        }
    }
}

pub fn append_repro_log(
    root: impl AsRef<Path>,
    entry: &ReproLogEntry,
) -> Result<PathBuf, LlmError> {
    let path = root.as_ref().join(REPRO_LOG_RELATIVE_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LlmError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|source| LlmError::Io {
            path: path.clone(),
            source,
        })?;
    let line = serde_json::to_string(entry)?;
    writeln!(file, "{line}").map_err(|source| LlmError::Io {
        path: path.clone(),
        source,
    })?;
    Ok(path)
}

fn timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    format!("unix:{secs}")
}

fn ensure_success(provider: Provider, response: &LlmHttpResponse) -> Result<(), LlmError> {
    if (200..300).contains(&response.status) {
        Ok(())
    } else {
        Err(LlmError::Http {
            provider,
            status: response.status,
            body: response.body.clone(),
        })
    }
}

fn parse_anthropic_text(body: &str) -> Result<String, LlmError> {
    let value: Value = serde_json::from_str(body)?;
    let text = value
        .get("content")
        .and_then(Value::as_array)
        .and_then(|items| {
            items.iter().find_map(|item| {
                if item.get("type").and_then(Value::as_str) == Some("text") {
                    item.get("text").and_then(Value::as_str)
                } else {
                    None
                }
            })
        })
        .ok_or(LlmError::MissingText {
            provider: Provider::Anthropic,
        })?;
    Ok(text.to_string())
}

fn parse_openai_text(body: &str) -> Result<String, LlmError> {
    let value: Value = serde_json::from_str(body)?;
    let text = value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .ok_or(LlmError::MissingText {
            provider: Provider::OpenAi,
        })?;
    Ok(text.to_string())
}

fn parse_ollama_text(body: &str) -> Result<String, LlmError> {
    let value: Value = serde_json::from_str(body)?;
    let text = value
        .get("response")
        .and_then(Value::as_str)
        .ok_or(LlmError::MissingText {
            provider: Provider::Ollama,
        })?;
    Ok(text.to_string())
}

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("failed to parse Garnet source for LLM suggestions: {0}")]
    Parse(String),
    #[error("{provider} provider returned HTTP {status}: {body}")]
    Http {
        provider: Provider,
        status: u16,
        body: String,
    },
    #[error("{provider} provider response did not contain generated text")]
    MissingText { provider: Provider },
    #[error("LLM transport error: {0}")]
    Transport(String),
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct RecordingTransport {
        response: LlmHttpResponse,
        requests: Arc<Mutex<Vec<LlmHttpRequest>>>,
    }

    impl RecordingTransport {
        fn new(response: LlmHttpResponse) -> Self {
            Self {
                response,
                requests: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn requests(&self) -> Vec<LlmHttpRequest> {
            self.requests.lock().expect("requests lock").clone()
        }
    }

    impl LlmTransport for RecordingTransport {
        fn send(&self, request: LlmHttpRequest) -> Result<LlmHttpResponse, LlmError> {
            self.requests.lock().expect("requests lock").push(request);
            Ok(self.response.clone())
        }
    }

    #[test]
    fn anthropic_client_builds_messages_request_and_parses_text() {
        let transport = RecordingTransport::new(LlmHttpResponse::ok(
            r#"{"content":[{"type":"text","text":"[{\"message\":\"Prefer a smaller helper\"}]"}]}"#,
        ));
        let client = AnthropicClient::new("secret", "claude-opus-4-7", 0.2, transport.clone());

        let completion = client
            .complete("review this", TokenBudget::Limited(128))
            .expect("completion");

        assert!(completion.text.contains("Prefer a smaller helper"));
        let request = transport.requests().pop().expect("request recorded");
        assert_eq!(Provider::Anthropic, request.provider);
        assert_eq!("https://api.anthropic.com/v1/messages", request.endpoint);
        assert!(request
            .headers
            .iter()
            .any(|(key, value)| key == "anthropic-version" && value == "2023-06-01"));
        let body: Value = serde_json::from_str(&request.body).expect("json body");
        assert_eq!("claude-opus-4-7", body["model"]);
        assert_eq!(128, body["max_tokens"]);
    }

    #[test]
    fn openai_client_builds_chat_request_and_parses_text() {
        let transport = RecordingTransport::new(LlmHttpResponse::ok(
            r#"{"choices":[{"message":{"content":"[{\"message\":\"Name the cap explicitly\"}]"}}]}"#,
        ));
        let client = OpenAiClient::new("secret", "gpt-5.4", 0.1, transport.clone());

        let completion = client
            .complete("review this", TokenBudget::Limited(256))
            .expect("completion");

        assert!(completion.text.contains("Name the cap"));
        let request = transport.requests().pop().expect("request recorded");
        assert_eq!(Provider::OpenAi, request.provider);
        assert!(request
            .headers
            .iter()
            .any(|(key, value)| key == "authorization" && value == "Bearer secret"));
        let body: Value = serde_json::from_str(&request.body).expect("json body");
        assert_eq!("gpt-5.4", body["model"]);
        assert_eq!(256, body["max_completion_tokens"]);
    }

    #[test]
    fn ollama_client_disables_streaming_and_parses_response() {
        let transport = RecordingTransport::new(LlmHttpResponse::ok(
            r#"{"response":"[{\"function\":\"main\",\"message\":\"Split the example\"}]","done":true}"#,
        ));
        let client = OllamaClient::new("gemma3", 0.0, transport.clone());

        let completion = client
            .complete("review this", TokenBudget::Limited(64))
            .expect("completion");

        assert!(completion.text.contains("Split the example"));
        let request = transport.requests().pop().expect("request recorded");
        assert_eq!("http://localhost:11434/api/generate", request.endpoint);
        let body: Value = serde_json::from_str(&request.body).expect("json body");
        assert_eq!(false, body["stream"]);
        assert_eq!(64, body["options"]["num_predict"]);
    }

    #[test]
    fn prompt_includes_deterministic_findings_history_and_source() {
        let source = "def helper(a, b, c, d) { }\n";
        let module = garnet_parser::parse_source(source).expect("parse source");
        let deterministic = suggest::suggest_for_module(&module);
        let history = CompilationHistory {
            episodes: vec![CompilationEpisode {
                raw: r#"{"status":"failed","error":"missing @caps"}"#.to_string(),
            }],
        };

        let prompt = build_prompt(source, &deterministic, Some(&history));

        assert!(prompt.contains("Deterministic findings"));
        assert!(prompt.contains("managed-fn-missing-caps"));
        assert!(prompt.contains(".garnet-cache/episodes.log"));
        assert!(prompt.contains("def helper"));
    }

    #[test]
    fn source_suggestion_run_logs_jsonl_without_replacing_deterministic_findings() {
        let temp = tempfile::tempdir().expect("tempdir");
        let response = r#"{"content":[{"type":"text","text":"[{\"function\":\"helper\",\"message\":\"Extract validation into a named function\"}]"}]}"#;
        let transport = RecordingTransport::new(LlmHttpResponse::ok(response));
        let client = AnthropicClient::new("secret", "claude-opus-4-7", 0.2, transport);
        let source = "def helper(a, b, c, d) { }\n";

        let report = suggest_for_source_with_llm(
            source,
            None,
            &client,
            LlmSuggestOptions::default().with_log_root(temp.path()),
        )
        .expect("suggestions");

        assert!(
            report
                .deterministic
                .iter()
                .any(|item| item.rule.id() == "managed-fn-missing-caps"),
            "{:?}",
            report.deterministic
        );
        assert_eq!(1, report.llm.len());
        assert!(report.llm[0].message.contains(NON_DETERMINISTIC_STABILITY));
        let log_path = report.log_path.expect("log path");
        let line = fs::read_to_string(log_path)
            .expect("read log")
            .lines()
            .next()
            .expect("one log line")
            .to_string();
        let entry: ReproLogEntry = serde_json::from_str(&line).expect("log json");
        assert_eq!(Provider::Anthropic, entry.provider);
        assert!(entry.prompt_hash.starts_with("blake3:"));
        assert_eq!(1, entry.suggestions_emitted.len());
        assert!(!line.contains("secret"));
    }

    #[test]
    fn unbounded_budget_is_logged_as_a_warning() {
        let temp = tempfile::tempdir().expect("tempdir");
        let transport = RecordingTransport::new(LlmHttpResponse::ok(
            r#"{"response":"[{\"message\":\"Keep this opt-in\"}]","done":true}"#,
        ));
        let client = OllamaClient::new("gemma3", 0.0, transport);

        let report = suggest_for_source_with_llm(
            "@caps()\ndef helper() { 1 }\n",
            None,
            &client,
            LlmSuggestOptions::default()
                .with_log_root(temp.path())
                .with_token_budget(TokenBudget::Unbounded),
        )
        .expect("suggestions");

        assert_eq!(1, report.warnings.len());
        assert!(report.warnings[0].contains("unbounded"));
    }
}
