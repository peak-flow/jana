# ADR-008: Configurable LLM Settings via SQLite

## Status
Proposed

## Context
The LLM endpoint URL and model name are hardcoded in `src-tauri/src/commands/llm.rs`. Users on different machines or networks cannot use the Summarize feature without rebuilding the app. Need a way to configure these at runtime.

Options considered:
- **Config file (JSON/TOML)** — Separate file in app data directory
- **SQLite settings table** — Key-value table in existing database
- **Environment variables** — Set before launch
- **Tauri plugin-store** — Tauri's built-in key-value store plugin

## Decision
**SQLite key-value `settings` table in the existing database.**

## Rationale
- Already have SQLite set up with sqlx — no new dependencies
- Settings persist across app restarts automatically
- Single source of truth (one DB file, not DB + config file)
- Simple key-value schema avoids over-engineering
- Defaults inserted on first run, so app works out of the box
- Tauri plugin-store adds an unnecessary dependency for two settings
- Environment variables are unfriendly for non-technical users
- Config file adds file management complexity (where to store, format parsing, error handling)

## Schema
```sql
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
)
```

## Default Values
| Key | Default Value |
|-----|--------------|
| `llm_url` | `http://192.168.77.1:1234/v1/chat/completions` |
| `llm_model` | `qwen3-vl-30b` |

## UI
- Gear icon button at bottom of sidebar
- Modal with two text inputs (URL, model name)
- Save persists immediately to SQLite
- No app restart required — next Summarize call uses updated values

## Consequences
- Settings are per-machine (stored in local SQLite, not synced)
- Adding new settings in future = add new key + default, no schema migration needed
- If DB is deleted, defaults are re-created on next launch
- Modal UI is minimal — no validation of URL format in v0.1 (fail on use with clear error)
