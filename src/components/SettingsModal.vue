<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import ModelCombobox from "./ModelCombobox.vue";
import {
  getSettings,
  saveSettings,
  setApiKey,
  hasApiKey,
  deleteApiKey,
  listModels,
  type AiProvider,
} from "../composables/useSettings";

const emit = defineEmits<{
  (e: "close"): void;
}>();

const PROVIDERS: { value: AiProvider; label: string }[] = [
  { value: "local", label: "Local (LM Studio)" },
  { value: "openai", label: "OpenAI" },
  { value: "anthropic", label: "Claude (Anthropic)" },
  { value: "gemini", label: "Gemini (Google)" },
];

// Curated default model lists so the dropdown isn't overwhelming. The field stays
// editable for anything else, and "Fetch all" pulls the live list per provider.
// Local has no sensible preset (depends on what's loaded in LM Studio).
const PROVIDER_MODELS: Record<AiProvider, string[]> = {
  local: [],
  openai: ["gpt-5.5", "gpt-5.4", "gpt-5.4-mini", "gpt-4o", "gpt-4o-mini"],
  anthropic: [
    "claude-opus-4-8",
    "claude-sonnet-4-6",
    "claude-haiku-4-5",
    "claude-fable-5",
    "claude-opus-4-7",
  ],
  gemini: [
    "gemini-2.5-pro",
    "gemini-2.5-flash",
    "gemini-2.0-flash",
    "gemini-2.0-flash-lite",
    "gemini-1.5-pro",
  ],
};

const provider = ref<AiProvider>("local");
const localUrl = ref("");
const localModel = ref("");
const openaiModel = ref("");
const anthropicModel = ref("");
const geminiModel = ref("");

// Whether a key is stored in the keychain, per cloud provider.
const keyStatus = ref<Record<string, boolean>>({});
// Transient key entry — never pre-filled (the backend won't return stored keys).
const apiKeyInput = ref("");

const modelOptions = ref<string[]>([]);
const comboRef = ref<InstanceType<typeof ModelCombobox> | null>(null);
const saving = ref(false);
const keySaving = ref(false);
const fetching = ref(false);
const notice = ref<string | null>(null);

const isCloud = computed(() => provider.value !== "local");

// Bind the model field for whichever cloud provider is active.
const cloudModel = computed<string>({
  get() {
    if (provider.value === "openai") return openaiModel.value;
    if (provider.value === "anthropic") return anthropicModel.value;
    return geminiModel.value;
  },
  set(v) {
    if (provider.value === "openai") openaiModel.value = v;
    else if (provider.value === "anthropic") anthropicModel.value = v;
    else geminiModel.value = v;
  },
});

const keyIsSet = computed(() => keyStatus.value[provider.value] === true);

onMounted(async () => {
  try {
    const s = await getSettings();
    provider.value = s.provider;
    localUrl.value = s.local_url;
    localModel.value = s.local_model;
    openaiModel.value = s.openai_model;
    anthropicModel.value = s.anthropic_model;
    geminiModel.value = s.gemini_model;
    for (const p of ["openai", "anthropic", "gemini"] as AiProvider[]) {
      keyStatus.value[p] = await hasApiKey(p);
    }
    modelOptions.value = [...PROVIDER_MODELS[provider.value]];
  } catch (e) {
    console.error("Failed to load settings:", e);
  }
});

// Switching providers resets the transient key field and shows that provider's
// curated model list.
function onProviderChange() {
  apiKeyInput.value = "";
  modelOptions.value = [...PROVIDER_MODELS[provider.value]];
  notice.value = null;
}

async function handleSaveKey() {
  if (!apiKeyInput.value.trim()) return;
  keySaving.value = true;
  notice.value = null;
  try {
    await setApiKey(provider.value, apiKeyInput.value.trim());
    keyStatus.value[provider.value] = true;
    apiKeyInput.value = "";
    notice.value = "Key saved to keychain.";
  } catch (e) {
    notice.value = `Failed to save key: ${e}`;
  } finally {
    keySaving.value = false;
  }
}

async function handleClearKey() {
  try {
    await deleteApiKey(provider.value);
    keyStatus.value[provider.value] = false;
    modelOptions.value = [];
    notice.value = "Key removed.";
  } catch (e) {
    notice.value = `Failed to remove key: ${e}`;
  }
}

async function handleFetchModels() {
  fetching.value = true;
  notice.value = null;
  try {
    modelOptions.value = await listModels(provider.value);
    if (modelOptions.value.length === 0) {
      notice.value = "No models returned.";
    } else {
      comboRef.value?.openList();
    }
  } catch (e) {
    notice.value = `Couldn't fetch models: ${e}`;
  } finally {
    fetching.value = false;
  }
}

async function handleSave() {
  saving.value = true;
  try {
    await saveSettings({
      provider: provider.value,
      local_url: localUrl.value,
      local_model: localModel.value,
      openai_model: openaiModel.value,
      anthropic_model: anthropicModel.value,
      gemini_model: geminiModel.value,
    });
    emit("close");
  } catch (e) {
    notice.value = `Failed to save settings: ${e}`;
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
        <label for="ai-provider">AI Provider</label>
        <select id="ai-provider" v-model="provider" @change="onProviderChange">
          <option v-for="p in PROVIDERS" :key="p.value" :value="p.value">
            {{ p.label }}
          </option>
        </select>
      </div>

      <!-- Local (LM Studio / OpenAI-compatible) -->
      <template v-if="provider === 'local'">
        <div class="form-group">
          <label for="llm-url">Endpoint URL</label>
          <input
            id="llm-url"
            v-model="localUrl"
            type="text"
            placeholder="http://192.168.77.1:1234/v1/chat/completions"
          />
        </div>
        <div class="form-group">
          <label for="local-model">Model</label>
          <div class="model-row">
            <ModelCombobox
              ref="comboRef"
              input-id="local-model"
              v-model="localModel"
              :options="modelOptions"
              placeholder="qwen3-vl-30b"
            />
            <button class="btn-secondary" :disabled="fetching" @click="handleFetchModels">
              {{ fetching ? "..." : "Fetch" }}
            </button>
          </div>
        </div>
      </template>

      <!-- Cloud providers: OpenAI / Claude / Gemini -->
      <template v-else>
        <div class="form-group">
          <label>API Key
            <span class="key-status" :class="{ set: keyIsSet }">
              {{ keyIsSet ? "✓ stored in keychain" : "not set" }}
            </span>
          </label>
          <div class="model-row">
            <input
              v-model="apiKeyInput"
              type="password"
              autocomplete="off"
              :placeholder="keyIsSet ? 'Enter a new key to replace' : 'Paste API key'"
            />
            <button
              class="btn-secondary"
              :disabled="keySaving || !apiKeyInput.trim()"
              @click="handleSaveKey"
            >
              {{ keySaving ? "..." : "Save Key" }}
            </button>
            <button v-if="keyIsSet" class="btn-danger" @click="handleClearKey">
              Clear
            </button>
          </div>
          <p class="hint">Stored locally in your OS keychain — never written to disk in plain text.</p>
        </div>
        <div class="form-group">
          <label :for="`${provider}-model`">Model</label>
          <div class="model-row">
            <ModelCombobox
              ref="comboRef"
              :input-id="`${provider}-model`"
              v-model="cloudModel"
              :options="modelOptions"
            />
            <button
              class="btn-secondary"
              :disabled="fetching || !keyIsSet"
              :title="keyIsSet ? 'List every model this key can access' : 'Save an API key first'"
              @click="handleFetchModels"
            >
              {{ fetching ? "..." : "Fetch all" }}
            </button>
          </div>
        </div>
      </template>

      <p v-if="notice" class="notice">{{ notice }}</p>

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
  width: 440px;
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
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  color: #a6adc8;
  margin-bottom: 4px;
}

.form-group input,
.form-group select {
  width: 100%;
  padding: 8px 10px;
  background: #181825;
  border: 1px solid #45475a;
  border-radius: 4px;
  color: #cdd6f4;
  font-size: 13px;
  box-sizing: border-box;
}

.form-group input:focus,
.form-group select:focus {
  outline: none;
  border-color: #89b4fa;
}

.model-row {
  display: flex;
  gap: 6px;
}

.model-row input {
  flex: 1;
}

.btn-secondary,
.btn-danger {
  padding: 0 12px;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
  border: 1px solid #45475a;
  background: #313244;
  color: #cdd6f4;
  white-space: nowrap;
}

.btn-secondary:hover:not(:disabled) {
  background: #45475a;
}

.btn-secondary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-danger {
  background: transparent;
  border-color: #f38ba8;
  color: #f38ba8;
}

.btn-danger:hover {
  background: rgba(243, 139, 168, 0.15);
}

.key-status {
  font-size: 11px;
  color: #6c7086;
  font-weight: normal;
}

.key-status.set {
  color: #a6e3a1;
}

.hint {
  font-size: 11px;
  color: #6c7086;
  margin: 4px 0 0 0;
}

.notice {
  font-size: 12px;
  color: #f9e2af;
  margin: 8px 0 0 0;
  word-break: break-word;
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
