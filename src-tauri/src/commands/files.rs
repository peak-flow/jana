use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::io::Write;
use std::path::Path;
use tauri::State;
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;

use crate::buffers::{self, BufferRegistry};
use crate::db::{dirs_next, DbState};
use crate::frontmatter;

/// Result of opening/creating a tab: the viewport identity plus the file content
/// to display in it.
#[derive(Debug, Serialize, Deserialize)]
pub struct TabResult {
    pub tab_id: String,
    pub window_id: String,
    pub file_path: String,
    pub jana_id: String,
    pub content: String,
    pub file_name: String,
}

/// Result of a Save As: the new path plus the re-keyed buffer id the editor
/// should switch to.
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveAsResult {
    pub file_path: String,
    pub buffer_id: String,
}

/// A persisted tab row (session state), without file content.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TabEntry {
    pub tab_id: String,
    pub window_id: String,
    pub file_path: String,
    pub jana_id: String,
    pub tab_order: i32,
    pub cursor_line: i32,
    pub cursor_col: i32,
    pub scroll_top: i32,
    pub last_opened: i64,
}

fn temp_dir() -> Result<std::path::PathBuf, String> {
    let mut path = dirs_next().ok_or("Cannot determine app support directory")?;
    path.push("jana");
    path.push("temp");
    std::fs::create_dir_all(&path).map_err(|e| format!("Failed to create temp dir: {}", e))?;
    Ok(path)
}

pub fn is_temp_file(file_path: &str) -> bool {
    if let Ok(td) = temp_dir() {
        file_path.starts_with(&td.to_string_lossy().to_string())
    } else {
        false
    }
}

fn file_name_from_path(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Ensure an `app_windows` row exists for this window. Covers fresh installs (no
/// legacy session to migrate) and any window created after the one-time migration.
async fn ensure_window(pool: &SqlitePool, window_id: &str) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp();
    let label = if window_id == "main" { "Main" } else { window_id };
    sqlx::query(
        "INSERT OR IGNORE INTO app_windows (window_id, label, active_tab_id, created_at, last_focused)
         VALUES (?, ?, NULL, ?, ?)",
    )
    .bind(window_id)
    .bind(label)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Open `file_path` as a tab in `window_id`, returning its content. If the file is
/// already open in this window, the existing tab is reused so we never create a
/// duplicate tab for the same file within one window.
async fn add_or_get_tab(
    pool: &SqlitePool,
    window_id: &str,
    file_path: &str,
) -> Result<TabResult, String> {
    let (jana_id, content) = frontmatter::ensure_jana_id(file_path)?;
    let file_name = file_name_from_path(file_path);
    let now = chrono::Utc::now().timestamp();

    ensure_window(pool, window_id).await?;

    let existing: Option<String> =
        sqlx::query_scalar("SELECT tab_id FROM open_tabs WHERE window_id = ? AND file_path = ?")
            .bind(window_id)
            .bind(file_path)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;

    let tab_id = match existing {
        Some(tid) => {
            // Disk is authoritative for jana_id (it may have changed via fork).
            sqlx::query("UPDATE open_tabs SET jana_id = ?, last_opened = ? WHERE tab_id = ?")
                .bind(&jana_id)
                .bind(now)
                .bind(&tid)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
            tid
        }
        None => {
            let tid = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO open_tabs
                 (tab_id, window_id, file_path, jana_id, tab_order, cursor_line, cursor_col, scroll_top, last_opened)
                 VALUES (?, ?, ?, ?, (SELECT COALESCE(MAX(tab_order), -1) + 1 FROM open_tabs WHERE window_id = ?), 1, 1, 0, ?)",
            )
            .bind(&tid)
            .bind(window_id)
            .bind(file_path)
            .bind(&jana_id)
            .bind(window_id)
            .bind(now)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
            tid
        }
    };

    Ok(TabResult {
        tab_id,
        window_id: window_id.to_string(),
        file_path: file_path.to_string(),
        jana_id,
        content,
        file_name,
    })
}

#[tauri::command]
pub async fn create_new_file(
    window_id: String,
    state: State<'_, DbState>,
) -> Result<TabResult, String> {
    let td = temp_dir()?;
    let jana_id = Uuid::new_v4().to_string();
    let full = frontmatter::compose_with_frontmatter(&jana_id, "");

    // Reserve the next available "Untitled N.md" and create it atomically:
    // create_new(true) makes name reservation and file creation one operation, so
    // two concurrent "New File" actions across windows can't both pick the same name.
    let mut n = 1u32;
    let file_path = loop {
        let path = td.join(format!("Untitled {}.md", n));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut f) => {
                f.write_all(full.as_bytes())
                    .map_err(|e| format!("Failed to create temp file: {}", e))?;
                break path.to_string_lossy().to_string();
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => n += 1,
            Err(e) => return Err(format!("Failed to create temp file: {}", e)),
        }
    };

    // A freshly-named temp file is always new, so this creates a new tab.
    add_or_get_tab(&state.pool, &window_id, &file_path).await
}

#[tauri::command]
pub async fn open_file_dialog(
    app: tauri::AppHandle,
    window_id: String,
    state: State<'_, DbState>,
) -> Result<Option<TabResult>, String> {
    let picked = app
        .dialog()
        .file()
        .add_filter("Text Files", &["md", "txt", "markdown"])
        .add_filter("All Files", &["*"])
        .blocking_pick_file();

    let file_path = match picked {
        Some(fp) => fp
            .into_path()
            .map_err(|e| format!("Invalid file path: {:?}", e))?
            .to_string_lossy()
            .to_string(),
        None => return Ok(None),
    };

    Ok(Some(add_or_get_tab(&state.pool, &window_id, &file_path).await?))
}

#[tauri::command]
pub async fn add_tab(
    window_id: String,
    file_path: String,
    state: State<'_, DbState>,
) -> Result<TabResult, String> {
    add_or_get_tab(&state.pool, &window_id, &file_path).await
}

/// Create a new editor window with its own independent session. Mints a fresh
/// `window_id`, records it in `app_windows`, then opens a webview at
/// `index.html?window_id=<id>` so the frontend restores only this window's tabs.
#[tauri::command]
pub async fn create_app_window(
    app: tauri::AppHandle,
    state: State<'_, DbState>,
) -> Result<String, String> {
    let window_id = Uuid::new_v4().to_string();
    ensure_window(&state.pool, &window_id).await?;

    // The label is prefixed so the `win-*` capability glob grants the new window
    // the same permissions as the main window; the URL carries the bare id that
    // the frontend reads from `?window_id=`.
    let label = format!("win-{}", window_id);
    let url = format!("index.html?window_id={}", window_id);
    tauri::WebviewWindowBuilder::new(&app, &label, tauri::WebviewUrl::App(url.into()))
        .title("Jana")
        .inner_size(1200.0, 800.0)
        .build()
        .map_err(|e| format!("Failed to create window: {}", e))?;

    Ok(window_id)
}

/// Map a Tauri window label to its session `window_id`. Secondary windows are
/// labelled `win-<id>` (see `create_app_window`); the main window's label is its id.
pub fn window_id_from_label(label: &str) -> String {
    label.strip_prefix("win-").unwrap_or(label).to_string()
}

/// Release one backend buffer reference for every tab a window held. Called when a
/// window is destroyed so its buffers' refcounts drop (and a dirty last view flushes)
/// instead of leaking. The `open_tabs` rows are left intact for session restore.
pub async fn release_window_buffers(
    pool: &SqlitePool,
    registry: &BufferRegistry,
    window_id: &str,
) {
    let paths: Vec<String> =
        sqlx::query_scalar("SELECT file_path FROM open_tabs WHERE window_id = ?")
            .bind(window_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
    for path in paths {
        let buffer_id = buffers::canonical_buffer_id(&path);
        if let Err(e) = buffers::release_in_registry(registry, &buffer_id) {
            eprintln!("window cleanup: failed to release {}: {}", path, e);
        }
    }
}

#[tauri::command]
pub async fn list_tabs(
    window_id: String,
    state: State<'_, DbState>,
) -> Result<Vec<TabEntry>, String> {
    sqlx::query_as::<_, TabEntry>(
        "SELECT tab_id, window_id, file_path, jana_id, tab_order, cursor_line, cursor_col, scroll_top, last_opened
         FROM open_tabs WHERE window_id = ? ORDER BY tab_order ASC",
    )
    .bind(&window_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn close_tab(tab_id: String, state: State<'_, DbState>) -> Result<(), String> {
    let file_path: Option<String> =
        sqlx::query_scalar("SELECT file_path FROM open_tabs WHERE tab_id = ?")
            .bind(&tab_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM open_tabs WHERE tab_id = ?")
        .bind(&tab_id)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    // Clear the window's active tab pointer if it referenced this tab.
    sqlx::query("UPDATE app_windows SET active_tab_id = NULL WHERE active_tab_id = ?")
        .bind(&tab_id)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    // Delete a temp file only when no other tab (in any window) still references it.
    if let Some(fp) = file_path {
        if is_temp_file(&fp) {
            let others: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM open_tabs WHERE file_path = ?")
                    .bind(&fp)
                    .fetch_one(&state.pool)
                    .await
                    .map_err(|e| e.to_string())?;
            if others == 0 {
                let _ = std::fs::remove_file(&fp);
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn update_tab_view(
    tab_id: String,
    cursor_line: i32,
    cursor_col: i32,
    scroll_top: i32,
    state: State<'_, DbState>,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE open_tabs SET cursor_line = ?, cursor_col = ?, scroll_top = ? WHERE tab_id = ?",
    )
    .bind(cursor_line)
    .bind(cursor_col)
    .bind(scroll_top)
    .bind(&tab_id)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn fork_file(
    file_path: String,
    state: State<'_, DbState>,
    registry: State<'_, BufferRegistry>,
) -> Result<String, String> {
    let new_id = Uuid::new_v4().to_string();

    // Read current content (stripping old frontmatter)
    let raw = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    let parsed = frontmatter::parse_frontmatter(&raw);

    // Write back with new jana_id
    let full = frontmatter::compose_with_frontmatter(&new_id, &parsed.content);
    std::fs::write(&file_path, &full).map_err(|e| format!("Failed to write file: {}", e))?;

    // Keep the loaded buffer's identity in sync, or the flush loop would overwrite
    // the file with the old jana_id.
    buffers::set_buffer_jana_id(&registry, &file_path, &new_id);

    // Update every tab pointing at this file (identity is per-file, not per-tab).
    sqlx::query("UPDATE open_tabs SET jana_id = ? WHERE file_path = ?")
        .bind(&new_id)
        .bind(&file_path)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(new_id)
}

#[tauri::command]
pub async fn clear_ai_history(jana_id: String, state: State<'_, DbState>) -> Result<(), String> {
    sqlx::query("DELETE FROM file_ai_interactions WHERE jana_id = ?")
        .bind(&jana_id)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn reveal_in_finder(file_path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&file_path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", &file_path))
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        let parent = std::path::Path::new(&file_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| file_path.clone());
        std::process::Command::new("xdg-open")
            .arg(&parent)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn save_temp_file_as(
    app: tauri::AppHandle,
    tab_id: String,
    state: State<'_, DbState>,
    registry: State<'_, BufferRegistry>,
) -> Result<Option<SaveAsResult>, String> {
    // The tab row is the source of truth for which file this tab edits. Deriving the
    // source path server-side (instead of trusting a caller-supplied path) prevents a
    // stale/mismatched caller from saving one tab's buffer into another tab's file.
    let source_path: String =
        sqlx::query_scalar("SELECT file_path FROM open_tabs WHERE tab_id = ?")
            .bind(&tab_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Tab not found")?;

    // Content is owned by the backend buffer (authoritative for active and
    // inactive tabs alike). The active editor flushes pending edits before calling.
    let (jana_id, content) = buffers::buffer_content(&registry, &source_path)
        .ok_or("No loaded buffer for this file")?;

    let picked = app
        .dialog()
        .file()
        .add_filter("Markdown", &["md"])
        .add_filter("Text", &["txt"])
        .blocking_save_file();

    let new_path = match picked {
        Some(fp) => fp
            .into_path()
            .map_err(|e| format!("Invalid file path: {:?}", e))?
            .to_string_lossy()
            .to_string(),
        None => return Ok(None),
    };

    // Refuse to save over a file already open in another tab/window: re-keying onto
    // its buffer id would replace that buffer and drop its unsaved content.
    if new_path != source_path {
        let already_open: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM open_tabs WHERE file_path = ? AND tab_id != ?",
        )
        .bind(&new_path)
        .bind(&tab_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| e.to_string())?;
        if already_open > 0 {
            return Err(format!(
                "\"{}\" is already open in another tab. Close it first, then Save As.",
                file_name_from_path(&new_path)
            ));
        }
    }

    // Write content to the new location.
    let full = frontmatter::compose_with_frontmatter(&jana_id, &content);
    std::fs::write(&new_path, &full).map_err(|e| format!("Failed to save file: {}", e))?;

    // Re-point this tab at the new path.
    sqlx::query("UPDATE open_tabs SET file_path = ? WHERE tab_id = ?")
        .bind(&new_path)
        .bind(&tab_id)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    // Re-key the buffer (temp path → real path) before the old file may be removed.
    let buffer_id = buffers::rekey_buffer(&registry, &source_path, &new_path, &jana_id, &content)
        .map(|(id, _version)| id)
        .unwrap_or_else(|| buffers::canonical_buffer_id(&new_path));

    // Remove the old temp file only if no other tab still references it.
    if is_temp_file(&source_path) {
        let others: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM open_tabs WHERE file_path = ?")
            .bind(&source_path)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| e.to_string())?;
        if others == 0 {
            let _ = std::fs::remove_file(&source_path);
        }
    }

    Ok(Some(SaveAsResult {
        file_path: new_path,
        buffer_id,
    }))
}
