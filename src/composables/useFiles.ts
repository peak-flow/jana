import { invoke } from "@tauri-apps/api/core";

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

export interface AcquireResult {
  buffer_id: string;
  jana_id: string;
  content: string;
  version: number;
  file_name: string;
}

export interface BufferSnapshot {
  jana_id: string;
  content: string;
  version: number;
}

export interface UpdateResult {
  version: number;
  conflict: boolean;
  content: string | null;
}

export interface SaveAsResult {
  file_path: string;
  buffer_id: string;
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

/// Open a brand-new editor window with its own session. Returns the new window_id.
export async function createAppWindow(): Promise<string> {
  return invoke<string>("create_app_window");
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

export async function saveTempFileAs(
  tabId: string
): Promise<SaveAsResult | null> {
  return invoke<SaveAsResult | null>("save_temp_file_as", { tabId });
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

// --- Buffer registry (backend is the content authority + single disk writer) ---

export async function acquireBuffer(filePath: string): Promise<AcquireResult> {
  return invoke<AcquireResult>("acquire_buffer", { filePath });
}

export async function getBuffer(bufferId: string): Promise<BufferSnapshot> {
  return invoke<BufferSnapshot>("get_buffer", { bufferId });
}

export async function updateBuffer(
  bufferId: string,
  content: string,
  changes: unknown,
  baseVersion: number,
  originWindowId: string
): Promise<UpdateResult> {
  return invoke<UpdateResult>("update_buffer", {
    bufferId,
    content,
    changes,
    baseVersion,
    originWindowId,
  });
}

export async function flushBuffer(bufferId: string): Promise<void> {
  return invoke("flush_buffer", { bufferId });
}

export async function releaseBuffer(bufferId: string): Promise<void> {
  return invoke("release_buffer", { bufferId });
}
