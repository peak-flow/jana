use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub llm_url: String,
    pub llm_model: String,
}

#[tauri::command]
pub async fn get_settings(state: State<'_, DbState>) -> Result<Settings, String> {
    let llm_url: String = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'llm_url'")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    let llm_model: String = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'llm_model'")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Settings { llm_url, llm_model })
}

#[tauri::command]
pub async fn save_settings(settings: Settings, state: State<'_, DbState>) -> Result<(), String> {
    sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES ('llm_url', ?)")
        .bind(&settings.llm_url)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES ('llm_model', ?)")
        .bind(&settings.llm_model)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
