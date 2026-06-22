<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import type { TabView } from "../App.vue";

defineProps<{
  tabs: TabView[];
  activeTabId: string | null;
  dirtyFiles: Set<string>;
}>();

const emit = defineEmits<{
  (e: "open-file"): void;
  (e: "new-file"): void;
  (e: "select-tab", tabId: string): void;
  (e: "close-tab", tabId: string): void;
  (e: "open-settings"): void;
  (e: "fork-file", filePath: string): void;
  (e: "clear-history", janaId: string): void;
  (e: "reveal-in-finder", filePath: string): void;
  (e: "save-as", tabId: string): void;
}>();

const contextMenu = ref<{
  x: number;
  y: number;
  tabId: string;
  filePath: string;
  janaId: string;
} | null>(null);

function handleCloseTab(event: Event, tabId: string) {
  event.stopPropagation();
  emit("close-tab", tabId);
}

function openContextMenu(event: MouseEvent, tab: TabView) {
  event.preventDefault();
  contextMenu.value = {
    x: event.clientX,
    y: event.clientY,
    tabId: tab.tabId,
    filePath: tab.filePath,
    janaId: tab.janaId,
  };
}

function closeContextMenu() {
  contextMenu.value = null;
}

function handleFork() {
  if (contextMenu.value) {
    emit("fork-file", contextMenu.value.filePath);
  }
  closeContextMenu();
}

function handleClearHistory() {
  if (contextMenu.value) {
    emit("clear-history", contextMenu.value.janaId);
  }
  closeContextMenu();
}

function handleReveal() {
  if (contextMenu.value) {
    emit("reveal-in-finder", contextMenu.value.filePath);
  }
  closeContextMenu();
}

function handleSaveAs() {
  if (contextMenu.value) {
    emit("save-as", contextMenu.value.tabId);
  }
  closeContextMenu();
}

function onDocumentClick() {
  closeContextMenu();
}

function isTempFile(filePath: string): boolean {
  return filePath.includes('/jana/temp/');
}

onMounted(() => {
  document.addEventListener("click", onDocumentClick);
});

onUnmounted(() => {
  document.removeEventListener("click", onDocumentClick);
});
</script>

<template>
  <div class="sidebar">
    <div class="sidebar-header">
      <span class="sidebar-title">Files</span>
      <div style="display: flex; gap: 4px;">
        <button class="header-btn" @click="emit('open-file')" title="Open File (Cmd+O)">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <path d="M2 3h5l2 2h5v8H2V3z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" fill="none"/>
          </svg>
        </button>
        <button class="header-btn" @click="emit('new-file')" title="New File (Cmd+N)">+</button>
      </div>
    </div>
    <div class="file-list">
      <div
        v-for="tab in tabs"
        :key="tab.tabId"
        class="file-item"
        :class="{ active: tab.tabId === activeTabId }"
        @click="emit('select-tab', tab.tabId)"
        @contextmenu="openContextMenu($event, tab)"
      >
        <span class="file-name" :class="{ 'temp-file': isTempFile(tab.filePath) }">
          <span v-if="isTempFile(tab.filePath)" class="temp-indicator" title="Temporary file (not saved to disk)">○</span>
          <span v-if="dirtyFiles.has(tab.filePath)" class="dirty-indicator" title="Unsaved changes">•</span>
          {{ tab.fileName }}
        </span>
        <button
          class="close-btn"
          @click="handleCloseTab($event, tab.tabId)"
          title="Close"
        >&times;</button>
      </div>
      <div v-if="tabs.length === 0" class="empty-state">
        No files open
      </div>
    </div>
    <Teleport to="body">
      <div
        v-if="contextMenu"
        class="context-menu"
        :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
        @click.stop
      >
        <button class="context-menu-item" @click="handleSaveAs">Save As...</button>
        <button class="context-menu-item" @click="handleReveal">Reveal in Finder</button>
        <div style="height: 1px; background: var(--ctp-surface0, #313244); margin: 4px 0;"></div>
        <button class="context-menu-item" @click="handleFork">Fork File (New Identity)</button>
        <button class="context-menu-item danger" @click="handleClearHistory">Clear AI History</button>
      </div>
    </Teleport>
    <div class="sidebar-footer">
      <button class="settings-btn" @click="emit('open-settings')" title="Settings">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <path d="M6.5 1L6.2 2.6C5.8 2.7 5.5 2.9 5.2 3.1L3.6 2.5L2.1 5L3.4 6.1C3.4 6.4 3.4 6.6 3.4 6.9L2.1 8L3.6 10.5L5.2 9.9C5.5 10.1 5.8 10.3 6.2 10.4L6.5 12H9.5L9.8 10.4C10.2 10.3 10.5 10.1 10.8 9.9L12.4 10.5L13.9 8L12.6 6.9C12.6 6.6 12.6 6.4 12.6 6.1L13.9 5L12.4 2.5L10.8 3.1C10.5 2.9 10.2 2.7 9.8 2.6L9.5 1H6.5Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/>
          <circle cx="8" cy="6.5" r="2" stroke="currentColor" stroke-width="1.2"/>
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.sidebar {
  border-right: 1px solid #313244;
  display: flex;
  flex-direction: column;
  background: #181825;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid #313244;
}

.sidebar-title {
  font-size: 13px;
  font-weight: 600;
  color: #cdd6f4;
}

.header-btn {
  background: none;
  border: 1px solid #45475a;
  color: #cdd6f4;
  width: 24px;
  height: 24px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.header-btn:hover {
  background: #313244;
}

.file-list {
  flex: 1;
  overflow-y: auto;
}

.file-item {
  padding: 8px 16px;
  cursor: pointer;
  border-bottom: 1px solid #313244;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 4px;
}

.file-item:hover {
  background: #1e1e2e;
}

.file-item.active {
  background: #313244;
}

.file-name {
  font-size: 13px;
  color: #cdd6f4;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.file-name.temp-file {
  font-style: italic;
  color: #a6adc8;
}

.temp-indicator {
  color: #89b4fa;
  margin-right: 2px;
  font-weight: bold;
}

.dirty-indicator {
  color: #f9e2af;
  margin-right: 4px;
  font-weight: bold;
}

.close-btn {
  background: none;
  border: none;
  color: #6c7086;
  cursor: pointer;
  font-size: 14px;
  padding: 0 2px;
  border-radius: 2px;
  line-height: 1;
  opacity: 0;
}

.file-item:hover .close-btn,
.file-item.active .close-btn {
  opacity: 1;
}

.close-btn:hover {
  color: #f38ba8;
  background: rgba(243, 139, 168, 0.1);
}

.empty-state {
  padding: 16px;
  text-align: center;
  color: #6c7086;
  font-size: 13px;
}

.sidebar-footer {
  border-top: 1px solid #313244;
  padding: 8px 16px;
}

.settings-btn {
  background: none;
  border: none;
  color: #6c7086;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.settings-btn:hover {
  color: #cdd6f4;
  background: #313244;
}
</style>

<style>
.context-menu {
  position: fixed;
  z-index: 1000;
  background: #1e1e2e;
  border: 1px solid #313244;
  border-radius: 6px;
  padding: 4px 0;
  min-width: 180px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
}

.context-menu-item {
  display: block;
  width: 100%;
  padding: 8px 12px;
  background: none;
  border: none;
  color: #cdd6f4;
  font-size: 13px;
  text-align: left;
  cursor: pointer;
}

.context-menu-item:hover {
  background: #313244;
}

.context-menu-item.danger:hover {
  background: rgba(243, 139, 168, 0.15);
  color: #f38ba8;
}
</style>
