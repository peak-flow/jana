# ADR-001: Desktop Framework — Tauri v2

## Status
Accepted

## Context
Need a desktop shell for a note-taking app. Requirements: fast startup, small binary, native feel on macOS. Options considered:
- **Electron** — Large binary (~150MB), high memory, JS backend
- **Tauri v2** — Small binary (~10MB), native webview, Rust backend
- **Wails** — Similar to Tauri but Go backend
- **Tauri + Go sidecar** — Tauri shell with Go binary for backend logic

## Decision
**Tauri v2 with Rust backend.** No sidecar, no Go.

## Rationale
- User prioritized maximum speed → Rust is faster than Go for this workload
- Tauri's `invoke()` is zero-overhead IPC (no HTTP, no sidecar process management)
- Single binary output — simpler deployment
- Wails was considered but Tauri has a larger ecosystem (plugins, updater, etc.)
- Go sidecar was rejected as over-engineered for v0.1

## Consequences
- Must write backend logic in Rust (learning curve if unfamiliar)
- Tied to Tauri's command/state patterns
- Cross-platform builds require platform-specific toolchains
