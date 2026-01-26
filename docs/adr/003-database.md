# ADR-003: Database — SQLite via sqlx

## Status
Accepted

## Context
Need persistent storage for notes and AI summaries. Options considered:
- **tauri-plugin-sql** — Frontend-focused SQL plugin, abstracts away backend
- **rusqlite** — Synchronous SQLite bindings for Rust
- **sqlx** — Async, compile-time checked SQL for Rust
- **sled/redb** — Embedded key-value stores

## Decision
**SQLite via `sqlx` with the `runtime-tokio` feature and bundled `libsqlite3-sys`.**

## Rationale
- SQLite is the right tool for a single-user desktop note app
- `sqlx` provides async queries (non-blocking UI), compile-time SQL validation, and migration support
- `rusqlite` is synchronous — would block the Tauri event loop on large operations
- `tauri-plugin-sql` exposes SQL to the frontend (security concern, wrong layer)
- Bundled `libsqlite3-sys` avoids system SQLite version dependencies

## Schema
```sql
CREATE TABLE notes (
    id TEXT PRIMARY KEY,
    title TEXT,
    content TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE ai_summaries (
    id TEXT PRIMARY KEY,
    note_id TEXT NOT NULL,
    summary TEXT NOT NULL,
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(note_id) REFERENCES notes(id)
);
```

## Consequences
- DB file stored at `~/Library/Application Support/cc-sublime/notes.db`
- Migrations run automatically on app startup
- sqlx requires tokio runtime (already needed by Tauri)
