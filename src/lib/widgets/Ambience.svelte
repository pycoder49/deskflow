<script lang="ts">
  import { onMount } from 'svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { visualEffect, type VisualEffect } from '$lib/stores/ambience';

  type Tab = 'sounds' | 'music' | 'visuals';
  type SoundPreset = 'rain' | 'white' | 'brown' | 'pink';

  let tab = $state<Tab>('sounds');
  let playing = $state<SoundPreset | null>(null);
  let volume = $state(0.5);

  // ─── Web Audio ───────────────────────────────────────────────────────────────
  let audioCtx: AudioContext | null = null;
  let gainNode: GainNode | null = null;
  let activeSource: AudioBufferSourceNode | null = null;
  let activeFilter: BiquadFilterNode | null = null;

  function getCtx(): AudioContext {
    if (!audioCtx) {
      audioCtx = new AudioContext();
      gainNode = audioCtx.createGain();
      gainNode.gain.value = volume;
      gainNode.connect(audioCtx.destination);
    }
    return audioCtx;
  }

  function makeWhiteBuffer(ctx: AudioContext): AudioBuffer {
    const len = ctx.sampleRate * 3;
    const buf = ctx.createBuffer(1, len, ctx.sampleRate);
    const data = buf.getChannelData(0);
    for (let i = 0; i < len; i++) data[i] = Math.random() * 2 - 1;
    return buf;
  }

  function makeBrownBuffer(ctx: AudioContext): AudioBuffer {
    const len = ctx.sampleRate * 3;
    const buf = ctx.createBuffer(1, len, ctx.sampleRate);
    const data = buf.getChannelData(0);
    let last = 0;
    for (let i = 0; i < len; i++) {
      last = (last + 0.02 * (Math.random() * 2 - 1)) / 1.02;
      data[i] = Math.min(1, Math.max(-1, last * 3.5));
    }
    return buf;
  }

  function makePinkBuffer(ctx: AudioContext): AudioBuffer {
    const len = ctx.sampleRate * 3;
    const buf = ctx.createBuffer(1, len, ctx.sampleRate);
    const data = buf.getChannelData(0);
    let b0 = 0, b1 = 0, b2 = 0, b3 = 0, b4 = 0, b5 = 0, b6 = 0;
    for (let i = 0; i < len; i++) {
      const w = Math.random() * 2 - 1;
      b0 = 0.99886 * b0 + w * 0.0555179;
      b1 = 0.99332 * b1 + w * 0.0750759;
      b2 = 0.96900 * b2 + w * 0.1538520;
      b3 = 0.86650 * b3 + w * 0.3104856;
      b4 = 0.55000 * b4 + w * 0.5329522;
      b5 = -0.7616 * b5 - w * 0.0168980;
      b6 = w * 0.115926;
      data[i] = Math.min(1, Math.max(-1, (b0 + b1 + b2 + b3 + b4 + b5 + b6 + w * 0.5362) * 0.11));
    }
    return buf;
  }

  function stopCurrent() {
    if (activeSource) {
      try { activeSource.stop(); } catch {}
      activeSource.disconnect();
      activeSource = null;
    }
    if (activeFilter) {
      activeFilter.disconnect();
      activeFilter = null;
    }
  }

  function playPreset(preset: SoundPreset) {
    if (playing === preset) {
      stopCurrent();
      playing = null;
      return;
    }
    stopCurrent();
    const ctx = getCtx();
    if (ctx.state === 'suspended') ctx.resume();

    let buf: AudioBuffer;
    if (preset === 'brown') buf = makeBrownBuffer(ctx);
    else if (preset === 'pink') buf = makePinkBuffer(ctx);
    else buf = makeWhiteBuffer(ctx);

    const src = ctx.createBufferSource();
    src.buffer = buf;
    src.loop = true;

    if (preset === 'rain') {
      const filter = ctx.createBiquadFilter();
      filter.type = 'lowpass';
      filter.frequency.value = 450;
      filter.Q.value = 0.8;
      const filter2 = ctx.createBiquadFilter();
      filter2.type = 'highpass';
      filter2.frequency.value = 80;
      src.connect(filter);
      filter.connect(filter2);
      filter2.connect(gainNode!);
      activeFilter = filter;
    } else {
      src.connect(gainNode!);
    }

    src.start();
    activeSource = src;
    playing = preset;
  }

  $effect(() => {
    if (gainNode) gainNode.gain.value = volume;
  });

  // ─── Visuals ─────────────────────────────────────────────────────────────────
  const effects: { id: VisualEffect; label: string; emoji: string; desc: string }[] = [
    { id: 'none',      label: 'Off',       emoji: '◯',  desc: 'No effect' },
    { id: 'aurora',    label: 'Aurora',    emoji: '🌌', desc: 'Northern lights' },
    { id: 'particles', label: 'Particles', emoji: '✦',  desc: 'Floating dust' },
    { id: 'rain',      label: 'Rain',      emoji: '🌧',  desc: 'Falling drops' },
    { id: 'matrix',    label: 'Matrix',    emoji: '⬛', desc: 'Green cascade' },
  ];

  const sounds: { id: SoundPreset; label: string; emoji: string; desc: string }[] = [
    { id: 'rain',  label: 'Rain',        emoji: '🌧', desc: 'Filtered noise' },
    { id: 'white', label: 'White',       emoji: '〰', desc: 'Flat spectrum' },
    { id: 'brown', label: 'Brown',       emoji: '🟤', desc: 'Deep rumble' },
    { id: 'pink',  label: 'Pink',        emoji: '🌸', desc: 'Balanced warm' },
  ];

  onMount(() => {
    return () => {
      stopCurrent();
      audioCtx?.close();
    };
  });
</script>

<div class="flex flex-col h-full gap-3">
  <!-- Tab bar -->
  <div class="flex gap-1 border-b border-line pb-2 shrink-0">
    {#each (['sounds', 'music', 'visuals'] as Tab[]) as t}
      <button
        onclick={() => (tab = t)}
        class="px-3 py-1 rounded-md text-xs font-medium capitalize transition {tab === t
          ? 'bg-accent text-white'
          : 'text-mute hover:text-ink hover:bg-surface'}"
      >
        {t}
      </button>
    {/each}
  </div>

  <!-- Sounds tab -->
  {#if tab === 'sounds'}
    <div class="flex flex-col gap-3 flex-1 min-h-0">
      <div class="grid grid-cols-2 gap-2">
        {#each sounds as s}
          <button
            onclick={() => playPreset(s.id)}
            class="flex flex-col items-start gap-0.5 p-3 rounded-xl border transition {playing === s.id
              ? 'border-accent bg-accent/10 text-accent'
              : 'border-line hover:border-accent/40 hover:bg-surface'}"
          >
            <div class="flex items-center justify-between w-full">
              <span class="text-lg leading-none">{s.emoji}</span>
              {#if playing === s.id}
                <span class="flex gap-0.5 items-end h-4">
                  {#each [1, 2, 3] as i}
                    <span
                      class="w-0.5 rounded-full bg-accent animate-bounce"
                      style="height: {8 + i * 3}px; animation-delay: {i * 0.1}s;"
                    ></span>
                  {/each}
                </span>
              {/if}
            </div>
            <span class="text-xs font-semibold">{s.label}</span>
            <span class="text-[10px] text-mute">{s.desc}</span>
          </button>
        {/each}
      </div>

      <!-- Lo-fi link -->
      <button
        onclick={() => openUrl('https://www.youtube.com/watch?v=jfKfPfyJRdk')}
        class="flex items-center gap-2 px-3 py-2 rounded-xl border border-line hover:border-accent/40 hover:bg-surface transition text-xs text-mute"
      >
        <span class="text-base">🎵</span>
        <span>Lo-fi Radio</span>
        <span class="ml-auto opacity-50">↗</span>
      </button>

      <!-- Volume -->
      <div class="flex items-center gap-3 mt-auto">
        <span class="text-xs text-mute shrink-0">Vol</span>
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          bind:value={volume}
          class="flex-1 accent-accent h-1 cursor-pointer"
        />
        <span class="text-xs text-mute w-6 text-right">{Math.round(volume * 100)}</span>
      </div>
    </div>

  <!-- Music tab -->
  {:else if tab === 'music'}
    <div class="flex flex-col items-center justify-center flex-1 gap-4 text-center">
      <div class="w-16 h-16 rounded-2xl flex items-center justify-center text-3xl"
           style="background: linear-gradient(135deg, #1db954 0%, #191414 100%);">
        &#9835;
      </div>
      <div>
        <p class="text-sm font-semibold text-ink">Spotify</p>
        <p class="text-xs text-mute mt-0.5">Now Playing controls</p>
      </div>
      <span class="px-3 py-1 rounded-full text-[10px] font-semibold tracking-wider border border-line text-mute">
        COMING SOON
      </span>
      <p class="text-[10px] text-mute max-w-[180px] leading-relaxed">
        Spotify Web API integration with OAuth — picking track, controls, and queue display.
      </p>
    </div>

  <!-- Visuals tab -->
  {:else}
    <div class="flex flex-col gap-2 flex-1 min-h-0">
      <p class="text-[10px] text-mute">Background effect — applies across the dashboard</p>
      <div class="grid grid-cols-1 gap-1.5 overflow-y-auto">
        {#each effects as fx}
          <button
            onclick={() => visualEffect.set(fx.id)}
            class="flex items-center gap-3 px-3 py-2.5 rounded-xl border transition text-left {$visualEffect === fx.id
              ? 'border-accent bg-accent/10'
              : 'border-line hover:border-accent/40 hover:bg-surface'}"
          >
            <span class="text-base w-6 text-center shrink-0">{fx.emoji}</span>
            <div class="flex flex-col min-w-0">
              <span class="text-xs font-semibold {$visualEffect === fx.id ? 'text-accent' : 'text-ink'}">{fx.label}</span>
              <span class="text-[10px] text-mute">{fx.desc}</span>
            </div>
            {#if $visualEffect === fx.id}
              <span class="ml-auto text-accent text-xs">●</span>
            {/if}
          </button>
        {/each}
      </div>
    </div>
  {/if}
</div>
