# ADR-006: Autosave Strategy — 2s Debounce + Blur

## Status
Accepted

## Context
PRD specifies "autosave every N seconds or on blur" with no save dialogs. Need to balance data safety vs SQLite write frequency.

## Decision
**2-second debounce after last keystroke + immediate save on window blur.**

## Rationale
- 2s debounce: user stops typing for 2 seconds → save fires. Prevents hammering SQLite on every keystroke
- Blur save: switching windows or tabbing away immediately persists content
- No periodic interval timer (unnecessary with debounce approach)
- Visual indicator shows "Saving..." / "Saved" state
- Title auto-derived from first line of content (no separate title input needed)

## Implementation
- Frontend: `lodash-es` `debounce(saveNote, 2000)` on CodeMirror's `onUpdate`
- Frontend: Tauri `listen('tauri://blur')` event triggers immediate save
- Backend: `save_note` uses `INSERT OR REPLACE` (upsert pattern)

## Consequences
- Maximum 2 seconds of potential data loss on crash
- SQLite writes are small (single row upsert) — no performance concern
- No undo/version history in v0.1 (future consideration)
