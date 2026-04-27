<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { getProjects, addProject, removeProject, type Project } from '$lib/services/projects';

  type ContextLine = {
    type: 'h1' | 'h2' | 'h3' | 'text' | 'bullet' | 'hr' | 'blank';
    content?: string;
  };

  const DOT_COLORS = ['#60a5fa', '#34d399', '#a78bfa', '#fb923c', '#f472b6'];
  function dotColor(i: number) { return DOT_COLORS[i % DOT_COLORS.length]; }

  let projects = $state<Project[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let expandedId = $state<string | null>(null);

  onMount(async () => {
    try {
      projects = await getProjects();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  async function handlePickFolder() {
    const path = await open({ directory: true, multiple: false });
    if (!path) return;
    try {
      projects = await addProject(path as string);
    } catch (e) {
      error = String(e);
    }
  }

  async function handleRemove(id: string) {
    try {
      projects = await removeProject(id);
      if (expandedId === id) expandedId = null;
    } catch (e) {
      error = String(e);
    }
  }

  function toggle(id: string) {
    expandedId = expandedId === id ? null : id;
  }

  function renderInline(s: string): string {
    return s
      .replace(/[&<>"']/g, (c) =>
        ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!
      )
      .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
      .replace(/`([^`]+)`/g, '<code class="px-1 rounded bg-line/40 text-[11px]">$1</code>');
  }

  function parseLines(md: string): ContextLine[] {
    return md.split('\n').map((line): ContextLine => {
      if (/^### /.test(line)) return { type: 'h3' as const, content: renderInline(line.slice(4)) };
      if (/^## /.test(line)) return { type: 'h2' as const, content: renderInline(line.slice(3)) };
      if (/^# /.test(line)) return { type: 'h1' as const, content: renderInline(line.slice(2)) };
      if (/^---/.test(line)) return { type: 'hr' as const };
      if (/^[-*] /.test(line)) return { type: 'bullet' as const, content: renderInline(line.slice(2)) };
      if (/^\d+\. /.test(line)) return { type: 'bullet' as const, content: renderInline(line.replace(/^\d+\. /, '')) };
      if (line.trim() === '') return { type: 'blank' as const };
      return { type: 'text' as const, content: renderInline(line) };
    });
  }

</script>

<div class="flex flex-col h-full">
  <!-- Header -->
  <div class="flex items-center justify-between mb-3">
    <h2 class="text-xs uppercase tracking-wider text-accent">Projects</h2>
    <button
      class="text-xs px-2 py-0.5 rounded border border-line hover:bg-surface transition text-mute hover:text-ink"
      onclick={handlePickFolder}
    >
      + Add
    </button>
  </div>

  {#if loading}
    <p class="text-xs text-mute italic">Loading…</p>
  {:else if error}
    <p class="text-xs text-red-400">{error}</p>
  {:else if projects.length === 0}
    <p class="text-sm text-mute italic">No projects yet — click + Add to get started.</p>
  {:else}
    <div class="border border-line rounded-lg overflow-hidden divide-y divide-line">
      {#each projects as project, i (project.id)}
        <div>
          <!-- Row -->
          <div class="project-row flex items-center px-3 py-2 gap-2 transition" style="--dot:{dotColor(i)}">
            <button
              class="flex-1 min-w-0 text-left flex items-center gap-2"
              onclick={() => toggle(project.id)}
            >
              <span
                class="w-6 h-6 rounded-full shrink-0 flex items-center justify-center text-[11px] font-bold uppercase"
                style="background-color:color-mix(in srgb,{dotColor(i)} 18%,transparent);color:{dotColor(i)}"
              >{project.name[0]}</span>
              <span class="text-sm font-medium text-ink truncate">{project.name}</span>
            </button>
            <span class="text-[10px] text-mute pointer-events-none select-none">
              {expandedId === project.id ? '▲' : '▼'}
            </span>
            <button
              class="text-base leading-none text-mute hover:text-red-400 transition px-1"
              onclick={() => handleRemove(project.id)}
              title="Remove project"
            >×</button>
          </div>

          <!-- Expanded: context.md -->
          {#if expandedId === project.id}
            <div class="border-t border-line px-3 py-3 max-h-80 overflow-y-auto bg-surface/40 border-l-2" style="border-left-color:{dotColor(i)}">
              {#if project.context}
                <div class="space-y-0.5">
                  {#each parseLines(project.context) as line}
                    {#if line.type === 'h1'}
                      <p class="font-bold text-sm mt-4 mb-1 text-ink">{@html line.content ?? ''}</p>
                    {:else if line.type === 'h2'}
                      <p class="font-semibold text-[11px] mt-4 mb-1 text-ink uppercase tracking-wide">{@html line.content ?? ''}</p>
                    {:else if line.type === 'h3'}
                      <p class="font-semibold text-xs mt-3 mb-0.5 text-ink">{@html line.content ?? ''}</p>
                    {:else if line.type === 'hr'}
                      <hr class="border-line my-2" />
                    {:else if line.type === 'bullet'}
                      <p class="text-xs leading-snug pl-3 flex gap-1.5">
                        <span class="text-mute shrink-0">·</span>
                        <span>{@html line.content ?? ''}</span>
                      </p>
                    {:else if line.type === 'blank'}
                      <div class="h-1.5"></div>
                    {:else}
                      <p class="text-xs leading-snug text-ink/80">{@html line.content ?? ''}</p>
                    {/if}
                  {/each}
                </div>
              {:else}
                <p class="text-xs text-mute italic">No <code class="px-1 rounded bg-line/40 text-[11px]">context.md</code> found.</p>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .project-row:hover {
    background-color: color-mix(in srgb, var(--dot) 8%, transparent);
  }
</style>

