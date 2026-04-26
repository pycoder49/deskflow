<script lang="ts">
  import {
    getTodayTasks,
    completeTask,
    uncheckTask,
    PRIORITY_META,
    type Task,
  } from '$lib/services/clickup';
  import {
    clickupVersion,
    completedTasks,
    markCompleted,
    unmarkCompleted,
    clearCompleted,
    bumpClickup,
    nowNextIds,
    todayIds,
  } from '$lib/stores/refresh';
  import { openUrl } from '@tauri-apps/plugin-opener';

  let tasks = $state<Task[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let pending = $state<Set<string>>(new Set());

  async function load() {
    loading = true;
    error = null;
    try {
      tasks = await getTodayTasks();
      todayIds.set(new Set(tasks.map((t) => t.id)));
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function manualRefresh() {
    clearCompleted();
    load();
  }

  async function toggle(task: Task) {
    if (pending.has(task.id)) return;
    pending = new Set([...pending, task.id]);
    const wasDone = $completedTasks.has(task.id);
    const inNowNext = $nowNextIds.has(task.id);

    if (wasDone) {
      unmarkCompleted(task.id);
      try {
        await uncheckTask(task.id, task.name);
        // If task was in NowNext display (rare for uncheck — would mean
        // we just completed → nowNext re-picked → unchecked again before
        // re-pick finished — keep symmetric). Else local refetch only.
        if (inNowNext) bumpClickup();
        else await load();
      } catch (e) {
        markCompleted(task);
        console.error('uncheck failed:', e);
      }
    } else {
      markCompleted(task);
      try {
        await completeTask(task.id, task.name);
        if (inNowNext) bumpClickup();
        else await load();
      } catch (e) {
        unmarkCompleted(task.id);
        console.error('complete failed:', e);
      }
    }

    pending = new Set([...pending].filter((id) => id !== task.id));
  }

  function priorityColor(task: Task): string {
    const id = task.priority?.id ?? null;
    return (id && PRIORITY_META[id]?.color) ?? '#9ca3af';
  }

  // Pick black or white text for a hex bg based on perceived luminance (YIQ).
  // Threshold 140 leans slightly toward white for borderline mid-bright colors.
  function readableText(bg: string): string {
    const hex = bg.replace('#', '');
    if (hex.length !== 6) return '#ffffff';
    const r = parseInt(hex.slice(0, 2), 16);
    const g = parseInt(hex.slice(2, 4), 16);
    const b = parseInt(hex.slice(4, 6), 16);
    const yiq = (r * 299 + g * 587 + b * 114) / 1000;
    return yiq > 140 ? '#0a0a0a' : '#ffffff';
  }

  // Merge: server tasks + overlay tasks not in server response.
  // Overlay keeps just-completed tasks visible (strikethrough) after refetch.
  const display = $derived.by(() => {
    const ids = new Set(tasks.map((t) => t.id));
    const overlay = [...$completedTasks.values()].filter((t) => !ids.has(t.id));
    return [...tasks, ...overlay];
  });

  const remaining = $derived(
    display.filter((t) => !$completedTasks.has(t.id)).length
  );

  $effect(() => {
    $clickupVersion;
    load();
  });
</script>

<div class="flex flex-col h-full">
  <div class="flex items-center justify-between mb-3">
    <h2 class="text-xs uppercase tracking-wider text-mute">Today</h2>
    <button
      class="text-xs text-mute hover:text-ink transition"
      onclick={manualRefresh}
      title="Refresh"
      aria-label="Refresh tasks"
    >↻</button>
  </div>

  {#if loading}
    <div class="space-y-2">
      {#each [1, 2, 3] as _}
        <div class="h-8 bg-line rounded animate-pulse"></div>
      {/each}
    </div>

  {:else if error}
    <p class="text-xs text-red-400 mt-2">Failed to load: {error}</p>

  {:else if display.length === 0}
    <p class="text-sm text-mute italic mt-2">All clear — nothing left today.</p>

  {:else}
    <ul class="space-y-1.5 overflow-y-auto flex-1">
      {#each display as task (task.id)}
        {@const done_ = $completedTasks.has(task.id)}
        <li class="flex items-center gap-2 group">
          <button
            class="w-4 h-4 rounded border flex-shrink-0 transition flex items-center justify-center
                   {done_ ? 'bg-green-500 border-green-500 hover:bg-green-600' : 'border-line hover:border-accent'}"
            onclick={() => toggle(task)}
            disabled={pending.has(task.id)}
            title={done_ ? 'Uncheck' : 'Mark complete'}
            aria-label="{done_ ? 'Uncheck' : 'Mark complete'} {task.name}"
          >
            {#if done_}
              <svg
                class="w-2.5 h-2.5 text-white"
                fill="none"
                viewBox="0 0 10 8"
                stroke="currentColor"
                stroke-width="2"
              >
                <path d="M1 4l3 3 5-6" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            {/if}
          </button>

          <span
            class="w-1.5 h-1.5 rounded-full flex-shrink-0"
            style="background:{priorityColor(task)}"
          ></span>

          <button
            class="flex-1 min-w-0 text-sm text-left truncate transition
                   {done_ ? 'line-through text-mute' : 'text-ink hover:text-accent'}"
            onclick={() => task.url && openUrl(task.url)}
            title={task.name}
          >
            {task.name}
          </button>

          {#if task.tags.length > 0}
            <div
              class="tag-strip flex items-center gap-1 max-w-[8rem] overflow-x-auto flex-shrink-0
                     {done_ ? 'opacity-30 saturate-[.35]' : ''}"
            >
              {#each task.tags as tag}
                {@const bg = tag.tag_bg ?? '#6b7280'}
                <span
                  class="px-1.5 py-0.5 text-[11px] rounded font-medium whitespace-nowrap"
                  style="background:{bg};color:{readableText(bg)}"
                  title={tag.name}
                >{tag.name}</span>
              {/each}
            </div>
          {/if}

          {#if task.time_estimate}
            <span class="text-xs text-mute flex-shrink-0 {done_ ? 'opacity-30' : ''}">
              {Math.round(task.time_estimate / 60000)}m
            </span>
          {/if}
        </li>
      {/each}
    </ul>
    <p class="text-xs text-mute mt-3">
      {remaining > 0 ? `${remaining} remaining` : 'All done!'}
    </p>
  {/if}
</div>

<style>
  .tag-strip {
    scrollbar-width: thin;
    scrollbar-color: rgba(127, 127, 127, 0.35) transparent;
  }
  .tag-strip::-webkit-scrollbar {
    height: 3px;
  }
  .tag-strip::-webkit-scrollbar-thumb {
    background: rgba(127, 127, 127, 0.35);
    border-radius: 2px;
  }
  .tag-strip::-webkit-scrollbar-track {
    background: transparent;
  }
</style>
