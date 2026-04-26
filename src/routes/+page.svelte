<script lang="ts">
  import { theme } from '$lib/stores/theme';
  import QuickCaptureModal from '$lib/widgets/QuickCaptureModal.svelte';
  import AmbientPanel from '$lib/widgets/AmbientPanel.svelte';
  import TodayTasks from '$lib/widgets/TodayTasks.svelte';
  import NowNext from '$lib/widgets/NowNext.svelte';
  import Notepad from '$lib/widgets/Notepad.svelte';
  import Calendar from '$lib/widgets/Calendar.svelte';
  import Vault from '$lib/widgets/Vault.svelte';

  let captureOpen = $state(false);

  const today = new Date().toLocaleDateString(undefined, {
    weekday: 'long', month: 'long', day: 'numeric',
  });

  const themeLabel = $derived(
    $theme === 'light' ? '☀  Light' : $theme === 'dark' ? '☾  Dark' : '✦  Space'
  );

  function handleKey(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'n' && !captureOpen) {
      e.preventDefault();
      captureOpen = true;
    }
  }
</script>

<svelte:window onkeydown={handleKey} />

<header class="flex items-center justify-between px-8 py-5 border-b border-line">
  <div class="flex items-baseline gap-4">
    <h1 class="text-2xl font-semibold tracking-tight">Aryan OS</h1>
    <span class="text-sm text-mute">{today}</span>
  </div>
  <div class="flex items-center gap-2">
    <button
      class="px-3 py-1.5 rounded-md border border-line text-sm hover:bg-surface transition flex items-center gap-2"
      onclick={() => (captureOpen = true)}
      title="Ctrl+N"
    >
      <span class="text-lg leading-none">+</span>
      <span>New Task</span>
      <span class="text-xs text-mute">Ctrl+N</span>
    </button>
    <button
      class="px-3 py-1.5 rounded-md border border-line text-sm hover:bg-surface transition"
      onclick={() => theme.cycle()}
    >
      {themeLabel}
    </button>
  </div>
</header>

<main class="p-8 grid grid-cols-12 gap-6 auto-rows-[minmax(8rem,auto)]">
  <!-- Row 1: Today / Now+Next / Notepad -->
  <section class="col-span-4 bg-surface border border-line rounded-xl p-5 min-h-64">
    <TodayTasks />
  </section>

  <section class="col-span-4 bg-surface border border-line rounded-xl p-5 min-h-64">
    <NowNext />
  </section>

  <section class="col-span-4 bg-surface border border-line rounded-xl p-5 min-h-64">
    <Notepad />
  </section>

  <!-- Row 2: Calendar / Vault -->
  <section class="col-span-7 bg-surface border border-line rounded-xl p-5 h-[28rem]">
    <Calendar />
  </section>

  <section class="col-span-5 bg-surface border border-line rounded-xl p-5 h-[28rem]">
    <Vault />
  </section>
</main>

<!-- Bottom: Projects / Ambient -->
<section class="px-8 pb-8 grid grid-cols-12 gap-6">
  <div class="col-span-7 flex flex-col gap-6">
    <section class="bg-surface border border-line rounded-xl p-5 min-h-32">
      <h2 class="text-xs uppercase tracking-wider text-mute">Projects</h2>
      <p class="mt-3 text-mute italic text-sm">Coming soon</p>
    </section>
    <section class="bg-surface border border-line rounded-xl p-5 min-h-32">
      <h2 class="text-xs uppercase tracking-wider text-mute">Workouts</h2>
      <p class="mt-3 text-mute italic text-sm">Coming soon</p>
    </section>
    <section class="bg-surface border border-line rounded-xl p-5 min-h-32">
      <h2 class="text-xs uppercase tracking-wider text-mute">Claude Terminal</h2>
      <p class="mt-3 text-mute italic text-sm">Coming soon</p>
    </section>
  </div>
  <div class="col-span-5">
    <AmbientPanel />
  </div>
</section>

<QuickCaptureModal bind:open={captureOpen} onCreated={() => {}} />
