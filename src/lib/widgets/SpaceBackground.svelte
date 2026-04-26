<script lang="ts">
  const dimStars: [number, number][] = [
    [56, 89], [145, 234], [267, 45], [389, 178], [478, 312],
    [567, 89], [623, 456], [745, 23], [834, 267], [912, 178],
    [1023, 345], [1145, 89], [1234, 456], [1312, 234], [1389, 78],
    [78, 456], [167, 567], [289, 678], [356, 789], [445, 867],
    [534, 123], [645, 234], [712, 567], [823, 678], [934, 789],
    [1012, 567], [1134, 678], [1245, 789], [1334, 867], [28, 534],
    [23, 678], [112, 789], [234, 845], [312, 23], [401, 567],
    [489, 234], [578, 678], [689, 789], [756, 123], [867, 456],
    [945, 234], [1056, 678], [1167, 789], [1256, 345], [1345, 156],
    [199, 145], [366, 512], [544, 389], [799, 534], [1099, 234],
  ];

  const brightStars: [number, number, number][] = [
    [89, 156, 0.80], [234, 89, 0.90], [456, 234, 0.75],
    [678, 345, 0.85], [845, 156, 0.70], [1023, 267, 0.90],
    [1189, 345, 0.80], [1323, 78, 0.75], [167, 678, 0.70],
    [389, 756, 0.85], [723, 812, 0.80], [1089, 723, 0.75],
  ];

  // Bright glint stars with cross sparkle
  const glintStars: [number, number, number][] = [
    [345, 267, 0], [867, 123, 2], [1145, 456, 4], [234, 523, 1], [1267, 678, 3],
  ];
</script>

<svg
  class="fixed inset-0 w-full h-full pointer-events-none"
  style="z-index: 0;"
  viewBox="0 0 1400 900"
  preserveAspectRatio="xMidYMid slice"
  xmlns="http://www.w3.org/2000/svg"
  aria-hidden="true"
>
  <defs>
    <filter id="sb-galaxy-blur" x="-60%" y="-60%" width="220%" height="220%">
      <feGaussianBlur stdDeviation="14" />
    </filter>
    <filter id="sb-star-glow" x="-400%" y="-400%" width="900%" height="900%">
      <feGaussianBlur stdDeviation="2" result="blur" />
      <feMerge>
        <feMergeNode in="blur" />
        <feMergeNode in="SourceGraphic" />
      </feMerge>
    </filter>
    <filter id="sb-sun-glow" x="-100%" y="-100%" width="300%" height="300%">
      <feGaussianBlur stdDeviation="8" result="blur" />
      <feMerge>
        <feMergeNode in="blur" />
        <feMergeNode in="SourceGraphic" />
      </feMerge>
    </filter>

    <radialGradient id="sb-g1" cx="50%" cy="50%" r="50%">
      <stop offset="0%"   stop-color="#9b8eff" stop-opacity="0.40" />
      <stop offset="55%"  stop-color="#7060d4" stop-opacity="0.15" />
      <stop offset="100%" stop-color="#9b8eff" stop-opacity="0" />
    </radialGradient>
    <radialGradient id="sb-g2" cx="50%" cy="50%" r="50%">
      <stop offset="0%"   stop-color="#5880c8" stop-opacity="0.30" />
      <stop offset="100%" stop-color="#5880c8" stop-opacity="0" />
    </radialGradient>
    <radialGradient id="sb-g3" cx="50%" cy="50%" r="50%">
      <stop offset="0%"   stop-color="#c0a0ff" stop-opacity="0.18" />
      <stop offset="100%" stop-color="#c0a0ff" stop-opacity="0" />
    </radialGradient>
    <radialGradient id="sb-sun" cx="50%" cy="50%" r="50%">
      <stop offset="0%"   stop-color="#fffbe0" stop-opacity="1.00" />
      <stop offset="20%"  stop-color="#ffd060" stop-opacity="0.70" />
      <stop offset="60%"  stop-color="#ff8020" stop-opacity="0.25" />
      <stop offset="100%" stop-color="#ff5000" stop-opacity="0" />
    </radialGradient>
  </defs>

  <!-- Galaxies / nebulae -->
  <ellipse cx="1100" cy="155" rx="230" ry="88"
    transform="rotate(-22 1100 155)"
    fill="url(#sb-g1)" filter="url(#sb-galaxy-blur)" opacity="0.85" />

  <ellipse cx="215" cy="720" rx="165" ry="62"
    transform="rotate(18 215 720)"
    fill="url(#sb-g2)" filter="url(#sb-galaxy-blur)" opacity="0.75" />

  <ellipse cx="660" cy="72" rx="295" ry="48"
    transform="rotate(4 660 72)"
    fill="url(#sb-g3)" filter="url(#sb-galaxy-blur)" opacity="0.55" />

  <!-- Sun as a horizon glow — peeking from the bottom edge, warm and subtle -->
  <circle cx="700" cy="960" r="130" fill="url(#sb-sun)" filter="url(#sb-sun-glow)" opacity="0.35" />
  <circle cx="700" cy="960" r="5" fill="#fffbe0" opacity="0.70" />

  <!-- Dim background stars -->
  {#each dimStars as [cx, cy], i}
    <circle
      {cx} {cy}
      r={0.5 + (i % 5) * 0.12}
      fill="white"
      opacity={0.25 + (i % 6) * 0.07}
    />
  {/each}

  <!-- Brighter mid-field stars -->
  {#each brightStars as [cx, cy, op]}
    <circle {cx} {cy} r="1.4" fill="white" opacity={op} filter="url(#sb-star-glow)" />
  {/each}

  <!-- Glint stars — 4-pointed sparkle + core -->
  {#each glintStars as [cx, cy, delay]}
    <g
      filter="url(#sb-star-glow)"
      class="space-twinkle"
      style="animation-delay: {delay}s"
    >
      <line x1={cx - 15} y1={cy}     x2={cx + 15} y2={cy}
        stroke="white" stroke-width="0.45" opacity="0.45" />
      <line x1={cx}     y1={cy - 15} x2={cx}      y2={cy + 15}
        stroke="white" stroke-width="0.45" opacity="0.45" />
      <line x1={cx - 8} y1={cy - 8}  x2={cx + 8}  y2={cy + 8}
        stroke="white" stroke-width="0.28" opacity="0.22" />
      <line x1={cx + 8} y1={cy - 8}  x2={cx - 8}  y2={cy + 8}
        stroke="white" stroke-width="0.28" opacity="0.22" />
      <circle {cx} {cy} r="2.2" fill="white" />
    </g>
  {/each}
</svg>

<style>
  .space-twinkle {
    animation: twinkle 6s ease-in-out infinite;
  }
  @keyframes twinkle {
    0%, 100% { opacity: 0.9; }
    50%       { opacity: 0.45; }
  }
</style>
