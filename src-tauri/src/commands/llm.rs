use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tauri::State;
use uuid::Uuid;

use crate::db::DbState;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AiSummary {
    pub id: String,
    pub note_id: String,
    pub summary: String,
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
pub async fn summarize_note(note_id: String, state: State<'_, DbState>) -> Result<AiSummary, String> {
    // Read LLM settings from database
    let llm_url: String = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'llm_url'")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    let llm_model: String = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'llm_model'")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    // Fetch note content
    let content: String = sqlx::query_scalar("SELECT content FROM notes WHERE id = ?")
        .bind(&note_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    // Call LM Studio
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
                content,
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

    // Store summary
    let now = chrono::Utc::now().timestamp();
    let summary = AiSummary {
        id: Uuid::new_v4().to_string(),
        note_id: note_id.clone(),
        summary: summary_text,
        model: llm_model,
        created_at: now,
    };

    // Upsert: replace existing summary for this note
    sqlx::query("DELETE FROM ai_summaries WHERE note_id = ?")
        .bind(&note_id)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("INSERT INTO ai_summaries (id, note_id, summary, model, created_at) VALUES (?, ?, ?, ?, ?)")
        .bind(&summary.id)
        .bind(&summary.note_id)
        .bind(&summary.summary)
        .bind(&summary.model)
        .bind(summary.created_at)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(summary)
}

#[tauri::command]
pub async fn get_summary(note_id: String, state: State<'_, DbState>) -> Result<Option<AiSummary>, String> {
    sqlx::query_as::<_, AiSummary>(
        "SELECT id, note_id, summary, model, created_at FROM ai_summaries WHERE note_id = ? ORDER BY created_at DESC LIMIT 1"
    )
    .bind(&note_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| e.to_string())
}
