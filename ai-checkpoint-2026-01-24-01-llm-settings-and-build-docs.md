# Checkpoint: 2026-01-24-01

**Created**: 2026-01-24 (session)
**Branch**: master
**Working Directory**: /Users/dabraham/CascadeProjects/cc-sublime/cc-sublime-app

## TL;DR
Implemented user-configurable LLM settings with SQLite persistence, added build guide and README documentation.

## Problem Statement
- Building a desktop note-taking app ("CC Sublime") with local AI summarization via Tauri + Vue 3 + Rust
- This session: make LLM URL and model name user-configurable instead of hardcoded, then document build/install process

## Files Modified / Created
- `src-tauri/src/db.rs` — added `settings` table migration with default rows for `llm_url` and `llm_model`
- `src-tauri/src/commands/settings.rs` — NEW: `get_settings`/`save_settings` Tauri commands with `Settings` struct
- `src-tauri/src/commands/llm.rs` — removed hardcoded constants, reads settings from DB before LLM call
- `src-tauri/src/commands/mod.rs` — added `pub mod settings;`
- `src-tauri/src/main.rs` — registered `get_settings` and `save_settings` in `generate_handler!`
- `src/composables/useSettings.ts` — NEW: typed invoke wrappers for settings CRUD
- `src/components/SettingsModal.vue` — NEW: modal with LLM URL and model name inputs
- `src/components/Sidebar.vue` — added gear icon button in sidebar footer, emits `open-settings`
- `src/App.vue` — added `showSettings` ref, wired SettingsModal with conditional render
- `docs/BUILD.md` — NEW: build instructions for macOS ARM/Intel, Windows, Linux
- `README.md` — NEW: app overview, unsigned app install guide, LLM configuration instructions

## Files Read / Referenced
- `src/composables/useNotes.ts` — checked import path (`@tauri-apps/api/core` not `@tauri-apps/api/tauri`)
- `src-tauri/tauri.conf.json` — checked app naming for potential rename to "Jana"
- `src-tauri/Cargo.toml` — referenced for rename discussion

## Key Decisions / Conclusions
- Decision: Key-value `settings` table — Reason: simple, extensible for future settings without schema changes
- Decision: `INSERT OR IGNORE` for defaults — Reason: safe for repeat migrations, won't overwrite user changes
- Decision: Settings read at summarize-time — Reason: always uses latest settings without app restart
- Decision: No API key field yet — Reason: out of scope, app targets local LLM endpoints
- Rename to "Jana" discussed: ~5 files, ~10 lines. Gotcha: db path change means new empty database unless folder renamed manually

## Implementation Details
- Settings table schema: `key TEXT PRIMARY KEY, value TEXT NOT NULL`
- Default settings: `llm_url` = `http://192.168.77.1:1234/v1/chat/completions`, `llm_model` = `qwen3-vl-30b`
- Frontend import path: `@tauri-apps/api/core` (Tauri v2 style)
- SettingsModal uses Catppuccin Mocha color scheme consistent with rest of app
- Intel Mac cross-compile: `rustup target add x86_64-apple-darwin` then `npm run tauri build -- --target x86_64-apple-darwin`
- Build outputs: `target/release/bundle/` (native), `target/x86_64-apple-darwin/release/bundle/` (cross)

## Current State
- DONE: Settings feature fully implemented and compiled (cargo check + npm run build pass)
- DONE: Both ARM and Intel DMGs built successfully
- DONE: README.md and docs/BUILD.md committed
- IN PROGRESS: Untracked `docs/adr/` and `docs/feature-specs/` from planning phase (not committed)

## Next Steps
1. Decide on app rename to "Jana" (5 files, ~10 line changes)
2. Commit or remove planning docs (`docs/adr/`, `docs/feature-specs/`)
3. Consider adding API key field to settings for remote LLM providers
4. Set up GitHub Actions for Windows builds if needed
5. Test settings persistence across app restarts

## Constraints
- Tauri v2 with Vue 3 frontend
- SQLite via sqlx (async)
- LM Studio at `http://192.168.77.1:1234` for local inference
- macOS ARM (Apple Silicon) development machine
- App is unsigned — requires Gatekeeper bypass on macOS
- Cannot cross-compile Windows from macOS (needs native Windows or CI)
- Port 8080 reserved by Laravel Herd

## Error Log
- Error: `TS2307: Cannot find module '@tauri-apps/api/tauri'` — Resolution: changed import to `@tauri-apps/api/core` (Tauri v2 path)
- Error: `TS6133: 'Settings' is declared but its value is never read` — Resolution: removed unused type import from SettingsModal.vue
