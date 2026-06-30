use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

use crate::db::DbState;

/// AI configuration. The active `provider` selects which backend Summarize uses;
/// each provider keeps its own model name. API keys live in the OS keychain
/// (see `secrets.rs`), never here.
#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub provider: String, // "local" | "openai" | "anthropic" | "gemini"
    pub local_url: String,
    pub local_model: String,
    pub openai_model: String,
    pub anthropic_model: String,
    pub gemini_model: String,
}

/// Read a single setting, falling back to `default` if the row is missing.
pub async fn get_setting(pool: &SqlitePool, key: &str, default: &str) -> String {
    sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| default.to_string())
}

async fn put_setting(pool: &SqlitePool, key: &str, value: &str) -> Result<(), String> {
    sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)")
        .bind(key)
        .bind(value)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_settings(state: State<'_, DbState>) -> Result<Settings, String> {
    let pool = &state.pool;
    Ok(Settings {
        provider: get_setting(pool, "ai_provider", "local").await,
        local_url: get_setting(pool, "llm_url", "http://192.168.77.1:1234/v1/chat/completions").await,
        local_model: get_setting(pool, "llm_model", "qwen3-vl-30b").await,
        openai_model: get_setting(pool, "openai_model", "gpt-4o-mini").await,
        anthropic_model: get_setting(pool, "anthropic_model", "claude-opus-4-8").await,
        gemini_model: get_setting(pool, "gemini_model", "gemini-2.0-flash").await,
    })
}

#[tauri::command]
pub async fn save_settings(settings: Settings, state: State<'_, DbState>) -> Result<(), String> {
    let pool = &state.pool;
    put_setting(pool, "ai_provider", &settings.provider).await?;
    put_setting(pool, "llm_url", &settings.local_url).await?;
    put_setting(pool, "llm_model", &settings.local_model).await?;
    put_setting(pool, "openai_model", &settings.openai_model).await?;
    put_setting(pool, "anthropic_model", &settings.anthropic_model).await?;
    put_setting(pool, "gemini_model", &settings.gemini_model).await?;
    Ok(())
}
