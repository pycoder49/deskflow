<script lang="ts">
  import {
    getNowNext,
    completeTask,
    PRIORITY_META,
    type Task,
    type NowNextResult,
  } from '$lib/services/clickup';
  import {
    clickupVersion,
    completedTasks,
    markCompleted,
    unmarkCompleted,
    clearCompleted,
    bumpClickup,
    nowNextIds,
  } from '$lib/stores/refresh';
  import { openUrl } from '@tauri-apps/plugin-opener';

  let result = $state<NowNextResult>({ now: null, next: [] });
  let loading = $state(true);
  let error = $state<string | null>(null);
  let completing = $state<string | null>(null);

  async function load(preserveNow = true) {
    loading = true;
    error = null;
    try {
      result = await getNowNext(preserveNow);
      const ids = new Set<string>();
      if (result.now) ids.add(result.now.id);
      for (const t of result.next) ids.add(t.id);
      nowNextIds.set(ids);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function manualRefresh() {
    clearCompleted();
    load(false);
  }

  async function done(task: Task) {
    if (completing) return;
    completing = task.id;
    markCompleted(task);
    try {
      await completeTask(task.id, task.name);
      // Task was in NowNext display by definition — re-pick.
      bumpClickup();
    } catch (e) {
      unmarkCompleted(task.id);
      console.error('complete failed:', e);
    } finally {
      completing = null;
    }
  }

  function priorityColor(task: Task): string {
    const id = task.priority?.id ?? null;
    return (id && PRIORITY_META[id]?.color) ?? '#9ca3af';
  }

  function estimateLabel(task: Task): string | null {
    if (!task.time_estimate) return null;
    return `${Math.round(task.time_estimate / 60000)}m`;
  }

  // Hide tasks marked complete in the overlay.
  const now = $derived(
    result.now && !$completedTasks.has(result.now.id) ? result.now : null
  );
  const next = $derived(result.next.filter((t) => !$completedTasks.has(t.id)));

  $effect(() => {
    $clickupVersion;
    load(true);
  });
</script>

<div class="flex flex-col h-full">
  <div class="flex items-center justify-between mb-4">
    <h2 class="text-xs uppercase tracking-wider text-mute">Now / Next</h2>
    <button
      class="text-xs text-mute hover:text-ink transition"
      onclick={manualRefresh}
      title="Refresh"
      aria-label="Refresh focus"
    >↻</button>
  </div>

  {#if loading}
    <div class="space-y-3">
      <div class="h-14 bg-line rounded-lg animate-pulse"></div>
      <div class="h-8 bg-line rounded animate-pulse"></div>
      <div class="h-8 bg-line rounded animate-pulse opacity-60"></div>
    </div>

  {:else if error}
    <p class="text-xs text-red-400 mt-2">Failed: {error}</p>

  {:else if !now && next.length === 0}
    <p class="text-sm text-mute italic mt-2">Nothing to focus on — you're clear.</p>

  {:else}
    {#if now}
      {@const task = now}
      <div class="mb-3">
        <p class="text-[10px] uppercase tracking-widest text-accent mb-1">Now</p>
        <div class="border border-accent/30 bg-accent/5 rounded-lg p-2.5 flex items-start gap-2">
          <button
            class="mt-0.5 w-4 h-4 rounded border border-accent/50 flex-shrink-0 hover:bg-accent/20 transition flex items-center justify-center"
            class:opacity-50={completing === task.id}
            onclick={() => done(task)}
            disabled={!!completing}
            title="Mark complete"
          >
            {#if completing === task.id}
              <span
                class="w-2 h-2 border border-accent border-t-transparent rounded-full animate-spin block"
              ></span>
            {/if}
          </button>
          <div class="flex items-center gap-2 flex-1 min-w-0">
            <span class="w-2 h-2 rounded-full flex-shrink-0" style="background:{priorityColor(task)}"></span>
            <button
              class="text-sm font-semibold text-ink text-left flex-1 min-w-0 truncate hover:text-accent transition"
              onclick={() => task.url && openUrl(task.url)}
              title={task.name}
            >{task.name}</button>
            {#if estimateLabel(task)}
              <span class="text-xs text-mute flex-shrink-0">{estimateLabel(task)}</span>
            {/if}
          </div>
        </div>
      </div>
    {/if}

    {#if next.length > 0}
      <p class="text-[10px] uppercase tracking-widest text-mute mb-1.5">Next</p>
      <ul class="space-y-2.5">
        {#each next as task (task.id)}
          <li class="flex items-center gap-2">
            <button
              class="w-3.5 h-3.5 rounded border border-line flex-shrink-0 hover:border-accent transition flex items-center justify-center"
              class:opacity-50={completing === task.id}
              onclick={() => done(task)}
              disabled={!!completing}
              title="Mark complete"
            >
              {#if completing === task.id}
                <span
                  class="w-1.5 h-1.5 border border-accent border-t-transparent rounded-full animate-spin block"
                ></span>
              {/if}
            </button>
            <span
              class="w-1.5 h-1.5 rounded-full flex-shrink-0"
              style="background:{priorityColor(task)}"
            ></span>
            <button
              class="flex-1 text-sm font-medium text-left text-ink truncate hover:text-accent transition"
              onclick={() => task.url && openUrl(task.url)}
              title={task.name}
            >{task.name}</button>
            {#if estimateLabel(task)}
              <span class="text-xs text-mute flex-shrink-0">{estimateLabel(task)}</span>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
</div>
