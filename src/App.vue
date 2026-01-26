<script setup lang="ts">
import { ref } from "vue";
import Sidebar from "./components/Sidebar.vue";
import Editor from "./components/Editor.vue";
import SummaryPanel from "./components/SummaryPanel.vue";
import SettingsModal from "./components/SettingsModal.vue";

const activeNoteId = ref<string | null>(null);
const showSettings = ref(false);

function onSelectNote(id: string) {
  activeNoteId.value = id;
}

function onNewNote(id: string) {
  activeNoteId.value = id;
}
</script>

<template>
  <div class="app-layout">
    <Sidebar
      :active-note-id="activeNoteId"
      @select-note="onSelectNote"
      @new-note="onNewNote"
      @open-settings="showSettings = true"
    />
    <Editor :note-id="activeNoteId" />
    <SummaryPanel :note-id="activeNoteId" />
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
