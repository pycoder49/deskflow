import { invoke } from '@tauri-apps/api/core';

export interface VaultCounts {
  sources: number;
  topics: number;
  entities: number;
  queries: number;
  study_guides: number;
}

export interface InboxItem {
  path: string;     // raw/notes/foo.md
  name: string;
  modified: string; // YYYY-MM-DD
}

export interface LogDay {
  date: string;
  bullets: string[];
}

export interface HotSection {
  title: string;
  body: string;
}

export interface VaultPulse {
  vault_name: string;
  hot_sections: HotSection[];
  hot_updated: string;
  hot_session: string;
  counts: VaultCounts;
  inbox: InboxItem[];
  recent_log: LogDay[];
}

export const getVaultPulse = () => invoke<VaultPulse>('get_vault_pulse');

// Build an obsidian:// URI to open a vault-relative file in Obsidian.
// Path must be vault-relative (e.g. "wiki/hot.md", "raw/notes/foo.md").
export function obsidianUri(vault: string, path: string): string {
  const params = new URLSearchParams({ vault, file: path });
  return `obsidian://open?${params.toString()}`;
}
