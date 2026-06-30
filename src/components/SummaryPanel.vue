<script setup lang="ts">
import { ref, watch } from "vue";
import {
  summarizeFile,
  getFileSummary,
  suggestName,
  getNameSuggestion,
  type AiInteraction,
} from "../composables/useLLM";

const props = defineProps<{
  janaId: string | null;
  filePath: string | null;
}>();

const summary = ref<AiInteraction | null>(null);
const isLoading = ref(false);
const error = ref<string | null>(null);

const nameSuggestion = ref<AiInteraction | null>(null);
const isNaming = ref(false);
const nameError = ref<string | null>(null);
const copied = ref(false);

async function loadExistingSummary(janaId: string) {
  try {
    summary.value = await getFileSummary(janaId);
  } catch (e) {
    summary.value = null;
  }
}

async function loadExistingName(janaId: string) {
  try {
    nameSuggestion.value = await getNameSuggestion(janaId);
  } catch (e) {
    nameSuggestion.value = null;
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

async function handleSuggestName() {
  if (!props.janaId || !props.filePath) return;

  isNaming.value = true;
  nameError.value = null;
  copied.value = false;

  try {
    nameSuggestion.value = await suggestName(props.janaId, props.filePath);
  } catch (e) {
    nameError.value = String(e);
  } finally {
    isNaming.value = false;
  }
}

async function copyName() {
  if (!nameSuggestion.value) return;
  try {
    await navigator.clipboard.writeText(nameSuggestion.value.response);
    copied.value = true;
    window.setTimeout(() => {
      copied.value = false;
    }, 1500);
  } catch (e) {
    nameError.value = "Couldn't copy to clipboard.";
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
    nameError.value = null;
    copied.value = false;
    if (newId) {
      loadExistingSummary(newId);
      loadExistingName(newId);
    } else {
      summary.value = null;
      nameSuggestion.value = null;
    }
  },
  { immediate: true }
);
</script>

<template>
  <div class="summary-panel">
    <div class="panel-header">
      <span class="panel-title">AI</span>
    </div>
    <div class="panel-content">
      <section class="ai-section">
        <div class="section-head">
          <span class="section-label">Suggested name</span>
          <button
            class="action-btn"
            :disabled="!janaId || isNaming"
            @click="handleSuggestName"
          >
            {{ isNaming ? "..." : "Suggest" }}
          </button>
        </div>
        <div v-if="isNaming" class="loading">Generating name...</div>
        <div v-else-if="nameError" class="error">{{ nameError }}</div>
        <div v-else-if="nameSuggestion" class="name-result">
          <code class="name-text">{{ nameSuggestion.response }}</code>
          <button class="copy-btn" @click="copyName">
            {{ copied ? "Copied" : "Copy" }}
          </button>
        </div>
        <div v-else class="empty">Suggest a file name from the content</div>
      </section>

      <section class="ai-section">
        <div class="section-head">
          <span class="section-label">Summary</span>
          <button
            class="action-btn"
            :disabled="!janaId || isLoading"
            @click="handleSummarize"
          >
            {{ isLoading ? "..." : "Summarize" }}
          </button>
        </div>
        <div v-if="isLoading" class="loading">Generating summary...</div>
        <div v-else-if="error" class="error">{{ error }}</div>
        <div v-else-if="summary" class="summary">
          <p class="summary-text">{{ summary.response }}</p>
          <div class="summary-meta">
            <span>{{ summary.model }}</span>
            <span>{{ formatDate(summary.created_at) }}</span>
          </div>
        </div>
        <div v-else class="empty">Open a file and click Summarize</div>
      </section>
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

.panel-content {
  flex: 1;
  padding: 16px;
  overflow-y: auto;
}

.ai-section + .ai-section {
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid #313244;
}

.section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
}

.section-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: #6c7086;
}

.action-btn {
  background: #89b4fa;
  color: #1e1e2e;
  border: none;
  padding: 4px 12px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.action-btn:hover:not(:disabled) {
  background: #b4d0fb;
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
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

.name-result {
  display: flex;
  align-items: center;
  gap: 8px;
}

.name-text {
  flex: 1;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 13px;
  color: #a6e3a1;
  background: #11111b;
  padding: 6px 8px;
  border-radius: 4px;
  word-break: break-all;
}

.copy-btn {
  background: transparent;
  color: #cdd6f4;
  border: 1px solid #45475a;
  padding: 4px 10px;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
  white-space: nowrap;
}

.copy-btn:hover {
  background: #313244;
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
