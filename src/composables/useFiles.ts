import { invoke } from "@tauri-apps/api/core";

export interface OpenFileResult {
  file_path: string;
  jana_id: string;
  content: string;
  file_name: string;
}

export interface TabResult {
  tab_id: string;
  window_id: string;
  file_path: string;
  jana_id: string;
  content: string;
  file_name: string;
}

export interface TabEntry {
  tab_id: string;
  window_id: string;
  file_path: string;
  jana_id: string;
  tab_order: number;
  cursor_line: number;
  cursor_col: number;
  scroll_top: number;
  last_opened: number;
}

export async function openFileDialog(windowId: string): Promise<TabResult | null> {
  return invoke<TabResult | null>("open_file_dialog", { windowId });
}

export async function addTab(windowId: string, filePath: string): Promise<TabResult> {
  return invoke<TabResult>("add_tab", { windowId, filePath });
}

export async function createNewFile(windowId: string): Promise<TabResult> {
  return invoke<TabResult>("create_new_file", { windowId });
}

export async function readFile(filePath: string): Promise<OpenFileResult> {
  return invoke<OpenFileResult>("read_file", { filePath });
}

export async function listTabs(windowId: string): Promise<TabEntry[]> {
  return invoke<TabEntry[]>("list_tabs", { windowId });
}

export async function closeTab(tabId: string): Promise<void> {
  return invoke("close_tab", { tabId });
}

export async function updateTabView(
  tabId: string,
  cursorLine: number,
  cursorCol: number,
  scrollTop: number
): Promise<void> {
  return invoke("update_tab_view", { tabId, cursorLine, cursorCol, scrollTop });
}

export async function saveFile(
  filePath: string,
  janaId: string,
  content: string
): Promise<void> {
  return invoke("save_file", { filePath, janaId, content });
}

export async function saveTempFileAs(
  tabId: string,
  filePath: string,
  janaId: string,
  content: string
): Promise<string | null> {
  return invoke<string | null>("save_temp_file_as", { tabId, filePath, janaId, content });
}

export async function forkFile(filePath: string): Promise<string> {
  return invoke<string>("fork_file", { filePath });
}

export async function clearAiHistory(janaId: string): Promise<void> {
  return invoke("clear_ai_history", { janaId });
}

export async function revealInFinder(filePath: string): Promise<void> {
  return invoke("reveal_in_finder", { filePath });
}
