# ADR-002: Frontend Framework — Vue 3

## Status
Accepted

## Context
Need a frontend framework for the Tauri webview. The app has: a code editor (CodeMirror 6), a note list sidebar, and a summary panel. Options considered:
- **Alpine.js** — User's preference, lightweight, mini-Vue
- **Vue 3** — Composition API, reactive bindings, component structure
- **React** — Large ecosystem, JSX
- **Svelte** — Compiled, minimal runtime

## Decision
**Vue 3 with Composition API and TypeScript.**

## Rationale
- Alpine.js is designed for sprinkling interactivity on server-rendered pages, not for managing complex component state (editor + sidebar + panels)
- CodeMirror 6 has clean integration with Vue via `vue-codemirror`
- Vue 3 Composition API is lightweight and similar to Alpine's reactive model
- TypeScript gives type-safe bindings with Tauri's `invoke()` calls
- User was open to Vue if it was the better fit

## Consequences
- Slightly more boilerplate than Alpine
- Need `vue-tsc` for type checking in build pipeline
- Component-based architecture scales well for v0.2+ features
