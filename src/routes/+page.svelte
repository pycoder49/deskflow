<script lang="ts">
  import { onMount } from 'svelte';
  import { theme } from '$lib/stores/theme';
  import { visualEffect, type VisualEffect } from '$lib/stores/ambience';
  import QuickCaptureModal from '$lib/widgets/QuickCaptureModal.svelte';
  import TodayTasks from '$lib/widgets/TodayTasks.svelte';
  import NowNext from '$lib/widgets/NowNext.svelte';
  import Calendar from '$lib/widgets/Calendar.svelte';
  import Vault from '$lib/widgets/Vault.svelte';
  import Projects from '$lib/widgets/Projects.svelte';
  import Notepad from '$lib/widgets/Notepad.svelte';
  import Ambience from '$lib/widgets/Ambience.svelte';

  let captureOpen = $state(false);
  let now = $state(new Date());
  let overlayCanvas = $state<HTMLCanvasElement | null>(null);
  let animId = 0;

  const dateLabel = $derived(
    now.toLocaleDateString(undefined, { weekday: 'long', month: 'long', day: 'numeric' })
  );
  const timeLabel = $derived(
    now.toLocaleTimeString(undefined, { hour: 'numeric', minute: '2-digit' })
  );

  const themeLabel = $derived(
    $theme === 'light' ? '☀  Light' : $theme === 'dark' ? '☾  Dark' : '✦  Space'
  );

  // ─── Visual effect canvas loops ──────────────────────────────────────────────
  function stopLoop() {
    if (animId) { cancelAnimationFrame(animId); animId = 0; }
  }

  function startAurora(canvas: HTMLCanvasElement) {
    const ctx = canvas.getContext('2d')!;
    let t = 0;
    function frame() {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      const grd = ctx.createRadialGradient(
        canvas.width * (0.3 + 0.2 * Math.sin(t * 0.3)),
        canvas.height * (0.4 + 0.15 * Math.cos(t * 0.2)),
        0,
        canvas.width * 0.5, canvas.height * 0.5,
        canvas.width * 0.8
      );
      grd.addColorStop(0, `hsla(${160 + 40 * Math.sin(t * 0.15)}, 80%, 55%, 0.25)`);
      grd.addColorStop(0.45, `hsla(${280 + 30 * Math.cos(t * 0.1)}, 70%, 50%, 0.18)`);
      grd.addColorStop(1, 'hsla(0,0%,0%,0)');
      ctx.fillStyle = grd;
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      const grd2 = ctx.createRadialGradient(
        canvas.width * (0.7 + 0.15 * Math.cos(t * 0.25)),
        canvas.height * (0.3 + 0.2 * Math.sin(t * 0.18)),
        0,
        canvas.width * 0.5, canvas.height * 0.4,
        canvas.width * 0.6
      );
      grd2.addColorStop(0, `hsla(${200 + 30 * Math.cos(t * 0.2)}, 90%, 60%, 0.2)`);
      grd2.addColorStop(1, 'hsla(0,0%,0%,0)');
      ctx.fillStyle = grd2;
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      t += 0.012;
      animId = requestAnimationFrame(frame);
    }
    frame();
  }

  interface Particle { x: number; y: number; vx: number; vy: number; r: number; alpha: number }
  function startParticles(canvas: HTMLCanvasElement) {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    const ctx = canvas.getContext('2d')!;
    const N = 90;
    const particles: Particle[] = Array.from({ length: N }, () => ({
      x: Math.random() * canvas.width,
      y: Math.random() * canvas.height,
      vx: (Math.random() - 0.5) * 0.3,
      vy: (Math.random() - 0.5) * 0.3,
      r: Math.random() * 2 + 0.5,
      alpha: Math.random() * 0.5 + 0.1,
    }));
    function frame() {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      for (const p of particles) {
        p.x = (p.x + p.vx + canvas.width) % canvas.width;
        p.y = (p.y + p.vy + canvas.height) % canvas.height;
        ctx.beginPath();
        ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(160,180,255,${p.alpha})`;
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
    const N = 120;
    const drops: Drop[] = Array.from({ length: N }, () => ({
      x: Math.random() * canvas.width,
      y: Math.random() * canvas.height,
      len: Math.random() * 18 + 8,
      speed: Math.random() * 3 + 2,
      alpha: Math.random() * 0.25 + 0.05,
    }));
    function frame() {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      for (const d of drops) {
        ctx.beginPath();
        ctx.moveTo(d.x, d.y);
        ctx.lineTo(d.x - 1, d.y + d.len);
        ctx.strokeStyle = `rgba(120,160,255,${d.alpha})`;
        ctx.lineWidth = 0.8;
        ctx.stroke();
        d.y += d.speed;
        if (d.y > canvas.height) { d.y = -d.len; d.x = Math.random() * canvas.width; }
      }
      animId = requestAnimationFrame(frame);
    }
    frame();
  }

  interface MatrixCol { x: number; y: number; speed: number; chars: string[] }
  function startMatrix(canvas: HTMLCanvasElement) {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    const ctx = canvas.getContext('2d')!;
    const fontSize = 13;
    const cols = Math.floor(canvas.width / fontSize);
    const charset = '01アイウエオカキクケコサシスセソタチツテト';
    const columns: MatrixCol[] = Array.from({ length: cols }, (_, i) => ({
      x: i * fontSize,
      y: Math.random() * canvas.height,
      speed: Math.random() * 1.5 + 0.5,
      chars: Array.from({ length: 20 }, () => charset[Math.floor(Math.random() * charset.length)]),
    }));
    function frame() {
      ctx.fillStyle = 'rgba(0,0,0,0.05)';
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      for (const col of columns) {
        const ch = col.chars[Math.floor(col.y / fontSize) % col.chars.length];
        ctx.fillStyle = `rgba(0, 255, 100, 0.18)`;
        ctx.font = `${fontSize}px monospace`;
        ctx.fillText(ch, col.x, col.y);
        col.y += col.speed;
        if (col.y > canvas.height) col.y = 0;
        if (Math.random() < 0.01) col.chars = col.chars.map(() => charset[Math.floor(Math.random() * charset.length)]);
      }
      animId = requestAnimationFrame(frame);
    }
    frame();
  }

  $effect(() => {
    const fx: VisualEffect = $visualEffect;
    stopLoop();
    if (!overlayCanvas) return;
    if (fx === 'aurora') startAurora(overlayCanvas);
    else if (fx === 'particles') startParticles(overlayCanvas);
    else if (fx === 'rain') startRain(overlayCanvas);
    else if (fx === 'matrix') { overlayCanvas.getContext('2d')!.clearRect(0, 0, overlayCanvas.width, overlayCanvas.height); startMatrix(overlayCanvas); }
    else overlayCanvas.getContext('2d')!.clearRect(0, 0, overlayCanvas.width || 0, overlayCanvas.height || 0);
  });

  onMount(() => {
    const interval = setInterval(() => { now = new Date(); }, 60_000);
    const resize = () => {
      if (overlayCanvas && $visualEffect !== 'none') {
        overlayCanvas.width = window.innerWidth;
        overlayCanvas.height = window.innerHeight;
      }
    };
    window.addEventListener('resize', resize);
    return () => { clearInterval(interval); window.removeEventListener('resize', resize); stopLoop(); };
  });

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
  style="z-index: 0; opacity: {$visualEffect === 'none' ? 0 : 1}; transition: opacity 0.6s;"
></canvas>

<div class="relative" style="z-index: 1;">
  <header class="flex items-center justify-between px-8 py-5 border-b border-line">
    <div class="flex items-baseline gap-3">
      <span class="text-lg font-semibold text-ink">{dateLabel}</span>
      <span class="text-sm text-mute">{timeLabel}</span>
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
    <!-- Row 1: Today / Now/Next / Notepad -->
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
    <section class="col-span-8 bg-surface border border-line rounded-xl p-5 h-[28rem]">
      <Calendar />
    </section>

    <section class="col-span-4 bg-surface border border-line rounded-xl p-5 h-[28rem]">
      <Ambience />
    </section>

    <!-- Row 3: Projects / Vault -->
    <section class="col-span-7 bg-surface border border-line rounded-xl p-5 h-[26rem] overflow-y-auto">
      <Projects />
    </section>

    <section class="col-span-5 bg-surface border border-line rounded-xl p-5 h-[26rem]">
      <Vault />
    </section>
  </main>
</div>

<QuickCaptureModal bind:open={captureOpen} onCreated={() => {}} />
