<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import Sidebar from "./components/Sidebar.vue";
import Editor from "./components/Editor.vue";
import SummaryPanel from "./components/SummaryPanel.vue";
import SettingsModal from "./components/SettingsModal.vue";
import {
  openFileDialog,
  readFile,
  closeFile,
  listOpenFiles,
  forkFile,
  clearAiHistory,
  type OpenFileResult,
} from "./composables/useFiles";

export interface ActiveFile {
  filePath: string;
  janaId: string;
  fileName: string;
  content: string;
}

const activeFile = ref<ActiveFile | null>(null);
const openFiles = ref<ActiveFile[]>([]);
const showSettings = ref(false);
const editorRef = ref<InstanceType<typeof Editor> | null>(null);

async function handleOpenFile() {
  try {
    const result = await openFileDialog();
    if (!result) return;
    addFileToList(result);
  } catch (e) {
    console.error("Failed to open file:", e);
  }
}

function addFileToList(result: OpenFileResult) {
  // Check if already open
  const existing = openFiles.value.find((f) => f.filePath === result.file_path);
  if (existing) {
    activeFile.value = existing;
    return;
  }

  const file: ActiveFile = {
    filePath: result.file_path,
    janaId: result.jana_id,
    fileName: result.file_name,
    content: result.content,
  };
  openFiles.value.push(file);
  activeFile.value = file;
}

function onSelectFile(filePath: string) {
  const file = openFiles.value.find((f) => f.filePath === filePath);
  if (file) {
    activeFile.value = file;
  }
}

async function onCloseFile(filePath: string) {
  try {
    await closeFile(filePath);
  } catch (e) {
    console.error("Failed to close file:", e);
  }
  openFiles.value = openFiles.value.filter((f) => f.filePath !== filePath);
  if (activeFile.value?.filePath === filePath) {
    activeFile.value = openFiles.value.length > 0 ? openFiles.value[0] : null;
  }
}

async function onForkFile(filePath: string) {
  try {
    const newJanaId = await forkFile(filePath);
    const file = openFiles.value.find((f) => f.filePath === filePath);
    if (file) {
      file.janaId = newJanaId;
    }
  } catch (e) {
    console.error("Failed to fork file:", e);
  }
}

async function onClearHistory(janaId: string) {
  try {
    await clearAiHistory(janaId);
  } catch (e) {
    console.error("Failed to clear AI history:", e);
  }
}

// Keyboard shortcuts
function handleKeydown(e: KeyboardEvent) {
  const mod = e.metaKey || e.ctrlKey;
  if (!mod) return;

  if (e.key === "o") {
    e.preventDefault();
    handleOpenFile();
  } else if (e.key === "s") {
    e.preventDefault();
    editorRef.value?.immediatelySave();
  } else if (e.key === "w") {
    e.preventDefault();
    if (activeFile.value) {
      onCloseFile(activeFile.value.filePath);
    }
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
});

// Session restore
onMounted(async () => {
  try {
    const entries = await listOpenFiles();
    for (const entry of entries.sort((a, b) => a.tab_order - b.tab_order)) {
      try {
        const result = await readFile(entry.file_path);
        openFiles.value.push({
          filePath: result.file_path,
          janaId: result.jana_id,
          fileName: result.file_name,
          content: result.content,
        });
      } catch {
        // File no longer exists — clean up
        await closeFile(entry.file_path).catch(() => {});
      }
    }
    if (openFiles.value.length > 0) {
      activeFile.value = openFiles.value[0];
    }
  } catch (e) {
    console.error("Session restore failed:", e);
  }
});
</script>

<template>
  <div class="app-layout">
    <Sidebar
      :open-files="openFiles"
      :active-file-path="activeFile?.filePath ?? null"
      @open-file="handleOpenFile"
      @select-file="onSelectFile"
      @close-file="onCloseFile"
      @fork-file="onForkFile"
      @clear-history="onClearHistory"
      @open-settings="showSettings = true"
    />
    <Editor
      ref="editorRef"
      :file-path="activeFile?.filePath ?? null"
      :jana-id="activeFile?.janaId ?? null"
      :content="activeFile?.content ?? ''"
    />
    <SummaryPanel
      :jana-id="activeFile?.janaId ?? null"
      :file-path="activeFile?.filePath ?? null"
    />
    <SettingsModal v-if="showSettings" @close="showSettings = false" />
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  height: 100%;
  width: 100%;
}
</style>
