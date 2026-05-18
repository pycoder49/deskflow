<script lang="ts">
  import { onMount } from 'svelte';
  import { theme, type Theme } from '$lib/stores/theme';
  import { backgroundEffect, type BackgroundEffect, sectionEffect, type SectionEffect } from '$lib/stores/ambience';
  import QuickCaptureModal from '$lib/widgets/QuickCaptureModal.svelte';
  import TodayTasks from '$lib/widgets/TodayTasks.svelte';
  import NowNext from '$lib/widgets/NowNext.svelte';
  import Calendar from '$lib/widgets/Calendar.svelte';
  import Notepad from '$lib/widgets/Notepad.svelte';
  import Ambience from '$lib/widgets/Ambience.svelte';
  import TaskStats from '$lib/widgets/TaskStats.svelte';
  import { startDay } from '$lib/services/clickup';
  import { logicalToday } from '$lib/stores/refresh';

  let captureOpen = $state(false);
  let now = $state(new Date());
  let logStatus = $state<'idle' | 'loading' | 'ok' | 'already' | 'err'>('idle');
  let overlayCanvas = $state<HTMLCanvasElement | null>(null);
  let animId = 0;

  const dateLabel = $derived(
    now.toLocaleDateString(undefined, { weekday: 'long', month: 'long', day: 'numeric' })
  );
  const timeLabel = $derived(
    now.toLocaleTimeString(undefined, { hour: 'numeric', minute: '2-digit' })
  );

  const THEME_OPTIONS: { value: Theme; label: string }[] = [
    { value: 'light',  label: '☀  Light'  },
    { value: 'dark',   label: '☾  Dark'   },
    { value: 'space',  label: '✦  Space'  },
    { value: 'nord',   label: '❄  Nord'   },
    { value: 'forest',  label: '⬡  Forest'  },
    { value: 'vintage', label: '✒  Vintage' },
    { value: 'slate',   label: '◈  Slate'   },
    { value: 'cloudy',  label: '☁  Cloudy'  },
  ];

  // ─── Visual effect canvas loops ──────────────────────────────────────────────
  function stopLoop() {
    if (animId) { cancelAnimationFrame(animId); animId = 0; }
  }

  function themeRgb(varName: string): [number, number, number] {
    const val = getComputedStyle(document.documentElement).getPropertyValue(varName).trim();
    const m = val.match(/^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i);
    if (m) return [parseInt(m[1], 16), parseInt(m[2], 16), parseInt(m[3], 16)];
    return [120, 160, 255];
  }

  interface Particle { x: number; y: number; vx: number; vy: number; r: number; alpha: number; useAccent: boolean }
  function startParticles(canvas: HTMLCanvasElement) {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    const ctx = canvas.getContext('2d')!;
    const [ar, ag, ab] = themeRgb('--color-accent');
    const [ir, ig, ib] = themeRgb('--color-ink');
    const N = 130;
    const particles: Particle[] = Array.from({ length: N }, () => ({
      x: Math.random() * canvas.width,
      y: Math.random() * canvas.height,
      vx: (Math.random() - 0.5) * 0.45,
      vy: (Math.random() - 0.5) * 0.45,
      r: Math.random() * 2.5 + 0.8,
      alpha: Math.random() * 0.45 + 0.3,
      useAccent: Math.random() > 0.25,
    }));
    function frame() {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      for (const p of particles) {
        p.x = (p.x + p.vx + canvas.width) % canvas.width;
        p.y = (p.y + p.vy + canvas.height) % canvas.height;
        const [cr, cg, cb] = p.useAccent ? [ar, ag, ab] : [ir, ig, ib];
        ctx.beginPath();
        ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(${cr},${cg},${cb},${p.alpha})`;
        ctx.fill();
      }
      animId = requestAnimationFrame(frame);
    }
    frame();
  }

  interface Drop { x: number; y: number; len: number; speed: number; alpha: number }
  function startRain(canvas: HTMLCanvasElement) {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    const ctx = canvas.getContext('2d')!;
    const [r, g, b] = themeRgb('--color-accent');
    const N = 220;
    const drops: Drop[] = Array.from({ length: N }, () => ({
      x: Math.random() * canvas.width,
      y: Math.random() * canvas.height,
      len: Math.random() * 22 + 12,
      speed: Math.random() * 4 + 3,
      alpha: Math.random() * 0.35 + 0.35,
    }));
    function frame() {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      for (const d of drops) {
        ctx.beginPath();
        ctx.moveTo(d.x, d.y);
        ctx.lineTo(d.x - 2, d.y + d.len);
        ctx.strokeStyle = `rgba(${r},${g},${b},${d.alpha})`;
        ctx.lineWidth = 1.4;
        ctx.stroke();
        d.y += d.speed;
        if (d.y > canvas.height) { d.y = -d.len; d.x = Math.random() * canvas.width; }
      }
      animId = requestAnimationFrame(frame);
    }
    frame();
  }

  interface Flake { x: number; y: number; r: number; speed: number; drift: number; driftPhase: number; alpha: number }
  function startSnow(canvas: HTMLCanvasElement) {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    const ctx = canvas.getContext('2d')!;
    const base = themeRgb('--color-base');
    const isDark = (base[0] + base[1] + base[2]) / 3 < 140;
    const [sr, sg, sb] = isDark ? [255, 255, 255] : [100, 130, 200];
    const N = 180;
    let t = 0;
    const flakes: Flake[] = Array.from({ length: N }, () => ({
      x: Math.random() * canvas.width,
      y: Math.random() * canvas.height,
      r: Math.random() * 2.5 + 0.8,
      speed: Math.random() * 1.2 + 0.4,
      drift: (Math.random() - 0.5) * 0.6,
      driftPhase: Math.random() * Math.PI * 2,
      alpha: Math.random() * 0.4 + 0.45,
    }));
    function frame() {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      for (const f of flakes) {
        f.y += f.speed;
        f.x += f.drift * Math.sin(t * 0.02 + f.driftPhase);
        f.x = (f.x + canvas.width) % canvas.width;
        if (f.y > canvas.height) { f.y = -f.r * 2; f.x = Math.random() * canvas.width; }
        ctx.beginPath();
        ctx.arc(f.x, f.y, f.r, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(${sr},${sg},${sb},${f.alpha})`;
        ctx.fill();
      }
      t++;
      animId = requestAnimationFrame(frame);
    }
    frame();
  }

  interface Firefly { x: number; y: number; vx: number; vy: number; r: number; phase: number; phaseSpeed: number; baseAlpha: number }
  function startFireflies(canvas: HTMLCanvasElement) {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    const ctx = canvas.getContext('2d')!;
    const [ar, ag, ab] = themeRgb('--color-accent');
    const N = 55;
    let t = 0;
    const flies: Firefly[] = Array.from({ length: N }, () => ({
      x: Math.random() * canvas.width,
      y: Math.random() * canvas.height,
      vx: (Math.random() - 0.5) * 0.35,
      vy: (Math.random() - 0.5) * 0.35,
      r: Math.random() * 2.5 + 1.5,
      phase: Math.random() * Math.PI * 2,
      phaseSpeed: Math.random() * 0.025 + 0.015,
      baseAlpha: Math.random() * 0.3 + 0.5,
    }));
    function frame() {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.shadowBlur = 12;
      ctx.shadowColor = `rgb(${ar},${ag},${ab})`;
      for (const f of flies) {
        f.x = (f.x + f.vx + canvas.width) % canvas.width;
        f.y = (f.y + f.vy + canvas.height) % canvas.height;
        const alpha = f.baseAlpha * (0.3 + 0.7 * (Math.sin(t * f.phaseSpeed + f.phase) * 0.5 + 0.5));
        ctx.beginPath();
        ctx.arc(f.x, f.y, f.r, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(${ar},${ag},${ab},${alpha})`;
        ctx.fill();
      }
      ctx.shadowBlur = 0;
      t++;
      animId = requestAnimationFrame(frame);
    }
    frame();
  }

  interface FogBlob { cx: number; cy: number; ax: number; ay: number; wx: number; wy: number; px: number; py: number; radius: number; a: number }
  function startFog(canvas: HTMLCanvasElement) {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    const ctx = canvas.getContext('2d')!;
    const [ar, ag, ab] = themeRgb('--color-accent');
    const [mr, mg, mb] = themeRgb('--color-mute');
    const blobs: FogBlob[] = [
      { cx: 0.25, cy: 0.35, ax: 0.15, ay: 0.12, wx: 0.18, wy: 0.14, px: 0.0, py: 1.2, radius: 0.45, a: 1.00 },
      { cx: 0.72, cy: 0.55, ax: 0.12, ay: 0.18, wx: 0.14, wy: 0.11, px: 2.1, py: 0.5, radius: 0.40, a: 0.85 },
      { cx: 0.50, cy: 0.20, ax: 0.18, ay: 0.10, wx: 0.22, wy: 0.16, px: 1.0, py: 2.8, radius: 0.38, a: 0.90 },
      { cx: 0.15, cy: 0.75, ax: 0.10, ay: 0.15, wx: 0.16, wy: 0.20, px: 3.4, py: 0.7, radius: 0.35, a: 0.75 },
    ];
    let t = 0;
    function frame() {
      const W = canvas.width, H = canvas.height;
      ctx.clearRect(0, 0, W, H);
      for (const blob of blobs) {
        const bx = W * (blob.cx + blob.ax * Math.sin(t * blob.wx + blob.px));
        const by = H * (blob.cy + blob.ay * Math.cos(t * blob.wy + blob.py));
        const rad = W * blob.radius;
        const grd = ctx.createRadialGradient(bx, by, 0, bx, by, rad);
        grd.addColorStop(0,   `rgba(${ar},${ag},${ab},${0.14 * blob.a})`);
        grd.addColorStop(0.5, `rgba(${mr},${mg},${mb},${0.08 * blob.a})`);
        grd.addColorStop(1,   `rgba(${ar},${ag},${ab},0)`);
        ctx.fillStyle = grd;
        ctx.fillRect(0, 0, W, H);
      }
      t += 0.003;
      animId = requestAnimationFrame(frame);
    }
    frame();
  }

  $effect(() => {
    const fx: BackgroundEffect = $backgroundEffect;
    $theme;
    stopLoop();
    if (!overlayCanvas) return;
    if (fx === 'particles')  startParticles(overlayCanvas);
    else if (fx === 'rain')      startRain(overlayCanvas);
    else if (fx === 'snow')      startSnow(overlayCanvas);
    else if (fx === 'fireflies') startFireflies(overlayCanvas);
    else if (fx === 'fog')       startFog(overlayCanvas);
    else overlayCanvas.getContext('2d')!.clearRect(0, 0, overlayCanvas.width || 0, overlayCanvas.height || 0);
  });

  onMount(() => {
    if (localStorage.getItem(`checkin_${logicalToday()}`) === 'done') {
      logStatus = 'already';
    }

    const interval = setInterval(() => { now = new Date(); }, 60_000);
    const resize = () => {
      if (overlayCanvas && $backgroundEffect !== 'none') {
        overlayCanvas.width = window.innerWidth;
        overlayCanvas.height = window.innerHeight;
      }
    };
    window.addEventListener('resize', resize);
    return () => { clearInterval(interval); window.removeEventListener('resize', resize); stopLoop(); };
  });

  async function runStartDay() {
    if (logStatus === 'loading' || logStatus === 'already') return;
    logStatus = 'loading';
    try {
      const status = await startDay();
      localStorage.setItem(`checkin_${logicalToday()}`, 'done');
      if (status === 'already') {
        logStatus = 'already';
      } else {
        logStatus = 'ok';
        setTimeout(() => { logStatus = 'idle'; }, 3000);
      }
    } catch {
      logStatus = 'err';
      setTimeout(() => { logStatus = 'idle'; }, 3000);
    }
  }

  function handleKey(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'n' && !captureOpen) {
      e.preventDefault();
      captureOpen = true;
    }
  }
</script>

<svelte:window onkeydown={handleKey} />

<!-- Visual effect overlay — fixed behind all content -->
<canvas
  bind:this={overlayCanvas}
  class="fixed inset-0 w-full h-full pointer-events-none"
  style="z-index: 0; opacity: {$backgroundEffect === 'none' ? 0 : 1}; transition: opacity 0.6s;"
></canvas>

<!-- Scanlines overlay — fixed on top of all content -->
{#if $sectionEffect === 'scanlines'}
  <div
    class="fixed inset-0 pointer-events-none"
    style="z-index: 50; background: repeating-linear-gradient(to bottom, transparent 0px, transparent 2px, color-mix(in srgb, var(--color-ink) 5%, transparent) 2px, color-mix(in srgb, var(--color-ink) 5%, transparent) 3px);"
  ></div>
{/if}

<div class="relative flex flex-col h-screen overflow-hidden" class:effect-breathe={$sectionEffect === 'breathe'} style="z-index: 1;">
  <header class="shrink-0 flex items-center justify-between px-8 py-5 border-b border-line">
    <div class="flex items-baseline gap-3">
      <span class="text-lg font-semibold text-ink">{dateLabel}</span>
      <span class="text-sm text-mute">{timeLabel}</span>
    </div>
    <div class="flex items-center gap-2">
      <!-- Start Day: ensures this month's log doc exists in ClickUp Logs folder -->
      <button
        class="px-3 py-1.5 rounded-md border text-sm transition flex items-center gap-1.5
               {logStatus === 'ok' || logStatus === 'already' ? 'border-green-500 text-green-500' :
                logStatus === 'err' ? 'border-red-400 text-red-400' :
                'border-line text-mute hover:text-ink hover:bg-surface'}"
        onclick={runStartDay}
        disabled={logStatus === 'loading' || logStatus === 'already'}
        title="Run your start-of-day Claude skill"
      >
        {#if logStatus === 'loading'}
          <svg class="w-3.5 h-3.5 animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4" stroke-linecap="round"/>
          </svg>
          <span>Checking in…</span>
        {:else if logStatus === 'ok'}
          <span>✓ Checked in</span>
        {:else if logStatus === 'already'}
          <span>✓ Already checked in</span>
        {:else if logStatus === 'err'}
          <span>Check-in failed</span>
        {:else}
          <svg class="w-3.5 h-3.5" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.6">
            <rect x="2" y="1" width="10" height="12" rx="1.5"/>
            <path d="M4 4h6M4 7h6M4 10h3" stroke-linecap="round"/>
          </svg>
          <span>Start Day</span>
        {/if}
      </button>

      <button
        class="px-3 py-1.5 rounded-md border border-line text-sm hover:bg-surface transition flex items-center gap-2"
        onclick={() => (captureOpen = true)}
        title="Ctrl+N"
      >
        <span class="text-lg leading-none">+</span>
        <span>New Task</span>
        <span class="text-xs text-mute">Ctrl+N</span>
      </button>
      <select
        class="px-3 py-1.5 rounded-md border border-line text-sm bg-surface text-ink hover:bg-line transition cursor-pointer focus:outline-none focus:border-accent"
        onchange={(e) => theme.set((e.currentTarget as HTMLSelectElement).value as Theme)}
      >
        {#each THEME_OPTIONS as opt}
          <option value={opt.value} selected={opt.value === $theme}>{opt.label}</option>
        {/each}
      </select>
    </div>
  </header>

  <main class="flex-1 min-h-0 p-5 flex gap-4">
    <!-- Left 8 cols: Today / NowNext / Stats / Calendar -->
    <div class="flex flex-col gap-4 min-h-0 min-w-0" style="flex: 8;">
      <!-- Row 1: Today + NowNext — ~40% of height -->
      <div class="flex gap-4" style="flex: 3; min-height: 0;">
        <section class="flex-1 bg-surface border border-line rounded-xl p-5 min-h-0 overflow-hidden">
          <TodayTasks />
        </section>
        <section class="flex-1 bg-surface border border-line rounded-xl p-5 min-h-0 overflow-hidden">
          <NowNext />
        </section>
      </div>
      <!-- Row 2: Stats — ~15% of height -->
      <section class="bg-surface border border-line rounded-xl p-5" style="flex: 1.2; min-height: 0;">
        <TaskStats />
      </section>
      <!-- Row 3: Calendar — ~47% of height -->
      <section class="bg-surface border border-line rounded-xl p-5" style="flex: 3.5; min-height: 0; overflow: hidden;">
        <Calendar />
      </section>
    </div>

    <!-- Right 4 cols: Ambience (60%) / Notepad (40%) -->
    <div class="flex flex-col gap-4 min-h-0 min-w-0" style="flex: 4;">
      <section class="bg-surface border border-line rounded-xl p-5" style="flex: 3; min-height: 0; overflow: hidden;">
        <Ambience />
      </section>
      <section class="bg-surface border border-line rounded-xl p-5" style="flex: 2; min-height: 0; overflow-y: auto;">
        <Notepad />
      </section>
    </div>
  </main>
</div>

<QuickCaptureModal bind:open={captureOpen} onCreated={() => {}} />
