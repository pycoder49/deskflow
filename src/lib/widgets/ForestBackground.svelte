<svg
  class="fixed inset-0 w-full h-full pointer-events-none"
  style="z-index: 0;"
  viewBox="0 0 1400 900"
  preserveAspectRatio="xMidYMax slice"
  xmlns="http://www.w3.org/2000/svg"
  aria-hidden="true"
>
  <defs>
    <linearGradient id="fb-fg" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%"   stop-color="#3a6028" stop-opacity="0.50" />
      <stop offset="100%" stop-color="#1e3a14" stop-opacity="0.85" />
    </linearGradient>
    <linearGradient id="fb-bg" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%"   stop-color="#2a4820" stop-opacity="0.24" />
      <stop offset="100%" stop-color="#162e10" stop-opacity="0.44" />
    </linearGradient>
    <radialGradient id="fb-mist" cx="50%" cy="100%" r="55%">
      <stop offset="0%"   stop-color="#2e5424" stop-opacity="0.20" />
      <stop offset="100%" stop-color="#111a0e" stop-opacity="0"    />
    </radialGradient>
  </defs>

  <!-- Ground mist -->
  <ellipse cx="700" cy="900" rx="900" ry="210" fill="url(#fb-mist)" />

  <!-- Background tree layer — scattered mid-edge -->
  <polygon points="40,820  18,900  62,900"   fill="url(#fb-bg)" />
  <polygon points="110,835 92,900  128,900"  fill="url(#fb-bg)" />
  <polygon points="195,808 169,900 221,900"  fill="url(#fb-bg)" />
  <polygon points="278,838 262,900 294,900"  fill="url(#fb-bg)" />

  <polygon points="1122,838 1106,900 1138,900" fill="url(#fb-bg)" />
  <polygon points="1205,808 1179,900 1231,900" fill="url(#fb-bg)" />
  <polygon points="1290,835 1272,900 1308,900" fill="url(#fb-bg)" />
  <polygon points="1360,820 1338,900 1382,900" fill="url(#fb-bg)" />

  <!-- Foreground tree layer — left cluster framing the edge -->
  <polygon points="0,735   -50,900  50,900"  fill="url(#fb-fg)" />
  <polygon points="72,748  28,900   116,900" fill="url(#fb-fg)" />
  <polygon points="162,765 125,900  199,900" fill="url(#fb-fg)" />
  <polygon points="245,785 215,900  275,900" fill="url(#fb-fg)" />

  <!-- Foreground tree layer — right cluster -->
  <polygon points="1400,735 1350,900 1450,900" fill="url(#fb-fg)" />
  <polygon points="1328,748 1284,900 1372,900" fill="url(#fb-fg)" />
  <polygon points="1238,765 1201,900 1275,900" fill="url(#fb-fg)" />
  <polygon points="1155,785 1125,900 1185,900" fill="url(#fb-fg)" />

  <!-- Top-left corner vine -->
  <path
    d="M -4,0 C 20,32 4,68 36,92 C 68,116 20,158 54,186 C 84,212 28,252 62,280"
    stroke="#2e5828" stroke-width="1.6" fill="none" stroke-opacity="0.44"
  />
  <!-- Side branches -->
  <path d="M 36,92  C 55,78 64,60 50,47"  stroke="#2e5828" stroke-width="1" fill="none" stroke-opacity="0.34" />
  <path d="M 54,186 C 76,172 82,152 66,140" stroke="#2e5828" stroke-width="1" fill="none" stroke-opacity="0.34" />
  <!-- Leaves -->
  <ellipse cx="50"  cy="47"  rx="13" ry="5.5" fill="#265020" fill-opacity="0.32" transform="rotate(38 50 47)"  />
  <ellipse cx="36"  cy="92"  rx="14" ry="6"   fill="#265020" fill-opacity="0.34" transform="rotate(-44 36 92)" />
  <ellipse cx="66"  cy="140" rx="13" ry="5.5" fill="#265020" fill-opacity="0.32" transform="rotate(26 66 140)" />
  <ellipse cx="54"  cy="186" rx="14" ry="6"   fill="#265020" fill-opacity="0.34" transform="rotate(-32 54 186)" />
  <ellipse cx="62"  cy="280" rx="12" ry="5"   fill="#265020" fill-opacity="0.28" transform="rotate(16 62 280)"  />

  <!-- Top-right corner vine (mirrored) -->
  <path
    d="M 1404,0 C 1380,32 1396,68 1364,92 C 1332,116 1380,158 1346,186 C 1316,212 1372,252 1338,280"
    stroke="#2e5828" stroke-width="1.6" fill="none" stroke-opacity="0.44"
  />
  <path d="M 1364,92  C 1345,78 1336,60 1350,47"  stroke="#2e5828" stroke-width="1" fill="none" stroke-opacity="0.34" />
  <path d="M 1346,186 C 1324,172 1318,152 1334,140" stroke="#2e5828" stroke-width="1" fill="none" stroke-opacity="0.34" />
  <ellipse cx="1350" cy="47"  rx="13" ry="5.5" fill="#265020" fill-opacity="0.32" transform="rotate(-38 1350 47)"  />
  <ellipse cx="1364" cy="92"  rx="14" ry="6"   fill="#265020" fill-opacity="0.34" transform="rotate(44 1364 92)"  />
  <ellipse cx="1334" cy="140" rx="13" ry="5.5" fill="#265020" fill-opacity="0.32" transform="rotate(-26 1334 140)" />
  <ellipse cx="1346" cy="186" rx="14" ry="6"   fill="#265020" fill-opacity="0.34" transform="rotate(32 1346 186)"  />
  <ellipse cx="1338" cy="280" rx="12" ry="5"   fill="#265020" fill-opacity="0.28" transform="rotate(-16 1338 280)" />
</svg>
