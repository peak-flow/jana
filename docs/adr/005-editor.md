# ADR-005: Editor — CodeMirror 6

## Status
Accepted

## Context
Need a text editor component for the note-taking app. Requirements: fast, markdown support, extensible. Options considered:
- **CodeMirror 6** — Modern, extensible, built for embedding
- **Monaco (VS Code editor)** — Feature-rich, heavy (~5MB)
- **ProseMirror** — Rich-text focused, WYSIWYG
- **Plain textarea** — Simplest possible

## Decision
**CodeMirror 6 with markdown language support and One Dark theme, integrated via `vue-codemirror`.**

## Rationale
- CodeMirror 6 is purpose-built for embedding in applications
- Lightweight compared to Monaco (~200KB vs ~5MB)
- Markdown syntax highlighting out of the box
- Extensible: can add vim mode, search, etc. in v0.2
- `vue-codemirror` provides clean Vue 3 Composition API bindings
- One Dark theme matches the app's dark UI

## Consequences
- Single large JS chunk (~568KB gzipped ~200KB) — acceptable for desktop app
- Editor state management handled by CodeMirror internally
- Autosave hooks into CodeMirror's `onUpdate` callback
