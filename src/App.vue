<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import Sidebar from "./components/Sidebar.vue";
import Editor from "./components/Editor.vue";
import SummaryPanel from "./components/SummaryPanel.vue";
import SettingsModal from "./components/SettingsModal.vue";
import {
  openFileDialog,
  readFile,
  listTabs,
  closeTab,
  forkFile,
  clearAiHistory,
  revealInFinder,
  createNewFile,
  saveTempFileAs,
  type TabResult,
} from "./composables/useFiles";

export interface TabView {
  tabId: string;
  filePath: string;
  janaId: string;
  fileName: string;
  content: string;
}

// Each window owns its own tabs. The first window has no `?window_id=` and
// falls back to "main", matching the migrated default window.
const windowId = new URLSearchParams(location.search).get("window_id") ?? "main";

const tabs = ref<TabView[]>([]);
const activeTabId = ref<string | null>(null);
// Dirty state is keyed by file path; within a single window each path maps to
// exactly one tab, so this stays 1:1 with the open tabs.
const dirtyFiles = ref<Set<string>>(new Set());
const showSettings = ref(false);
const editorRef = ref<InstanceType<typeof Editor> | null>(null);
const sidebarWidth = ref(200);
const isResizing = ref(false);
const showSummaryPanel = ref(true);

const activeTab = computed(
  () => tabs.value.find((t) => t.tabId === activeTabId.value) ?? null
);

function onResizeStart(e: MouseEvent) {
  e.preventDefault();
  isResizing.value = true;
  const onMove = (ev: MouseEvent) => {
    const w = Math.min(Math.max(ev.clientX, 120), 500);
    sidebarWidth.value = w;
  };
  const onUp = () => {
    isResizing.value = false;
    document.removeEventListener("mousemove", onMove);
    document.removeEventListener("mouseup", onUp);
  };
  document.addEventListener("mousemove", onMove);
  document.addEventListener("mouseup", onUp);
}

function onDirtyChange(filePath: string, isDirty: boolean) {
  if (isDirty) {
    dirtyFiles.value.add(filePath);
  } else {
    dirtyFiles.value.delete(filePath);
  }
  // Trigger reactivity
  dirtyFiles.value = new Set(dirtyFiles.value);
}

async function handleOpenFile() {
  try {
    const result = await openFileDialog(windowId);
    if (!result) return;
    addTabToList(result);
  } catch (e) {
    console.error("Failed to open file:", e);
  }
}

function addTabToList(result: TabResult) {
  // Backend dedups by (window_id, file_path), so reopening a file returns its
  // existing tab_id — just activate it instead of pushing a duplicate.
  const existing = tabs.value.find((t) => t.tabId === result.tab_id);
  if (existing) {
    activeTabId.value = existing.tabId;
    return;
  }

  const tab: TabView = {
    tabId: result.tab_id,
    filePath: result.file_path,
    janaId: result.jana_id,
    fileName: result.file_name,
    content: result.content,
  };
  tabs.value.push(tab);
  activeTabId.value = tab.tabId;
}

function onSelectTab(tabId: string) {
  activeTabId.value = tabId;
}

async function onCloseTab(tabId: string) {
  const tab = tabs.value.find((t) => t.tabId === tabId);
  try {
    await closeTab(tabId);
  } catch (e) {
    console.error("Failed to close tab:", e);
  }
  tabs.value = tabs.value.filter((t) => t.tabId !== tabId);
  if (tab) {
    dirtyFiles.value.delete(tab.filePath);
    dirtyFiles.value = new Set(dirtyFiles.value);
  }
  if (activeTabId.value === tabId) {
    activeTabId.value = tabs.value.length > 0 ? tabs.value[0].tabId : null;
  }
}

async function onForkFile(filePath: string) {
  try {
    const newJanaId = await forkFile(filePath);
    // Identity is per-file: update every tab pointing at this path.
    for (const t of tabs.value) {
      if (t.filePath === filePath) t.janaId = newJanaId;
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

async function onRevealInFinder(filePath: string) {
  try {
    await revealInFinder(filePath);
  } catch (e) {
    console.error("Failed to reveal in finder:", e);
  }
}

async function handleNewFile() {
  try {
    const result = await createNewFile(windowId);
    addTabToList(result);
  } catch (e) {
    console.error("Failed to create new file:", e);
  }
}

async function handleSaveAs(tabId: string) {
  try {
    const tab = tabs.value.find((t) => t.tabId === tabId);
    if (!tab) return;
    // Flush editor if this is the active tab, then grab its current content.
    const isActive = activeTabId.value === tabId;
    if (isActive) {
      editorRef.value?.immediatelySave();
    }
    const content = isActive
      ? (editorRef.value?.getContent() ?? tab.content)
      : tab.content;
    const newPath = await saveTempFileAs(tabId, tab.filePath, tab.janaId, content);
    if (!newPath) return;
    // The file moved temp → real: update the tab and clear its dirty marker.
    dirtyFiles.value.delete(tab.filePath);
    tab.filePath = newPath;
    tab.fileName = newPath.split("/").pop() ?? newPath;
    tab.content = content;
    dirtyFiles.value = new Set(dirtyFiles.value);
  } catch (e) {
    console.error("Failed to save as:", e);
  }
}

// Keyboard shortcuts
function handleKeydown(e: KeyboardEvent) {
  const mod = e.metaKey || e.ctrlKey;
  if (!mod) return;

  if (e.key === "n") {
    e.preventDefault();
    handleNewFile();
  } else if (e.key === "o") {
    e.preventDefault();
    handleOpenFile();
  } else if ((e.key === "s" || e.key === "S") && e.shiftKey) {
    e.preventDefault();
    if (activeTabId.value) handleSaveAs(activeTabId.value);
  } else if (e.key === "s") {
    e.preventDefault();
    editorRef.value?.immediatelySave();
  } else if (e.key === "w") {
    e.preventDefault();
    if (activeTabId.value) onCloseTab(activeTabId.value);
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
});

// Session restore — load this window's tabs and hydrate each with file content.
onMounted(async () => {
  try {
    const entries = await listTabs(windowId);
    for (const entry of entries.sort((a, b) => a.tab_order - b.tab_order)) {
      try {
        const result = await readFile(entry.file_path);
        tabs.value.push({
          tabId: entry.tab_id,
          filePath: result.file_path,
          janaId: result.jana_id,
          fileName: result.file_name,
          content: result.content,
        });
      } catch {
        // File no longer exists on disk — drop the stale tab.
        await closeTab(entry.tab_id).catch(() => {});
      }
    }
    if (tabs.value.length > 0) {
      activeTabId.value = tabs.value[0].tabId;
    }
  } catch (e) {
    console.error("Session restore failed:", e);
  }
});
</script>

<template>
  <div class="app-layout" :class="{ resizing: isResizing }">
    <Sidebar
      :tabs="tabs"
      :active-tab-id="activeTabId"
      :dirty-files="dirtyFiles"
      :style="{ width: sidebarWidth + 'px', minWidth: sidebarWidth + 'px' }"
      @new-file="handleNewFile"
      @open-file="handleOpenFile"
      @select-tab="onSelectTab"
      @close-tab="onCloseTab"
      @fork-file="onForkFile"
      @clear-history="onClearHistory"
      @reveal-in-finder="onRevealInFinder"
      @save-as="handleSaveAs"
      @open-settings="showSettings = true"
    />
    <div class="resize-handle" @mousedown="onResizeStart" />
    <Editor
      ref="editorRef"
      :file-path="activeTab?.filePath ?? null"
      :jana-id="activeTab?.janaId ?? null"
      :content="activeTab?.content ?? ''"
      @dirty-change="onDirtyChange"
    />
    <button class="panel-toggle" @click="showSummaryPanel = !showSummaryPanel" :title="showSummaryPanel ? 'Hide AI panel' : 'Show AI panel'">
      {{ showSummaryPanel ? '›' : '‹' }}
    </button>
    <SummaryPanel
      v-show="showSummaryPanel"
      :jana-id="activeTab?.janaId ?? null"
      :file-path="activeTab?.filePath ?? null"
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

.app-layout.resizing {
  user-select: none;
  cursor: col-resize;
}

.resize-handle {
  width: 4px;
  cursor: col-resize;
  background: transparent;
  flex-shrink: 0;
  position: relative;
}

.resize-handle:hover,
.app-layout.resizing .resize-handle {
  background: #89b4fa;
}

.panel-toggle {
  background: #181825;
  border: none;
  border-left: 1px solid #313244;
  color: #6c7086;
  cursor: pointer;
  font-size: 16px;
  padding: 0 4px;
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.panel-toggle:hover {
  color: #cdd6f4;
  background: #1e1e2e;
}
</style>
