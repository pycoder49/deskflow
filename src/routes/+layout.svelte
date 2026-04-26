<script lang="ts">
  import '../app.css';
  import { theme } from '$lib/stores/theme';
  import { startTaskPolling } from '$lib/stores/refresh';
  import SpaceBackground from '$lib/widgets/SpaceBackground.svelte';

  let { children } = $props();

  $effect(() => startTaskPolling());
</script>

{#if $theme === 'space'}
  <SpaceBackground />
{/if}

<div class="relative min-h-screen" style="z-index: 1;">
  {@render children()}
</div>
