<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from "vue";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers, highlightActiveLine } from "@codemirror/view";
import { markdown } from "@codemirror/lang-markdown";
import { oneDark } from "@codemirror/theme-one-dark";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { debounce } from "lodash-es";
import { saveFile } from "../composables/useFiles";

const props = defineProps<{
  filePath: string | null;
  janaId: string | null;
  content: string;
}>();

const editorRef = ref<HTMLDivElement>();
const saveStatus = ref<"saved" | "saving" | "idle">("idle");
let editorView: EditorView | null = null;
let currentFilePath: string | null = null;
let currentJanaId: string | null = null;

const debouncedSave = debounce(async (filePath: string, janaId: string, content: string) => {
  saveStatus.value = "saving";
  try {
    await saveFile(filePath, janaId, content);
    saveStatus.value = "saved";
  } catch (e) {
    console.error("Save failed:", e);
    saveStatus.value = "idle";
  }
}, 2000);

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
      keymap.of([...defaultKeymap, ...historyKeymap]),
      markdown(),
      oneDark,
      EditorView.updateListener.of((update) => {
        if (update.docChanged && currentFilePath && currentJanaId) {
          saveStatus.value = "idle";
          debouncedSave(currentFilePath, currentJanaId, update.state.doc.toString());
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

function loadFile(filePath: string, janaId: string, content: string) {
  currentFilePath = filePath;
  currentJanaId = janaId;
  saveStatus.value = "idle";
  createEditor(content);
}

// Save on window blur
function onBlur() {
  if (currentFilePath && editorView) {
    debouncedSave.flush();
  }
}

onMounted(() => {
  window.addEventListener("blur", onBlur);
  if (!props.filePath) {
    createEditor("");
  }
});

onUnmounted(() => {
  window.removeEventListener("blur", onBlur);
  debouncedSave.cancel();
  editorView?.destroy();
});

watch(
  () => props.filePath,
  (newPath) => {
    // Flush any pending save for previous file
    debouncedSave.flush();
    if (newPath && props.janaId) {
      loadFile(newPath, props.janaId, props.content);
    } else {
      currentFilePath = null;
      currentJanaId = null;
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
</style>
