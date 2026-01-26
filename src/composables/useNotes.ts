import { invoke } from "@tauri-apps/api/core";

export interface Note {
  id: string;
  title: string | null;
  content: string | null;
  created_at: number;
  updated_at: number;
}

export interface NoteListItem {
  id: string;
  title: string | null;
  updated_at: number;
}

export async function createNote(): Promise<Note> {
  return invoke<Note>("create_note");
}

export async function saveNote(
  id: string,
  title: string | null,
  content: string | null
): Promise<void> {
  return invoke("save_note", { id, title, content });
}

export async function getNote(id: string): Promise<Note> {
  return invoke<Note>("get_note", { id });
}

export async function listNotes(): Promise<NoteListItem[]> {
  return invoke<NoteListItem[]>("list_notes");
}

export async function deleteNote(id: string): Promise<void> {
  return invoke("delete_note", { id });
}
