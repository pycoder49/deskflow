<script lang="ts">
  import '../app.css';
  import { theme } from '$lib/stores/theme';
  import { startTaskPolling, triggerStartupRetry } from '$lib/stores/refresh';
  import { loadConfig } from '$lib/stores/config';
  import SpaceBackground from '$lib/widgets/SpaceBackground.svelte';
  import ForestBackground from '$lib/widgets/ForestBackground.svelte';
  import CloudBackground from '$lib/widgets/CloudBackground.svelte';

  let { children } = $props();
  let view = $state<'dashboard' | 'workout' | 'research'>('dashboard');

  $effect(() => { loadConfig(); });
  $effect(() => startTaskPolling());
  $effect(() => {
    const timer = setTimeout(() => triggerStartupRetry(), 2500);
    return () => clearTimeout(timer);
  });
</script>

{#if $theme === 'space'}
  <SpaceBackground />
{/if}
{#if $theme === 'forest'}
  <ForestBackground />
{/if}
{#if $theme === 'cloudy'}
  <CloudBackground />
{/if}

<div class="relative flex h-screen overflow-hidden" style="z-index: 1;">
  <!-- Sidebar nav rail -->
  <nav class="flex flex-col items-center py-5 gap-1 border-r border-line bg-surface w-12 shrink-0" style="z-index: 10;">
    <button
      class="w-8 h-8 flex items-center justify-center rounded-lg transition-colors {view === 'dashboard' ? 'bg-accent/10 text-accent' : 'text-mute hover:text-ink hover:bg-line'}"
      title="Dashboard"
      onclick={() => view = 'dashboard'}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/>
        <rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/>
      </svg>
    </button>
    <button
      class="w-8 h-8 flex items-center justify-center rounded-lg transition-colors {view === 'workout' ? 'bg-accent/10 text-accent' : 'text-mute hover:text-ink hover:bg-line'}"
      title="Workout"
      onclick={() => view = 'workout'}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M6 4v16M18 4v16M6 8h12M6 16h12M2 8h2M20 8h2M2 16h2M20 16h2"/>
      </svg>
    </button>
    <button
      class="w-8 h-8 flex items-center justify-center rounded-lg transition-colors {view === 'research' ? 'bg-accent/10 text-accent' : 'text-mute hover:text-ink hover:bg-line'}"
      title="Research"
      onclick={() => view = 'research'}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
      </svg>
    </button>
  </nav>

  <!-- Content area -->
  <div class="flex-1 overflow-auto">
    {#if view === 'dashboard'}
      {@render children()}
    {:else}
      <div class="flex items-center justify-center h-full text-mute text-sm">
        {view.charAt(0).toUpperCase() + view.slice(1)} — coming soon
      </div>
    {/if}
  </div>
</div>
