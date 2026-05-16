import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { ListEntry } from '$lib/services/clickup';

export interface AreaList {
  list_id: string;
  label: string;
  slug: string;
}

export interface AppConfig {
  clickup: {
    workspace_id: string;
    daily_list_id: string;
    areas: AreaList[];
  };
  calendar: {
    personal_email: string;
  };
  commands: {
    start_day_skill: string;
  };
  logging: {
    mode: 'clickup_doc' | 'local_file' | 'none';
    clickup_logs_folder_id: string;
    local_file_path: string;
  };
}

export const config = writable<AppConfig | null>(null);

export async function loadConfig() {
  const cfg = await invoke<AppConfig>('get_config');
  config.set(cfg);
}

// Derived list including the daily list as a special "no slug" entry plus
// all configured area lists. Drives the QuickCaptureModal dropdown.
export const lists = derived(config, ($cfg): ListEntry[] => {
  if (!$cfg) return [];
  const out: ListEntry[] = [];
  if ($cfg.clickup.daily_list_id) {
    out.push({ id: $cfg.clickup.daily_list_id, label: 'Daily To-Do', slug: null });
  }
  for (const a of $cfg.clickup.areas) {
    out.push({ id: a.list_id, label: a.label, slug: a.slug });
  }
  return out;
});

export const areas = derived(lists, ($lists) =>
  $lists.filter((l): l is ListEntry & { slug: string } => l.slug !== null)
);
