<script setup lang="ts">
import { ref, watch } from "vue";
import { summarizeFile, getFileSummary, type AiInteraction } from "../composables/useLLM";

const props = defineProps<{
  janaId: string | null;
  filePath: string | null;
}>();

const summary = ref<AiInteraction | null>(null);
const isLoading = ref(false);
const error = ref<string | null>(null);

async function loadExistingSummary(janaId: string) {
  try {
    summary.value = await getFileSummary(janaId);
  } catch (e) {
    summary.value = null;
  }
}

async function handleSummarize() {
  if (!props.janaId || !props.filePath) return;

  isLoading.value = true;
  error.value = null;

  try {
    summary.value = await summarizeFile(props.janaId, props.filePath);
  } catch (e) {
    error.value = String(e);
  } finally {
    isLoading.value = false;
  }
}

function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

watch(
  () => props.janaId,
  (newId) => {
    error.value = null;
    if (newId) {
      loadExistingSummary(newId);
    } else {
      summary.value = null;
    }
  },
  { immediate: true }
);
</script>

<template>
  <div class="summary-panel">
    <div class="panel-header">
      <span class="panel-title">AI</span>
      <button
        class="summarize-btn"
        :disabled="!janaId || isLoading"
        @click="handleSummarize"
      >
        {{ isLoading ? "..." : "Summarize" }}
      </button>
    </div>
    <div class="panel-content">
      <div v-if="isLoading" class="loading">Generating summary...</div>
      <div v-else-if="error" class="error">{{ error }}</div>
      <div v-else-if="summary" class="summary">
        <p class="summary-text">{{ summary.response }}</p>
        <div class="summary-meta">
          <span>{{ summary.model }}</span>
          <span>{{ formatDate(summary.created_at) }}</span>
        </div>
      </div>
      <div v-else class="empty">
        Open a file and click Summarize
      </div>
    </div>
  </div>
</template>

<style scoped>
.summary-panel {
  width: 280px;
  min-width: 280px;
  border-left: 1px solid #313244;
  display: flex;
  flex-direction: column;
  background: #181825;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid #313244;
}

.panel-title {
  font-size: 13px;
  font-weight: 600;
  color: #cdd6f4;
}

.summarize-btn {
  background: #89b4fa;
  color: #1e1e2e;
  border: none;
  padding: 4px 12px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.summarize-btn:hover:not(:disabled) {
  background: #b4d0fb;
}

.summarize-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.panel-content {
  flex: 1;
  padding: 16px;
  overflow-y: auto;
}

.loading {
  color: #6c7086;
  font-size: 13px;
}

.error {
  color: #f38ba8;
  font-size: 13px;
  word-break: break-word;
}

.summary-text {
  font-size: 13px;
  line-height: 1.6;
  color: #cdd6f4;
  white-space: pre-wrap;
}

.summary-meta {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  font-size: 11px;
  color: #6c7086;
}

.empty {
  color: #6c7086;
  font-size: 13px;
}
</style>
