# ADR-004: LLM Integration — Local LM Studio

## Status
Accepted

## Context
Need AI summarization for notes. Options considered:
- **Remote API (Anthropic/OpenAI)** — Reliable, fast, requires API key + internet
- **Local LM Studio** — Privacy-preserving, no cost per call, user already runs it
- **Ollama** — Alternative local inference
- **Embedded model (GGUF sidecar)** — Ship model with app

## Decision
**Local LM Studio at `192.168.77.1:1234` using OpenAI-compatible API, model `qwen3-vl-30b`.**

## Rationale
- User already has LM Studio running with qwen3-vl-30b loaded
- OpenAI-compatible endpoint means we can swap to any provider later (just change URL + model)
- No API key management needed for v0.1
- Privacy: notes never leave the local network
- 60-second timeout for large note summarization

## API Details
- Endpoint: `POST http://192.168.77.1:1234/v1/chat/completions`
- Model: `qwen3-vl-30b`
- Temperature: 0.3 (low for consistent summaries)
- System prompt: "Summarize the following note concisely. Focus on key points and action items."
- HTTP client: `reqwest` (async, built into Rust backend)

## Consequences
- App requires LM Studio to be running for summarization to work
- Summarization fails gracefully with error message if LM Studio is down
- Can easily add remote API fallback in v0.2
- Port 1234 does not conflict with Laravel Herd (port 8080)
