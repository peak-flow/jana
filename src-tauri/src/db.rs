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

        // Run migrations
        Self::run_migrations(&pool).await?;

        Ok(DbState { pool })
    }

    fn db_path() -> PathBuf {
        let mut path = dirs_next().unwrap_or_else(|| PathBuf::from("."));
        path.push("jana");
        std::fs::create_dir_all(&path).ok();
        path.push("notes.db");
        path
    }

    async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                title TEXT,
                content TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )"
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS ai_summaries (
                id TEXT PRIMARY KEY,
                note_id TEXT NOT NULL,
                summary TEXT NOT NULL,
                model TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY(note_id) REFERENCES notes(id)
            )"
        )
        .execute(pool)
        .await?;

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

fn dirs_next() -> Option<PathBuf> {
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
