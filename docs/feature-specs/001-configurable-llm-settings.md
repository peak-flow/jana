# Feature Spec: User-configurable LLM Settings

## Summary
Add a settings panel where users can configure the LM Studio URL and model name, persisted in SQLite.

## Current State
- `src-tauri/src/commands/llm.rs` has hardcoded constants:
  - `LM_STUDIO_URL = "http://192.168.77.1:1234/v1/chat/completions"`
  - `MODEL_NAME = "qwen3-vl-30b"`
- No settings table exists in the database

## Changes

### 1. Database: Add `settings` table
**File:** `src-tauri/src/db.rs`

Add to `run_migrations()`:
```sql
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
)
```

Default rows inserted on first run:
- `llm_url` → `http://192.168.77.1:1234/v1/chat/completions`
- `llm_model` → `qwen3-vl-30b`

### 2. Rust: Settings commands
**New file:** `src-tauri/src/commands/settings.rs`

Commands:
- `get_settings() -> Settings` — reads all settings as a struct
- `save_settings(settings: Settings)` — upserts all settings

```rust
struct Settings {
    llm_url: String,
    llm_model: String,
}
```

### 3. Rust: Update LLM command to use settings
**File:** `src-tauri/src/commands/llm.rs`

- Remove hardcoded `LM_STUDIO_URL` and `MODEL_NAME` constants
- `summarize_note` reads `llm_url` and `llm_model` from the `settings` table before making the HTTP call

### 4. Rust: Register new commands
**File:** `src-tauri/src/main.rs`

Add `commands::settings::get_settings` and `commands::settings::save_settings` to `generate_handler![]`

### 5. Frontend: Settings composable
**New file:** `src/composables/useSettings.ts`

- Types: `Settings { llm_url: string, llm_model: string }`
- `getSettings()` → `invoke('get_settings')`
- `saveSettings(s)` → `invoke('save_settings', { settings: s })`

### 6. Frontend: Settings modal
**New file:** `src/components/SettingsModal.vue`

- Simple modal overlay with:
  - Text input: "LLM URL" (placeholder: `http://192.168.77.1:1234/v1/chat/completions`)
  - Text input: "Model Name" (placeholder: `qwen3-vl-30b`)
  - Save button
  - Cancel button
- Loads current settings on open
- Saves and closes on submit

### 7. Frontend: Settings trigger button
**File:** `src/components/Sidebar.vue`

- Add a gear icon button at the bottom of the sidebar
- Emits `open-settings` event

### 8. Frontend: Wire modal into App.vue
**File:** `src/App.vue`

- `showSettings` ref (boolean)
- Sidebar emits `open-settings` → sets `showSettings = true`
- `<SettingsModal>` rendered conditionally

## Files Modified
- `src-tauri/src/db.rs` — add settings table + defaults
- `src-tauri/src/commands/llm.rs` — read settings instead of constants
- `src-tauri/src/commands/mod.rs` — add `pub mod settings;`
- `src-tauri/src/main.rs` — register settings commands
- `src/App.vue` — add modal state + component
- `src/components/Sidebar.vue` — add gear button

## Files Created
- `src-tauri/src/commands/settings.rs`
- `src/composables/useSettings.ts`
- `src/components/SettingsModal.vue`

## Verification
1. `cargo check` compiles clean
2. `npm run build` passes TypeScript
3. Launch app → click gear → modal shows current defaults
4. Change URL → save → click Summarize → uses new URL
5. Restart app → settings persist
