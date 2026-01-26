# Jana

A lightweight desktop note-taking app with local AI summarization. Write notes, and get concise AI-generated summaries powered by your own local LLM.

## Features

- **Distraction-free editor** — CodeMirror 6 with autosave
- **AI summaries** — One-click note summarization via any OpenAI-compatible API
- **Local-first** — All data stored in SQLite on your machine
- **Configurable LLM** — Point to any local or remote LLM endpoint

## Installation (Unsigned App)

This app is not signed with an Apple Developer certificate. macOS will block it by default.

### macOS — First Launch

1. Open the `.dmg` and drag **Jana** to Applications
2. On first launch, macOS will show "Jana can't be opened because Apple cannot check it for malicious software"
3. Open **System Settings > Privacy & Security**
4. Scroll down — you'll see a message about Jana being blocked
5. Click **Open Anyway**
6. Confirm the dialog

Alternatively, right-click the app and select **Open** — this bypasses Gatekeeper for that session.

### Windows

If Windows Defender SmartScreen blocks the app:

1. Click **More info**
2. Click **Run anyway**

## Configuring Your LLM

Jana works with any OpenAI-compatible chat completions API (LM Studio, Ollama, vLLM, OpenAI, etc.).

### Setup

1. Click the **gear icon** at the bottom of the sidebar
2. Set **LLM URL** — the full chat completions endpoint, e.g.:
   - LM Studio: `http://localhost:1234/v1/chat/completions`
   - Ollama: `http://localhost:11434/v1/chat/completions`
   - OpenAI: `https://api.openai.com/v1/chat/completions`
3. Set **Model Name** — the model identifier your server expects, e.g.:
   - `qwen3-vl-30b`
   - `llama3`
   - `gpt-4o`
4. Click **Save**

Settings persist across app restarts.

### Requirements

- Your LLM server must be running and accessible at the configured URL
- The endpoint must accept the OpenAI chat completions format
- No API key management yet — use endpoints that don't require auth, or servers running locally

## Data Storage

Notes and settings are stored in SQLite at:

- **macOS:** `~/Library/Application Support/jana/notes.db`
- **Linux:** `~/.local/share/jana/notes.db`
- **Windows:** `%APPDATA%/jana/notes.db`
