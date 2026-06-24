use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tauri::State;
use uuid::Uuid;

use crate::commands::settings::get_setting;
use crate::db::DbState;
use crate::frontmatter;
use crate::secrets;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AiInteraction {
    pub id: String,
    pub jana_id: String,
    pub interaction_type: String,
    pub prompt: Option<String>,
    pub response: String,
    pub model: String,
    pub created_at: i64,
}

const SUMMARY_SYSTEM: &str = "Summarize the following note concisely. Focus on key points and action items. Return only the summary, no preamble.";

const NAME_SYSTEM: &str = "Suggest a short, descriptive file name for the following note based on its content. Use lowercase kebab-case (words separated by hyphens), 2 to 5 words, with no file extension, no path, and no surrounding quotes. Respond with only the file name and nothing else.";

fn http_client() -> Result<reqwest::Client, String> {
    // Generous timeout — reasoning models (gpt-5.x, o-series, Claude with thinking)
    // can take well over a minute to produce a summary.
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())
}

/// Send a request, retrying transient failures (429 rate-limit, 5xx including
/// 529 overloaded, and network errors) with exponential backoff. Raw HTTP doesn't
/// get the official SDKs' built-in retry, so we add a small one here.
async fn send_with_retry(builder: reqwest::RequestBuilder) -> Result<reqwest::Response, String> {
    const MAX_RETRIES: u32 = 3;
    let mut attempt = 0u32;
    loop {
        let req = builder
            .try_clone()
            .ok_or("request could not be retried")?;
        match req.send().await {
            Ok(resp) => {
                let code = resp.status().as_u16();
                if (code == 429 || code >= 500) && attempt < MAX_RETRIES {
                    attempt += 1;
                    // 1s, 2s, 4s
                    tokio::time::sleep(std::time::Duration::from_millis(500u64 << attempt)).await;
                    continue;
                }
                return Ok(resp);
            }
            Err(e) => {
                // Retry genuine connection failures, but not timeouts — retrying a
                // slow model just makes the user wait another full timeout.
                if e.is_connect() && attempt < MAX_RETRIES {
                    attempt += 1;
                    tokio::time::sleep(std::time::Duration::from_millis(500u64 << attempt)).await;
                    continue;
                }
                return Err(if e.is_timeout() {
                    "Request timed out — the model is taking too long. Try again, or pick a faster model.".to_string()
                } else {
                    format!("Request failed: {}", e)
                });
            }
        }
    }
}

/// Read a response body once, surface non-2xx status with a friendly message
/// (provider error bodies are more useful than a generic parse failure), then parse.
async fn json_or_err<T: DeserializeOwned>(resp: reqwest::Response, what: &str) -> Result<T, String> {
    let status = resp.status();
    let code = status.as_u16();
    let text = resp.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        let msg = match code {
            429 => format!("{} is rate limited — wait a moment and try again.", what),
            529 => format!("{} is temporarily overloaded — try again in a moment.", what),
            401 | 403 => format!(
                "{} rejected the API key (HTTP {}). Check your key in Settings.",
                what, code
            ),
            _ => format!("{} error (HTTP {}): {}", what, code, text),
        };
        return Err(msg);
    }
    serde_json::from_str(&text).map_err(|e| format!("Failed to parse {} response: {}", what, e))
}

// --- OpenAI-compatible (OpenAI cloud + local LM Studio) ---

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

async fn complete_openai_compat(
    url: &str,
    model: &str,
    api_key: Option<&str>,
    system: &str,
    content: String,
) -> Result<String, String> {
    let request = OpenAIRequest {
        model: model.to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content,
            },
        ],
        temperature: 0.3,
    };

    let mut builder = http_client()?.post(url).json(&request);
    if let Some(key) = api_key {
        builder = builder.bearer_auth(key);
    }
    let resp = send_with_retry(builder).await?;
    let body: OpenAIResponse = json_or_err(resp, "LLM").await?;
    body.choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "No response from LLM".to_string())
}

// --- OpenAI Responses API (cloud) — required for gpt-5.x; also serves gpt-4o ---
// The legacy /v1/chat/completions endpoint rejects newer models ("not a chat
// model"); the Responses API is OpenAI's unified endpoint across all current
// models. `instructions` is the system prompt; output text is in
// output[].content[] where type == "output_text".

#[derive(Serialize)]
struct ResponsesRequest {
    model: String,
    instructions: String,
    input: String,
}

#[derive(Deserialize)]
struct ResponsesResponse {
    #[serde(default)]
    output: Vec<ResponsesItem>,
}

#[derive(Deserialize)]
struct ResponsesItem {
    #[serde(default)]
    content: Vec<ResponsesContent>,
}

#[derive(Deserialize)]
struct ResponsesContent {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(default)]
    text: String,
}

async fn complete_openai_responses(
    model: &str,
    api_key: &str,
    system: &str,
    content: String,
) -> Result<String, String> {
    // No `temperature`: reasoning models (gpt-5.x, o-series) reject it.
    let request = ResponsesRequest {
        model: model.to_string(),
        instructions: system.to_string(),
        input: content,
    };

    let builder = http_client()?
        .post("https://api.openai.com/v1/responses")
        .bearer_auth(api_key)
        .json(&request);
    let resp = send_with_retry(builder).await?;
    let body: ResponsesResponse = json_or_err(resp, "OpenAI").await?;

    let text: String = body
        .output
        .iter()
        .flat_map(|item| item.content.iter())
        .filter(|c| c.content_type == "output_text")
        .map(|c| c.text.as_str())
        .collect();
    if text.is_empty() {
        return Err("OpenAI returned no text (the request may have been refused or truncated).".to_string());
    }
    Ok(text)
}

// --- Anthropic (Claude) — top-level `system`, no sampling params on Opus 4.x ---

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<ChatMessage>,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    #[serde(default)]
    content: Vec<AnthropicBlock>,
    #[serde(default)]
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[serde(default)]
    text: String,
}

async fn complete_anthropic(
    model: &str,
    api_key: &str,
    system: &str,
    content: String,
    max_tokens: u32,
) -> Result<String, String> {
    let request = AnthropicRequest {
        model: model.to_string(),
        max_tokens,
        system: system.to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content,
        }],
    };

    let builder = http_client()?
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&request);
    let resp = send_with_retry(builder).await?;
    let body: AnthropicResponse = json_or_err(resp, "Claude").await?;

    if body.stop_reason.as_deref() == Some("refusal") {
        return Err("Claude declined to summarize this content.".to_string());
    }
    body.content
        .into_iter()
        .find(|b| b.block_type == "text")
        .map(|b| b.text)
        .ok_or_else(|| "No response from Claude".to_string())
}

// --- Google Gemini — systemInstruction + contents/parts ---

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Serialize)]
struct GeminiSystem {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiRequest {
    #[serde(rename = "systemInstruction")]
    system_instruction: GeminiSystem,
    contents: Vec<GeminiContent>,
}

#[derive(Deserialize)]
struct GeminiResponse {
    #[serde(default)]
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiRespContent,
}

#[derive(Deserialize)]
struct GeminiRespContent {
    #[serde(default)]
    parts: Vec<GeminiRespPart>,
}

#[derive(Deserialize)]
struct GeminiRespPart {
    #[serde(default)]
    text: String,
}

async fn complete_gemini(
    model: &str,
    api_key: &str,
    system: &str,
    content: String,
) -> Result<String, String> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        model
    );
    let request = GeminiRequest {
        system_instruction: GeminiSystem {
            parts: vec![GeminiPart {
                text: system.to_string(),
            }],
        },
        contents: vec![GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart { text: content }],
        }],
    };

    let builder = http_client()?
        .post(&url)
        .header("x-goog-api-key", api_key)
        .json(&request);
    let resp = send_with_retry(builder).await?;
    let body: GeminiResponse = json_or_err(resp, "Gemini").await?;
    body.candidates
        .into_iter()
        .next()
        .and_then(|c| c.content.parts.into_iter().next())
        .map(|p| p.text)
        .ok_or_else(|| "No response from Gemini".to_string())
}

fn missing_key(provider: &str) -> String {
    format!("No API key set for {}. Add it in Settings.", provider)
}

/// Run `content` through whichever provider is configured, using `system` as the
/// instruction. `anthropic_max_tokens` caps the Anthropic response (other providers
/// rely on the prompt to bound length). Returns the generated text and the resolved
/// model id. Shared by `summarize_file` and `suggest_name`.
async fn run_completion(
    pool: &sqlx::SqlitePool,
    system: &str,
    content: String,
    anthropic_max_tokens: u32,
) -> Result<(String, String), String> {
    let provider = get_setting(pool, "ai_provider", "local").await;
    match provider.as_str() {
        "openai" => {
            let model = get_setting(pool, "openai_model", "gpt-4o-mini").await;
            let key = secrets::get_api_key("openai").ok_or_else(|| missing_key("OpenAI"))?;
            Ok((complete_openai_responses(&model, &key, system, content).await?, model))
        }
        "anthropic" => {
            let model = get_setting(pool, "anthropic_model", "claude-opus-4-8").await;
            let key = secrets::get_api_key("anthropic").ok_or_else(|| missing_key("Claude"))?;
            Ok((
                complete_anthropic(&model, &key, system, content, anthropic_max_tokens).await?,
                model,
            ))
        }
        "gemini" => {
            let model = get_setting(pool, "gemini_model", "gemini-2.0-flash").await;
            let key = secrets::get_api_key("gemini").ok_or_else(|| missing_key("Gemini"))?;
            Ok((complete_gemini(&model, &key, system, content).await?, model))
        }
        _ => {
            // Local OpenAI-compatible endpoint (e.g. LM Studio) — no API key.
            let url = get_setting(
                pool,
                "llm_url",
                "http://192.168.77.1:1234/v1/chat/completions",
            )
            .await;
            let model = get_setting(pool, "llm_model", "qwen3-vl-30b").await;
            Ok((
                complete_openai_compat(&url, &model, None, system, content).await?,
                model,
            ))
        }
    }
}

/// Upsert an AI interaction for a file: replace any existing row of the same
/// `interaction_type` for this `jana_id`, then insert the new one.
async fn store_interaction(
    pool: &sqlx::SqlitePool,
    jana_id: &str,
    interaction_type: &str,
    response: String,
    model: String,
) -> Result<AiInteraction, String> {
    let interaction = AiInteraction {
        id: Uuid::new_v4().to_string(),
        jana_id: jana_id.to_string(),
        interaction_type: interaction_type.to_string(),
        prompt: None,
        response,
        model,
        created_at: chrono::Utc::now().timestamp(),
    };

    sqlx::query("DELETE FROM file_ai_interactions WHERE jana_id = ? AND interaction_type = ?")
        .bind(jana_id)
        .bind(interaction_type)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query(
        "INSERT INTO file_ai_interactions (id, jana_id, interaction_type, prompt, response, model, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&interaction.id)
    .bind(&interaction.jana_id)
    .bind(&interaction.interaction_type)
    .bind(&interaction.prompt)
    .bind(&interaction.response)
    .bind(&interaction.model)
    .bind(interaction.created_at)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(interaction)
}

#[tauri::command]
pub async fn summarize_file(
    jana_id: String,
    file_path: String,
    state: State<'_, DbState>,
) -> Result<AiInteraction, String> {
    let raw = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    let content = frontmatter::parse_frontmatter(&raw).content;

    let (summary_text, model) = run_completion(&state.pool, SUMMARY_SYSTEM, content, 2048).await?;
    store_interaction(&state.pool, &jana_id, "summary", summary_text, model).await
}

/// Trim an LLM name suggestion down to a single clean file name: first non-empty
/// line, with surrounding quotes/backticks and a trailing period stripped.
fn clean_name_suggestion(raw: &str) -> String {
    raw.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("")
        .trim_matches(|c| c == '"' || c == '\'' || c == '`' || c == '.')
        .trim()
        .to_string()
}

#[tauri::command]
pub async fn suggest_name(
    jana_id: String,
    file_path: String,
    state: State<'_, DbState>,
) -> Result<AiInteraction, String> {
    let raw = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    let content = frontmatter::parse_frontmatter(&raw).content;

    // A file name is short, so cap Anthropic output tightly.
    let (name_text, model) = run_completion(&state.pool, NAME_SYSTEM, content, 64).await?;
    let name = clean_name_suggestion(&name_text);
    if name.is_empty() {
        return Err("The model returned an empty name suggestion.".to_string());
    }
    store_interaction(&state.pool, &jana_id, "name_suggestion", name, model).await
}

#[tauri::command]
pub async fn get_file_summary(
    jana_id: String,
    state: State<'_, DbState>,
) -> Result<Option<AiInteraction>, String> {
    sqlx::query_as::<_, AiInteraction>(
        "SELECT id, jana_id, interaction_type, prompt, response, model, created_at
         FROM file_ai_interactions
         WHERE jana_id = ? AND interaction_type = 'summary'
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&jana_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_name_suggestion(
    jana_id: String,
    state: State<'_, DbState>,
) -> Result<Option<AiInteraction>, String> {
    sqlx::query_as::<_, AiInteraction>(
        "SELECT id, jana_id, interaction_type, prompt, response, model, created_at
         FROM file_ai_interactions
         WHERE jana_id = ? AND interaction_type = 'name_suggestion'
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&jana_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| e.to_string())
}

// --- Model discovery (populate the Settings model dropdown) ---

#[derive(Deserialize)]
struct IdList {
    #[serde(default)]
    data: Vec<IdEntry>,
}

#[derive(Deserialize)]
struct IdEntry {
    id: String,
}

#[derive(Deserialize)]
struct GeminiModelList {
    #[serde(default)]
    models: Vec<GeminiModelEntry>,
}

#[derive(Deserialize)]
struct GeminiModelEntry {
    name: String,
    #[serde(default, rename = "supportedGenerationMethods")]
    methods: Vec<String>,
}

/// Keep only text-capable OpenAI models — the list also includes embeddings,
/// audio, image, and search models that can't summarize.
fn is_openai_chat_model(id: &str) -> bool {
    let prefix_ok = id.starts_with("gpt")
        || id.starts_with("chatgpt")
        || id.starts_with("o1")
        || id.starts_with("o3")
        || id.starts_with("o4");
    if !prefix_ok {
        return false;
    }
    const EXCLUDE: &[&str] = &[
        "image", "audio", "realtime", "transcribe", "tts", "embedding", "moderation", "search",
        "dall",
    ];
    !EXCLUDE.iter().any(|kw| id.contains(kw))
}

/// List the models available for a provider, using its stored key. Powers the
/// "Fetch models" button in Settings; manual entry remains available regardless.
#[tauri::command]
pub async fn list_models(
    provider: String,
    state: State<'_, DbState>,
) -> Result<Vec<String>, String> {
    let client = http_client()?;
    match provider.as_str() {
        "openai" => {
            let key = secrets::get_api_key("openai").ok_or_else(|| missing_key("OpenAI"))?;
            let resp = send_with_retry(client.get("https://api.openai.com/v1/models").bearer_auth(&key))
                .await?;
            let list: IdList = json_or_err(resp, "OpenAI").await?;
            let mut ids: Vec<String> = list
                .data
                .into_iter()
                .map(|m| m.id)
                .filter(|id| is_openai_chat_model(id))
                .collect();
            ids.sort();
            Ok(ids)
        }
        "anthropic" => {
            let key = secrets::get_api_key("anthropic").ok_or_else(|| missing_key("Claude"))?;
            let resp = send_with_retry(
                client
                    .get("https://api.anthropic.com/v1/models")
                    .header("x-api-key", &key)
                    .header("anthropic-version", "2023-06-01"),
            )
            .await?;
            let list: IdList = json_or_err(resp, "Claude").await?;
            Ok(list.data.into_iter().map(|m| m.id).collect())
        }
        "gemini" => {
            let key = secrets::get_api_key("gemini").ok_or_else(|| missing_key("Gemini"))?;
            let resp = send_with_retry(
                client
                    .get("https://generativelanguage.googleapis.com/v1beta/models")
                    .header("x-goog-api-key", &key),
            )
            .await?;
            let list: GeminiModelList = json_or_err(resp, "Gemini").await?;
            Ok(list
                .models
                .into_iter()
                .filter(|m| m.methods.iter().any(|x| x == "generateContent"))
                .map(|m| m.name.trim_start_matches("models/").to_string())
                .collect())
        }
        _ => {
            // Local OpenAI-compatible: GET <base>/models.
            let url = get_setting(
                &state.pool,
                "llm_url",
                "http://192.168.77.1:1234/v1/chat/completions",
            )
            .await;
            let base = url.strip_suffix("/chat/completions").unwrap_or(&url);
            let resp = send_with_retry(client.get(format!("{}/models", base))).await?;
            let list: IdList = json_or_err(resp, "LLM").await?;
            Ok(list.data.into_iter().map(|m| m.id).collect())
        }
    }
}
