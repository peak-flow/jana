<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from "vue";
import { EditorState, ChangeSet, Annotation } from "@codemirror/state";
import { EditorView, keymap, lineNumbers, highlightActiveLine } from "@codemirror/view";
import { markdown } from "@codemirror/lang-markdown";
import { oneDark } from "@codemirror/theme-one-dark";
import { defaultKeymap, history, historyKeymap, insertTab, indentLess } from "@codemirror/commands";
import { search, searchKeymap, highlightSelectionMatches } from "@codemirror/search";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getBuffer, updateBuffer, flushBuffer } from "../composables/useFiles";

const props = defineProps<{
  filePath: string | null;
  bufferId: string | null;
  windowId: string;
}>();

// Remote edits (applied from a peer window's `buffer-updated`) are tagged so the
// update listener doesn't echo them back to the backend.
const Remote = Annotation.define<boolean>();

interface BufferUpdatedPayload {
  buffer_id: string;
  changes: any;
  version: number;
  origin_window_id: string;
}

const emit = defineEmits<{
  (e: "dirty-change", filePath: string, isDirty: boolean): void;
}>();

const editorRef = ref<HTMLDivElement>();
const saveStatus = ref<"saved" | "saving" | "idle">("idle");
let editorView: EditorView | null = null;
let currentBufferId: string | null = null;
let currentFilePath: string | null = null;

// Optimistic version the editor believes the buffer is at. Seeded from get_buffer
// on load and advanced by each accepted update_buffer ack.
let localVersion = 0;
// Single-flight push: only one update_buffer is in flight at a time. The newest
// content and the composed ChangeSet covering all un-sent keystrokes are coalesced
// and sent next. This keeps base_version in order, avoids hammering IPC on every
// keystroke, and still relays a change set peers can apply to their own view.
let inFlight = false;
let pendingContent: string | null = null;
let pendingChanges: ChangeSet | null = null;
// Guards the doc-replace we do on conflict resync so it isn't treated as an edit.
let applyingRemote = false;
// Bumped on every loadBuffer so a stale async load (whose tab has since been
// switched away) can detect it lost the race and skip mutating editor state.
let loadGeneration = 0;
// Active subscription to peer `buffer-updated` broadcasts.
let unlisten: UnlistenFn | null = null;

function flushUpdate() {
  if (pendingContent === null || pendingChanges === null || !currentBufferId) {
    inFlight = false;
    return;
  }
  // Capture the buffer this batch belongs to. If the tab is switched mid-flight the
  // completion handlers must not touch the new buffer's state.
  const bufferId = currentBufferId;
  const filePath = currentFilePath;
  const content = pendingContent;
  const changes = pendingChanges;
  pendingContent = null;
  pendingChanges = null;
  inFlight = true;
  const base = localVersion;
  localVersion = base + 1;
  saveStatus.value = "saving";
  let failed = false;
  updateBuffer(bufferId, content, changes.toJSON(), base, props.windowId)
    .then((r) => {
      if (currentBufferId !== bufferId) return; // switched away — response is stale
      localVersion = r.version;
      // Backend is ahead — adopt its authoritative content (OCC resync).
      if (r.conflict && r.content !== null) reloadContent(r.content);
    })
    .catch((e) => {
      console.error("update_buffer failed:", e);
      if (currentBufferId !== bufferId) return; // switched away — drop the stale retry
      failed = true;
      localVersion = base; // retry from the right base
      // Re-queue the unsent batch instead of dropping it. Edits that arrived while
      // this one was in flight came *after* it, so the failed changes compose first.
      pendingChanges = pendingChanges ? changes.compose(pendingChanges) : changes;
      if (pendingContent === null) pendingContent = content;
    })
    .finally(() => {
      inFlight = false;
      if (currentBufferId !== bufferId) return; // a switch manages its own state
      if (pendingContent !== null) {
        // Still dirty — keep the indicator honest and keep trying. Defer the retry
        // when the last attempt failed so a persistent error can't hot-spin.
        if (filePath) emit("dirty-change", filePath, true);
        if (failed) {
          window.setTimeout(() => {
            if (!inFlight && pendingContent !== null && currentBufferId === bufferId) {
              flushUpdate();
            }
          }, 500);
        } else {
          flushUpdate();
        }
      } else {
        saveStatus.value = "saved";
        if (filePath) emit("dirty-change", filePath, false);
      }
    });
}

function scheduleUpdate(changes: ChangeSet, content: string) {
  pendingContent = content;
  // Compose so a single flush carries every un-sent keystroke as one change set.
  pendingChanges = pendingChanges ? pendingChanges.compose(changes) : changes;
  if (currentFilePath) emit("dirty-change", currentFilePath, true);
  if (!inFlight) flushUpdate();
}

function reloadContent(content: string) {
  if (!editorView) return;
  applyingRemote = true;
  editorView.dispatch({
    changes: { from: 0, to: editorView.state.doc.length, insert: content },
  });
  applyingRemote = false;
}

function createEditor(content: string) {
  if (editorView) {
    editorView.destroy();
  }

  const state = EditorState.create({
    doc: content,
    extensions: [
      lineNumbers(),
      highlightActiveLine(),
      history(),
      // Tab indents (inserts at cursor, or block-indents a multi-line selection),
      // Shift-Tab dedents — IDE-style. Listed first so it wins over defaultKeymap.
      keymap.of([
        { key: "Tab", run: insertTab, shift: indentLess },
        ...defaultKeymap,
        ...historyKeymap,
        ...searchKeymap,
      ]),
      search(),
      highlightSelectionMatches(),
      markdown(),
      oneDark,
      EditorView.updateListener.of((update) => {
        if (!update.docChanged || applyingRemote) return;
        // Don't echo a remote-applied change back to the backend.
        if (update.transactions.some((tr) => tr.annotation(Remote))) return;
        if (currentBufferId) {
          saveStatus.value = "idle";
          scheduleUpdate(update.changes, update.state.doc.toString());
        }
      }),
      EditorView.theme({
        "&": { height: "100%" },
        ".cm-scroller": { overflow: "auto" },
        ".cm-content": { padding: "16px 24px", fontSize: "15px", lineHeight: "1.6" },
      }),
    ],
  });

  editorView = new EditorView({
    state,
    parent: editorRef.value!,
  });
}

// Push the current buffer's pending edits to the backend and wait (bounded) until
// the queue is empty. Re-triggers flushUpdate each tick so a requeued failure keeps
// retrying; the ~2s cap stops a persistent backend error from blocking forever.
async function drainPending(): Promise<void> {
  for (let i = 0; i < 200; i++) {
    if (pendingContent === null && !inFlight) return;
    if (pendingContent !== null && !inFlight) flushUpdate();
    await new Promise((r) => setTimeout(r, 10));
  }
}

async function loadBuffer(bufferId: string, filePath: string) {
  // Drain edits queued for the buffer we're leaving before swapping — a fast tab
  // switch must not discard the previous buffer's unsent keystrokes.
  await drainPending();

  const gen = ++loadGeneration;
  currentBufferId = bufferId;
  currentFilePath = filePath;
  pendingContent = null;
  pendingChanges = null;
  inFlight = false;
  saveStatus.value = "idle";
  try {
    const snap = await getBuffer(bufferId);
    if (gen !== loadGeneration) return; // a newer switch superseded this load
    localVersion = snap.version;
    createEditor(snap.content);
  } catch (e) {
    if (gen !== loadGeneration) return;
    console.error("get_buffer failed:", e);
    localVersion = 0;
    createEditor("");
  }
}

// Manual save: drain pending IPC into the backend buffer, then force the backend to
// persist it to disk so "Saved" means the bytes are actually on disk.
async function immediatelySave(): Promise<void> {
  await drainPending();
  if (currentBufferId) {
    try {
      await flushBuffer(currentBufferId);
    } catch (e) {
      console.error("flush_buffer failed:", e);
    }
  }
}

function getContent(): string {
  return editorView?.state.doc.toString() ?? "";
}

defineExpose({ immediatelySave, getContent });

// Flush pending edits to the backend on window blur.
function onBlur() {
  if (pendingContent !== null && !inFlight) flushUpdate();
}

// Apply a peer window's edit to this view. Three echo guards keep it from looping:
// 1. origin filter — ignore our own broadcast; 2. version monotonicity — ignore
// stale/duplicate events; 3. the Remote annotation (applied here) stops the update
// listener from re-sending it. CodeMirror maps this view's cursor through `changes`.
function onBufferUpdated(payload: BufferUpdatedPayload) {
  if (payload.origin_window_id === props.windowId) return;
  if (payload.buffer_id !== currentBufferId) return;
  if (payload.version <= localVersion) return;
  if (!editorView) return;
  editorView.dispatch({
    changes: ChangeSet.fromJSON(payload.changes),
    annotations: Remote.of(true),
  });
  localVersion = payload.version;
}

onMounted(async () => {
  window.addEventListener("blur", onBlur);
  unlisten = await listen<BufferUpdatedPayload>("buffer-updated", (e) =>
    onBufferUpdated(e.payload)
  );
  if (props.bufferId && props.filePath) {
    loadBuffer(props.bufferId, props.filePath);
  } else {
    createEditor("");
  }
});

onUnmounted(() => {
  window.removeEventListener("blur", onBlur);
  if (unlisten) unlisten();
  editorView?.destroy();
});

watch(
  () => props.bufferId,
  async (newBufferId) => {
    if (newBufferId && props.filePath) {
      loadBuffer(newBufferId, props.filePath);
    } else {
      // No buffer to show: drain the outgoing one first so its edits aren't lost.
      await drainPending();
      loadGeneration++;
      currentBufferId = null;
      currentFilePath = null;
      pendingContent = null;
      pendingChanges = null;
      inFlight = false;
      createEditor("");
    }
  }
);
</script>

<template>
  <div class="editor-wrapper">
    <div class="editor-status">
      <span v-if="saveStatus === 'saving'" class="status-text">Saving...</span>
      <span v-else-if="saveStatus === 'saved'" class="status-text saved">Saved</span>
    </div>
    <div ref="editorRef" class="editor-container" />
  </div>
</template>

<style scoped>
.editor-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  position: relative;
}

.editor-status {
  position: absolute;
  top: 8px;
  right: 16px;
  z-index: 10;
}

.status-text {
  font-size: 12px;
  color: #6c7086;
}

.status-text.saved {
  color: #a6e3a1;
}

.editor-container {
  flex: 1;
  overflow: hidden;
}

.editor-container :deep(.cm-editor) {
  height: 100%;
}

.editor-container :deep(.cm-search) {
  background: #1e1e2e;
  border-bottom: 1px solid #313244;
}

.editor-container :deep(.cm-search label),
.editor-container :deep(.cm-search button) {
  color: #cdd6f4;
}

.editor-container :deep(.cm-search input) {
  background: #313244;
  color: #cdd6f4;
  border: 1px solid #45475a;
  border-radius: 3px;
}

.editor-container :deep(.cm-search button:hover) {
  background: #313244;
}

.editor-container :deep(.cm-selectionMatch) {
  background: rgba(137, 180, 250, 0.2);
}

.editor-container :deep(.cm-searchMatch) {
  background: rgba(249, 226, 175, 0.3);
}

.editor-container :deep(.cm-searchMatch-selected) {
  background: rgba(249, 226, 175, 0.5);
}
</style>
