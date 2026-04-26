import { writable, get } from 'svelte/store';
import { getTodayTasks, type Task } from '$lib/services/clickup';

// Bumped when both widgets should refetch — task creation, external poll, or a
// completion that affects NowNext's current pick. Bumping triggers a NowNext
// re-pick (preserve-Now path), which costs an AI call, so callers should only
// bump when warranted. For Today-only mutations (toggling a task that isn't in
// NowNext display), call `load()` locally instead.
// IMPORTANT: bump only AFTER the API call succeeds — bumping during the optimistic
// mark/unmark races against stale server state.
export const clickupVersion = writable(0);
export const bumpClickup = () => clickupVersion.update(n => n + 1);

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
// Cleared when either widget is manually refreshed.
export const completedTasks = writable<Map<string, Task>>(new Map());

export const markCompleted = (task: Task) =>
  completedTasks.update((m) => {
    const next = new Map(m);
    next.set(task.id, task);
    return next;
  });

export const unmarkCompleted = (id: string) =>
  completedTasks.update((m) => {
    const next = new Map(m);
    next.delete(id);
    return next;
  });

export const clearCompleted = () => completedTasks.set(new Map());

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
