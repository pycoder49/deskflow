<script lang="ts">
  import {
    getCalendarEvents,
    type CalendarEvent,
  } from '$lib/services/calendar';
  import { untrack } from 'svelte';
  import { logicalToday as getLogicalToday, startupRetry } from '$lib/stores/refresh';

  type View = 'week' | 'list' | 'day' | 'month';

  let events = $state<CalendarEvent[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let view = $state<View>('week');
  let dayOffset = $state(0);
  let weekOffset = $state(0);
  let monthOffset = $state(0);
  let hoveredCell = $state<string | null>(null);

  const HOUR_START = 7;
  const HOUR_END = 23;
  const PX_PER_HOUR = 40;
  const TIMELINE_HEIGHT = (HOUR_END - HOUR_START) * PX_PER_HOUR;

  const logicalToday = getLogicalToday();

  function addDays(iso: string, n: number): string {
    const d = new Date(iso + 'T12:00:00');
    d.setDate(d.getDate() + n);
    return d.toISOString().slice(0, 10);
  }

  const currentDate = $derived(addDays(logicalToday, dayOffset));

  // Week: Mon–Sun containing logicalToday + weekOffset weeks
  const weekStart = $derived.by(() => {
    const base = new Date(logicalToday + 'T12:00:00');
    const dow = base.getDay(); // 0=Sun,1=Mon…
    const toMonday = dow === 0 ? -6 : 1 - dow;
    const mon = new Date(base);
    mon.setDate(base.getDate() + toMonday + weekOffset * 7);
    return mon.toISOString().slice(0, 10);
  });

  const weekDates = $derived.by(() => {
    const out: string[] = [];
    for (let i = 0; i < 7; i++) out.push(addDays(weekStart, i));
    return out;
  });

  function weekLabel(dates: string[]): string {
    const a = new Date(dates[0] + 'T12:00:00');
    const b = new Date(dates[6] + 'T12:00:00');
    const am = a.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
    if (a.getMonth() === b.getMonth() && a.getFullYear() === b.getFullYear()) {
      return `${am} – ${b.getDate()}`;
    }
    return `${am} – ${b.toLocaleDateString(undefined, { month: 'short', day: 'numeric' })}`;
  }

  const monthAnchor = $derived.by(() => {
    const base = new Date(logicalToday + 'T12:00:00');
    base.setDate(1);
    base.setMonth(base.getMonth() + monthOffset);
    return base;
  });

  const monthDates = $derived.by(() => {
    const first = new Date(monthAnchor);
    const start = new Date(first);
    start.setDate(1 - first.getDay());
    const out: string[] = [];
    for (let i = 0; i < 42; i++) {
      const d = new Date(start);
      d.setDate(start.getDate() + i);
      out.push(d.toISOString().slice(0, 10));
    }
    return out;
  });

  function dateLabel(iso: string): string {
    if (iso === logicalToday) return 'Today';
    const today = new Date(logicalToday + 'T12:00:00');
    const target = new Date(iso + 'T12:00:00');
    const diffDays = Math.round(
      (target.getTime() - today.getTime()) / (24 * 60 * 60 * 1000)
    );
    if (diffDays === 1) return 'Tomorrow';
    if (diffDays === -1) return 'Yesterday';
    return target.toLocaleDateString(undefined, {
      weekday: 'short',
      month: 'short',
      day: 'numeric',
    });
  }

  function fullDateLabel(iso: string): string {
    const base = dateLabel(iso);
    const d = new Date(iso + 'T12:00:00');
    const md = d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
    if (['Today', 'Tomorrow', 'Yesterday'].includes(base)) return `${base} · ${md}`;
    return base;
  }

  function monthLabel(d: Date): string {
    return d.toLocaleDateString(undefined, { month: 'long', year: 'numeric' });
  }

  function formatTime(t: string): string {
    if (!t) return '';
    const [h, m] = t.split(':').map(Number);
    if (Number.isNaN(h)) return t;
    const ampm = h >= 12 ? 'pm' : 'am';
    const h12 = h % 12 === 0 ? 12 : h % 12;
    return m === 0 ? `${h12}${ampm}` : `${h12}:${String(m).padStart(2, '0')}${ampm}`;
  }

  function formatHour(h: number): string {
    if (h === 0) return '12am';
    if (h === 12) return '12pm';
    return h < 12 ? `${h}am` : `${h - 12}pm`;
  }

  function toMinutes(t: string): number {
    const [h, m] = t.split(':').map(Number);
    return h * 60 + (m || 0);
  }

  const fetchRange = $derived.by(() => {
    if (view === 'month') {
      return { start: monthDates[0], end: addDays(monthDates[41], 1) };
    }
    if (view === 'day') {
      return { start: currentDate, end: addDays(currentDate, 1) };
    }
    if (view === 'week') {
      return { start: weekDates[0], end: addDays(weekDates[6], 1) };
    }
    // list
    return { start: logicalToday, end: addDays(logicalToday, 2) };
  });

  async function load(start: string, end: string) {
    loading = true;
    error = null;
    try {
      events = await getCalendarEvents(start, end);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    load(fetchRange.start, fetchRange.end);
  });

  $effect(() => {
    $startupRetry;
    untrack(() => { if (error) load(fetchRange.start, fetchRange.end); });
  });

  const grouped = $derived.by(() => {
    const map = new Map<string, CalendarEvent[]>();
    for (const ev of events) {
      const arr = map.get(ev.start_date) ?? [];
      arr.push(ev);
      map.set(ev.start_date, arr);
    }
    return [...map.entries()];
  });

  const dayAllDay = $derived(
    events.filter((e) => e.start_date === currentDate && e.all_day)
  );
  const dayTimed = $derived(
    events.filter((e) => e.start_date === currentDate && !e.all_day)
  );

  const timedBlocks = $derived.by(() =>
    dayTimed
      .map((ev) => {
        const startM = toMinutes(ev.start_time);
        const endM = toMinutes(ev.end_time);
        let top = ((startM - HOUR_START * 60) / 60) * PX_PER_HOUR;
        let bottom = ((endM - HOUR_START * 60) / 60) * PX_PER_HOUR;
        top = Math.max(0, top);
        bottom = Math.min(TIMELINE_HEIGHT, bottom);
        const height = Math.max(20, bottom - top);
        return { ev, top, height };
      })
      .filter((b) => b.top < TIMELINE_HEIGHT && b.top + b.height > 0)
  );

  const weekGrid = $derived.by(() =>
    weekDates.map((iso) => {
      const d = new Date(iso + 'T12:00:00');
      return {
        iso,
        dayNum: d.getDate(),
        dayName: d.toLocaleDateString(undefined, { weekday: 'short' }),
        isToday: iso === logicalToday,
        events: events
          .filter((e) => e.start_date === iso)
          .sort((a, b) => {
            if (a.all_day && !b.all_day) return -1;
            if (!a.all_day && b.all_day) return 1;
            return toMinutes(a.start_time) - toMinutes(b.start_time);
          }),
      };
    })
  );

  const monthGrid = $derived.by(() => {
    const monthIdx = monthAnchor.getMonth();
    return monthDates.map((iso) => {
      const d = new Date(iso + 'T12:00:00');
      return {
        iso,
        day: d.getDate(),
        isCurrentMonth: d.getMonth() === monthIdx,
        isToday: iso === logicalToday,
        events: events.filter((e) => e.start_date === iso),
      };
    });
  });

  function jumpToWeek(iso: string) {
    const target = new Date(iso + 'T12:00:00');
    const today = new Date(logicalToday + 'T12:00:00');
    const targetDow = target.getDay();
    const targetMon = new Date(target);
    targetMon.setDate(target.getDate() + (targetDow === 0 ? -6 : 1 - targetDow));
    const todayDow = today.getDay();
    const todayMon = new Date(today);
    todayMon.setDate(today.getDate() + (todayDow === 0 ? -6 : 1 - todayDow));
    weekOffset = Math.round((targetMon.getTime() - todayMon.getTime()) / (7 * 24 * 60 * 60 * 1000));
    view = 'week';
  }

  function refresh() {
    load(fetchRange.start, fetchRange.end);
  }
</script>

<div class="cal-root flex flex-col h-full">
  <!-- Header -->
  <div class="flex items-center justify-between mb-3">
    <h2 class="text-xs uppercase tracking-wider cal-label">Calendar</h2>
    <div class="flex items-center gap-3 text-xs">
      <div class="flex cal-tab-border rounded-md p-0.5 gap-0.5">
        {#each ['list', 'day', 'week', 'month'] as v (v)}
          <button
            class="px-2 py-0.5 rounded transition capitalize
                   {view === v ? 'cal-tab-active' : 'text-mute hover:text-ink'}"
            onclick={() => (view = v as View)}
          >{v}</button>
        {/each}
      </div>
      <button
        class="text-mute hover:text-ink transition"
        onclick={refresh}
        title="Refresh"
        aria-label="Refresh calendar"
      >↻</button>
    </div>
  </div>

  {#if loading}
    <div class="space-y-2">
      {#each [1, 2, 3] as _}
        <div class="h-6 bg-line rounded animate-pulse"></div>
      {/each}
    </div>

  {:else if error}
    <p class="text-xs text-red-400 mt-2">Failed to load: {error}</p>

  <!-- ─── LIST ─────────────────────────────────────────────────── -->
  {:else if view === 'list'}
    {#if events.length === 0}
      <p class="text-sm text-mute italic mt-2">Nothing on the calendar.</p>
    {:else}
      <div class="space-y-3 overflow-y-auto flex-1">
        {#each grouped as [date, dayEvents] (date)}
          <div>
            <h3 class="text-[11px] uppercase tracking-wider text-mute mb-1.5">
              {dateLabel(date)}
            </h3>
            <ul class="space-y-1">
              {#each dayEvents as ev}
                <li
                  class="flex items-center gap-2 text-sm px-2 py-1 rounded"
                  style="border-left:2px solid var(--color-accent);background:color-mix(in srgb, var(--color-accent) 10%, transparent)"
                >
                  <span class="text-mute text-xs w-14 flex-shrink-0 tabular-nums">
                    {ev.all_day ? 'all day' : formatTime(ev.start_time)}
                  </span>
                  <span class="flex-1 min-w-0 truncate text-base" title="{ev.title} · {ev.calendar}">
                    {ev.title}
                  </span>
                </li>
              {/each}
            </ul>
          </div>
        {/each}
      </div>
    {/if}

  <!-- ─── WEEK ──────────────────────────────────────────────────── -->
  {:else if view === 'week'}
    <div class="flex items-center mb-2 text-xs">
      <div class="flex-1"></div>
      <div class="flex items-center gap-1">
        <button class="text-mute hover:text-ink transition px-1" onclick={() => weekOffset--} aria-label="Previous week">‹</button>
        <span class="text-ink px-1">{weekLabel(weekDates)}</span>
        <button class="text-mute hover:text-ink transition px-1" onclick={() => weekOffset++} aria-label="Next week">›</button>
      </div>
      <div class="flex-1 flex justify-end">
        <button
          class="transition {weekOffset === 0 ? 'text-mute opacity-30 cursor-default' : 'text-mute hover:text-ink'}"
          onclick={() => (weekOffset = 0)}
          disabled={weekOffset === 0}
          title="Jump to current week"
        >today</button>
      </div>
    </div>

    <div class="grid grid-cols-7 flex-1 min-h-0 gap-px bg-line rounded-lg overflow-hidden">
      {#each weekGrid as day (day.iso)}
        <div class="bg-surface flex flex-col overflow-hidden {day.isToday ? 'bg-accent/[0.03]' : ''}">
          <!-- Day header -->
          <div class="px-1 py-1.5 border-b border-line text-center shrink-0">
            <div class="text-[9px] uppercase tracking-wide text-mute leading-none">{day.dayName}</div>
            <div
              class="text-sm font-medium leading-none tabular-nums mt-1 w-6 h-6 flex items-center justify-center rounded-full mx-auto
                     {day.isToday ? 'bg-accent text-white' : 'text-ink'}"
            >{day.dayNum}</div>
          </div>
          <!-- Events -->
          <div class="flex-1 overflow-y-auto p-0.5 space-y-0.5">
            {#each day.events as ev}
              <div
                class="rounded px-1 py-0.5 text-[10px] leading-tight"
                style="border-left:2px solid var(--color-accent);background:color-mix(in srgb, var(--color-accent) 10%, transparent)"
                title="{ev.title}{ev.all_day ? '' : ` · ${formatTime(ev.start_time)}`}"
              >
                {#if !ev.all_day}
                  <div class="text-mute leading-none mb-0.5 text-[9px]">{formatTime(ev.start_time)}</div>
                {/if}
                <div class="truncate text-ink">{ev.title}</div>
              </div>
            {/each}
          </div>
        </div>
      {/each}
    </div>

  <!-- ─── DAY ───────────────────────────────────────────────────── -->
  {:else if view === 'day'}
    <div class="flex items-center mb-2 text-xs">
      <div class="flex-1"></div>
      <div class="flex items-center gap-1">
        <button class="text-mute hover:text-ink transition px-1" onclick={() => dayOffset--} aria-label="Previous day">‹</button>
        <span class="text-ink px-1">{fullDateLabel(currentDate)}</span>
        <button class="text-mute hover:text-ink transition px-1" onclick={() => dayOffset++} aria-label="Next day">›</button>
      </div>
      <div class="flex-1 flex justify-end">
        <button
          class="transition {dayOffset === 0 ? 'text-mute opacity-30 cursor-default' : 'text-mute hover:text-ink'}"
          onclick={() => (dayOffset = 0)}
          disabled={dayOffset === 0}
          title="Jump to today"
        >today</button>
      </div>
    </div>

    {#if dayAllDay.length > 0}
      <div class="flex flex-wrap gap-1 mb-2">
        {#each dayAllDay as ev}
          <span
            class="px-2 py-0.5 text-[11px] rounded"
            style="background:color-mix(in srgb, var(--color-accent) 15%, transparent);border-left:2px solid var(--color-accent)"
            title="{ev.title} · {ev.calendar}"
          >{ev.title}</span>
        {/each}
      </div>
    {/if}

    <div class="overflow-y-auto flex-1">
      <div class="relative" style="height:{TIMELINE_HEIGHT}px">
        {#each Array.from({ length: HOUR_END - HOUR_START }, (_, i) => HOUR_START + i) as h}
          <div
            class="absolute left-0 right-0 border-t border-line"
            style="top:{(h - HOUR_START) * PX_PER_HOUR}px;height:{PX_PER_HOUR}px"
          >
            <span class="text-[10px] text-mute pl-1 pt-0.5 inline-block tabular-nums">
              {formatHour(h)}
            </span>
          </div>
        {/each}

        {#each timedBlocks as { ev, top, height } (ev.title + ev.start_time)}
          <div
            class="absolute rounded px-2 py-1 overflow-hidden"
            style="left:3rem;right:0.25rem;top:{top}px;height:{height}px;background:color-mix(in srgb, var(--color-accent) 15%, transparent);border-left:3px solid var(--color-accent)"
            title="{ev.title} · {ev.calendar}"
          >
            <div class="text-sm font-medium truncate">{ev.title}</div>
            {#if height >= 32}
              <div class="text-[10px] text-mute truncate">
                {formatTime(ev.start_time)} – {formatTime(ev.end_time)}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>

  <!-- ─── MONTH ─────────────────────────────────────────────────── -->
  {:else}
    <div class="flex items-center mb-2 text-xs">
      <div class="flex-1"></div>
      <div class="flex items-center gap-1">
        <button class="text-mute hover:text-ink transition px-1" onclick={() => monthOffset--} aria-label="Previous month">‹</button>
        <span class="text-ink px-1">{monthLabel(monthAnchor)}</span>
        <button class="text-mute hover:text-ink transition px-1" onclick={() => monthOffset++} aria-label="Next month">›</button>
      </div>
      <div class="flex-1 flex justify-end">
        <button
          class="transition {monthOffset === 0 ? 'text-mute opacity-30 cursor-default' : 'text-mute hover:text-ink'}"
          onclick={() => (monthOffset = 0)}
          disabled={monthOffset === 0}
          title="Jump to current month"
        >today</button>
      </div>
    </div>

    <div class="grid grid-cols-7 gap-1 text-[10px] text-mute uppercase tracking-wider mb-1 text-center">
      {#each ['S', 'M', 'T', 'W', 'T', 'F', 'S'] as d}
        <div>{d}</div>
      {/each}
    </div>

    <div class="grid grid-cols-7 grid-rows-6 gap-1 flex-1 min-h-0">
      {#each monthGrid as cell (cell.iso)}
        <div
          class="relative border border-line rounded p-1 flex flex-col cursor-pointer
                 {cell.isCurrentMonth ? '' : 'opacity-40'}
                 {cell.isToday ? 'ring-1 ring-accent/60' : ''}"
          onmouseenter={() => (hoveredCell = cell.iso)}
          onmouseleave={() => (hoveredCell = null)}
          ondblclick={() => jumpToWeek(cell.iso)}
          role="gridcell"
          tabindex="0"
          title="Double-click to open week view"
        >
          <!-- Hover popup -->
          {#if hoveredCell === cell.iso && cell.events.length > 0}
            <div class="cal-popup absolute z-50 bottom-full left-0 mb-1 w-44 bg-surface border border-line rounded-lg shadow-lg p-2.5 pointer-events-none">
              <div class="text-[9px] uppercase tracking-wider text-mute mb-2">
                {cell.day} {monthAnchor.toLocaleDateString(undefined, { month: 'short' })}
              </div>
              {#each cell.events as ev}
                <div class="flex items-start gap-1.5 py-0.5">
                  <span
                    class="w-1.5 h-1.5 rounded-full shrink-0 mt-[3px]"
                    style="background:var(--color-accent)"
                  ></span>
                  <div class="flex-1 min-w-0">
                    <div class="text-xs text-ink leading-snug truncate">{ev.title}</div>
                    <div class="text-[9px] text-mute">
                      {ev.all_day ? 'all day' : `${formatTime(ev.start_time)} – ${formatTime(ev.end_time)}`}
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          {/if}

          <div
            class="text-[11px] tabular-nums leading-none
                   {cell.isToday ? 'text-accent font-semibold' : 'text-ink'}"
          >
            {cell.day}
          </div>
          {#if cell.events.length > 0}
            <div class="flex flex-wrap gap-0.5 mt-1">
              {#each cell.events.slice(0, 5) as ev}
                <span
                  class="w-1.5 h-1.5 rounded-full"
                  style="background:var(--color-accent)"
                ></span>
              {/each}
              {#if cell.events.length > 5}
                <span class="text-[9px] text-mute leading-none">
                  +{cell.events.length - 5}
                </span>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  /* Amber in light mode, theme accent in dark/space — mirrors Vault accent logic */
  .cal-root { --cal-accent: var(--color-accent); }

  .cal-label { color: var(--cal-accent); }

  .cal-tab-active {
    background: var(--cal-accent);
    color: #fff;
  }

  .cal-tab-border {
    border: 1px solid color-mix(in srgb, var(--cal-accent) 30%, transparent);
  }

  /* Popup needs to escape the grid cell stacking context */
  .cal-popup {
    box-shadow: 0 4px 16px rgba(0,0,0,0.12), 0 1px 4px rgba(0,0,0,0.08);
  }
</style>
