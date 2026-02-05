import { invoke } from "@tauri-apps/api/core";

export interface AiInteraction {
  id: string;
  jana_id: string;
  interaction_type: string;
  prompt: string | null;
  response: string;
  model: string;
  created_at: number;
}

export async function summarizeFile(
  janaId: string,
  filePath: string
): Promise<AiInteraction> {
  return invoke<AiInteraction>("summarize_file", { janaId, filePath });
}

export async function getFileSummary(
  janaId: string
): Promise<AiInteraction | null> {
  return invoke<AiInteraction | null>("get_file_summary", { janaId });
}
