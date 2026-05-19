<script lang="ts">
  import { onMount } from 'svelte';
  import { getTaskStats, getCompletedTodayTasks, PRIORITY_META, type StatEntry, type Task } from '$lib/services/clickup';
  import { clickupVersion, completionVersion } from '$lib/stores/refresh';

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

  $effect(() => {
    $completionVersion;
    getCompletedTodayTasks().then(t => { doneTasks = t; });
  });

  const DAY_ABBR = ['SU', 'MO', 'TU', 'WE', 'TH', 'FR', 'SA'];
  const DAY_FULL = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

  const data = $derived.by(() => {
    const d = stats.map(s => s.count);
    if (d.length > 0) d[d.length - 1] = doneTasks.length;
    return d;
  });
  const labels = $derived(stats.map(s => {
    const [y, m, d] = s.date.split('-').map(Number);
    const dow = new Date(y, m - 1, d).getDay();
    return `${DAY_FULL[dow]} ${m}/${d}`;
  }));
  const dayLabels = $derived(stats.map(s => {
    const [y, m, d] = s.date.split('-').map(Number);
    return DAY_ABBR[new Date(y, m - 1, d).getDay()];
  }));

  const max = $derived(Math.max(...data, 1));
  const n   = $derived(data.length);

  let cw = $state(0);
  let ch = $state(0);

  const PAD_T = 10;
  const PAD_B = 16;

  const chartH = $derived(ch - PAD_T - PAD_B);
  const baseY  = $derived(PAD_T + chartH);

  // ── Line view ────────────────────────────────────────────────────────────────
  const pts = $derived(
    cw && ch && n > 1
      ? data.map((v, i) => ({
          x: (i / (n - 1)) * cw,
          y: PAD_T + (1 - v / max) * chartH,
        }))
      : []
  );

  const smoothLine = $derived(
    pts.map((p, i) => `${i === 0 ? 'M' : 'L'}${p.x},${p.y}`).join(' ')
  );

  // ── Bars view ────────────────────────────────────────────────────────────────
  function topRoundedRect(x: number, y: number, w: number, h: number, r: number): string {
    const cr = Math.min(r, h, w / 2);
    return `M${x+cr},${y} h${w-2*cr} a${cr},${cr} 0 0 1 ${cr},${cr} v${h-cr} h${-w} v${-(h-cr)} a${cr},${cr} 0 0 1 ${cr},${-cr} z`;
  }

  const BAR_GAP = 3;
  const bars = $derived(
    cw && ch && n > 0
      ? data.map((v, i) => {
          const slotW = cw / n;
          const barW  = Math.max(4, slotW - BAR_GAP);
          const barH  = Math.max(v === 0 ? 0 : 2, (v / max) * chartH);
          const y     = PAD_T + chartH - barH;
          return {
            x: i * slotW + (slotW - barW) / 2,
            y,
            w: barW,
            h: barH,
            v,
            label: labels[i],
            dayLabel: dayLabels[i],
            isLast: i === n - 1,
            path: barH > 0 ? topRoundedRect(i * slotW + (slotW - barW) / 2, y, barW, barH, 3) : '',
          };
        })
      : []
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
            <linearGradient id="stat-bar" gradientUnits="objectBoundingBox" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%"   stop-color="var(--color-chart-a)" stop-opacity="0.9" />
              <stop offset="100%" stop-color="var(--color-chart-b)" stop-opacity="0.6" />
            </linearGradient>
            <linearGradient id="stat-track" gradientUnits="userSpaceOnUse" x1="0" y1="0" x2="0" y2={ch}>
              <stop offset="0%"   stop-color="var(--color-chart-a)" stop-opacity="0.07" />
              <stop offset="100%" stop-color="var(--color-chart-b)" stop-opacity="0.03" />
            </linearGradient>
          </defs>

          <!-- Baseline -->
          <line x1="0" y1={baseY} x2={cw} y2={baseY}
            stroke="var(--color-line)" stroke-width="1" />

          <!-- Mid gridline (dashed) -->
          {#if ch}
            <line
              x1="0" y1={PAD_T + chartH * 0.5}
              x2={cw} y2={PAD_T + chartH * 0.5}
              stroke="var(--color-line)" stroke-width="1"
              stroke-dasharray="2 5" stroke-opacity="0.45"
            />
          {/if}

          {#if view === 'line'}
            <!-- Vertical drop lines: thin dashes from each point to baseline -->
            {#each pts as p}
              <line
                x1={p.x} y1={p.y + 7}
                x2={p.x} y2={baseY}
                stroke="var(--color-chart-a)"
                stroke-width="1"
                stroke-dasharray="1.5 3"
                stroke-opacity="0.17"
              />
            {/each}

            <!-- Smooth bezier line -->
            <path
              d={smoothLine}
              fill="none"
              stroke="url(#stat-line)"
              stroke-width="1.5"
              stroke-linejoin="round"
              stroke-linecap="round"
            />

            <!-- Dots, labels, hit targets -->
            {#each pts as p, i}
              {#if i === pts.length - 1}
                <!-- Today: pulse ring + filled dot + count badge -->
                <circle cx={p.x} cy={p.y} r="6" class="pulse-ring"
                  fill="none" stroke="var(--color-chart-a)" stroke-width="1" />
                <circle cx={p.x} cy={p.y} r="3.5" fill="var(--color-chart-a)" />
                <text
                  x={p.x} y={p.y - 9}
                  text-anchor="middle"
                  fill="var(--color-chart-a)"
                  font-size="9"
                  font-family="var(--font-mono)"
                  font-weight="600"
                >{data[i]}</text>
              {:else if hovered?.label === labels[i]}
                <!-- Hovered: solid fill -->
                <circle cx={p.x} cy={p.y} r="3" fill="var(--color-chart-a)" />
              {:else}
                <!-- Past: hollow ring -->
                <circle
                  cx={p.x} cy={p.y} r="2"
                  fill="none"
                  stroke="var(--color-chart-a)"
                  stroke-width="1.5"
                  stroke-opacity="0.4"
                />
              {/if}
              <!-- Hit target -->
              <circle
                cx={p.x} cy={p.y} r="14"
                fill="transparent"
                class="cursor-default"
                onmouseenter={() => hovered = { x: p.x, y: p.y, value: data[i], label: labels[i] }}
                onmouseleave={() => hovered = null}
              />
            {/each}

            <!-- X-axis day labels -->
            {#each pts as p, i}
              <text
                x={p.x} y={baseY + 12}
                text-anchor="middle"
                fill="var(--color-mute)"
                font-size="8"
                font-family="var(--font-mono)"
                opacity={i === pts.length - 1 ? '1' : '0.55'}
              >{dayLabels[i]}</text>
            {/each}

          {:else}
            <!-- Bars + day labels -->
            {#each bars as b}
              <!-- Actual bar -->
              {#if b.path}
                <path
                  d={b.path}
                  fill="url(#stat-bar)"
                  opacity={hovered && hovered.label !== b.label ? '0.3' : b.isLast ? '1' : '0.72'}
                  style="transition: opacity 0.15s"
                />
              {/if}
              <!-- Value above today's or hovered bar -->
              {#if (b.isLast || hovered?.label === b.label) && b.v > 0}
                <text
                  x={b.x + b.w / 2} y={b.h > 0 ? b.y - 5 : PAD_T + 10}
                  text-anchor="middle"
                  fill="var(--color-chart-a)"
                  font-size="9"
                  font-family="var(--font-mono)"
                  font-weight="600"
                >{b.v}</text>
              {/if}
              <!-- Day label -->
              <text
                x={b.x + b.w / 2} y={baseY + 12}
                text-anchor="middle"
                fill="var(--color-mute)"
                font-size="8"
                font-family="var(--font-mono)"
                opacity={hovered && hovered.label !== b.label ? '0.3' : b.isLast ? '1' : '0.55'}
                style="transition: opacity 0.15s"
              >{b.dayLabel}</text>
              <!-- Hit target -->
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

        <!-- Tooltip pill -->
        {#if hovered}
          <div
            class="absolute pointer-events-none z-10 whitespace-nowrap"
            style="left:{hovered.x}px;top:{hovered.y}px;transform:translate(-50%,calc(-100% - 10px))"
          >
            <div style="font-family:var(--font-mono);font-size:10px;padding:2px 8px;border-radius:999px;background:color-mix(in srgb,var(--color-surface) 88%,transparent);border:1px solid color-mix(in srgb,var(--color-line) 70%,transparent);color:var(--color-ink);backdrop-filter:blur(6px)">
              <span style="color:var(--color-chart-a);font-weight:700">{hovered.value}</span>
              <span style="color:var(--color-mute)"> · {hovered.label}</span>
            </div>
          </div>
        {/if}
      {:else}
        <div class="flex items-center justify-center h-full text-xs text-mute">loading…</div>
      {/if}
    </div>
  {/if}
</div>

<style>
  @keyframes pulse-ring {
    0%   { transform: scale(0.3); opacity: 0.75; }
    100% { transform: scale(2.4); opacity: 0; }
  }
  .pulse-ring {
    transform-box: fill-box;
    transform-origin: center;
    animation: pulse-ring 2.8s ease-out infinite;
  }
</style>
