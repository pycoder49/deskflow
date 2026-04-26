<script lang="ts">
  import { onMount } from 'svelte';

  let now = $state(new Date());

  onMount(() => {
    const id = setInterval(() => { now = new Date(); }, 1000);
    return () => clearInterval(id);
  });

  const timeStr = $derived(
    now.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })
  );
  const secondsStr = $derived(
    now.toLocaleTimeString(undefined, { second: '2-digit' })
  );
  const dateStr = $derived(
    now.toLocaleDateString(undefined, { weekday: 'long', month: 'long', day: 'numeric' })
  );
</script>

<section
  class="bg-surface border border-line rounded-xl p-6 h-full flex flex-col justify-center items-center"
  aria-label="Clock"
>
  <div class="text-6xl font-semibold tracking-tight tabular-nums">
    {timeStr}<span class="text-2xl text-mute align-top ml-1">:{secondsStr}</span>
  </div>
  <div class="text-sm text-mute mt-2">{dateStr}</div>
</section>
