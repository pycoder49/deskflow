<script lang="ts">
  import { onMount } from 'svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import {
    forceSimulation,
    forceLink,
    forceManyBody,
    forceCenter,
    forceCollide,
    forceX,
    forceY,
    type SimulationNodeDatum,
  } from 'd3-force';
  import { getVaultPulse, obsidianUri, type VaultPulse, type GraphNode } from '$lib/services/vault';

  type View = 'hot' | 'inbox' | 'graph';

  let pulse = $state<VaultPulse | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let view = $state<View>('hot');

  // ─── Graph simulation ───────────────────────────────────────────────────────
  type SimNode = SimulationNodeDatum & GraphNode;

  let graphDiv = $state<HTMLDivElement | null>(null);
  let simNodes = $state<SimNode[]>([]);
  let simEdges = $state<Array<{ sx: number; sy: number; tx: number; ty: number }>>([]);

  // ─── Pan / zoom / drag ──────────────────────────────────────────────────────
  let svgEl = $state<SVGSVGElement | null>(null);
  let vt = $state({ x: 0, y: 0, k: 1 });
  let isPanning = $state(false);
  let isDragging = $state(false);
  let panOrigin = { x: 0, y: 0 };
  let dragNode: SimNode | null = null;
  let activeSim: ReturnType<typeof forceSimulation<SimNode>> | null = null;
  let activeNodes: SimNode[] = [];

  function toLocal(cx: number, cy: number) {
    const rect = svgEl!.getBoundingClientRect();
    return { x: (cx - rect.left - vt.x) / vt.k, y: (cy - rect.top - vt.y) / vt.k };
  }

  function onWheel(e: WheelEvent) {
    e.preventDefault();
    const factor = e.deltaY < 0 ? 1.12 : 1 / 1.12;
    const newK = Math.max(0.15, Math.min(6, vt.k * factor));
    const rect = svgEl!.getBoundingClientRect();
    const mx = e.clientX - rect.left;
    const my = e.clientY - rect.top;
    vt = { x: mx + (vt.x - mx) * (newK / vt.k), y: my + (vt.y - my) * (newK / vt.k), k: newK };
  }

  function onSvgDown(e: PointerEvent) {
    if ((e.target as Element).closest('.gnode')) return;
    isPanning = true;
    panOrigin = { x: e.clientX - vt.x, y: e.clientY - vt.y };
    (e.currentTarget as SVGSVGElement).setPointerCapture(e.pointerId);
  }

  function onSvgMove(e: PointerEvent) {
    if (isPanning) {
      vt = { ...vt, x: e.clientX - panOrigin.x, y: e.clientY - panOrigin.y };
    } else if (dragNode) {
      const { x, y } = toLocal(e.clientX, e.clientY);
      dragNode.fx = x;
      dragNode.fy = y;
      dragNode.x = x;
      dragNode.y = y;
      simNodes = activeNodes.map(n => ({ ...n }));
    }
  }

  function onSvgUp() {
    isPanning = false;
    if (dragNode) {
      // keep fx/fy set — node stays pinned where dropped (Obsidian-style)
      activeSim?.alphaTarget(0);
      dragNode = null;
      isDragging = false;
    }
  }

  function onNodeDown(e: PointerEvent, nodeId: string) {
    e.stopPropagation();
    const node = activeNodes.find(n => n.id === nodeId);
    if (!node) return;
    dragNode = node;
    isDragging = true;
    node.fx = node.x;
    node.fy = node.y;
    activeSim?.alphaTarget(0.3).restart();
    (e.currentTarget as SVGGElement).setPointerCapture(e.pointerId);
  }

  const NODE_TYPES = ['source', 'topic', 'entity', 'query', 'raw'] as const;
  function nodeColor(t: string) { return `var(--node-${t}-color, #94a3b8)`; }

  $effect(() => {
    if (view !== 'graph' || !pulse?.graph || !graphDiv) return;
    const { width: W, height: H } = graphDiv.getBoundingClientRect();
    if (!W || !H) return;

    const nodes: SimNode[] = pulse.graph.nodes.map(n => ({
      ...n,
      x: W / 2 + (Math.random() - 0.5) * W * 0.4,
      y: H / 2 + (Math.random() - 0.5) * H * 0.4,
    }));
    const idSet = new Set(nodes.map(n => n.id));
    const rawLinks = pulse.graph.edges
      .filter(e => idSet.has(e.source) && idSet.has(e.target))
      .map(e => ({ source: e.source, target: e.target }));

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const linkForce = (forceLink(rawLinks) as any)
      .id((d: SimNode) => d.id)
      .distance(70)
      .strength(0.5);

    // Nodes that appear in at least one edge
    const connectedIds = new Set<string>(
      rawLinks.flatMap((l: { source: string; target: string }) => [l.source, l.target])
    );
    const isOrphan = (n: SimNode) => !connectedIds.has(n.id);

    const sim = forceSimulation<SimNode>(nodes)
      .force('link', linkForce)
      .force('charge', forceManyBody<SimNode>().strength(-130))
      .force('center', forceCenter(W / 2, H / 2))
      .force('collision', forceCollide<SimNode>(20))
      // Pull orphan nodes toward center so they don't drift to the edges
      .force('orphan-x', forceX<SimNode>(W / 2).strength(n => isOrphan(n) ? 0.2 : 0))
      .force('orphan-y', forceY<SimNode>(H / 2).strength(n => isOrphan(n) ? 0.2 : 0))
      .alphaDecay(0.025);

    activeSim = sim;
    activeNodes = nodes;

    function flush() {
      simNodes = nodes.map(n => ({ ...n }));
      simEdges = linkForce.links().map((l: any) => ({
        sx: (l.source as SimNode).x ?? 0,
        sy: (l.source as SimNode).y ?? 0,
        tx: (l.target as SimNode).x ?? 0,
        ty: (l.target as SimNode).y ?? 0,
      }));
    }
    sim.on('tick', flush).on('end', flush);

    return () => { sim.stop(); activeSim = null; activeNodes = []; };
  });

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

    <!-- ─── GRAPH TAB ────────────────────────────────────────── -->
    {:else if view === 'graph'}
      <div class="flex-1 min-h-0 relative overflow-hidden rounded-lg border border-line" bind:this={graphDiv}>
        {#if !pulse.graph || pulse.graph.nodes.length === 0}
          <div class="absolute inset-0 flex items-center justify-center text-mute italic text-sm">
            No graph data
          </div>
        {:else}
          <svg
            class="w-full h-full select-none"
            class:cursor-grab={!isPanning && !isDragging}
            class:cursor-grabbing={isPanning || isDragging}
            bind:this={svgEl}
            onwheel={onWheel}
            onpointerdown={onSvgDown}
            onpointermove={onSvgMove}
            onpointerup={onSvgUp}
          >
            <g transform="translate({vt.x},{vt.y}) scale({vt.k})">
              {#each simEdges as edge, i (i)}
                <line
                  x1={edge.sx} y1={edge.sy}
                  x2={edge.tx} y2={edge.ty}
                  stroke="currentColor"
                  stroke-width="1"
                  stroke-opacity="0.18"
                />
              {/each}
              {#each simNodes as node (node.id)}
                <g
                  class="gnode"
                  transform="translate({node.x ?? 0},{node.y ?? 0})"
                  onpointerdown={(e) => onNodeDown(e, node.id)}
                >
                  <circle r="6" style="fill:{nodeColor(node.node_type)}" fill-opacity="0.88" />
                  <text
                    y="16"
                    text-anchor="middle"
                    font-size="8"
                    fill="currentColor"
                    fill-opacity="0.6"
                    class="pointer-events-none"
                  >{node.label.length > 14 ? node.label.slice(0, 13) + '…' : node.label}</text>
                </g>
              {/each}
            </g>
          </svg>
          <div class="absolute bottom-2 right-2 flex gap-2.5">
            {#each NODE_TYPES as type}
              <div class="flex items-center gap-1 text-[9px] text-mute">
                <span class="w-2 h-2 rounded-full shrink-0" style="background:{nodeColor(type)}"></span>
                {type}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  {/if}
</div>

<style>
  /* Amber (light) / purple (dark/space). */
  .vault-root { --vault-accent: #b87333; }
  :global(.dark) .vault-root,
  :global(.space) .vault-root { --vault-accent: #a78bfa; }

  /* Graph node colors — darker in light mode for contrast on white bg */
  .vault-root {
    --node-source-color: #b45309;
    --node-topic-color:  #1d4ed8;
    --node-entity-color: #059669;
    --node-query-color:  #dc2626;
    --node-raw-color:    #64748b;
  }
  :global(.dark) .vault-root,
  :global(.space) .vault-root {
    --node-source-color: #f59e0b;
    --node-topic-color:  #60a5fa;
    --node-entity-color: #34d399;
    --node-query-color:  #f87171;
    --node-raw-color:    #94a3b8;
  }

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
