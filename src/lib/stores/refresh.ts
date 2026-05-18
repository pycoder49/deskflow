import { writable, get } from 'svelte/store';
import { getTodayTasks, type Task } from '$lib/services/clickup';

// Shared 4am-cutoff logical date. Mirrors logical_today() in scripts/log_action.py — update both if the rule changes.
export function logicalToday(): string {
  const shifted = new Date(Date.now() - 4 * 60 * 60 * 1000);
  const y = shifted.getFullYear();
  const m = String(shifted.getMonth() + 1).padStart(2, '0');
  const d = String(shifted.getDate()).padStart(2, '0');
  return `${y}-${m}-${d}`;
}

// Bumped when both widgets should refetch — task creation, external poll, or a
// completion that affects NowNext's current pick. Bumping triggers a NowNext
// re-pick (preserve-Now path), which costs an AI call, so callers should only
// bump when warranted. For Today-only mutations (toggling a task that isn't in
// NowNext display), call `load()` locally instead.
// IMPORTANT: bump only AFTER the API call succeeds — bumping during the optimistic
// mark/unmark races against stale server state.
export const clickupVersion = writable(0);
export const bumpClickup = () => clickupVersion.update(n => n + 1);

// Bumped on every successful complete/uncheck — drives TaskStats real-time count
// without triggering a NowNext re-pick (which clickupVersion does).
export const completionVersion = writable(0);
export const bumpCompletion = () => completionVersion.update(n => n + 1);

// When a task is created in Daily To-Do, inject it directly into Today without
// a server reload. Set suppressNextTodayLoad before bumpClickup so Today's
// clickupVersion effect skips the load while NowNext still re-picks.
export const pendingNewTask = writable<Task | null>(null);
export const pushNewTask = (task: Task) => pendingNewTask.set(task);
export const clearNewTask = () => pendingNewTask.set(null);
export const suppressNextTodayLoad = writable(false);

// Set of task IDs currently rendered in NowNext (Now + Next). NowNext writes
// to this after each successful load. TodayTasks reads it to decide whether
// a check/uncheck should trigger a NowNext re-pick or stay local.
export const nowNextIds = writable<Set<string>>(new Set());

// Set of task IDs currently in Today's `tasks` array (server-truth, pre-overlay).
// TodayTasks writes after every successful load. The poll compares its fetch
// against this to decide whether anything changed externally — keeps in-app
// mutations from triggering false positives.
export const todayIds = writable<Set<string>>(new Set());

// Locally completed tasks (snapshot at completion). Display overlay:
// Today renders these with strikethrough even after server refetch excludes them;
// NowNext hides any task whose id is in this map.
// Persisted to localStorage keyed by logical date; cleared on new day or manual refresh.
const COMPLETED_KEY = 'aryan-dashboard-completed';

function loadPersistedCompleted(): Map<string, Task> {
  if (typeof localStorage === 'undefined') return new Map();
  try {
    const raw = localStorage.getItem(COMPLETED_KEY);
    if (!raw) return new Map();
    const parsed = JSON.parse(raw) as { date: string; tasks: [string, Task][] };
    if (parsed.date !== logicalToday()) return new Map();
    return new Map(parsed.tasks);
  } catch {
    return new Map();
  }
}

function saveCompleted(m: Map<string, Task>): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(COMPLETED_KEY, JSON.stringify({
      date: logicalToday(),
      tasks: [...m.entries()],
    }));
  } catch {}
}

export const completedTasks = writable<Map<string, Task>>(loadPersistedCompleted());

export const markCompleted = (task: Task) =>
  completedTasks.update((m) => {
    const next = new Map(m);
    next.set(task.id, task);
    saveCompleted(next);
    return next;
  });

export const unmarkCompleted = (id: string) =>
  completedTasks.update((m) => {
    const next = new Map(m);
    next.delete(id);
    saveCompleted(next);
    return next;
  });

export const clearCompleted = () => {
  completedTasks.set(new Map());
  if (typeof localStorage !== 'undefined') {
    try { localStorage.removeItem(COMPLETED_KEY); } catch {}
  }
};

// ─── External-mutation poll ─────────────────────────────────────────────────
// Polls Daily To-Do every 30s while the window is visible; bumps clickupVersion
// only when the task ID set has changed since the last successful in-app load
// (`todayIds`). In-app toggles refresh `todayIds` locally, so they don't trigger
// false-positive bumps. Pauses on `document.hidden`, resumes on visible.

const POLL_INTERVAL_MS = 30_000;

function setsEqual(a: Set<string>, b: Set<string>): boolean {
  if (a.size !== b.size) return false;
  for (const id of a) if (!b.has(id)) return false;
  return true;
}

// Fires once ~2.5s after mount; each widget retries if it has an error.
export const startupRetry = writable(0);
export const triggerStartupRetry = () => startupRetry.update(n => n + 1);

export function startTaskPolling(): () => void {
  let timer: ReturnType<typeof setInterval> | null = null;

  async function tick() {
    try {
      const tasks = await getTodayTasks();
      const fresh = new Set(tasks.map((t) => t.id));
      if (!setsEqual(fresh, get(todayIds))) {
        bumpClickup();
      }
    } catch (e) {
      console.error('[poll] getTodayTasks failed:', e);
    }
  }

  function start() {
    if (timer !== null) return;
    timer = setInterval(tick, POLL_INTERVAL_MS);
  }
  function stop() {
    if (timer === null) return;
    clearInterval(timer);
    timer = null;
  }
  function onVisibility() {
    if (document.hidden) stop();
    else start();
  }

  if (!document.hidden) start();
  document.addEventListener('visibilitychange', onVisibility);

  return () => {
    document.removeEventListener('visibilitychange', onVisibility);
    stop();
  };
}
