use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;

#[derive(Clone)]
pub struct DbState {
    pub pool: SqlitePool,
}

impl DbState {
    pub async fn init() -> Result<Self, Box<dyn std::error::Error>> {
        let db_path = Self::db_path();

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        Self::run_migrations(&pool).await?;

        Ok(DbState { pool })
    }

    fn db_path() -> PathBuf {
        let mut path = dirs_next().unwrap_or_else(|| PathBuf::from("."));
        path.push("jana");
        std::fs::create_dir_all(&path).ok();
        path.push("jana.db");
        path
    }

    async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        // Drop old tables from v0.1 notes-based schema
        let has_notes_table: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='notes'"
        )
        .fetch_one(pool)
        .await?;

        if has_notes_table {
            sqlx::query("DROP TABLE IF EXISTS ai_summaries").execute(pool).await?;
            sqlx::query("DROP TABLE IF EXISTS notes").execute(pool).await?;
        }

        // Session restore: which files are currently open
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS open_files (
                file_path TEXT PRIMARY KEY,
                jana_id TEXT NOT NULL,
                tab_order INTEGER NOT NULL DEFAULT 0,
                cursor_line INTEGER NOT NULL DEFAULT 1,
                cursor_col INTEGER NOT NULL DEFAULT 1,
                last_opened INTEGER NOT NULL
            )"
        )
        .execute(pool)
        .await?;

        // AI interactions keyed to jana_id
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS file_ai_interactions (
                id TEXT PRIMARY KEY,
                jana_id TEXT NOT NULL,
                interaction_type TEXT NOT NULL,
                prompt TEXT,
                response TEXT NOT NULL,
                model TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )"
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_fai_jana_id ON file_ai_interactions(jana_id)"
        )
        .execute(pool)
        .await?;

        // Settings table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )"
        )
        .execute(pool)
        .await?;

        // Insert default settings if not present
        sqlx::query(
            "INSERT OR IGNORE INTO settings (key, value) VALUES ('llm_url', 'http://192.168.77.1:1234/v1/chat/completions')"
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "INSERT OR IGNORE INTO settings (key, value) VALUES ('llm_model', 'qwen3-vl-30b')"
        )
        .execute(pool)
        .await?;

        // Multi-provider AI defaults. `ai_provider` selects the active backend;
        // each provider keeps its own model name. API keys are NOT stored here —
        // they live in the OS keychain (see secrets.rs).
        for (key, value) in [
            ("ai_provider", "local"),
            ("openai_model", "gpt-4o-mini"),
            ("anthropic_model", "claude-opus-4-8"),
            ("gemini_model", "gemini-2.0-flash"),
        ] {
            sqlx::query("INSERT OR IGNORE INTO settings (key, value) VALUES (?, ?)")
                .bind(key)
                .bind(value)
                .execute(pool)
                .await?;
        }

        // --- Multi-window B-lite schema (Milestone 1) ---
        // One row per editor window. `active_tab_id` is the tab focused in that window.
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS app_windows (
                window_id     TEXT PRIMARY KEY,
                label         TEXT,
                active_tab_id TEXT,
                created_at    INTEGER NOT NULL,
                last_focused  INTEGER NOT NULL
            )"
        )
        .execute(pool)
        .await?;

        // Per-window viewports onto files. `file_path` is intentionally NOT unique here:
        // the same file can be open in multiple windows/tabs (this is what `open_files`
        // could not represent). Buffers themselves are reconstructed at runtime, not stored.
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS open_tabs (
                tab_id      TEXT PRIMARY KEY,
                window_id   TEXT NOT NULL,
                file_path   TEXT NOT NULL,
                jana_id     TEXT NOT NULL,
                tab_order   INTEGER NOT NULL DEFAULT 0,
                cursor_line INTEGER NOT NULL DEFAULT 1,
                cursor_col  INTEGER NOT NULL DEFAULT 1,
                scroll_top  INTEGER NOT NULL DEFAULT 0,
                last_opened INTEGER NOT NULL
            )"
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_open_tabs_window ON open_tabs(window_id)"
        )
        .execute(pool)
        .await?;

        // One-time migration: seed a default window from the legacy `open_files` session.
        // Runs only when no windows exist yet but legacy rows do, so it is idempotent.
        // `open_files` is deliberately kept as legacy and NOT dropped during the transition.
        let windows_empty: bool = sqlx::query_scalar("SELECT COUNT(*) = 0 FROM app_windows")
            .fetch_one(pool)
            .await?;
        let has_legacy_files: bool = sqlx::query_scalar("SELECT COUNT(*) > 0 FROM open_files")
            .fetch_one(pool)
            .await?;

        if windows_empty && has_legacy_files {
            // Wrap the copy in a transaction. Inserting the window row flips the
            // `windows_empty` guard, so a partial failure mid-copy would otherwise
            // strand the remaining legacy rows permanently on the next launch.
            let mut tx = pool.begin().await?;

            // Stable id "main" matches the frontend's URL-less fallback
            // (`window_id ?? "main"`), so Milestone 2's `list_tabs` finds these tabs.
            let window_id = "main".to_string();
            let now = chrono::Utc::now().timestamp();

            let legacy: Vec<(String, String, i64, i64, i64, i64)> = sqlx::query_as(
                "SELECT file_path, jana_id, tab_order, cursor_line, cursor_col, last_opened
                 FROM open_files ORDER BY tab_order"
            )
            .fetch_all(&mut *tx)
            .await?;

            // Create the default window first; its active tab is set once tabs exist.
            sqlx::query(
                "INSERT INTO app_windows (window_id, label, active_tab_id, created_at, last_focused)
                 VALUES (?, 'Main', NULL, ?, ?)"
            )
            .bind(&window_id)
            .bind(now)
            .bind(now)
            .execute(&mut *tx)
            .await?;

            let mut first_tab_id: Option<String> = None;
            for (file_path, jana_id, tab_order, cursor_line, cursor_col, last_opened) in legacy {
                let tab_id = uuid::Uuid::new_v4().to_string();
                if first_tab_id.is_none() {
                    first_tab_id = Some(tab_id.clone());
                }
                sqlx::query(
                    "INSERT INTO open_tabs
                     (tab_id, window_id, file_path, jana_id, tab_order, cursor_line, cursor_col, scroll_top, last_opened)
                     VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?)"
                )
                .bind(&tab_id)
                .bind(&window_id)
                .bind(&file_path)
                .bind(&jana_id)
                .bind(tab_order)
                .bind(cursor_line)
                .bind(cursor_col)
                .bind(last_opened)
                .execute(&mut *tx)
                .await?;
            }

            if let Some(active_tab_id) = first_tab_id {
                sqlx::query("UPDATE app_windows SET active_tab_id = ? WHERE window_id = ?")
                    .bind(&active_tab_id)
                    .bind(&window_id)
                    .execute(&mut *tx)
                    .await?;
            }

            tx.commit().await?;
        }

        Ok(())
    }
}

pub fn dirs_next() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join("Library").join("Application Support"))
    }
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_DATA_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".local").join("share")))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(PathBuf::from)
    }
}
