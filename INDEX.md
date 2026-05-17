# DeskFlow — File Index

**Last updated:** 2026-05-13 (cleanup pass: removed dead Projects/Vault/AmbientPanel/LayoutMocks; ambience store split)
**Update rule:** Touch this file any time a source file is added, deleted, or renamed. One line per entry.
**How to use:** Match user's task keywords against the descriptions below. If you find a hit, open that file directly — don't grep.

---

## Frontend — Routes (`src/routes/`)

- `+layout.svelte` — App shell: imports `app.css`, initializes theme store, starts `startTaskPolling()`, fires `triggerStartupRetry()` after 2.5s on mount, renders 48px left sidebar (Dashboard/Workout/Research nav — Workout/Research show "coming soon"), conditionally renders SpaceBackground / ForestBackground / CloudBackground per theme. Tags: shell, sidebar, polling start, startup retry, theme backgrounds.
- `+layout.ts` — Disables SSR (Tauri SPA mode). Tags: SSR, SPA.
- `+page.svelte` — Dashboard grid: header (Start Day + New Task + 8-theme dropdown), flex layout (TodayTasks, NowNext, TaskStats, Calendar, Ambience, Notepad), fixed canvas overlay driven by `backgroundEffect` store (particles/rain/snow/fireflies/fog), scanlines overlay + breathe class driven by `sectionEffect`. Tags: layout, grid, mount widgets, theme dropdown, canvas overlay.

## Frontend — Widgets (`src/lib/widgets/`)

- `TodayTasks.svelte` — Daily To-Do list. Checkbox toggles complete/uncheck; pencil = edit modal (name/priority/due/time-estimate); trash = two-click delete. Merges server tasks + `completedTasks` overlay; writes IDs to `todayIds`. Tag pills with dynamic overflow (canvas `measureText` + ResizeObserver, +N badge). Tags: tasks, complete, uncheck, delete, edit, tag pills, overflow.
- `NowNext.svelte` — AI-picked focus: 1 Now + up to 2 Next. Hides `completedTasks`; auto-reloads on `clickupVersion` (`preserveNow=true`); writes display IDs to `nowNextIds`. Tags: AI picker, Now/Next, focus, refresh.
- `QuickCaptureModal.svelte` — Add-task modal (title + list + Area + priority); auto-tags with list/Area slug; Ctrl+N open, Ctrl+Enter save, Esc close; bumps `clickupVersion`. Tags: add task, create, modal, keyboard.
- `Calendar.svelte` — List/Day/Week/Month views (default Week 7-col). Day = 7am–11pm hour grid; Month = 7×6 grid with colored dots + hover popup. Logical-day aware (`getDate()`, never `toISOString()`). Tags: calendar, events, week, day, month.
- `Notepad.svelte` — Quick-notes scratchpad. Loads on mount, auto-saves debounced 800ms, manual Save button flashes "Saved ✓". Tags: notepad, save, autosave.
- `Ambience.svelte` — Tabs: Sounds (4 ambient tiles + Spotify: top-5 MRU playlists scrollable when no session, player when active), Visuals (bg + section toggles + 6 named Mood Presets — each sets sound+bg+section via `applyPreset()`), Terminal (xterm.js + real PTY). `PAIRINGS` array drives presets with `{ name, emoji, sound, bg, sec }`. Tags: ambience, sounds, spotify, music, playlists, mood presets, visual effect, xterm, PTY.
- `TaskStats.svelte` — 3-tab completion widget. "line": 14-day SVG area+line+dots chart. "bars": same data as rounded bar chart. "done": today's completed tasks (from `get_completed_today_tasks`) in up to 4 columns of 3, priority dot + truncated name; re-fetches on `clickupVersion` bump. Refresh button reloads all data. Tags: stats, chart, completion count, done today, hover tooltip.
- `SpaceBackground.svelte` — Fixed SVG starfield + galaxies. Active when theme = "space". Tags: theme background, space.
- `ForestBackground.svelte` — Fixed SVG forest layer (pines + corner vines). Active when theme = "forest". Tags: theme background, forest.
- `CloudBackground.svelte` — Fixed SVG cloud layer (5 cloud formations + distant wisps, anchored to top via `preserveAspectRatio="xMidYMin slice"`). Active when theme = "cloudy". Tags: theme background, cloudy, clouds.

## Frontend — Services (`src/lib/services/`)

- `clickup.ts` — Types (`Task`, `TaskTag`, `NowNextResult`, `StatEntry`); constants (`LISTS`, `AREAS`, `PRIORITY_META`); invoke wrappers (`getTodayTasks`, `getCompletedTodayTasks`, `getNowNext`, `createTask`, `completeTask`, `uncheckTask`, `deleteTask`, `updateTask`, `ensureLogDoc`, `startDay`, `getTaskStats`). Tags: ClickUp types, lists, areas, priorities.
- `calendar.ts` — `CalendarEvent` type, `CALENDAR_COLORS` map, `eventColor(name)` helper, `getCalendarEvents(start, end)` wrapper. Tags: calendar service.
- `notepad.ts` — `getNotepad()`, `saveNotepad(content)` wrappers. Tags: notepad service.

## Frontend — Stores (`src/lib/stores/`)

- `theme.ts` — Eight-mode store (light/dark/space/nord/forest/vintage/slate/cloudy); persists to localStorage; sets class on `<html>`; exposes `cycle()`. Tags: theme, dark mode.
- `refresh.ts` — Cross-widget signals + poll loop. Exports: `logicalToday()` (4am-shifted), `clickupVersion` (bump AFTER API ack, only when mutated task is in `$nowNextIds`), `nowNextIds`, `todayIds`, `completedTasks` (Map overlay), `startTaskPolling()` (30s interval, pauses on hidden, diffs IDs), `startupRetry` + `triggerStartupRetry()` (one-shot startup retry signal). Tags: refresh, logicalToday, polling, completed overlay, version bump, startup retry.
- `ambience.ts` — `BackgroundEffect` + `SectionEffect` types; `backgroundEffect` and `sectionEffect` writables. Read by `+page.svelte` for canvas overlay (particles/rain/snow/fireflies/fog) and section effects (scanlines/breathe). Tags: visual effect, canvas overlay, section effect.
- `config.ts` — `AppConfig` type + `config` writable + `loadConfig()` (calls `get_config` Tauri command on app start). Derived stores: `lists` (Daily + areas) and `areas` (areas only) feed QuickCaptureModal. Tags: config store, lists, areas, setup.

## Frontend — Shared

- `src/app.html` — HTML shell (title: "DeskFlow").
- `src/app.css` — Tailwind entry + `@theme` tokens + theme overrides (`.dark`, `.space`, `.nord`, `.forest`, `.vintage`, `.slate`, `.cloudy`); per-theme `--color-chart-*` vars; themed scrollbars. Tags: theme CSS, colors, scrollbar.
- `src/lib/types.ts` — Shared TS types (currently just `Theme` enum). Tags: shared types.

## Backend — Rust Modules (`src-tauri/src/`)

- `main.rs` — Binary entry; delegates to `lib::run()`.
- `lib.rs` — Tauri app setup; loads `.env` via `dotenvy`; registers all commands in `generate_handler![…]`. Tags: command registration, env loading.
- `clickup.rs` — ClickUp v2 tasks HTTP; `claude -p` shell-out for AI pick (`claude_pick_full`, `claude_pick_next`); Today-doc logging via `scripts/log_action.py`; `extract_json()` helper; logical-day comparison; calendar context injected into AI prompts via `calendar::fetch_events`. Daily list ID comes from `config::get().clickup.daily_list_id`. Tags: ClickUp HTTP, AI picker, doc logging, claude -p.
- `calendar.rs` — gcalcli shell-out (`--tsv --details=calendar`); 6-column TSV parse; personal calendar name comes from `config::get().calendar.personal_email`, secondary names in `EXTRA_CALENDARS` const; requires `PYTHONIOENCODING=utf-8`. Public `fetch_events(start, end)` for inter-module use. Tags: gcalcli, calendar shell-out, calendar filter.
- `config.rs` — Loads `os-config.json` (project root) at app start into a `OnceLock<Config>`. Exposes `config::get()` for Rust callers and a `get_config` Tauri command for the frontend. Schema: `clickup{workspace_id, logs_folder_id, daily_list_id, areas[]}`, `calendar{personal_email}`. Tags: config, setup, personalization.
- `notepad.rs` — `get_notepad`, `save_notepad`, `save_to_path` commands. Resolves `notepad.md` via `CARGO_MANIFEST_DIR/../notepad.md`. Tags: notepad I/O, project root path.
- `terminal.rs` — Real PTY via `portable-pty`: `pty_create(rows,cols)`, `pty_write(data)`, `pty_resize(rows,cols)`, `pty_kill()`. Output streamed via `pty-data` Tauri event. Tags: terminal, PTY, xterm.
- `spotify.rs` — Spotify Web API: OAuth flow (local redirect server on 127.0.0.1:8888), token storage at `~/.claude/spotify-tokens.json`, auto-refresh. Scopes include `user-read-recently-played` so `spotify_get_playlists` returns MRU-sorted (decorates each `Playlist` with `last_played_at` from `/me/player/recently-played`). `SpotifyState` includes `track_id`. Tags: spotify, music, OAuth, playback, MRU sort, recently played, last_played_at, audio features, energy, tempo, waveform.

## Backend — Commands Registered

(Defined in `lib.rs::generate_handler!`. Source module in parens.)

- `get_config` (config) — Returns the loaded `os-config.json` (clickup IDs + calendar email) to the frontend
- `get_today_tasks` (clickup) — GET Daily To-Do incomplete tasks
- `get_completed_today_tasks` (clickup) — GET tasks completed since today's 4am logical-day cutoff
- `get_now_next(preserve_now)` (clickup) — Cache HIT if pool unchanged; MISS + preserve_now → pick Next only; otherwise full re-pick
- `create_task(name, list_id, priority, tags)` (clickup) — POST task; appends `**add**` to Today doc
- `complete_task(task_id, task_name)` (clickup) — PUT status=complete; appends `**complete**` to Today doc
- `uncheck_task(task_id, task_name, tags)` (clickup) — PUT status="in progress"; logs via `log_action.py`
- `delete_task(task_id, task_name, tags)` (clickup) — DELETE task; logs via `log_action.py`
- `update_task(task_id, task_name, tags, new_name?, new_priority?, new_due_date?, new_time_estimate?, details?)` (clickup) — PUT field changes; logs diff via `log_action.py`
- `get_task_stats()` (clickup) — Reads `~/.claude/task-stats.json`; returns last 14 logical days as `[{date, count}]`
- `ensure_log_doc()` (clickup) — Shells `log_action.py --ensure`; creates monthly doc if missing
- `start_day()` (clickup) — Shells `claude -p /<commands.start_day_skill>` from project root (default skill: `start-day`, shipped at `.claude/skills/start-day/`). Idempotency lives in the skill.
- `get_calendar_events(start, end)` (calendar) — gcalcli agenda; ISO dates or phrases ("today", "in 7 days")
- `get_notepad()` / `save_notepad(content)` / `save_to_path(content, path)` (notepad)
- `pty_create(rows, cols)` / `pty_write(data)` / `pty_resize(rows, cols)` / `pty_kill()` (terminal)
- `spotify_is_authenticated()` / `spotify_auth()` / `spotify_get_state()` / `spotify_play()` / `spotify_pause()` / `spotify_next()` / `spotify_prev()` / `spotify_get_playlists()` / `spotify_get_playlist_tracks(playlist_id)` / `spotify_play_context(...)` / `spotify_set_shuffle(...)` / `spotify_play_uri(...)` / `spotify_search(query)` / `spotify_get_token()` / `spotify_transfer_playback(device_id)` / `spotify_get_devices()` / `spotify_get_beats(track_id)` / `spotify_get_audio_features(track_id)` (spotify)

## Config

- `package.json` — JS deps + scripts (`dev`, `build`, `check`, `tauri`)
- `svelte.config.js` — adapter-static SPA mode (`fallback: index.html`)
- `vite.config.js` — Vite + `tailwindcss()` + `sveltekit()`; dev port 1420
- `tsconfig.json` — Extends SvelteKit default
- `src-tauri/Cargo.toml` — Rust deps: tauri 2, tauri-plugin-opener, tauri-plugin-dialog, serde, reqwest 0.12, dotenvy, tokio (process + io-util + rt), chrono, dirs, portable-pty
- `src-tauri/tauri.conf.json` — Window "DeskFlow" 1400×900 (min 1100×700); identifier `com.deskflow.app`
- `src-tauri/capabilities/default.json` — Tauri permission scopes (must include `opener:allow-open-url` for `obsidian://`; `dialog:allow-save` + `dialog:allow-open`)
- `.env.example` — Template for `CLICKUP_TOKEN`, `VAULT_PATH`, `SPOTIFY_CLIENT_ID`, `SPOTIFY_CLIENT_SECRET`
- `.gitignore` — `.env`, `notepad.md`, `hot.md`, `BUILD_NOTES.md`, `projects.json` excluded (personal/session data)
- `notepad.md` — User quick-notes data file; project root
- `static/sounds/` — 4 ambient mp3s (cafe, cyberpunk, medieval, rain); populated by `scripts/download_sounds.py`

## Scripts (`scripts/`)

- `setup.py` — Interactive first-run wizard. Reads `CLICKUP_TOKEN` from `.env`, walks the user through picking workspace + daily list + area lists + logs folder, runs `gcalcli list` for calendar pick, writes `os-config.json` at project root, then invokes `download_sounds.py` for ambient sounds. If `RANDOM_COMPLETIONS_ON_DOWNLOAD=True` and `~/.claude/task-stats.json` doesn't exist, seeds 14 days of demo completion counts (never overwrites real data). Re-runnable. Tags: setup, wizard, configuration, demo seed.
- `tag-tasks.sh` — Bulk-add ClickUp tags by ID pairs or list ID; reads `CLICKUP_TOKEN` from `.env`
- `log_action.py` — Append a task action entry to whichever destination is configured in `os-config.json::logging.mode`. Three modes: `clickup_doc` (monthly docs in a ClickUp Logs folder; caches doc ID in `~/.claude/clickup-log-state.json`), `local_file` (Markdown/text file at `logging.local_file_path`, absolute or relative to project root), `none` (silent no-op). 4am-aware (`logical_today()`); same date-header / same-day-append logic for both file backends. Args: `--action`, `--task-name`, `--tags`, `--details`, `--ensure`. Tags: log action, logging mode, local file, ClickUp doc.
- `record_stats.py` — Write today's completion count to `~/.claude/task-stats.json` (logical day). Called by `/check-out` with `--count N`. Tags: stats writer.
- `start_day.py` — Bookkeeping for the dashboard's "Start Day" button. Three job groups + three operating modes. **Jobs:** (1) refresh + backfill completion stats — re-records yesterday's count, backfills missing days in the last 14 by querying ClickUp for `complete` tasks with `date_closed` in each 4 am → 4 am window; (2) pulls today's calendar via `gcalcli` using `calendar.personal_email`; (3) moves incomplete tasks from each area list to the daily list (`PUT /task/{id}` with `list`), applies area slug tag (POST `/task/{id}/tag/{slug}`), calls `log_action.py --action move ...` for each — gated to once per logical day via `~/.claude/dashboard-state.json`. **Modes:** `--audit` (emit JSON snapshot of stats + calendar + per-area task inventory; no moves — for skill consumption), `--moves-only` (skip stats + calendar; just do the gated move step; pair with `--skip "id1,id2"` to defer tasks), default (all three jobs, terminal-friendly output). `--force` bypasses the gate. Invoked by the shipped AI-driven `/start-day` skill via the audit → judgment → moves-only-with-skip flow. Tags: start day, audit, moves-only, skip, slug tag, stats backfill, calendar context, idempotency gate.
- `download_sounds.py` — Download the 4 ambient sound clips via `yt-dlp` + `ffmpeg` into `static/sounds/`. Reads URLs from `os-config.json` (`ambience.sounds`); falls back to hardcoded defaults if missing. Tracks last-downloaded URL per slot in `static/sounds/.manifest.json` and only re-pulls slots whose URL changed. `--force` re-downloads everything. Called automatically by `setup.py`. Tags: ambience setup, sounds, yt-dlp, manifest.

## Skills (`.claude/skills/`)

- `start-day/SKILL.md` — Shipped default for the "Start Day" button. AI-driven: runs `start_day.py --audit` to get stats + today's calendar + per-area task inventory as JSON, reasons about whether the day's tasks fit available work time (calendar-aware), then runs `start_day.py --moves-only --skip "ids"` to execute moves with deferred tasks excluded. Generic judgment only — no personal heuristics. Override by setting `commands.start_day_skill` in `os-config.json` (e.g. to `check-in` for a personal skill in `~/.claude/skills/`). Tags: start day, skill, AI-driven, calendar-aware, audit, defer.

## Docs

- `CLAUDE.md` — Routing rules (read first every session)
- `INDEX.md` — This file
- `hot.md` — Session handoff / active work / what's next
- `BUILD_NOTES.md` — Build log (append only, newest on top)
- `IDEAS.md` — Layout critique, future widget direction, open design questions
- `README.md` — Project intro, setup walkthrough, troubleshooting
