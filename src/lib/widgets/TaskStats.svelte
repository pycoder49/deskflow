<script lang="ts">
  import { onMount } from 'svelte';
  import { getTaskStats, getCompletedTodayTasks, PRIORITY_META, type StatEntry, type Task } from '$lib/services/clickup';
  import { clickupVersion } from '$lib/stores/refresh';

  let stats     = $state<StatEntry[]>([]);
  let doneTasks = $state<Task[]>([]);
  let view      = $state<'line' | 'bars' | 'done'>('line');

  onMount(async () => {
    [stats, doneTasks] = await Promise.all([getTaskStats(), getCompletedTodayTasks()]);
  });

  $effect(() => {
    $clickupVersion;
    getCompletedTodayTasks().then(t => { doneTasks = t; });
  });

  const DAY_ABBR = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

  const data   = $derived(stats.map(s => s.count));
  const labels = $derived(stats.map(s => {
    const [y, m, d] = s.date.split('-').map(Number);
    const dow = new Date(y, m - 1, d).getDay();
    return `${DAY_ABBR[dow]} ${m}/${d}`;
  }));

  const max = $derived(Math.max(...data, 1));
  const n   = $derived(data.length);

  let cw = $state(0);
  let ch = $state(0);

  const PAD_T = 10;
  const PAD_B = 8;

  const chartH = $derived(ch - PAD_T - PAD_B);

  // ── Line view ────────────────────────────────────────────────────────────────
  const pts = $derived(
    cw && ch && n > 1
      ? data.map((v, i) => ({
          x: (i / (n - 1)) * cw,
          y: PAD_T + (1 - v / max) * chartH,
        }))
      : []
  );

  const line = $derived(pts.map((p, i) => `${i === 0 ? 'M' : 'L'}${p.x},${p.y}`).join(' '));
  const area = $derived(
    pts.length ? `${line} L${cw},${PAD_T + chartH} L0,${PAD_T + chartH} Z` : ''
  );

  // ── Bars view ────────────────────────────────────────────────────────────────
  const BAR_GAP = 3;
  const bars = $derived(
    cw && ch && n > 0
      ? data.map((v, i) => {
          const slotW = cw / n;
          const barW  = Math.max(4, slotW - BAR_GAP);
          const barH  = Math.max(v === 0 ? 0 : 2, (v / max) * chartH);
          return {
            x: i * slotW + (slotW - barW) / 2,
            y: PAD_T + chartH - barH,
            w: barW,
            h: barH,
            v,
            label: labels[i],
          };
        })
      : []
  );

  const gridYs = $derived(
    ch ? [0.25, 0.5, 0.75].map(f => PAD_T + (1 - f) * chartH) : []
  );

  let hovered = $state<{ x: number; y: number; value: number; label: string } | null>(null);

  const doneColumns = $derived.by(() => {
    const cols = [];
    for (let i = 0; i < doneTasks.length; i += 3) cols.push(doneTasks.slice(i, i + 3));
    return cols.slice(0, 4);
  });

  async function refresh() {
    [stats, doneTasks] = await Promise.all([getTaskStats(), getCompletedTodayTasks()]);
  }
</script>

<div class="flex flex-col h-full">
  <div class="flex items-center justify-between mb-3">
    <span class="text-xs uppercase tracking-wider font-semibold text-mute">Completions</span>
    <div class="flex items-center gap-3 text-xs">
      <div class="flex border border-line rounded-md p-0.5 gap-0.5">
        {#each (['line', 'bars', 'done'] as const) as v}
          <button
            onclick={() => (view = v)}
            class="px-2 py-0.5 rounded transition capitalize {view === v ? 'bg-accent text-white' : 'text-mute hover:text-ink'}"
          >{v}</button>
        {/each}
      </div>
      <button onclick={refresh} class="text-mute hover:text-ink transition" title="Refresh">↻</button>
    </div>
  </div>

  <!-- ── Done tab ─────────────────────────────────────────────────────────── -->
  {#if view === 'done'}
    <div class="flex-1 min-h-0 overflow-y-auto">
      {#if doneTasks.length === 0}
        <div class="flex items-center justify-center h-full text-xs text-mute italic">
          Nothing completed yet today.
        </div>
      {:else}
        <div class="flex gap-1.5 overflow-hidden">
          {#each doneColumns as col}
            <div class="flex flex-col gap-px flex-1 min-w-0">
              {#each col as task}
                {@const p = task.priority?.id ? PRIORITY_META[task.priority.id] : null}
                <div class="flex items-center gap-1.5 px-2 rounded-lg bg-surface/50 min-w-0" style="padding-top:1.6px;padding-bottom:1.6px">
                  <span class="w-1.5 h-1.5 rounded-full shrink-0" style="background:{p?.color ?? '#6b7280'}"></span>
                  <span class="text-xs text-ink truncate">{task.name}</span>
                </div>
              {/each}
            </div>
          {/each}
        </div>
      {/if}
    </div>

  <!-- ── Chart tabs ───────────────────────────────────────────────────────── -->
  {:else}
    <div class="flex-1 min-h-0 relative" bind:clientWidth={cw} bind:clientHeight={ch}>
      {#if pts.length || bars.length}
        <svg width={cw} height={ch} class="absolute inset-0 overflow-visible">
          <defs>
            <linearGradient id="stat-line" gradientUnits="userSpaceOnUse" x1="0" y1="0" x2={cw} y2="0">
              <stop offset="0%"   stop-color="var(--color-chart-b)" />
              <stop offset="100%" stop-color="var(--color-chart-a)" />
            </linearGradient>
            <linearGradient id="stat-area" gradientUnits="userSpaceOnUse" x1="0" y1="0" x2="0" y2={ch}>
              <stop offset="0%"   stop-color="var(--color-chart-a)" stop-opacity="0.5" />
              <stop offset="100%" stop-color="var(--color-chart-b)" stop-opacity="0.08" />
            </linearGradient>
            <linearGradient id="stat-bar" gradientUnits="userSpaceOnUse" x1="0" y1="0" x2="0" y2={ch}>
              <stop offset="0%"   stop-color="var(--color-chart-a)" stop-opacity="0.9" />
              <stop offset="100%" stop-color="var(--color-chart-b)" stop-opacity="0.5" />
            </linearGradient>
          </defs>

          <!-- Gridlines -->
          {#each gridYs as y}
            <line x1="0" y1={y} x2={cw} y2={y} stroke="var(--color-line)" stroke-width="1" stroke-opacity="0.35" />
          {/each}

          {#if view === 'line'}
            <path d={area} fill="url(#stat-area)" />
            <path
              d={line}
              fill="none"
              stroke="url(#stat-line)"
              stroke-width="2.5"
              stroke-linejoin="round"
              stroke-linecap="round"
            />
            {#each pts as p, i}
              {#if i === pts.length - 1}
                <circle cx={p.x} cy={p.y} r="7"   fill="var(--color-chart-a)" fill-opacity="0.18" />
                <circle cx={p.x} cy={p.y} r="3.5" fill="var(--color-chart-a)" />
              {:else}
                <circle cx={p.x} cy={p.y} r="2.5" fill="url(#stat-line)" />
              {/if}
              <circle
                cx={p.x} cy={p.y} r="12"
                fill="transparent"
                class="cursor-default"
                onmouseenter={() => hovered = { x: p.x, y: p.y, value: data[i], label: labels[i] }}
                onmouseleave={() => hovered = null}
              />
            {/each}

          {:else}
            {#each bars as b}
              {#if b.h > 0}
                <rect x={b.x} y={b.y} width={b.w} height={b.h} rx="3" ry="3" fill="url(#stat-bar)" />
              {/if}
              <rect
                x={b.x} y={PAD_T}
                width={b.w} height={chartH}
                fill="transparent"
                class="cursor-default"
                onmouseenter={() => hovered = { x: b.x + b.w / 2, y: b.y, value: b.v, label: b.label }}
                onmouseleave={() => hovered = null}
              />
            {/each}
          {/if}
        </svg>

        {#if hovered}
          <div
            class="absolute pointer-events-none z-10 bg-surface border border-line rounded-md px-2 py-1 text-xs text-ink shadow-md whitespace-nowrap"
            style="left:{hovered.x}px;top:{hovered.y}px;transform:translate(-50%,calc(-100% - 8px))"
          >
            {hovered.value} task{hovered.value === 1 ? '' : 's'} · {hovered.label}
          </div>
        {/if}
      {:else}
        <div class="flex items-center justify-center h-full text-xs text-mute">loading…</div>
      {/if}
    </div>
  {/if}
</div>
