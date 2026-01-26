# Jana - Architecture Documentation

## Overview
A lightweight desktop note-taking app with local AI summarization — v0.1

## Technology Stack

### Core Framework
- **Tauri v2**: Cross-platform desktop app framework
- **Frontend**: Vue 3 with TypeScript
- **Backend**: Rust (Tauri commands)

### Editor
- **CodeMirror 6**: Text editor component
- **Theme**: One Dark
- **Features**: Markdown support, autosave

### Database
- **SQLite**: Local file-based database via sqlx
- **Location**: OS app data directory
- **Schema**: 3-table design (notes, ai_summaries, settings)

### AI Integration
- **API**: OpenAI-compatible chat completions endpoint
- **Default**: Local LM Studio instance
- **Configurable**: URL and model name via Settings modal

## Data Model

### `notes` table
```sql
CREATE TABLE notes (
    id TEXT PRIMARY KEY,           -- UUID
    title TEXT,                     -- Optional, derived from content
    content TEXT,                   -- Main note content
    created_at INTEGER NOT NULL,    -- Unix timestamp
    updated_at INTEGER NOT NULL     -- Unix timestamp
);
```

### `ai_summaries` table
```sql
CREATE TABLE ai_summaries (
    id TEXT PRIMARY KEY,           -- UUID
    note_id TEXT NOT NULL,          -- Foreign key to notes.id
    summary TEXT NOT NULL,          -- AI-generated summary
    model TEXT NOT NULL,            -- Model name used
    created_at INTEGER NOT NULL,    -- Unix timestamp
    FOREIGN KEY(note_id) REFERENCES notes(id)
);
```

### `settings` table
```sql
CREATE TABLE settings (
    key TEXT PRIMARY KEY,           -- Setting identifier
    value TEXT NOT NULL             -- Setting value
);
-- Default rows: llm_url, llm_model
```

## Application Structure

### Frontend Components (Vue 3)
- **App.vue**: Root component, layout orchestration
- **Sidebar.vue**: Note list, new note button, settings trigger
- **Editor.vue**: CodeMirror instance with autosave
- **SummaryPanel.vue**: AI summary display and trigger
- **SettingsModal.vue**: LLM configuration

### Frontend Composables
- **useNotes.ts**: Note CRUD operations
- **useLLM.ts**: AI summarization
- **useSettings.ts**: Settings get/save

### Backend Commands (Rust)
- `create_note() -> Note`: Create new note
- `save_note(id, title, content)`: Update existing note
- `get_note(id) -> Note`: Retrieve note
- `list_notes() -> Vec<NoteListItem>`: List all notes
- `delete_note(id)`: Delete note
- `summarize_note(note_id) -> AiSummary`: Generate AI summary
- `get_summary(note_id) -> Option<AiSummary>`: Get existing summary
- `get_settings() -> Settings`: Read LLM settings
- `save_settings(settings)`: Update LLM settings

### User Flow
1. App opens → loads most recent note or creates new
2. User types/pastes content
3. Autosave triggers on content change (debounced)
4. User clicks "Summarize"
5. AI call generates summary using configured LLM
6. Summary stored and displayed in side panel

## File Structure
```
src/
├── App.vue
├── main.ts
├── style.css
├── components/
│   ├── Editor.vue
│   ├── Sidebar.vue
│   ├── SummaryPanel.vue
│   └── SettingsModal.vue
└── composables/
    ├── useNotes.ts
    ├── useLLM.ts
    └── useSettings.ts

src-tauri/
├── src/
│   ├── main.rs
│   ├── db.rs
│   └── commands/
│       ├── mod.rs
│       ├── notes.rs
│       ├── llm.rs
│       └── settings.rs
├── Cargo.toml
└── tauri.conf.json
```

## Database Location

| OS | Path |
|----|------|
| macOS | `~/Library/Application Support/jana/notes.db` |
| Linux | `~/.local/share/jana/notes.db` |
| Windows | `%APPDATA%/jana/notes.db` |

## Configuration

### Tauri Settings
- Window: 1200x800, resizable
- Security: CSP disabled for local LLM calls
- Network: HTTP calls to AI endpoints

### AI Settings (User-Configurable)
- Stored in `settings` table (key-value pairs)
- `llm_url`: API endpoint (default: `http://192.168.77.1:1234/v1/chat/completions`)
- `llm_model`: Model name (default: `qwen3-vl-30b`)
- Configurable via Settings modal (gear icon in sidebar)
- Timeout: 60 seconds

## Security Considerations
- No network access except configured AI endpoint
- Local file system access limited to app data directory
- No telemetry or analytics
- All data stored locally

## Performance Targets
- App startup: <2 seconds
- Save operation: <100ms
- AI summary: <60 seconds (depends on LLM)
- Memory usage: <100MB idle

## Future Architecture (v0.2+)
- Vector embeddings for semantic search
- Note linking and backreferences
- Export functionality (Markdown, JSON)
- Multi-note AI reasoning
- API key management for cloud LLMs
