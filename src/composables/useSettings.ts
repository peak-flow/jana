import { invoke } from "@tauri-apps/api/core";

export type AiProvider = "local" | "openai" | "anthropic" | "gemini";

export interface Settings {
  provider: AiProvider;
  local_url: string;
  local_model: string;
  openai_model: string;
  anthropic_model: string;
  gemini_model: string;
}

export async function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export async function saveSettings(settings: Settings): Promise<void> {
  return invoke("save_settings", { settings });
}

// --- API keys (stored in the OS keychain; never returned to the frontend) ---

export async function setApiKey(provider: AiProvider, key: string): Promise<void> {
  return invoke("set_api_key", { provider, key });
}

export async function hasApiKey(provider: AiProvider): Promise<boolean> {
  return invoke<boolean>("has_api_key", { provider });
}

export async function deleteApiKey(provider: AiProvider): Promise<void> {
  return invoke("delete_api_key", { provider });
}

// --- Model discovery ---

export async function listModels(provider: AiProvider): Promise<string[]> {
  return invoke<string[]>("list_models", { provider });
}
