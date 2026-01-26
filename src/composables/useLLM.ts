import { invoke } from "@tauri-apps/api/core";

export interface AiSummary {
  id: string;
  note_id: string;
  summary: string;
  model: string;
  created_at: number;
}

export async function summarizeNote(noteId: string): Promise<AiSummary> {
  return invoke<AiSummary>("summarize_note", { noteId });
}

export async function getSummary(noteId: string): Promise<AiSummary | null> {
  return invoke<AiSummary | null>("get_summary", { noteId });
}
