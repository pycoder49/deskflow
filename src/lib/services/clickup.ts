import { invoke } from '@tauri-apps/api/core';

export interface TaskTag {
  name: string;
  tag_fg: string | null;
  tag_bg: string | null;
}

export interface Task {
  id: string;
  name: string;
  status: { status: string; color: string | null };
  priority: { id: string | null; priority: string | null; color: string | null } | null;
  time_estimate: number | null;
  url: string | null;
  due_date: string | null;
  tags: TaskTag[];
}

export interface NowNextResult {
  now: Task | null;
  next: Task[];
}

export const LISTS = [
  { id: '901414961997', label: 'Daily To-Do',     slug: null },
  { id: '901415158864', label: 'School',          slug: 'school' },
  { id: '901415158865', label: 'Learning',        slug: 'learning' },
  { id: '901415121407', label: 'Work & Career',   slug: 'work-career' },
  { id: '901415121408', label: 'Projects',        slug: 'projects' },
  { id: '901415121409', label: 'Health & Habits', slug: 'health-habits' },
] as const;

export type ListId = (typeof LISTS)[number]['id'];

// Areas = lists with a slug. Used for the "Area" dropdown when Daily To-Do is selected.
export const AREAS = LISTS.filter((l): l is typeof l & { slug: string } => l.slug !== null);

export const PRIORITY_META: Record<string, { label: string; color: string }> = {
  '1': { label: 'Urgent', color: '#ef4444' },
  '2': { label: 'High',   color: '#f97316' },
  '3': { label: 'Normal', color: '#3b82f6' },
  '4': { label: 'Low',    color: '#6b7280' },
};

export const getTodayTasks = () => invoke<Task[]>('get_today_tasks');
export const getNowNext    = (preserveNow: boolean) =>
  invoke<NowNextResult>('get_now_next', { preserveNow });
export const completeTask  = (taskId: string, taskName: string) =>
  invoke<void>('complete_task', { taskId, taskName });
export const uncheckTask   = (taskId: string, taskName: string) =>
  invoke<void>('uncheck_task', { taskId, taskName });
export const createTask    = (name: string, listId: string, priority: number, tags: string[]) =>
  invoke<Task>('create_task', { name, listId, priority, tags });
