# Jana v0.1 — Milestone Checklist

**Focus: Eleusis (Typing, Saving, Remembering)**

> v0.1 answers one question well:
> **"Where do I type and safely store my thoughts?"**

Everything else waits.

---

## Milestone 0 — Project Skeleton ✅

**Goal:** App boots, nothing fancy.

* [x] Create repo `jana`
* [x] Initialize Tauri v2 app
* [x] Frontend builds and runs (Vue 3)
* [x] Backend commands exist (Rust)
* [x] App launches on macOS

✅ Definition of done:
You can open Jana and see *something* on screen.

---

## Milestone 1 — Editor Core ✅

**Goal:** Typing feels instant and reliable.

* [x] Integrate CodeMirror 6
* [x] Single editor instance renders
* [x] Can type freely
* [x] Can paste large blocks of text
* [x] Editor state readable from Vue

❌ Not required:
* Syntax highlighting polish
* Themes (using One Dark)
* Markdown shortcuts

✅ Definition of done:
You trust the editor to hold text without lag or weirdness.

---

## Milestone 2 — Notes as First-Class Objects ✅

**Goal:** Every note is a real object.

* [x] Define `Note` model (id, title, content, timestamps)
* [x] Generate UUID for each new note
* [x] Create "New Note" action
* [x] Editor loads note content
* [x] Switching notes swaps editor state (via sidebar)

❌ Not required:
* Folders
* Search

✅ Definition of done:
You can open multiple notes and switch between them.

---

## Milestone 3 — Persistence ✅

**Goal:** Nothing is lost. Ever.

* [x] Local SQLite database created
* [x] Notes table implemented
* [x] Autosave on content change (debounced)
* [x] Load most recent note on startup
* [x] Update timestamps on change

❌ Not required:
* Manual save button
* Export
* Sync

✅ Definition of done:
You close the app mid-thought and reopen to find everything intact.

---

## Milestone 4 — Note Navigation ✅

**Goal:** Sublime-like mental model.

* [x] Sidebar note list UI
* [x] Visual indicator for active note
* [x] Note titles derived from content
* [x] Sorted by last updated

**Changed from original plan:** Implemented sidebar list instead of tabs. Simpler, works well for v0.1.

❌ Not required:
* Tab bar
* Drag reorder
* Split panes

✅ Definition of done:
You can comfortably work with multiple notes.

---

## Milestone 5 — Eleusis Identity

**Goal:** Make Eleusis *felt*, not explained.

* [ ] Eleusis referenced internally (naming, comments)
* [ ] Optional subtle UI label
* [ ] No lore dumps in UI
* [x] README explains app purpose clearly

❌ Not required:
* Custom icons
* Branding
* Animations

✅ Definition of done:
Eleusis feels like a place, not a feature.

---

## Milestone 6 — AI Summary ✅

**Goal:** Jana remembers *why* a note mattered.

* [x] "Summarize" button (manual trigger)
* [x] AI call with note content
* [x] Summary stored in DB
* [x] Summary linked to note
* [x] Summary displayed in side panel
* [x] Model name recorded with summary

❌ Not required:
* Auto summaries
* Embeddings
* RAG
* Multi-note reasoning

✅ Definition of done:
You can paste a transcript, summarize it, and come back later understanding its purpose in seconds.

---

## Milestone 6.5 — Configurable LLM ✅

**Goal:** Works with any local or remote LLM.

* [x] Settings table in SQLite
* [x] Configurable LLM URL
* [x] Configurable model name
* [x] Settings modal UI (gear icon)
* [x] Settings persist across restarts

✅ Definition of done:
You can point Jana at LM Studio, Ollama, or any OpenAI-compatible endpoint.

---

## Milestone 7 — Stability Pass

**Goal:** Trustworthiness.

* [x] Handle AI failures gracefully (shows error)
* [ ] Handle DB write failures
* [ ] Prevent data loss on crash
* [x] No console spam
* [ ] App feels calm, not fragile

✅ Definition of done:
You'd actually use this daily.

---

# Explicit v0.1 Non-Goals

To protect your sanity:

* ❌ Page links
* ❌ Toggles
* ❌ Backlinks
* ❌ Knowledge graph
* ❌ Task management
* ❌ Sync
* ❌ Accounts
* ❌ API key management (for now)

Those are **earned features**, not starting points.

---

## v0.1 Success Criteria

Jana v0.1 is successful if:

* You open it instead of other apps for raw notes
* You trust it not to lose text
* You feel relief dumping ideas into it
* You don't think about the app while typing
* AI summaries actually help you recall note purpose

That's the bar.
