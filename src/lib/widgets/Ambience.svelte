<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { backgroundEffect, type BackgroundEffect, sectionEffect, type SectionEffect } from '$lib/stores/ambience';
  import { theme } from '$lib/stores/theme';
  import 'xterm/css/xterm.css';

  type Tab = 'sounds' | 'visuals' | 'terminal';

  let tab = $state<Tab>('sounds');
  let volume = $state(0.5);

  // ─── Sounds ──────────────────────────────────────────────────────────────────
  const SOUNDS = [
    { id: 'cafe',      label: 'Coffee Shop', emoji: '☕', desc: 'Café background noise',    file: '/sounds/cafe.mp3' },
    { id: 'rain',      label: 'Rain & Storm', emoji: '🌧', desc: 'Heavy rain with thunder',  file: '/sounds/rain.mp3' },
    { id: 'medieval',  label: 'Medieval',    emoji: '🏰', desc: 'Celtic ambient music',      file: '/sounds/medieval.mp3' },
    { id: 'cyberpunk', label: 'Synthwave',   emoji: '🌆', desc: 'Cyberpunk retrowave',       file: '/sounds/cyberpunk.mp3' },
  ] as const;

  type SoundId = typeof SOUNDS[number]['id'];
  let playing = $state<Record<SoundId, boolean>>({ cafe: false, rain: false, medieval: false, cyberpunk: false });
  let audios: Partial<Record<SoundId, HTMLAudioElement>> = {};

  function toggleSound(id: SoundId) {
    if (playing[id]) {
      audios[id]?.pause();
      delete audios[id];
      playing[id] = false;
    } else {
      const sound = SOUNDS.find(s => s.id === id)!;
      const audio = new Audio(sound.file);
      audio.loop = true;
      audio.volume = volume;
      audio.play().catch(() => {});
      audios[id] = audio;
      playing[id] = true;
    }
  }

  $effect(() => {
    Object.values(audios).forEach(a => { if (a) a.volume = volume; });
  });

  function applyPreset(sound: SoundId | null, bg: BackgroundEffect, sec: SectionEffect) {
    for (const id of SOUNDS.map(s => s.id)) {
      if (playing[id]) {
        audios[id]?.pause();
        delete audios[id];
        playing[id] = false;
      }
    }
    if (sound) toggleSound(sound);
    backgroundEffect.set(bg);
    sectionEffect.set(sec);
  }

  function clearAll() {
    for (const id of SOUNDS.map(s => s.id)) {
      if (playing[id]) {
        audios[id]?.pause();
        delete audios[id];
        playing[id] = false;
      }
    }
    backgroundEffect.set('none');
    sectionEffect.set('none');
  }

  const anyActive = $derived(
    SOUNDS.some(s => playing[s.id]) || $backgroundEffect !== 'none' || $sectionEffect !== 'none'
  );

  // ─── Terminal (xterm.js + PTY) ───────────────────────────────────────────────
  let terminalEl = $state<HTMLDivElement | undefined>(undefined);
  let termBg = $state('');
  let xtermReady = false;
  let xtermInstance: import('xterm').Terminal | null = null;
  let fitAddon: import('@xterm/addon-fit').FitAddon | null = null;
  let xtermRo: ResizeObserver | null = null;
  let unlistenPty: (() => void) | null = null;

  function readThemeColors() {
    const cs = getComputedStyle(document.documentElement);
    return {
      tBg:  cs.getPropertyValue('--color-term-bg').trim()  || 'rgba(0,0,0,0.6)',
      tFg:  cs.getPropertyValue('--color-term-fg').trim()  || '#a8ff78',
      tCmd: cs.getPropertyValue('--color-term-cmd').trim() || '#f0c060',
      tErr: cs.getPropertyValue('--color-term-err').trim() || '#ff6060',
    };
  }

  function buildXtermTheme(c: ReturnType<typeof readThemeColors>) {
    return {
      background: 'rgba(0,0,0,0)',
      foreground: c.tFg,
      cursor: c.tCmd,
      cursorAccent: c.tBg,
      selectionBackground: 'rgba(255,255,255,0.15)',
      red: c.tErr, green: c.tFg, yellow: c.tCmd,
      blue: '#bd93f9', magenta: '#ff79c6', cyan: '#5dd4f8', white: '#f8f8f2',
      brightBlack: '#4d5566', brightRed: c.tErr, brightGreen: c.tFg,
      brightYellow: c.tCmd, brightBlue: '#d6b4fc', brightMagenta: '#ff99de',
      brightCyan: '#7ee5ff', brightWhite: '#ffffff',
    };
  }

  async function initXterm() {
    if (xtermReady || !terminalEl) return;

    const { Terminal } = await import('xterm');
    const { FitAddon } = await import('@xterm/addon-fit');

    const colors = readThemeColors();
    termBg = colors.tBg;

    xtermInstance = new Terminal({
      cursorBlink: true,
      fontSize: 12,
      fontFamily: 'Cascadia Code, Cascadia Mono, Consolas, "Courier New", monospace',
      scrollback: 2000,
      allowTransparency: true,
      theme: buildXtermTheme(colors),
    });

    fitAddon = new FitAddon();
    xtermInstance.loadAddon(fitAddon);
    xtermInstance.open(terminalEl);
    fitAddon.fit();

    const { rows, cols } = xtermInstance;

    unlistenPty = await listen<string>('pty-data', (e) => {
      xtermInstance?.write(e.payload);
    });

    xtermInstance.onData((data) => invoke('pty_write', { data }));

    await invoke('pty_create', { rows, cols });

    xtermRo = new ResizeObserver(() => {
      fitAddon?.fit();
      if (xtermInstance) invoke('pty_resize', { rows: xtermInstance.rows, cols: xtermInstance.cols });
    });
    xtermRo.observe(terminalEl);

    xtermReady = true;
  }

  async function openTerminalTab() {
    tab = 'terminal';
    await tick();
    if (!xtermReady) {
      await initXterm();
    } else {
      fitAddon?.fit();
      if (xtermInstance) invoke('pty_resize', { rows: xtermInstance.rows, cols: xtermInstance.cols });
    }
    xtermInstance?.focus();
  }

  $effect(() => {
    $theme;
    if (!xtermInstance) return;
    queueMicrotask(() => {
      if (!xtermInstance) return;
      xtermInstance.options.theme = buildXtermTheme(readThemeColors());
    });
  });

  // ─── Spotify ─────────────────────────────────────────────────────────────────
  interface SpotifyPlaybackState {
    is_playing: boolean;
    track_name: string;
    artist: string;
    album: string;
    album_art: string | null;
    progress_ms: number;
    duration_ms: number;
    device: string | null;
    track_id: string | null;
  }

  interface SpotifyPlaylist {
    id: string;
    uri: string;
    name: string;
    image: string | null;
    track_count: number;
    owner: string;
  }

  interface SpotifyDevice {
    id: string;
    name: string;
    is_active: boolean;
    device_type: string;
  }

  let spAuthed    = $state(false);
  let spState     = $state<SpotifyPlaybackState | null>(null);
  let spPlaylists = $state<SpotifyPlaylist[]>([]);
  let spView      = $state<'player' | 'playlists'>('player');
  let spLoading   = $state(false);
  let spError     = $state('');
  let spShuffle   = $state(false);
  let spPollId: ReturnType<typeof setInterval> | null = null;
  let spAudioFeatures = $state<{ energy: number; tempo: number; valence: number } | null>(null);
  let spLastFetchedTrackId = '';
  let spSelectedPlaylist = $state<string | null>(null);
  let spProgressMs = $state(0);
  let spLastPollAt = 0;
  let spBeats = $state<number[]>([]);
  let spBeatPulse = $state(false);
  let spLastBeat = -1;

  function formatMs(ms: number): string {
    const s = Math.floor(ms / 1000);
    return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
  }

  async function spInit() {
    spAuthed = await invoke<boolean>('spotify_is_authenticated');
    if (spAuthed) { await spPoll(); spStartPoll(); }
  }

  async function spPoll() {
    try {
      spState = await invoke<SpotifyPlaybackState | null>('spotify_get_state');
      if (spState) { spProgressMs = spState.progress_ms; spLastPollAt = Date.now(); }
    }
    catch { /* ignore transient poll errors */ }
  }

  function spStartPoll() {
    if (spPollId) return;
    spPollId = setInterval(spPoll, 5000);
  }

  function spStopPoll() {
    if (spPollId) { clearInterval(spPollId); spPollId = null; }
  }

  async function spConnect() {
    spLoading = true; spError = '';
    try {
      await invoke('spotify_auth');
      spAuthed = true;
      await spPoll();
      spStartPoll();
    } catch (e) { spError = String(e); }
    spLoading = false;
  }

  async function spLoadPlaylists() {
    spLoading = true; spError = '';
    try {
      spPlaylists = await invoke<SpotifyPlaylist[]>('spotify_get_playlists');
      spView = 'playlists';
    } catch (e) { spError = String(e); }
    spLoading = false;
  }

  // Auto-fetch playlists whenever authed with no active playback
  $effect(() => {
    if (spAuthed && !spState && spPlaylists.length === 0 && !spLoading) {
      spLoading = true;
      invoke<SpotifyPlaylist[]>('spotify_get_playlists')
        .then(pl => { spPlaylists = pl; })
        .catch(e => { spError = String(e); })
        .finally(() => { spLoading = false; });
    }
  });

  // Interpolate progress + fire beat pulses at 100ms resolution
  $effect(() => {
    if (!spState?.is_playing) return;
    const id = setInterval(() => {
      if (!spState) return;
      const elapsed = Date.now() - spLastPollAt;
      spProgressMs = Math.min(spState.duration_ms, spState.progress_ms + elapsed);
      // Beat detection: find any beat in the last 110ms window
      if (spBeats.length > 0) {
        const posS = spProgressMs / 1000;
        const beat = spBeats.find(b => b >= posS - 0.11 && b < posS);
        if (beat !== undefined && beat !== spLastBeat) {
          spLastBeat = beat;
          spBeatPulse = true;
          setTimeout(() => { spBeatPulse = false; }, 90);
        }
      }
    }, 100);
    return () => clearInterval(id);
  });

  // Fetch audio features + beats when track changes; reset on no playback
  $effect(() => {
    const tid = spState?.track_id ?? '';
    if (!tid) {
      spAudioFeatures = null;
      spBeats = [];
      spLastBeat = -1;
      spLastFetchedTrackId = '';
      return;
    }
    if (tid === spLastFetchedTrackId) return;
    spLastFetchedTrackId = tid;
    spLastBeat = -1;
    invoke<{ energy: number; tempo: number; valence: number }>('spotify_get_audio_features', { trackId: tid })
      .then(f => { spAudioFeatures = f; })
      .catch(() => {});
    invoke<number[]>('spotify_get_beats', { trackId: tid })
      .then(b => { spBeats = b; })
      .catch(() => {});
  });

  async function spPickDevice(): Promise<string> {
    const devices = await invoke<SpotifyDevice[]>('spotify_get_devices');
    if (devices.length === 0) throw new Error('Open Spotify on any device first, then try again.');
    return (devices.find(d => d.is_active) ?? devices[0]).id;
  }

  async function spPlayPlaylist(uri: string) {
    spError = '';
    try {
      const deviceId = await spPickDevice();
      await invoke('spotify_play_context', { contextUri: uri, deviceId, offset: 0 });
      spSelectedPlaylist = null;
      spView = 'player';
      setTimeout(spPoll, 1000);
    } catch (e) { spError = String(e); }
  }

  async function spPlayShuffled(uri: string, trackCount = 0) {
    spError = '';
    try {
      const deviceId = await spPickDevice();
      await invoke('spotify_set_shuffle', { state: true, deviceId });
      const offset = trackCount > 1 ? Math.floor(Math.random() * trackCount) : 0;
      await invoke('spotify_play_context', { contextUri: uri, deviceId, offset });
      spShuffle = true;
      spSelectedPlaylist = null;
      spView = 'player';
      setTimeout(spPoll, 1000);
    } catch (e) { spError = String(e); }
  }

  async function spTogglePlay() {
    if (!spState) return;
    try {
      if (spState.is_playing) await invoke('spotify_pause');
      else await invoke('spotify_play');
      setTimeout(spPoll, 500);
    } catch (e) { spError = String(e); }
  }

  async function spSkipNext() {
    try { await invoke('spotify_next'); setTimeout(spPoll, 800); } catch { }
  }

  async function spSkipPrev() {
    try { await invoke('spotify_prev'); setTimeout(spPoll, 800); } catch { }
  }

  async function spRefresh() {
    spError = '';
    await spPoll();
  }

  function spShowAllPlaylists() {
    spSelectedPlaylist = null;
    if (spPlaylists.length === 0) { spLoadPlaylists(); return; }
    spView = 'playlists';
  }

  async function spToggleShuffle() {
    spShuffle = !spShuffle;
    try { await invoke('spotify_set_shuffle', { state: spShuffle, deviceId: '' }); }
    catch { spShuffle = !spShuffle; }
  }

  // ─── Visuals ─────────────────────────────────────────────────────────────────
  const bgEffects: { id: BackgroundEffect; label: string; emoji: string }[] = [
    { id: 'none',      label: 'Off',       emoji: '◯'  },
    { id: 'particles', label: 'Particles', emoji: '✦'  },
    { id: 'rain',      label: 'Rain',      emoji: '🌧' },
    { id: 'snow',      label: 'Snow',      emoji: '❄'  },
    { id: 'fireflies', label: 'Fireflies', emoji: '✿'  },
    { id: 'fog',       label: 'Fog',       emoji: '🌫' },
  ];

  const secEffects: { id: SectionEffect; label: string; emoji: string }[] = [
    { id: 'none',      label: 'Off',       emoji: '◯' },
    { id: 'breathe',   label: 'Breathe',   emoji: '◈' },
    { id: 'scanlines', label: 'Scanlines', emoji: '▤' },
  ];

  const PAIRINGS: { name: string; emoji: string; desc: string; sound: SoundId | null; bg: BackgroundEffect; sec: SectionEffect }[] = [
    // Rain + scanlines is the user's confirmed favourite — keep it first
    { name: 'Rainy Night',        emoji: '🌧', sound: 'rain',      bg: 'rain',      sec: 'scanlines', desc: 'Rain mask + CRT drops — proven focus state, blocks distracting noise'      },
    // Warm café ambience + drifting fog = medium ambient noise boosts creative cognition
    { name: 'Coffee Haze',        emoji: '☕', sound: 'cafe',      bg: 'fog',       sec: 'breathe',   desc: 'Café murmur + rolling mist — 70 dB sweet-spot for creative thinking'         },
    // Nature sounds reduce cortisol; fireflies + breathe = grounding, low-anxiety
    { name: 'Deep Forest',        emoji: '🌲', sound: 'medieval',  bg: 'fireflies', sec: 'breathe',   desc: 'Celtic drones + firefly drift — nature sounds lower stress hormones'         },
    // Nostalgia from synthwave boosts motivation; particles on scanlines = retro energy
    { name: 'Midnight Synthwave', emoji: '🌆', sound: 'cyberpunk', bg: 'particles', sec: 'scanlines', desc: 'Retro pulse + particle dust — nostalgia-driven motivation for late sessions'  },
    // No sound = zero audio distraction; snow = visual movement without cognitive load
    { name: 'Winter Silence',     emoji: '❄',  sound: null,        bg: 'snow',      sec: 'none',      desc: 'Snow only, no audio — visual movement without cognitive load for deep work'   },
    // Rain audio + fog = atmospheric without CRT edge — brooding, slower tempo
    { name: 'Storm Drift',        emoji: '⛅', sound: 'rain',      bg: 'fog',       sec: 'breathe',   desc: 'Rain audio + slow fog — same calm without the retro edge'                    },
  ];

  // ─── Lifecycle ───────────────────────────────────────────────────────────────
  function handleKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.key === '`') {
      e.preventDefault();
      openTerminalTab();
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
    spInit();
    return () => {
      window.removeEventListener('keydown', handleKeydown);
      Object.values(audios).forEach(a => a?.pause());
    };
  });

  onDestroy(() => {
    unlistenPty?.();
    xtermRo?.disconnect();
    xtermInstance?.dispose();
    invoke('pty_kill');
    spStopPoll();
  });
</script>

<div class="flex flex-col h-full gap-3">
  <!-- Tab bar -->
  <div class="flex gap-1 border-b border-line pb-2 shrink-0">
    {#each (['sounds', 'visuals'] as Tab[]) as t}
      <button
        onclick={() => (tab = t)}
        class="px-3 py-1 rounded-md text-xs font-medium capitalize transition {tab === t
          ? 'bg-accent text-white'
          : 'text-mute hover:text-ink hover:bg-surface'}"
      >
        {t}
      </button>
    {/each}
    {#if anyActive}
      <button
        onclick={clearAll}
        title="Clear all sounds & effects"
        class="px-2 py-1 rounded-md text-xs font-medium transition text-mute hover:text-red-400 hover:bg-surface"
      >✕ clear</button>
    {/if}
    <button
      onclick={openTerminalTab}
      title="Ctrl+`"
      class="ml-auto px-3 py-1 rounded-md text-xs font-medium capitalize transition {tab === 'terminal'
        ? 'bg-accent text-white'
        : 'text-mute hover:text-ink hover:bg-surface'}"
    >
      terminal
    </button>
  </div>

  <!-- Sounds tab -->
  {#if tab === 'sounds'}
    <div class="flex flex-col gap-3 flex-1 min-h-0">
      <div class="grid grid-cols-2 gap-2 shrink-0">
        {#each SOUNDS as s}
          <button
            onclick={() => toggleSound(s.id)}
            class="flex flex-col items-start gap-0.5 p-3 rounded-xl border transition {playing[s.id]
              ? 'border-accent bg-accent/10 text-accent'
              : 'border-line hover:border-accent/40 hover:bg-surface'}"
          >
            <div class="flex items-center justify-between w-full">
              <span class="text-lg leading-none">{s.emoji}</span>
              {#if playing[s.id]}
                <span class="flex gap-0.5 items-end h-4">
                  {#each [1, 2, 3] as i}
                    <span class="w-0.5 rounded-full bg-accent animate-bounce" style="height: {8 + i * 3}px; animation-delay: {i * 0.1}s;"></span>
                  {/each}
                </span>
              {/if}
            </div>
            <span class="text-xs font-semibold">{s.label}</span>
            <span class="text-[10px] text-mute">{s.desc}</span>
          </button>
        {/each}
      </div>

      <!-- Spotify Card -->
      <div
        class="flex flex-col flex-1 min-h-0 rounded-xl overflow-hidden border border-white/5"
        style="background:#121212"
      >
        {#if !spAuthed}
          <!-- Connect prompt -->
          <button
            onclick={spConnect}
            disabled={spLoading}
            class="flex flex-col items-center gap-2 m-3 py-6 rounded-lg border border-dashed hover:bg-white/5 transition"
            style="border-color:rgba(255,255,255,0.1)"
          >
            <svg viewBox="0 0 24 24" class="w-6 h-6" style="color:#1db954">
              <path fill="currentColor" d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.419 1.56-.299.421-1.02.599-1.559.3z"/>
            </svg>
            <span class="text-[10px]" style="color:rgba(255,255,255,0.5)">
              {spLoading ? 'Connecting…' : 'Connect Spotify'}
            </span>
          </button>

        {:else if spView === 'playlists'}
          <!-- Playlist picker -->
          <div class="flex items-center justify-between px-3 pt-3 pb-2 shrink-0">
            <span class="text-xs font-bold text-white">Playlists</span>
            <button
              onclick={() => (spView = 'player')}
              class="text-[10px] px-2 py-0.5 rounded-full border hover:text-white transition"
              style="color:rgba(255,255,255,0.4);border-color:rgba(255,255,255,0.15)"
            >← back</button>
          </div>
          <div class="flex-1 min-h-0 overflow-y-auto flex flex-col">
            {#if spLoading}
              <p class="text-[10px] px-3 pb-3" style="color:rgba(255,255,255,0.3)">Loading…</p>
            {:else}
              {#each spPlaylists as pl}
                <button
                  onclick={() => spPlayPlaylist(pl.uri)}
                  class="flex items-center gap-2.5 px-3 py-2 text-left hover:bg-white/5 transition shrink-0"
                >
                  {#if pl.image}
                    <img src={pl.image} alt="" class="w-7 h-7 rounded object-cover shrink-0" />
                  {:else}
                    <div class="w-7 h-7 rounded shrink-0" style="background:rgba(255,255,255,0.1)"></div>
                  {/if}
                  <div class="flex flex-col min-w-0">
                    <span class="text-[10px] font-semibold text-white truncate">{pl.name}</span>
                    <span class="text-[9px]" style="color:rgba(255,255,255,0.4)">{pl.track_count} tracks</span>
                  </div>
                </button>
              {/each}
            {/if}
          </div>

        {:else}
          <!-- Player view -->
          <div class="flex items-center justify-between px-3 pt-3 pb-2 shrink-0">
            <div class="flex items-center gap-1.5">
              <svg viewBox="0 0 24 24" class="w-3.5 h-3.5 shrink-0" style="color:#1db954">
                <path fill="currentColor" d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.419 1.56-.299.421-1.02.599-1.559.3z"/>
              </svg>
              <span class="text-xs font-bold text-white">Spotify</span>
            </div>
            <div class="flex items-center gap-1.5">
              {#if spState}
                <button
                  onclick={spLoadPlaylists}
                  disabled={spLoading}
                  class="text-[10px] px-2.5 py-0.5 rounded-full border hover:text-white transition"
                  style="color:rgba(255,255,255,0.5);border-color:rgba(255,255,255,0.15)"
                >{spLoading ? '…' : 'Playlists'}</button>
              {/if}
              <button onclick={spRefresh} title="Refresh" class="p-1 rounded hover:bg-white/10 transition" style="color:rgba(255,255,255,0.4)">
                <svg viewBox="0 0 24 24" class="w-3.5 h-3.5"><path fill="currentColor" d="M17.65 6.35A7.958 7.958 0 0 0 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08A5.99 5.99 0 0 1 12 18c-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/></svg>
              </button>
            </div>
          </div>

          {#if !spState}
            <div class="flex-1 min-h-0 flex flex-col px-3 pb-3 gap-2">
              <p class="text-[9px] font-medium" style="color:rgba(255,255,255,0.25)">No active session</p>
              {#if spLoading}
                <p class="text-[10px]" style="color:rgba(255,255,255,0.3)">Loading…</p>
              {:else}
                <div class="overflow-y-auto" style="max-height:12rem;scrollbar-width:thin;scrollbar-color:rgba(255,255,255,0.15) transparent">
                  <div class="grid grid-cols-3 gap-2.5">
                    {#each spPlaylists as pl}
                      <div class="relative aspect-square rounded-lg overflow-hidden group" title={pl.name}>
                        {#if pl.image}
                          <img src={pl.image} alt={pl.name} class="w-full h-full object-cover transition duration-150 group-hover:brightness-50" />
                        {:else}
                          <div class="w-full h-full transition duration-150 group-hover:brightness-50" style="background:rgba(255,255,255,0.1)"></div>
                        {/if}
                        <div class="absolute inset-0 flex flex-col items-center justify-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-150">
                          <button
                            onclick={() => { spShuffle = !spShuffle; }}
                            class="p-1 rounded-full transition hover:bg-white/10"
                            style="color:{spShuffle ? '#1db954' : 'rgba(255,255,255,0.75)'}"
                            title="Toggle shuffle"
                          >
                            <svg viewBox="0 0 24 24" class="w-3.5 h-3.5"><path fill="currentColor" d="M10.59 9.17 5.41 4 4 5.41l5.17 5.17 1.42-1.41zM14.5 4l2.04 2.04L4 18.59 5.41 20 17.96 8.46 20 10.5V4h-5.5zm.33 9.41-1.41 1.41 3.13 3.13L14.5 20H20v-5.5l-2.04 2.04-3.13-3.13z"/></svg>
                          </button>
                          <button
                            onclick={() => spShuffle ? spPlayShuffled(pl.uri, pl.track_count) : spPlayPlaylist(pl.uri)}
                            class="w-7 h-7 rounded-full flex items-center justify-center hover:scale-105 transition-transform"
                            style="background:#1db954"
                            title="Play"
                          >
                            <svg viewBox="0 0 24 24" class="w-4 h-4" style="color:#000"><path fill="currentColor" d="M8 5v14l11-7z"/></svg>
                          </button>
                          <span class="text-[8px] text-white font-medium px-1 text-center leading-tight line-clamp-2">{pl.name}</span>
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          {:else}
            <!-- Now playing -->
            <div class="flex items-center gap-2.5 px-3 pb-2 min-w-0">
              {#if spState.album_art}
                <img src={spState.album_art} alt="" class="w-9 h-9 rounded object-cover shrink-0" />
              {/if}
              <div class="flex flex-col min-w-0">
                <span class="text-[11px] font-semibold text-white truncate">{spState.track_name}</span>
                <span class="text-[9px] truncate" style="color:rgba(255,255,255,0.5)">{spState.artist}</span>
              </div>
            </div>

            <!-- Audio waveform -->
            {#if spState.is_playing}
              {@const energy = spAudioFeatures?.energy ?? 0.6}
              {@const tempo = spAudioFeatures?.tempo ?? 120}
              {@const speed = Math.max(120, Math.round(28000 / tempo))}
              {@const maxH = Math.round(12 + energy * 24)}
              <div class="flex items-end w-full justify-between px-3 pb-1.5" style="height:36px;transform-origin:bottom;transform:{spBeatPulse ? 'scaleY(1.25)' : 'scaleY(1)'};transition:transform {spBeatPulse ? '40ms' : '80ms'} ease-out">
                {#each Array(30) as _, i}
                  {@const v = 0.25 + 0.75 * Math.abs(Math.sin(i * 1.9 + 0.8))}
                  {@const peak = Math.max(3, Math.round(maxH * v))}
                  {@const dur = Math.round(speed * (0.65 + v * 0.7))}
                  {@const delay = Math.round((i / 30) * speed * 0.85)}
                  <div style="width:2.5px;height:{peak}px;background:#1db954;border-radius:9999px;transform-origin:bottom;animation:sp-wave {dur}ms ease-in-out {delay}ms infinite alternate both"></div>
                {/each}
              </div>
            {:else}
              <div style="height:36px"></div>
            {/if}

            <!-- Progress bar -->
            <div class="flex items-center gap-2 px-3 pb-2.5">
              <span class="text-[9px] tabular-nums w-7 shrink-0" style="color:rgba(255,255,255,0.35)">{formatMs(spProgressMs)}</span>
              <div class="flex-1 h-1 rounded-full" style="background:rgba(255,255,255,0.12)">
                <div class="h-full rounded-full" style="background:#1db954;width:{spState.duration_ms > 0 ? (spProgressMs / spState.duration_ms * 100) : 0}%"></div>
              </div>
              <span class="text-[9px] tabular-nums w-7 text-right shrink-0" style="color:rgba(255,255,255,0.35)">{formatMs(spState.duration_ms)}</span>
            </div>

            <!-- Controls -->
            <div class="flex items-center justify-center gap-4 pb-3">
              <button onclick={spToggleShuffle} class="p-1.5 rounded-full transition hover:bg-white/10" style="color:{spShuffle ? '#1db954' : 'rgba(255,255,255,0.4)'}" title="Shuffle">
                <svg viewBox="0 0 24 24" class="w-3.5 h-3.5"><path fill="currentColor" d="M10.59 9.17 5.41 4 4 5.41l5.17 5.17 1.42-1.41zM14.5 4l2.04 2.04L4 18.59 5.41 20 17.96 8.46 20 10.5V4h-5.5zm.33 9.41-1.41 1.41 3.13 3.13L14.5 20H20v-5.5l-2.04 2.04-3.13-3.13z"/></svg>
              </button>
              <button onclick={spSkipPrev} class="p-1.5 rounded-full transition hover:bg-white/10" style="color:rgba(255,255,255,0.5)">
                <svg viewBox="0 0 24 24" class="w-4 h-4"><path fill="currentColor" d="M6 6h2v12H6zm3.5 6 8.5 6V6z"/></svg>
              </button>
              <button
                onclick={spTogglePlay}
                class="w-10 h-10 rounded-full flex items-center justify-center text-black hover:scale-105 transition-transform shrink-0"
                style="background:#1db954"
              >
                {#if spState.is_playing}
                  <svg viewBox="0 0 24 24" class="w-5 h-5"><path fill="currentColor" d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/></svg>
                {:else}
                  <svg viewBox="0 0 24 24" class="w-5 h-5"><path fill="currentColor" d="M8 5v14l11-7z"/></svg>
                {/if}
              </button>
              <button onclick={spSkipNext} class="p-1.5 rounded-full transition hover:bg-white/10" style="color:rgba(255,255,255,0.5)">
                <svg viewBox="0 0 24 24" class="w-4 h-4"><path fill="currentColor" d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z"/></svg>
              </button>
            </div>
          {/if}
        {/if}

        {#if spError}
          <div class="mx-2 mb-2 flex items-start gap-2 px-2.5 py-2 rounded-lg shrink-0" style="background:rgba(255,90,50,0.5);border:1px solid rgba(255,90,50,0.6)">
            <svg viewBox="0 0 24 24" class="w-3.5 h-3.5 shrink-0 mt-px" style="color:rgba(255,150,120,1)">
              <path fill="currentColor" d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
            </svg>
            <span class="text-[9px] leading-relaxed flex-1" style="color:rgba(255,200,185,1)">{spError}</span>
            <button onclick={() => spError = ''} aria-label="Dismiss" class="shrink-0 rounded hover:bg-white/10 transition p-0.5 -mr-0.5" style="color:rgba(255,255,255,0.25)">
              <svg viewBox="0 0 24 24" class="w-3 h-3"><path fill="currentColor" d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg>
            </button>
          </div>
        {/if}
      </div>

    </div>

  <!-- Visuals tab -->
  {:else if tab === 'visuals'}
    <div class="flex flex-col gap-3 flex-1 min-h-0 overflow-y-auto pr-0.5">

      <!-- Background -->
      <div class="flex flex-col gap-1.5">
        <p class="text-[10px] font-semibold text-mute uppercase tracking-wide">Background</p>
        <div class="grid grid-cols-3 gap-1.5">
          {#each bgEffects as fx}
            <button
              onclick={() => backgroundEffect.set(fx.id)}
              class="flex flex-col items-center gap-1 py-2.5 px-1 rounded-xl border transition {$backgroundEffect === fx.id
                ? 'border-accent bg-accent/10'
                : 'border-line hover:border-accent/40 hover:bg-surface'}"
            >
              <span class="text-2xl leading-none">{fx.emoji}</span>
              <span class="text-[10px] font-semibold {$backgroundEffect === fx.id ? 'text-accent' : 'text-ink'}">{fx.label}</span>
            </button>
          {/each}
        </div>
      </div>

      <!-- Sections -->
      <div class="flex flex-col gap-1.5">
        <p class="text-[10px] font-semibold text-mute uppercase tracking-wide">Sections</p>
        <div class="grid grid-cols-3 gap-1.5">
          {#each secEffects as fx}
            <button
              onclick={() => sectionEffect.set(fx.id)}
              class="flex flex-col items-center gap-1 py-2.5 px-1 rounded-xl border transition {$sectionEffect === fx.id
                ? 'border-accent bg-accent/10'
                : 'border-line hover:border-accent/40 hover:bg-surface'}"
            >
              <span class="text-2xl leading-none">{fx.emoji}</span>
              <span class="text-[10px] font-semibold {$sectionEffect === fx.id ? 'text-accent' : 'text-ink'}">{fx.label}</span>
            </button>
          {/each}
        </div>
      </div>

      <!-- Pairings -->
      <div class="flex flex-col gap-0.5">
        <p class="text-[10px] font-semibold text-mute uppercase tracking-wide mb-1">✦ Mood Presets</p>
        {#each PAIRINGS as p}
          {@const active = $backgroundEffect === p.bg && $sectionEffect === p.sec}
          <button
            onclick={() => applyPreset(p.sound, p.bg, p.sec)}
            class="flex items-center gap-2.5 px-2.5 py-2 rounded-lg text-left transition group border
                   {active ? 'border-accent/30 bg-accent/5' : 'border-transparent hover:border-line hover:bg-surface'}"
          >
            <span class="text-base leading-none shrink-0">{p.emoji}</span>
            <div class="flex flex-col min-w-0 flex-1">
              <span class="text-[10px] font-semibold {active ? 'text-accent' : 'text-ink group-hover:text-accent'} transition">{p.name}</span>
              <span class="text-[9px] text-mute leading-snug">{p.desc}</span>
            </div>
            {#if p.sound}
              <span class="text-[8px] text-mute shrink-0 opacity-60">{SOUNDS.find(s => s.id === p.sound)?.emoji}</span>
            {/if}
          </button>
        {/each}
      </div>

    </div>

  {/if}

  <!-- Terminal — always in DOM so xterm.js persists across tab switches -->
  <div
    class="flex-1 min-h-0 rounded-xl overflow-hidden"
    style:display={tab !== 'terminal' ? 'none' : 'block'}
    style:background="var(--color-term-bg)"
    style:padding-left="8px"
  >
    <div bind:this={terminalEl} class="h-full"></div>
  </div>
</div>
