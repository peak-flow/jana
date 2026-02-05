use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tauri::State;
use uuid::Uuid;

use crate::db::DbState;
use crate::frontmatter;

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

#[tauri::command]
pub async fn summarize_file(
    jana_id: String,
    file_path: String,
    state: State<'_, DbState>,
) -> Result<AiInteraction, String> {
    // Read LLM settings
    let llm_url: String = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'llm_url'")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    let llm_model: String = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'llm_model'")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    // Read content from file, strip frontmatter
    let raw = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    let parsed = frontmatter::parse_frontmatter(&raw);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())?;

    let request = OpenAIRequest {
        model: llm_model.clone(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "Summarize the following note concisely. Focus on key points and action items. Return only the summary, no preamble.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: parsed.content,
            },
        ],
        temperature: 0.3,
    };

    let response = client
        .post(&llm_url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("LLM request failed: {}", e))?;

    let response_body = response
        .json::<OpenAIResponse>()
        .await
        .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

    let summary_text = response_body
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "No response from LLM".to_string())?;

    let now = chrono::Utc::now().timestamp();
    let interaction = AiInteraction {
        id: Uuid::new_v4().to_string(),
        jana_id: jana_id.clone(),
        interaction_type: "summary".to_string(),
        prompt: None,
        response: summary_text,
        model: llm_model,
        created_at: now,
    };

    // Upsert: replace existing summary for this jana_id
    sqlx::query("DELETE FROM file_ai_interactions WHERE jana_id = ? AND interaction_type = 'summary'")
        .bind(&jana_id)
        .execute(&state.pool)
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
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(interaction)
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
         ORDER BY created_at DESC LIMIT 1"
    )
    .bind(&jana_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| e.to_string())
}
