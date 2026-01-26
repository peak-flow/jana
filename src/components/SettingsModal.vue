<script setup lang="ts">
import { ref, onMounted } from "vue";
import { getSettings, saveSettings } from "../composables/useSettings";

const emit = defineEmits<{
  (e: "close"): void;
}>();

const llmUrl = ref("");
const llmModel = ref("");
const saving = ref(false);

onMounted(async () => {
  try {
    const settings = await getSettings();
    llmUrl.value = settings.llm_url;
    llmModel.value = settings.llm_model;
  } catch (e) {
    console.error("Failed to load settings:", e);
  }
});

async function handleSave() {
  saving.value = true;
  try {
    await saveSettings({
      llm_url: llmUrl.value,
      llm_model: llmModel.value,
    });
    emit("close");
  } catch (e) {
    console.error("Failed to save settings:", e);
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal">
      <h2 class="modal-title">Settings</h2>
      <div class="form-group">
        <label for="llm-url">LLM URL</label>
        <input
          id="llm-url"
          v-model="llmUrl"
          type="text"
          placeholder="http://192.168.77.1:1234/v1/chat/completions"
        />
      </div>
      <div class="form-group">
        <label for="llm-model">Model Name</label>
        <input
          id="llm-model"
          v-model="llmModel"
          type="text"
          placeholder="qwen3-vl-30b"
        />
      </div>
      <div class="modal-actions">
        <button class="btn-cancel" @click="emit('close')">Cancel</button>
        <button class="btn-save" :disabled="saving" @click="handleSave">
          {{ saving ? "Saving..." : "Save" }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal {
  background: #1e1e2e;
  border: 1px solid #313244;
  border-radius: 8px;
  padding: 24px;
  width: 400px;
  max-width: 90vw;
}

.modal-title {
  font-size: 16px;
  font-weight: 600;
  color: #cdd6f4;
  margin: 0 0 16px 0;
}

.form-group {
  margin-bottom: 12px;
}

.form-group label {
  display: block;
  font-size: 12px;
  color: #a6adc8;
  margin-bottom: 4px;
}

.form-group input {
  width: 100%;
  padding: 8px 10px;
  background: #181825;
  border: 1px solid #45475a;
  border-radius: 4px;
  color: #cdd6f4;
  font-size: 13px;
  box-sizing: border-box;
}

.form-group input:focus {
  outline: none;
  border-color: #89b4fa;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 20px;
}

.btn-cancel,
.btn-save {
  padding: 6px 14px;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  border: none;
}

.btn-cancel {
  background: #313244;
  color: #cdd6f4;
}

.btn-cancel:hover {
  background: #45475a;
}

.btn-save {
  background: #89b4fa;
  color: #1e1e2e;
  font-weight: 500;
}

.btn-save:hover {
  background: #74c7ec;
}

.btn-save:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
