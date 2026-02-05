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
