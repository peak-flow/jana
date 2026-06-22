<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from "vue";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers, highlightActiveLine } from "@codemirror/view";
import { markdown } from "@codemirror/lang-markdown";
import { oneDark } from "@codemirror/theme-one-dark";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { search, searchKeymap, highlightSelectionMatches } from "@codemirror/search";
import { getBuffer, updateBuffer } from "../composables/useFiles";

const props = defineProps<{
  filePath: string | null;
  bufferId: string | null;
}>();

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
// Single-flight push: only one update_buffer is in flight at a time; the newest
// content is coalesced into `pending` and sent next. This keeps base_version in
// order and avoids hammering IPC on every keystroke.
let inFlight = false;
let pending: string | null = null;
// Guards the doc-replace we do on conflict resync so it isn't treated as an edit.
let applyingRemote = false;

function flushUpdate() {
  if (pending === null || !currentBufferId) {
    inFlight = false;
    return;
  }
  const content = pending;
  pending = null;
  inFlight = true;
  const base = localVersion;
  localVersion = base + 1;
  saveStatus.value = "saving";
  updateBuffer(currentBufferId, content, base)
    .then((r) => {
      if (r.conflict) {
        // Backend is ahead — adopt its version and content.
        localVersion = r.version;
        if (r.content !== null) reloadContent(r.content);
      } else {
        localVersion = r.version;
      }
    })
    .catch((e) => {
      console.error("update_buffer failed:", e);
      localVersion = base; // allow the next attempt to retry from the right base
    })
    .finally(() => {
      inFlight = false;
      if (pending !== null) {
        flushUpdate();
      } else {
        saveStatus.value = "saved";
        if (currentFilePath) emit("dirty-change", currentFilePath, false);
      }
    });
}

function scheduleUpdate(content: string) {
  pending = content;
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
      keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap]),
      search(),
      highlightSelectionMatches(),
      markdown(),
      oneDark,
      EditorView.updateListener.of((update) => {
        if (!update.docChanged || applyingRemote) return;
        if (currentBufferId) {
          saveStatus.value = "idle";
          scheduleUpdate(update.state.doc.toString());
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

async function loadBuffer(bufferId: string, filePath: string) {
  currentBufferId = bufferId;
  currentFilePath = filePath;
  pending = null;
  inFlight = false;
  saveStatus.value = "idle";
  try {
    const snap = await getBuffer(bufferId);
    localVersion = snap.version;
    createEditor(snap.content);
  } catch (e) {
    console.error("get_buffer failed:", e);
    localVersion = 0;
    createEditor("");
  }
}

// Push any pending edit to the backend immediately and resolve once it lands.
async function immediatelySave(): Promise<void> {
  if (pending !== null && !inFlight) flushUpdate();
  while (inFlight || pending !== null) {
    await new Promise((r) => setTimeout(r, 10));
  }
}

function getContent(): string {
  return editorView?.state.doc.toString() ?? "";
}

defineExpose({ immediatelySave, getContent });

// Flush pending edits to the backend on window blur.
function onBlur() {
  if (pending !== null && !inFlight) flushUpdate();
}

onMounted(() => {
  window.addEventListener("blur", onBlur);
  if (props.bufferId && props.filePath) {
    loadBuffer(props.bufferId, props.filePath);
  } else {
    createEditor("");
  }
});

onUnmounted(() => {
  window.removeEventListener("blur", onBlur);
  editorView?.destroy();
});

watch(
  () => props.bufferId,
  (newBufferId) => {
    if (newBufferId && props.filePath) {
      loadBuffer(newBufferId, props.filePath);
    } else {
      currentBufferId = null;
      currentFilePath = null;
      pending = null;
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
