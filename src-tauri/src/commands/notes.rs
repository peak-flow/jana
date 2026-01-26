use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tauri::State;
use uuid::Uuid;

use crate::db::DbState;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Note {
    pub id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NoteListItem {
    pub id: String,
    pub title: Option<String>,
    pub updated_at: i64,
}

#[tauri::command]
pub async fn create_note(state: State<'_, DbState>) -> Result<Note, String> {
    let now = chrono::Utc::now().timestamp();
    let note = Note {
        id: Uuid::new_v4().to_string(),
        title: None,
        content: None,
        created_at: now,
        updated_at: now,
    };

    sqlx::query("INSERT INTO notes (id, title, content, created_at, updated_at) VALUES (?, ?, ?, ?, ?)")
        .bind(&note.id)
        .bind(&note.title)
        .bind(&note.content)
        .bind(note.created_at)
        .bind(note.updated_at)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(note)
}

#[tauri::command]
pub async fn save_note(
    id: String,
    title: Option<String>,
    content: Option<String>,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp();

    sqlx::query("UPDATE notes SET title = ?, content = ?, updated_at = ? WHERE id = ?")
        .bind(&title)
        .bind(&content)
        .bind(now)
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_note(id: String, state: State<'_, DbState>) -> Result<Note, String> {
    sqlx::query_as::<_, Note>("SELECT id, title, content, created_at, updated_at FROM notes WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_notes(state: State<'_, DbState>) -> Result<Vec<NoteListItem>, String> {
    sqlx::query_as::<_, NoteListItem>("SELECT id, title, updated_at FROM notes ORDER BY updated_at DESC")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_note(id: String, state: State<'_, DbState>) -> Result<(), String> {
    sqlx::query("DELETE FROM ai_summaries WHERE note_id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
