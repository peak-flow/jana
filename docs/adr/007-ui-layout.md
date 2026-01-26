# ADR-007: UI Layout — Three-Column Dark Theme

## Status
Accepted

## Context
Need a layout that supports: note list, editor, and AI summary display. User requested minimal sidebar.

## Decision
**Three-column layout: Sidebar (200px) | Editor (flex) | Summary Panel (280px, always visible).**

## Rationale
- Sidebar shows note titles + timestamps — minimal, scannable
- Editor takes remaining space — maximizes writing area
- Summary panel is always visible (no modal/popup) — results are persistent
- Catppuccin Mocha dark theme — modern, easy on eyes, consistent with CodeMirror One Dark
- No collapsible panels in v0.1 (keep simple)

## Layout
```
┌──────────┬─────────────────────────┬────────────┐
│ Sidebar  │        Editor           │  Summary   │
│ 200px    │      (flex: 1)          │   280px    │
│          │                         │            │
│ [notes]  │   [CodeMirror 6]        │ [Summarize]│
│          │                         │ [result]   │
└──────────┴─────────────────────────┴────────────┘
```

## Consequences
- Fixed sidebar width — no resize handles (v0.1 simplicity)
- Summary panel always takes space even when empty (acceptable trade-off)
- Dark theme only — no light mode toggle in v0.1
