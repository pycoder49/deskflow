<script lang="ts">
  import {
    getTodayTasks,
    getCompletedTodayTasks,
    completeTask,
    uncheckTask,
    deleteTask,
    updateTask,
    PRIORITY_META,
    type Task,
  } from '$lib/services/clickup';
  import { untrack } from 'svelte';
  import {
    clickupVersion,
    completedTasks,
    markCompleted,
    unmarkCompleted,
    clearCompleted,
    bumpClickup,
    bumpCompletion,
    nowNextIds,
    todayIds,
    pendingNewTask,
    clearNewTask,
    suppressNextTodayLoad,
    startupRetry,
  } from '$lib/stores/refresh';
  import { openUrl } from '@tauri-apps/plugin-opener';

  let tasks = $state<Task[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let pending = $state<Set<string>>(new Set());

  // Edit modal state
  let editTask = $state<Task | null>(null);
  let editName = $state('');
  let editPriority = $state<number | null>(null);
  let editDueDateStr = $state('');     // 'YYYY-MM-DD' or ''
  let editTimeEstimateMin = $state(''); // numeric string or ''
  let editSaving = $state(false);

  // Tag expansion: task ID whose overflow tags are shown
  let expandedTagTask = $state<string | null>(null);

  // Dynamic tag overflow: measure list container width + per-task name width
  let listEl = $state<HTMLUListElement | null>(null);
  let containerWidth = $state(0);

  $effect(() => {
    if (!listEl) return;
    containerWidth = listEl.clientWidth;
    const ro = new ResizeObserver(([entry]) => {
      containerWidth = entry.contentRect.width;
    });
    ro.observe(listEl);
    return () => ro.disconnect();
  });

  // Canvas text measurement — gives natural (unwrapped) text width per task name
  let _cvs: HTMLCanvasElement | null = null;
  let _ctx: CanvasRenderingContext2D | null = null;
  function measureTextWidth(text: string): number {
    if (!_cvs) { _cvs = document.createElement('canvas'); _ctx = _cvs.getContext('2d'); }
    if (!_ctx) return text.length * 7;
    _ctx.font = '14px ui-sans-serif, system-ui, -apple-system, sans-serif';
    return _ctx.measureText(text).width;
  }

  // Per-task: how many tag pills fit without pushing the task name.
  // First tag always shown (guarantee ≥1 if tags exist).
  // Subsequent tags only added if they + a potential "+N" badge still fit.
  function visibleTagCount(taskName: string, tags: { name: string }[]): number {
    if (tags.length === 0) return 0;
    if (containerWidth === 0) return tags.length; // show all until measured
    // Fixed overhead: checkbox(16) + dot(6) + three gap-2(24) + buffer(10)
    const overhead = 56;
    const nameWidth = measureTextWidth(taskName);
    const budget = containerWidth - overhead - nameWidth;
    // If name already fills the row, still guarantee 1 tag
    if (budget <= 0) return 1;
    let used = 0;
    let count = 0;
    for (let i = 0; i < tags.length; i++) {
      // tag pill: 11px font ≈ 0.786× canvas-14px measurement, plus 12px padding + 4px gap
      const tagW = measureTextWidth(tags[i].name) * 0.786 + 16;
      const hasMore = i + 1 < tags.length;
      const plusW = hasMore ? 22 : 0; // "+N" badge width
      // First tag always fits (enforces ≥1 guarantee)
      if (count > 0 && used + tagW + plusW > budget) break;
      used += tagW;
      count++;
    }
    return count;
  }
  // Right-click context menu
  let contextMenu = $state<{ task: Task; x: number; y: number } | null>(null);
  let menuDeleteConfirm = $state(false);

  function openContextMenu(e: MouseEvent, task: Task) {
    if ($completedTasks.has(task.id)) return;
    e.preventDefault();
    contextMenu = { task, x: e.clientX, y: e.clientY };
    menuDeleteConfirm = false;
  }

  function closeContextMenu() {
    contextMenu = null;
    menuDeleteConfirm = false;
  }

  async function menuDelete() {
    if (!contextMenu) return;
    if (!menuDeleteConfirm) { menuDeleteConfirm = true; return; }
    const task = contextMenu.task;
    closeContextMenu();
    pending = new Set([...pending, task.id]);
    try {
      await deleteTask(task.id, task.name, task.tags);
      await load();
    } catch (e) {
      console.error('delete failed:', e);
    }
    pending = new Set([...pending].filter((id) => id !== task.id));
  }

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

  async function manualRefresh() {
    clearCompleted();
    await load();
    try {
      const done = await getCompletedTodayTasks();
      for (const task of done) markCompleted(task);
    } catch (e) {
      console.error('failed to fetch completed tasks:', e);
    }
  }

  async function toggle(task: Task) {
    if (pending.has(task.id)) return;
    pending = new Set([...pending, task.id]);
    const wasDone = $completedTasks.has(task.id);
    const inNowNext = $nowNextIds.has(task.id);

    if (wasDone) {
      unmarkCompleted(task.id);
      try {
        await uncheckTask(task.id, task.name, task.tags);
        bumpCompletion();
        if (inNowNext) bumpClickup();
        // Only reload if task was already evicted from `tasks` by a prior server sync;
        // otherwise it pops back to the top via the sort in `display`.
        else if (!tasks.find((t) => t.id === task.id)) await load();
      } catch (e) {
        markCompleted(task);
        console.error('uncheck failed:', e);
      }
    } else {
      markCompleted(task);
      try {
        await completeTask(task.id, task.name, task.tags);
        bumpCompletion();
        if (inNowNext) bumpClickup();
        // No reload — task sinks to bottom instantly via display sort.
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
  // Completed tasks always sorted to the bottom regardless of origin.
  const display = $derived.by(() => {
    const ids = new Set(tasks.map((t) => t.id));
    const overlay = [...$completedTasks.values()].filter((t) => !ids.has(t.id));
    const all = [...tasks, ...overlay];
    return [
      ...all.filter((t) => !$completedTasks.has(t.id)),
      ...all.filter((t) => $completedTasks.has(t.id)),
    ];
  });

  const remaining = $derived(
    display.filter((t) => !$completedTasks.has(t.id)).length
  );

  function taskDueDateStr(task: Task): string {
    if (!task.due_date) return '';
    const ms = parseInt(task.due_date);
    if (isNaN(ms)) return '';
    const d = new Date(ms);
    // en-CA gives YYYY-MM-DD, uses local time (not UTC)
    return d.toLocaleDateString('en-CA');
  }

  function taskTimeEstimateMin(task: Task): string {
    if (!task.time_estimate || task.time_estimate <= 0) return '';
    return String(Math.round(task.time_estimate / 60000));
  }

  function openEdit(task: Task) {
    editTask = task;
    editName = task.name;
    editPriority = task.priority?.id ? Number(task.priority.id) : null;
    editDueDateStr = taskDueDateStr(task);
    editTimeEstimateMin = taskTimeEstimateMin(task);
  }

  function closeEdit() {
    editTask = null;
    editName = '';
    editPriority = null;
    editDueDateStr = '';
    editTimeEstimateMin = '';
    editSaving = false;
  }

  async function saveEdit() {
    if (!editTask) return;
    editSaving = true;
    const task = editTask;
    const nameChanged = editName.trim() !== task.name;
    const priorityChanged = editPriority !== (task.priority?.id ? Number(task.priority.id) : null);
    const origDueStr = taskDueDateStr(task);
    const dueDateChanged = editDueDateStr !== origDueStr;
    const origEstMin = taskTimeEstimateMin(task);
    const timeChanged = editTimeEstimateMin !== origEstMin;

    if (!nameChanged && !priorityChanged && !dueDateChanged && !timeChanged) {
      closeEdit();
      return;
    }

    const parts: string[] = [];
    if (nameChanged) parts.push(`name: "${task.name}" → "${editName.trim()}"`);
    if (priorityChanged) {
      const oldLabel = task.priority?.id ? (PRIORITY_META[task.priority.id]?.label ?? task.priority.id) : 'none';
      const newLabel = editPriority ? (PRIORITY_META[String(editPriority)]?.label ?? String(editPriority)) : 'none';
      parts.push(`priority: ${oldLabel} → ${newLabel}`);
    }
    if (dueDateChanged) {
      parts.push(`due: ${origDueStr || 'none'} → ${editDueDateStr || 'none'}`);
    }
    if (timeChanged) {
      parts.push(`estimate: ${origEstMin ? origEstMin + 'm' : 'none'} → ${editTimeEstimateMin ? editTimeEstimateMin + 'm' : 'none'}`);
    }

    let newDueDate: number | undefined;
    if (dueDateChanged) {
      newDueDate = editDueDateStr
        ? new Date(editDueDateStr).getTime()
        : 0; // 0 = clear
    }

    let newTimeEstimate: number | undefined;
    if (timeChanged) {
      const minutes = parseInt(editTimeEstimateMin) || 0;
      newTimeEstimate = minutes * 60000; // 0 = clear
    }

    try {
      await updateTask(
        task.id, task.name, task.tags,
        nameChanged ? editName.trim() : undefined,
        priorityChanged ? (editPriority ?? undefined) : undefined,
        newDueDate,
        newTimeEstimate,
        parts.join(', '),
      );
      closeEdit();
      await load();
    } catch (e) {
      console.error('update failed:', e);
      editSaving = false;
    }
  }

  // Inject tasks created via QuickCapture directly — no server round-trip.
  $effect(() => {
    const t = $pendingNewTask;
    if (!t) return;
    tasks = [t, ...tasks];
    todayIds.set(new Set(tasks.map((t) => t.id)));
    clearNewTask();
  });

  $effect(() => {
    $clickupVersion;
    if ($suppressNextTodayLoad) {
      suppressNextTodayLoad.set(false);
      return;
    }
    load();
  });

  $effect(() => {
    $startupRetry;
    untrack(() => { if (error) load(); });
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
    <ul class="space-y-1.5 overflow-y-auto flex-1" bind:this={listEl}>
      {#each display as task (task.id)}
        {@const done_ = $completedTasks.has(task.id)}
        <li
          class="flex items-center gap-2"
          oncontextmenu={(e) => openContextMenu(e, task)}
        >
          <button
            class="w-4 h-4 rounded border flex-shrink-0 transition flex items-center justify-center
                   {done_ ? 'bg-green-500 border-green-500 hover:bg-green-600' : 'border-line hover:border-accent'}"
            onclick={() => toggle(task)}
            disabled={pending.has(task.id)}
            title={done_ ? 'Uncheck' : 'Mark complete'}
            aria-label="{done_ ? 'Uncheck' : 'Mark complete'} {task.name}"
          >
            {#if done_}
              <svg class="w-2.5 h-2.5 text-white" fill="none" viewBox="0 0 10 8" stroke="currentColor" stroke-width="2">
                <path d="M1 4l3 3 5-6" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            {/if}
          </button>

          <span class="w-1.5 h-1.5 rounded-full flex-shrink-0" style="background:{priorityColor(task)}"></span>

          <button
            class="flex-1 min-w-0 text-sm text-left truncate transition
                   {done_ ? 'line-through text-mute' : 'text-ink hover:text-accent'}"
            onclick={() => task.url && openUrl(task.url)}
            title={task.name}
          >{task.name}</button>

          {#if task.tags.length > 0}
            {@const visible = visibleTagCount(task.name, task.tags)}
            {@const overflow = task.tags.length - visible}
            {@const showAll = expandedTagTask === task.id}
            <div
              class="flex items-center gap-1 flex-shrink-0 {done_ ? 'opacity-30 saturate-[.35]' : ''}"
              onmouseenter={() => { if (overflow > 0) expandedTagTask = task.id; }}
              onmouseleave={() => { expandedTagTask = null; }}
            >
              {#each (showAll ? task.tags : task.tags.slice(0, visible)) as tag}
                {@const bg = tag.tag_bg ?? '#6b7280'}
                <span
                  class="px-1.5 py-0.5 text-[11px] rounded font-medium whitespace-nowrap max-w-[6rem] truncate"
                  style="background:{bg};color:{readableText(bg)}"
                >{tag.name}</span>
              {/each}
              {#if overflow > 0 && !showAll}
                <span class="text-[10px] text-mute cursor-default">+{overflow}</span>
              {/if}
            </div>
          {/if}

          {#if task.time_estimate}
            <span class="text-xs text-mute flex-shrink-0 {done_ ? 'opacity-30' : ''}">{Math.round(task.time_estimate / 60000)}m</span>
          {/if}

        </li>
      {/each}
    </ul>
    <p class="text-xs text-mute mt-3">
      {remaining > 0 ? `${remaining} remaining` : 'All done!'}
    </p>
  {/if}
</div>

<!-- Right-click context menu -->
{#if contextMenu}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-40" onclick={closeContextMenu}></div>
  <div
    class="fixed z-50 bg-surface border border-line rounded-lg shadow-xl py-1 w-32"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px"
    onkeydown={(e) => { if (e.key === 'Escape') closeContextMenu(); }}
  >
    <button
      class="w-full px-3 py-1.5 text-left text-sm text-ink hover:bg-line transition-colors flex items-center gap-2"
      onclick={() => { openEdit(contextMenu!.task); closeContextMenu(); }}
    >
      <svg class="w-3 h-3 flex-shrink-0" fill="none" viewBox="0 0 12 12" stroke="currentColor" stroke-width="1.8">
        <path d="M8 2l2 2-6 6H2V8l6-6z" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
      Edit
    </button>
    <button
      class="w-full px-3 py-1.5 text-left text-sm transition-colors flex items-center gap-2
             {menuDeleteConfirm ? 'text-red-500 hover:bg-red-500/10' : 'text-ink hover:bg-line'}"
      onclick={menuDelete}
      disabled={pending.has(contextMenu.task.id)}
    >
      <svg class="w-3 h-3 flex-shrink-0" fill="none" viewBox="0 0 12 12" stroke="currentColor" stroke-width="1.8">
        <path d="M2 3h8M5 3V2h2v1M4 3v7h4V3H4z" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
      {menuDeleteConfirm ? 'Confirm?' : 'Delete'}
    </button>
  </div>
{/if}

<!-- Edit modal -->
{#if editTask}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
    onclick={(e) => { if (e.target === e.currentTarget) closeEdit(); }}
  >
    <div class="bg-surface border border-line rounded-2xl shadow-2xl w-[480px] overflow-hidden">

      <!-- Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-line">
        <div class="flex items-center gap-2.5">
          <span class="w-2 h-2 rounded-full flex-shrink-0" style="background:{priorityColor(editTask)}"></span>
          <h3 class="text-sm font-semibold text-ink">Edit Task</h3>
        </div>
        <button
          class="text-mute hover:text-ink transition p-1.5 rounded-lg hover:bg-line"
          onclick={closeEdit}
          aria-label="Close"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
            <path d="M2 2l10 10M12 2L2 12"/>
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="px-6 py-5 space-y-5">

        <!-- Name -->
        <div>
          <span class="text-[10px] font-semibold text-mute uppercase tracking-widest block mb-1.5">Name</span>
          <input
            class="w-full bg-base border border-line rounded-xl px-4 py-2.5 text-sm text-ink focus:outline-none focus:border-accent transition"
            bind:value={editName}
            onkeydown={(e) => { if (e.key === 'Enter') saveEdit(); if (e.key === 'Escape') closeEdit(); }}
          />
        </div>

        <!-- Priority (visual toggle buttons) -->
        <div>
          <span class="text-[10px] font-semibold text-mute uppercase tracking-widest block mb-1.5">Priority</span>
          <div class="flex gap-1.5">
            <button
              class="flex-1 px-2 py-2 rounded-xl text-xs font-medium border transition
                     {editPriority === null
                       ? 'bg-line border-accent text-ink'
                       : 'bg-base border-line text-mute hover:border-line/70 hover:text-ink'}"
              onclick={() => editPriority = null}
            >None</button>
            {#each Object.entries(PRIORITY_META).sort(([a], [b]) => Number(b) - Number(a)) as [id, meta]}
              <button
                class="flex-1 px-2 py-2 rounded-xl text-xs font-medium border transition"
                style="{editPriority === Number(id)
                  ? `background:${meta.color}22;border-color:${meta.color};color:${meta.color}`
                  : 'background:var(--color-base);border-color:var(--color-line);color:var(--color-mute)'}"
                onmouseenter={(e) => {
                  if (editPriority !== Number(id)) {
                    (e.currentTarget as HTMLElement).style.borderColor = meta.color;
                    (e.currentTarget as HTMLElement).style.color = meta.color;
                  }
                }}
                onmouseleave={(e) => {
                  if (editPriority !== Number(id)) {
                    (e.currentTarget as HTMLElement).style.borderColor = 'var(--color-line)';
                    (e.currentTarget as HTMLElement).style.color = 'var(--color-mute)';
                  }
                }}
                onclick={() => editPriority = Number(id)}
              >{meta.label}</button>
            {/each}
          </div>
        </div>

        <!-- Due Date + Time Estimate -->
        <div class="grid grid-cols-2 gap-4">
          <div>
            <span class="text-[10px] font-semibold text-mute uppercase tracking-widest block mb-1.5">Due Date</span>
            <input
              type="date"
              class="w-full bg-base border border-line rounded-xl px-3 py-2.5 text-sm text-ink focus:outline-none focus:border-accent transition"
              bind:value={editDueDateStr}
            />
          </div>
          <div>
            <span class="text-[10px] font-semibold text-mute uppercase tracking-widest block mb-1.5">Estimate (min)</span>
            <input
              type="number"
              min="0"
              placeholder="—"
              class="w-full bg-base border border-line rounded-xl px-3 py-2.5 text-sm text-ink focus:outline-none focus:border-accent transition placeholder:text-mute"
              bind:value={editTimeEstimateMin}
            />
          </div>
        </div>

      </div>

      <!-- Footer -->
      <div class="flex items-center justify-between px-6 py-3.5 border-t border-line bg-base/40">
        <span class="text-[11px] text-mute/60">⏎ save · esc cancel</span>
        <div class="flex gap-2">
          <button
            class="px-4 py-1.5 text-sm text-mute hover:text-ink transition rounded-lg hover:bg-line"
            onclick={closeEdit}
          >Cancel</button>
          <button
            class="px-4 py-1.5 text-sm bg-accent text-white rounded-lg hover:opacity-90 transition disabled:opacity-50 font-medium"
            onclick={saveEdit}
            disabled={editSaving}
          >{editSaving ? 'Saving…' : 'Save changes'}</button>
        </div>
      </div>

    </div>
  </div>
{/if}

