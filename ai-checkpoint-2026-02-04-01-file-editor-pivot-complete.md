# Checkpoint: 2026-02-04-01

**Created**: 2026-02-04 21:15
**Branch**: main
**Working Directory**: /Users/dabraham/CascadeProjects/jana

## TL;DR
Pivoted Jana from a DB-backed notes app to a Sublime Text-like file editor with YAML frontmatter `jana_id` for persistent AI interaction history.

## Problem Statement
- User wanted a Sublime Text clone with AI capabilities — not a notes app
- The original DB-backed notes model wasn't portable/shareable like real files
- Files on disk should be the source of truth; DB should only store AI metadata and session state
- Each file gets a `jana_id` UUID in YAML frontmatter that survives moves/renames and links to AI history

## Files Modified / Created
- `src-tauri/src/frontmatter.rs` — **CREATED** — YAML frontmatter parser/writer with `jana_id` extraction, composition, and `ensure_jana_id` for auto-injection
- `src-tauri/src/commands/files.rs` — **CREATED** — 9 Tauri commands: open_file_dialog, read_file, save_file, save_file_as, close_file, list_open_files, update_cursor_position, fork_file, clear_ai_history
- `src-tauri/capabilities/default.json` — **CREATED** — Tauri v2 capability permissions for core and dialog plugins
- `src/composables/useFiles.ts` — **CREATED** — TypeScript invoke wrappers for all file commands
- `src-tauri/src/db.rs` — Rewrote schema: dropped `notes`/`ai_summaries` tables, added `open_files` (session) and `file_ai_interactions` (AI data keyed to jana_id). Renamed DB from `notes.db` to `jana.db`
- `src-tauri/src/commands/llm.rs` — Renamed `summarize_note`→`summarize_file`, `get_summary`→`get_file_summary`. Now reads content from disk (not DB), uses `jana_id`, stores in `file_ai_interactions`
- `src-tauri/src/commands/mod.rs` — Changed `pub mod notes` to `pub mod files`
- `src-tauri/src/main.rs` — Added `mod frontmatter`, dialog plugin registration, replaced note commands with file commands
- `src-tauri/Cargo.toml` — Added `tauri-plugin-dialog = "2"`, `regex = "1"`
- `src-tauri/tauri.conf.json` — Changed identifier from `com.jana.notes` to `com.jana.editor`
- `package.json` — Added `@tauri-apps/plugin-dialog: "^2.0.0"`
- `src/App.vue` — Rewrote: `ActiveFile` interface, `openFiles` array, session restore on mount, file open/select/close handlers
- `src/components/Sidebar.vue` — Rewrote: shows open files with close (x) buttons, "Open File" button triggers native dialog
- `src/components/Editor.vue` — Rewrote: props changed to `filePath`/`janaId`/`content`, auto-save writes to file on disk
- `src/components/SummaryPanel.vue` — Updated: uses `janaId`/`filePath` props, `AiInteraction` type, `summary.response` field
- `src/composables/useLLM.ts` — Updated: `AiInteraction` interface, `summarizeFile(janaId, filePath)`, `getFileSummary(janaId)`
- `src-tauri/icons/32x32.png` — Converted from RGB to RGBA (required by `tauri::generate_context!()`)
- `src/composables/useNotes.ts` — **DELETED**
- `src-tauri/src/commands/notes.rs` — **DELETED**

## Files Read / Referenced
- `src/components/SettingsModal.vue` — Unchanged, still works (settings are independent of notes/files)
- `src/composables/useSettings.ts` — Unchanged
- `src-tauri/src/commands/settings.rs` — Unchanged

## Key Decisions / Conclusions
- Decision: Files on disk are source of truth — Reason: User wants Sublime Text-like behavior, not a notes app
- Decision: YAML frontmatter with `jana_id` UUID — Reason: Widely compatible (Obsidian, Jekyll, Hugo), visible in other editors, easy to parse
- Decision: DB stores AI metadata keyed to `jana_id` (not file path) — Reason: Files can be moved/renamed and keep their AI history
- Decision: Fork = new UUID + fresh AI history — Reason: User can start clean AI interactions for a file
- Decision: Clear = same UUID, delete AI data — Reason: Keep identity but wipe interactions
- Decision: No untitled file support in v0.2 — Reason: Adds complexity; user must open or save-as first
- Decision: Drop old notes tables in migration — Reason: v0.1 app with no real user data to preserve

## Implementation Details
- **Frontmatter format**: `---\njana_id: <uuid-v4>\n---\n<content>`
- **Frontmatter parser**: regex `jana_id:\s*([0-9a-fA-F\-]{36})` to extract UUID from YAML block
- **DB tables**: `open_files` (file_path PK, jana_id, tab_order, cursor_line, cursor_col, last_opened), `file_ai_interactions` (id PK, jana_id, interaction_type, prompt, response, model, created_at), `settings` (unchanged)
- **DB path**: `~/Library/Application Support/jana/jana.db` (macOS)
- **FilePath API**: `tauri_plugin_dialog::FilePath` is an enum (`Url`/`Path`), use `.into_path()` which returns `Result<PathBuf, FilePath>`
- **Dialog usage**: `app.dialog().file().add_filter(...).blocking_pick_file()` returns `Option<FilePath>`
- **Auto-save**: 2-second debounced save via lodash-es, writes frontmatter + content to disk
- **Session restore**: `onMounted` in App.vue reads `open_files` table, reopens each file, handles missing files gracefully

## Current State
- DONE: Full architectural pivot — notes app → file editor
- DONE: Rust backend compiles (cargo build succeeds)
- DONE: Vue frontend builds (vite build succeeds)
- DONE: Full Tauri build produces Jana.app and .dmg
- DONE: Session restore logic in App.vue
- NOT YET TESTED: Runtime behavior (opening files, editing, auto-save, AI summarization)
- NOT YET COMMITTED: All changes are unstaged

## Next Steps
1. Run `npx tauri dev` and manually test: open file dialog, edit, auto-save, close/reopen
2. Verify frontmatter injection on first open of a plain .md file
3. Verify session restore (close app, reopen, files should reappear)
4. Test AI summarization with a file (requires LM Studio running)
5. Add keyboard shortcuts: Cmd+O (open), Cmd+S (immediate save), Cmd+W (close file)
6. Commit all changes with conventional commits
7. Consider adding fork/clear AI history UI in sidebar context menu

## Constraints
- LM Studio endpoint: `http://192.168.77.1:1234/v1/chat/completions` (default)
- LM Studio model: `qwen3-vl-30b` (default)
- Tauri v2 — plugins require capability declarations in `src-tauri/capabilities/default.json`
- 32x32.png icon MUST be RGBA or `tauri::generate_context!()` panics at compile time
- `vue-tsc --noEmit` has a broken module resolution (workaround: skip it, use `vite build` directly)
- Port 8080 reserved by Laravel Herd — don't use for dev servers

## Error Log
- Error: `icon /Users/dabraham/.../32x32.png is not RGBA` — Resolution: Converted with Python PIL `img.convert('RGBA')`
- Error: `vue-tsc` module not found (`../index.js`) — Resolution: Skipped vue-tsc, used `vite build` directly (pre-existing issue)
