import { invoke } from '@tauri-apps/api/core';

export async function getNotepad(): Promise<string> {
  return invoke<string>('get_notepad');
}

export async function saveNotepad(content: string): Promise<void> {
  return invoke<void>('save_notepad', { content });
}
