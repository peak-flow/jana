import { invoke } from "@tauri-apps/api/core";

export interface OpenFileResult {
  file_path: string;
  jana_id: string;
  content: string;
  file_name: string;
}

export interface OpenFileEntry {
  file_path: string;
  jana_id: string;
  tab_order: number;
  cursor_line: number;
  cursor_col: number;
  last_opened: number;
}

export async function openFileDialog(): Promise<OpenFileResult | null> {
  return invoke<OpenFileResult | null>("open_file_dialog");
}

export async function readFile(filePath: string): Promise<OpenFileResult> {
  return invoke<OpenFileResult>("read_file", { filePath });
}

export async function saveFile(
  filePath: string,
  janaId: string,
  content: string
): Promise<void> {
  return invoke("save_file", { filePath, janaId, content });
}

export async function saveFileAs(
  janaId: string,
  content: string
): Promise<string | null> {
  return invoke<string | null>("save_file_as", { janaId, content });
}

export async function closeFile(filePath: string): Promise<void> {
  return invoke("close_file", { filePath });
}

export async function listOpenFiles(): Promise<OpenFileEntry[]> {
  return invoke<OpenFileEntry[]>("list_open_files");
}

export async function updateCursorPosition(
  filePath: string,
  cursorLine: number,
  cursorCol: number
): Promise<void> {
  return invoke("update_cursor_position", { filePath, cursorLine, cursorCol });
}

export async function forkFile(filePath: string): Promise<string> {
  return invoke<string>("fork_file", { filePath });
}

export async function clearAiHistory(janaId: string): Promise<void> {
  return invoke("clear_ai_history", { janaId });
}
