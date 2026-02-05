use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::path::Path;
use tauri::State;
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;

use crate::db::DbState;
use crate::frontmatter;

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenFileResult {
    pub file_path: String,
    pub jana_id: String,
    pub content: String,
    pub file_name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct OpenFileEntry {
    pub file_path: String,
    pub jana_id: String,
    pub tab_order: i32,
    pub cursor_line: i32,
    pub cursor_col: i32,
    pub last_opened: i64,
}

fn file_name_from_path(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

#[tauri::command]
pub async fn open_file_dialog(
    app: tauri::AppHandle,
    state: State<'_, DbState>,
) -> Result<Option<OpenFileResult>, String> {
    let file_path = app
        .dialog()
        .file()
        .add_filter("Text Files", &["md", "txt", "markdown"])
        .add_filter("All Files", &["*"])
        .blocking_pick_file();

    let file_path = match file_path {
        Some(fp) => fp.into_path()
            .map_err(|e| format!("Invalid file path: {:?}", e))?
            .to_string_lossy()
            .to_string(),
        None => return Ok(None),
    };

    let (jana_id, content) = frontmatter::ensure_jana_id(&file_path)?;
    let file_name = file_name_from_path(&file_path);
    let now = chrono::Utc::now().timestamp();

    // Upsert into open_files
    sqlx::query(
        "INSERT OR REPLACE INTO open_files (file_path, jana_id, tab_order, cursor_line, cursor_col, last_opened)
         VALUES (?, ?, COALESCE((SELECT tab_order FROM open_files WHERE file_path = ?), (SELECT COALESCE(MAX(tab_order), 0) + 1 FROM open_files)), 1, 1, ?)"
    )
    .bind(&file_path)
    .bind(&jana_id)
    .bind(&file_path)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Some(OpenFileResult {
        file_path,
        jana_id,
        content,
        file_name,
    }))
}

#[tauri::command]
pub async fn read_file(
    file_path: String,
    state: State<'_, DbState>,
) -> Result<OpenFileResult, String> {
    let (jana_id, content) = frontmatter::ensure_jana_id(&file_path)?;
    let file_name = file_name_from_path(&file_path);
    let now = chrono::Utc::now().timestamp();

    // Update open_files entry
    sqlx::query(
        "INSERT OR REPLACE INTO open_files (file_path, jana_id, tab_order, cursor_line, cursor_col, last_opened)
         VALUES (?, ?, COALESCE((SELECT tab_order FROM open_files WHERE file_path = ?), (SELECT COALESCE(MAX(tab_order), 0) + 1 FROM open_files)), COALESCE((SELECT cursor_line FROM open_files WHERE file_path = ?), 1), COALESCE((SELECT cursor_col FROM open_files WHERE file_path = ?), 1), ?)"
    )
    .bind(&file_path)
    .bind(&jana_id)
    .bind(&file_path)
    .bind(&file_path)
    .bind(&file_path)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(OpenFileResult {
        file_path,
        jana_id,
        content,
        file_name,
    })
}

#[tauri::command]
pub async fn save_file(
    file_path: String,
    jana_id: String,
    content: String,
) -> Result<(), String> {
    let full = frontmatter::compose_with_frontmatter(&jana_id, &content);
    std::fs::write(&file_path, &full)
        .map_err(|e| format!("Failed to save file: {}", e))
}

#[tauri::command]
pub async fn save_file_as(
    app: tauri::AppHandle,
    jana_id: String,
    content: String,
    state: State<'_, DbState>,
) -> Result<Option<String>, String> {
    let file_path = app
        .dialog()
        .file()
        .add_filter("Markdown", &["md"])
        .add_filter("Text", &["txt"])
        .blocking_save_file();

    let file_path = match file_path {
        Some(fp) => fp.into_path()
            .map_err(|e| format!("Invalid file path: {:?}", e))?
            .to_string_lossy()
            .to_string(),
        None => return Ok(None),
    };

    let full = frontmatter::compose_with_frontmatter(&jana_id, &content);
    std::fs::write(&file_path, &full)
        .map_err(|e| format!("Failed to save file: {}", e))?;

    let now = chrono::Utc::now().timestamp();

    // Add to open_files
    sqlx::query(
        "INSERT OR REPLACE INTO open_files (file_path, jana_id, tab_order, cursor_line, cursor_col, last_opened)
         VALUES (?, ?, (SELECT COALESCE(MAX(tab_order), 0) + 1 FROM open_files), 1, 1, ?)"
    )
    .bind(&file_path)
    .bind(&jana_id)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Some(file_path))
}

#[tauri::command]
pub async fn close_file(
    file_path: String,
    state: State<'_, DbState>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM open_files WHERE file_path = ?")
        .bind(&file_path)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn list_open_files(
    state: State<'_, DbState>,
) -> Result<Vec<OpenFileEntry>, String> {
    sqlx::query_as::<_, OpenFileEntry>(
        "SELECT file_path, jana_id, tab_order, cursor_line, cursor_col, last_opened
         FROM open_files ORDER BY tab_order ASC"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_cursor_position(
    file_path: String,
    cursor_line: i32,
    cursor_col: i32,
    state: State<'_, DbState>,
) -> Result<(), String> {
    sqlx::query("UPDATE open_files SET cursor_line = ?, cursor_col = ? WHERE file_path = ?")
        .bind(cursor_line)
        .bind(cursor_col)
        .bind(&file_path)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn fork_file(
    file_path: String,
    state: State<'_, DbState>,
) -> Result<String, String> {
    let new_id = Uuid::new_v4().to_string();

    // Read current content (stripping old frontmatter)
    let raw = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    let parsed = frontmatter::parse_frontmatter(&raw);

    // Write back with new jana_id
    let full = frontmatter::compose_with_frontmatter(&new_id, &parsed.content);
    std::fs::write(&file_path, &full)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    // Update open_files with new jana_id
    sqlx::query("UPDATE open_files SET jana_id = ? WHERE file_path = ?")
        .bind(&new_id)
        .bind(&file_path)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(new_id)
}

#[tauri::command]
pub async fn clear_ai_history(
    jana_id: String,
    state: State<'_, DbState>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM file_ai_interactions WHERE jana_id = ?")
        .bind(&jana_id)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
