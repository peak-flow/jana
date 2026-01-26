# CC-Sublime: Implementation Steps Taken

This file documents every step taken to build v0.1, in order. Use this to understand what was done, revert changes, or pick up where we left off.

---

## Step 1: Project Structure Created

**What:** Created the directory layout manually (Tauri scaffold failed in non-TTY).

```
cc-sublime-app/
├── src/                    # Vue 3 frontend
├── src-tauri/              # Rust backend
│   ├── src/commands/       # Tauri commands
│   ├── migrations/         # SQL migrations
│   └── icons/              # App icons
├── package.json
├── vite.config.ts
└── tsconfig.json
```

**Files created:**
- `package.json` — npm dependencies (Vue 3, CodeMirror 6, Tauri API, lodash-es, vite)
- `vite.config.ts` — Vite config with Vue plugin, port 1420
- `tsconfig.json` — TypeScript strict mode, bundler resolution
- `tsconfig.node.json` — Node-side TS config for vite.config
- `index.html` — Entry point loading `/src/main.ts`

---

## Step 2: Rust Backend — Cargo.toml + Main

**What:** Set up the Tauri Rust backend with all dependencies.

**Files created:**
- `src-tauri/Cargo.toml` — Dependencies: tauri 2, sqlx (sqlite + tokio), reqwest, serde, chrono, uuid
- `src-tauri/build.rs` — Standard `tauri_build::build()`
- `src-tauri/tauri.conf.json` — App config: window 1200x800, identifier `com.ccsublime.notes`, dev URL port 1420
- `src-tauri/src/main.rs` — Bootstraps Tauri, initializes DB, registers all commands

**Key decisions in main.rs:**
- DB path: `~/Library/Application Support/cc-sublime/notes.db`
- Tokio runtime created for async DB init
- DbState managed via `tauri::Builder::manage()`
- All commands registered in `generate_handler![]`

---

## Step 3: SQLite Database Setup

**What:** Created database module and migration.

**Files created:**
- `src-tauri/src/db.rs` — `DbState` struct with `SqlitePool`, `init()` creates dir + DB file + runs migrations
- `src-tauri/migrations/001_initial.sql` — Creates `notes` and `ai_summaries` tables

**Schema:**
```sql
notes: id, title, content, created_at, updated_at
ai_summaries: id, note_id, summary, model, created_at
```

---

## Step 4: Note CRUD Commands

**What:** Implemented all note operations as Tauri commands.

**File created:**
- `src-tauri/src/commands/notes.rs`

**Commands:**
- `create_note()` → creates new note with UUID, empty content, returns Note
- `save_note(id, title, content)` → upserts note, updates `updated_at`
- `get_note(id)` → fetches single note by ID
- `list_notes()` → returns all notes (id, title, updated_at) ordered by updated_at DESC
- `delete_note(id)` → deletes note and its summaries

**File created:**
- `src-tauri/src/commands/mod.rs` — Module declarations

---

## Step 5: LLM Summarize Command

**What:** Implemented AI summarization via LM Studio.

**File created:**
- `src-tauri/src/commands/llm.rs`

**Commands:**
- `summarize_note(note_id)` → fetches note content, calls LM Studio, stores summary, returns AiSummary
- `get_summary(note_id)` → fetches latest summary for a note

**LLM config:**
- Endpoint: `http://192.168.77.1:1234/v1/chat/completions`
- Model: `qwen3-vl-30b`
- Temperature: 0.3
- Timeout: 60 seconds
- System prompt: "Summarize concisely, focus on key points and action items"

---

## Step 6: Frontend — Vue 3 Entry Point + App Layout

**Files created:**
- `src/main.ts` — Creates Vue app, mounts to `#app`
- `src/App.vue` — Three-column layout (sidebar | editor | summary), manages active note state
- `src/style.css` — Global styles, Catppuccin Mocha dark theme, CSS reset

---

## Step 7: Frontend — Composables (useNotes, useLLM)

**Files created:**
- `src/composables/useNotes.ts` — Type definitions + invoke wrappers for note CRUD
- `src/composables/useLLM.ts` — Type definitions + invoke wrappers for summarize/getSummary

**Types exported:**
- `Note`, `NoteListItem`, `AiSummary`

---

## Step 8: Frontend — Editor Component

**File created:**
- `src/components/Editor.vue`

**Features:**
- CodeMirror 6 with markdown language support
- One Dark theme
- `onUpdate` callback triggers debounced autosave (2s via lodash-es)
- Watches `noteId` prop to load/switch notes
- Save status indicator ("Saving..." / "Saved" / "")

---

## Step 9: Frontend — Sidebar Component

**File created:**
- `src/components/Sidebar.vue`

**Features:**
- Lists notes (title + relative timestamp)
- "New Note" button
- Active note highlighted
- Loads note list on mount
- Exposes `refreshList()` for parent to call after create/delete

---

## Step 10: Frontend — Summary Panel

**File created:**
- `src/components/SummaryPanel.vue`

**Features:**
- "Summarize" button (disabled when no note selected or loading)
- Loading state
- Error display
- Shows summary text + model name + timestamp
- Loads existing summary when switching notes

---

## Step 11: TypeScript Config + Vite Declaration

**Files created:**
- `src/vite-env.d.ts` — Vite client types + .vue module declaration

**Fixes applied:**
- Removed `useDefineForExpose` from tsconfig.json (not a valid TS option)
- Removed unused `props` variable warning in Sidebar.vue (TS strict mode)

---

## Step 12: Icons + Bundle Config

**What:** Generated placeholder icons (blue square) using pure Python PNG generation.

**Files created:**
- `src-tauri/icons/32x32.png`
- `src-tauri/icons/128x128.png`
- `src-tauri/icons/128x128@2x.png`
- `src-tauri/icons/icon.icns`
- `src-tauri/icons/icon.ico`

**Fix:** Changed bundle identifier from `com.cc-sublime.app` to `com.ccsublime.notes` (macOS doesn't allow `.app` suffix in identifiers).

---

## Step 13: Build Verification

**Commands run:**
1. `npm install` — 76 packages installed
2. `cargo check` — Rust compiles clean
3. `npm run build` — Vue + TypeScript + Vite builds clean (568KB JS bundle)
4. `cargo tauri build` — Full production build successful

**Output:**
- `src-tauri/target/release/bundle/macos/CC Sublime.app`
- `src-tauri/target/release/bundle/dmg/CC Sublime_0.1.0_aarch64.dmg`

---

## How to Modify

### Change LLM endpoint:
Edit `src-tauri/src/commands/llm.rs` → `LM_STUDIO_URL` constant

### Change autosave interval:
Edit `src/components/Editor.vue` → `SAVE_DEBOUNCE_MS` constant (currently 2000)

### Add a new Tauri command:
1. Add function in `src-tauri/src/commands/`
2. Register in `src-tauri/src/main.rs` → `generate_handler![]`
3. Call from frontend via `invoke('command_name', { args })`

### Change theme colors:
Edit `src/style.css` → CSS variables use Catppuccin Mocha palette

### Run in development:
```bash
cd cc-sublime-app && cargo tauri dev
```

### Rebuild production:
```bash
cd cc-sublime-app && cargo tauri build
```
