<svg
  class="fixed inset-0 w-full h-full pointer-events-none"
  style="z-index: 0;"
  viewBox="0 0 1400 900"
  preserveAspectRatio="xMidYMin slice"
  xmlns="http://www.w3.org/2000/svg"
  aria-hidden="true"
>
  <defs>
    <!-- Cloud fill: slightly lighter than the cloudy base -->
    <linearGradient id="cb-cloud-fill" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%"   stop-color="#f4f8fc" stop-opacity="0.55" />
      <stop offset="100%" stop-color="#dce8f2" stop-opacity="0.30" />
    </linearGradient>
    <!-- Darker underbelly for depth -->
    <linearGradient id="cb-cloud-shadow" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%"   stop-color="#c8d8e8" stop-opacity="0.28" />
      <stop offset="100%" stop-color="#b0c4d6" stop-opacity="0.12" />
    </linearGradient>
    <!-- Subtle horizon haze -->
    <linearGradient id="cb-haze" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%"   stop-color="#9ab8d0" stop-opacity="0.14" />
      <stop offset="100%" stop-color="#9ab8d0" stop-opacity="0"    />
    </linearGradient>
  </defs>

  <!-- Horizon atmospheric haze — very faint band at top -->
  <rect x="0" y="0" width="1400" height="220" fill="url(#cb-haze)" />

  <!-- ── Cloud 1 — large, far left, mostly off-screen top ─────────────── -->
  <g>
    <ellipse cx="90"  cy="-18" rx="105" ry="72" fill="url(#cb-cloud-fill)" />
    <ellipse cx="175" cy="-30" rx="88"  ry="64" fill="url(#cb-cloud-fill)" />
    <ellipse cx="140" cy="18"  rx="120" ry="60" fill="url(#cb-cloud-fill)" />
    <ellipse cx="230" cy="8"   rx="75"  ry="52" fill="url(#cb-cloud-fill)" />
    <!-- underbelly shadow -->
    <ellipse cx="155" cy="48"  rx="130" ry="28" fill="url(#cb-cloud-shadow)" />
  </g>

  <!-- ── Cloud 2 — medium, upper centre-left ───────────────────────────── -->
  <g>
    <ellipse cx="420" cy="-22" rx="90"  ry="60" fill="url(#cb-cloud-fill)" />
    <ellipse cx="510" cy="-38" rx="72"  ry="55" fill="url(#cb-cloud-fill)" />
    <ellipse cx="475" cy="12"  rx="105" ry="52" fill="url(#cb-cloud-fill)" />
    <ellipse cx="560" cy="2"   rx="64"  ry="44" fill="url(#cb-cloud-fill)" />
    <ellipse cx="480" cy="42"  rx="112" ry="24" fill="url(#cb-cloud-shadow)" />
  </g>

  <!-- ── Cloud 3 — large, top centre ──────────────────────────────────── -->
  <g>
    <ellipse cx="730"  cy="-40" rx="120" ry="80"  fill="url(#cb-cloud-fill)" />
    <ellipse cx="840"  cy="-55" rx="95"  ry="70"  fill="url(#cb-cloud-fill)" />
    <ellipse cx="790"  cy="10"  rx="135" ry="65"  fill="url(#cb-cloud-fill)" />
    <ellipse cx="660"  cy="-10" rx="80"  ry="58"  fill="url(#cb-cloud-fill)" />
    <ellipse cx="920"  cy="0"   rx="70"  ry="50"  fill="url(#cb-cloud-fill)" />
    <!-- underbelly -->
    <ellipse cx="790"  cy="50"  rx="148" ry="30"  fill="url(#cb-cloud-shadow)" />
  </g>

  <!-- ── Cloud 4 — small wisp, upper centre-right ──────────────────────── -->
  <g>
    <ellipse cx="1050" cy="-15" rx="75"  ry="48" fill="url(#cb-cloud-fill)" />
    <ellipse cx="1130" cy="-25" rx="60"  ry="42" fill="url(#cb-cloud-fill)" />
    <ellipse cx="1090" cy="15"  rx="88"  ry="40" fill="url(#cb-cloud-fill)" />
    <ellipse cx="1090" cy="38"  rx="92"  ry="20" fill="url(#cb-cloud-shadow)" />
  </g>

  <!-- ── Cloud 5 — large, far right ───────────────────────────────────── -->
  <g>
    <ellipse cx="1260" cy="-20" rx="100" ry="68" fill="url(#cb-cloud-fill)" />
    <ellipse cx="1355" cy="-32" rx="82"  ry="60" fill="url(#cb-cloud-fill)" />
    <ellipse cx="1315" cy="16"  rx="115" ry="55" fill="url(#cb-cloud-fill)" />
    <ellipse cx="1315" cy="46"  rx="118" ry="26" fill="url(#cb-cloud-shadow)" />
  </g>

  <!-- ── Distant small wisps for depth ────────────────────────────────── -->
  <ellipse cx="320"  cy="72"  rx="55" ry="28" fill="url(#cb-cloud-fill)" opacity="0.45" />
  <ellipse cx="320"  cy="88"  rx="60" ry="14" fill="url(#cb-cloud-shadow)" opacity="0.35" />

  <ellipse cx="640"  cy="82"  rx="48" ry="24" fill="url(#cb-cloud-fill)" opacity="0.38" />
  <ellipse cx="640"  cy="96"  rx="52" ry="12" fill="url(#cb-cloud-shadow)" opacity="0.28" />

  <ellipse cx="990"  cy="68"  rx="60" ry="26" fill="url(#cb-cloud-fill)" opacity="0.40" />
  <ellipse cx="990"  cy="84"  rx="64" ry="13" fill="url(#cb-cloud-shadow)" opacity="0.30" />

  <ellipse cx="1190" cy="78"  rx="44" ry="20" fill="url(#cb-cloud-fill)" opacity="0.35" />
</svg>
