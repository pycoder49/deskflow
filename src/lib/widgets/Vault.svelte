<script lang="ts">
  import { onMount } from 'svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { getVaultPulse, obsidianUri, type VaultPulse } from '$lib/services/vault';

  type View = 'hot' | 'inbox' | 'graph';

  let pulse = $state<VaultPulse | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let view = $state<View>('hot');

  async function load() {
    loading = true;
    error = null;
    try {
      pulse = await getVaultPulse();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  onMount(load);

  function open(path: string) {
    if (!pulse) return;
    openUrl(obsidianUri(pulse.vault_name, path)).catch((e) => console.warn(e));
  }

  // Strict-ish inline markdown: escape, then re-introduce <strong> and <code>.
  function renderInline(s: string): string {
    const esc = s.replace(/[&<>"']/g, (c) =>
      ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!
    );
    return esc
      .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
      .replace(
        /`([^`]+)`/g,
        '<code class="px-1 rounded bg-line/40 text-[11px]">$1</code>'
      );
  }

  // Parse bullets with indent depth. Lines like "  - foo" become depth=1.
  // Two-space indents per level (markdown convention).
  function bullets(md: string): { depth: number; text: string }[] {
    const out: { depth: number; text: string }[] = [];
    for (const line of md.split('\n')) {
      const m = line.match(/^( *)- (.*)$/);
      if (!m) continue;
      out.push({ depth: Math.floor(m[1].length / 2), text: m[2] });
    }
    return out;
  }

  function shortDate(iso: string): string {
    const d = new Date(iso + 'T12:00:00');
    if (isNaN(d.getTime())) return iso;
    return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
  }

  // Each hot.md section is either bullets (possibly nested) or prose.
  function sectionBlocks(body: string) {
    const bs = bullets(body);
    return bs.length > 0
      ? { kind: 'bullets' as const, bullets: bs }
      : { kind: 'prose' as const, prose: body.trim() };
  }
</script>

<div class="vault-root h-full flex flex-col">
  <!-- Header: title + tab toggle + scheme swatches + refresh -->
  <div class="flex items-center justify-between mb-3">
    <h2 class="text-xs uppercase tracking-wider vault-label">Vault</h2>
    <div class="flex items-center gap-3 text-xs">
      <div class="flex border border-line rounded-md p-0.5 gap-0.5">
        {#each ['hot', 'inbox', 'graph'] as v (v)}
          <button
            class="px-2 py-0.5 rounded transition capitalize
                   {view === v ? 'vault-tab-active' : 'text-mute hover:text-ink'}"
            onclick={() => (view = v as View)}
          >{v}</button>
        {/each}
      </div>
      <button
        class="text-mute hover:text-ink transition"
        onclick={load}
        title="Refresh"
        disabled={loading}
        aria-label="Refresh vault"
      >↻</button>
    </div>
  </div>

  {#if error}
    <p class="text-sm text-red-400">Failed to load vault: {error}</p>
  {:else if loading && !pulse}
    <p class="text-sm text-mute italic">Loading…</p>
  {:else if pulse}

    <!-- ─── HOT TAB ───────────────────────────────────────────── -->
    {#if view === 'hot'}
      <div class="flex items-baseline justify-between mb-3 text-[11px] text-mute">
        <span>
          {pulse.counts.sources} sources · {pulse.counts.topics} topics ·
          {pulse.counts.entities} entities · {pulse.counts.queries} queries
        </span>
        {#if pulse.hot_session}
          <span class="truncate ml-3">{pulse.hot_session}</span>
        {/if}
      </div>

      {#if pulse.hot_sections.length > 0}
        <div class="flex-1 min-h-0 border border-line rounded-lg p-4 overflow-y-auto space-y-4">
          {#each pulse.hot_sections as section}
            {@const block = sectionBlocks(section.body)}
            <div>
              <div class="text-[10px] uppercase tracking-wider text-mute mb-1.5">
                {section.title}
              </div>
              {#if block.kind === 'bullets'}
                <ul class="space-y-1.5 text-sm leading-snug">
                  {#each block.bullets as b}
                    <li
                      class="flex gap-2"
                      style="padding-left: {b.depth * 0.9}rem"
                    >
                      <span class="text-mute select-none">{b.depth > 0 ? '◦' : '•'}</span>
                      <span>{@html renderInline(b.text)}</span>
                    </li>
                  {/each}
                </ul>
              {:else}
                <p class="text-sm leading-snug">{@html renderInline(block.prose)}</p>
              {/if}
            </div>
          {/each}
        </div>
      {:else}
        <p class="text-sm text-mute italic">No active session context in hot.md.</p>
      {/if}

    <!-- ─── INBOX TAB (inbox + recent) ────────────────────────── -->
    {:else if view === 'inbox'}
      <div class="flex-1 min-h-0 flex flex-col">
        <div class="text-[10px] uppercase tracking-wider text-mute mb-1.5 flex items-center gap-2">
          <span>Inbox</span>
          <span class="vault-badge px-1.5 rounded-full text-[10px] tracking-normal">{pulse.inbox.length}</span>
        </div>
        {#if pulse.inbox.length === 0}
          <p class="text-xs text-mute italic">All clear — nothing waiting to ingest.</p>
        {:else}
          <ul class="flex-1 overflow-y-auto space-y-0.5 pr-1">
            {#each pulse.inbox as item}
              <li>
                <button
                  class="w-full flex items-center justify-between gap-3 px-2 py-1 rounded hover:bg-line/30 text-left text-sm transition"
                  onclick={() => open(item.path)}
                  title={item.path}
                >
                  <span class="truncate">{item.name}</span>
                  <span class="text-[11px] text-mute shrink-0">{item.modified}</span>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>

      {#if pulse.recent_log.length > 0}
        <div class="border-t border-line pt-2 mt-3">
          <div class="text-[10px] uppercase tracking-wider text-mute mb-1.5">Recent</div>
          <ul class="space-y-1 text-xs">
            {#each pulse.recent_log.slice(0, 4) as day}
              {#each day.bullets.slice(0, 1) as bullet}
                <li class="flex gap-2 leading-snug">
                  <span class="text-mute shrink-0 w-12">{shortDate(day.date)}</span>
                  <span class="truncate">{@html renderInline(bullet)}</span>
                </li>
              {/each}
            {/each}
          </ul>
        </div>
      {/if}

    <!-- ─── GRAPH TAB (placeholder) ──────────────────────────── -->
    {:else if view === 'graph'}
      <div class="flex-1 min-h-0 flex items-center justify-center text-mute italic text-sm">
        Coming soon
      </div>
    {/if}
  {/if}
</div>

<style>
  /* Amber (light) / purple (dark/space). */
  .vault-root { --vault-accent: #b87333; }
  :global(.dark) .vault-root,
  :global(.space) .vault-root { --vault-accent: #a78bfa; }

  .vault-label { color: var(--vault-accent); }

  .vault-tab-active {
    background: var(--vault-accent);
    color: #fff;
  }

  .vault-badge {
    background: color-mix(in srgb, var(--vault-accent) 18%, transparent);
    color: var(--vault-accent);
  }
</style>
