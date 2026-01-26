<script setup lang="ts">
import { ref, onMounted } from "vue";
import { listNotes, createNote, type NoteListItem } from "../composables/useNotes";

defineProps<{
  activeNoteId: string | null;
}>();

const emit = defineEmits<{
  (e: "select-note", id: string): void;
  (e: "new-note", id: string): void;
  (e: "open-settings"): void;
}>();

const notes = ref<NoteListItem[]>([]);

async function refreshNotes() {
  try {
    notes.value = await listNotes();
  } catch (e) {
    console.error("Failed to list notes:", e);
  }
}

async function handleNewNote() {
  try {
    const note = await createNote();
    await refreshNotes();
    emit("new-note", note.id);
  } catch (e) {
    console.error("Failed to create note:", e);
  }
}

function selectNote(id: string) {
  emit("select-note", id);
}

function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
  });
}

onMounted(async () => {
  await refreshNotes();
  // Auto-select first note or create one
  if (notes.value.length > 0) {
    emit("select-note", notes.value[0].id);
  } else {
    await handleNewNote();
  }
});

// Refresh list periodically to catch autosaved title changes
let interval: ReturnType<typeof setInterval>;
onMounted(() => {
  interval = setInterval(refreshNotes, 5000);
});

import { onUnmounted } from "vue";
onUnmounted(() => {
  clearInterval(interval);
});
</script>

<template>
  <div class="sidebar">
    <div class="sidebar-header">
      <span class="sidebar-title">Notes</span>
      <button class="new-note-btn" @click="handleNewNote" title="New Note">+</button>
    </div>
    <div class="note-list">
      <div
        v-for="note in notes"
        :key="note.id"
        class="note-item"
        :class="{ active: note.id === activeNoteId }"
        @click="selectNote(note.id)"
      >
        <span class="note-title">{{ note.title || "Untitled" }}</span>
        <span class="note-date">{{ formatDate(note.updated_at) }}</span>
      </div>
      <div v-if="notes.length === 0" class="empty-state">
        No notes yet
      </div>
    </div>
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
  width: 200px;
  min-width: 200px;
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

.new-note-btn {
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

.new-note-btn:hover {
  background: #313244;
}

.note-list {
  flex: 1;
  overflow-y: auto;
}

.note-item {
  padding: 10px 16px;
  cursor: pointer;
  border-bottom: 1px solid #313244;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.note-item:hover {
  background: #1e1e2e;
}

.note-item.active {
  background: #313244;
}

.note-title {
  font-size: 13px;
  color: #cdd6f4;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.note-date {
  font-size: 11px;
  color: #6c7086;
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
