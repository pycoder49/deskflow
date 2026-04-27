# Aryan OS Dashboard — File Index

**Last updated:** 2026-04-26 (Ambience widget — Sounds/Music/Visuals tabs + visual overlay)
**Update rule:** Touch this file any time a source file is added, deleted, or renamed. One line per entry.

---

## Frontend — Routes (`src/routes/`)

- `+layout.svelte` — App shell: imports `app.css`, initializes theme store, starts `startTaskPolling()`, renders children
- `+layout.ts` — Disables SSR (Tauri SPA mode)
- `+page.svelte` — Dashboard main: header (New Task + theme toggle), grid row 1 (Today col-4 + NowNext col-4 + Notepad col-4), row 2 (Calendar col-8 + Ambience col-4), row 3 (Vault col-5 + Projects col-7); fixed canvas visual-effect overlay driven by `visualEffect` store

## Frontend — Widgets (`src/lib/widgets/`)

- `TodayTasks.svelte` — Truth list of all Daily To-Do tasks; checkbox toggles complete↔uncheck; post-toggle branches on `nowNextIds` (in-display → `bumpClickup()`, else local `load()`); merges server tasks + `completedTasks` overlay (strikethrough + dulled); writes server IDs to `todayIds` (poll baseline); tag pills
- `NowNext.svelte` — AI-picked focus: 1 Now (accent border) + up to 2 Next; hides `completedTasks`; auto-reloads on `clickupVersion` with `preserveNow=true`; writes display IDs to `nowNextIds` after each load; own ↻ always bumps (task is in display by definition)
- `QuickCaptureModal.svelte` — Add-task modal (title + list + Area + priority); auto-tags with list slug or Area slug for Daily To-Do; Ctrl+N open, Ctrl+Enter save, Esc close; bumps `clickupVersion` on success
- `Calendar.svelte` — Four views (List / Day / Week / Month); default = Week (7-col Mon–Sun grid); Day = 7am–11pm hour grid; Month = 7×6 grid with colored dots + hover popup; event colors = left-border + tint throughout; tab nav uses theme accent; logical-day aware (local `getDate()`, not `toISOString()`)
- `Vault.svelte` — Three tabs (Hot / Inbox / Graph); Hot = all `## ` sections from `hot.md` in order + counts; Inbox = uningested `raw/` files + recent log strip; Graph = d3-force wikilink graph (nodes by type: source/topic/entity/query, edges from `[[wikilinks]]`); file clicks open in Obsidian via `obsidian://`
- `Notepad.svelte` — Quick-notes scratchpad; loads from disk on mount; auto-saves (debounced 800ms) on every keystroke; manual Save button flashes "Saved ✓" for 1.5s; mounted in row-1 col-3 slot (top right)
- `Projects.svelte` — Per-project context viewer; cards expand to show `context.md` from each project root; Add Project modal (path input, auto-derives name); remove button; persists to `projects.json`
- `Ambience.svelte` — Three tabs: Sounds (Web Audio API: Rain, White, Brown, Pink noise + Lo-fi link + volume), Music (Spotify coming soon), Visuals (background effect picker → writes `visualEffect` store)
- `SpaceBackground.svelte` — Fixed SVG starfield + galaxies; active in "space" theme only

## Frontend — Services (`src/lib/services/`)

- `clickup.ts` — Task/TaskTag/NowNextResult types, LISTS (with slug), AREAS, PRIORITY_META; invoke wrappers: `getTodayTasks`, `getNowNext(preserveNow)`, `createTask`, `completeTask`, `uncheckTask`
- `calendar.ts` — CalendarEvent type, CALENDAR_COLORS map, `eventColor(name)` helper, `getCalendarEvents(start, end)` invoke wrapper
- `vault.ts` — VaultPulse/VaultCounts/InboxItem/LogDay/HotSection/GraphNode/GraphEdge/GraphData types, `getVaultPulse()` invoke wrapper, `obsidianUri(vault, path)` helper
- `notepad.ts` — `getNotepad()` / `saveNotepad(content)` invoke wrappers
- `projects.ts` — `Project` type, `getProjects()` / `addProject(path)` / `removeProject(id)` invoke wrappers

## Frontend — Stores (`src/lib/stores/`)

- `theme.ts` — Three-mode store (light/dark/space); persists to localStorage; sets `.dark` or `.space` on `<html>`; exposes `cycle()`
- `refresh.ts` — Cross-widget signals + external poll. `clickupVersion`: bump AFTER API ack to trigger NowNext re-pick — only bump when mutated task was in `$nowNextIds`. `nowNextIds`: NowNext writes display IDs after each load; TodayTasks reads to branch on toggle. `todayIds`: TodayTasks writes server IDs after each load; poll diffs against this. `completedTasks`: Map overlay (Today = strikethrough, NowNext = hidden). `startTaskPolling()`: 30s interval, pauses on hidden, diffs IDs, bumps only on change.
- `ambience.ts` — `VisualEffect` type + `visualEffect` writable store; read by `+page.svelte` to drive canvas overlay (aurora/particles/rain/matrix/none)

## Frontend — Shared

- `src/app.html` — HTML shell (title: "Aryan OS")
- `src/app.css` — Tailwind entry + `@theme` tokens + `.dark` and `.space` overrides
- `src/lib/types.ts` — Shared TypeScript types (currently just `Theme` enum)

## Backend — Rust Modules (`src-tauri/src/`)

- `main.rs` — Binary entry; delegates to `lib::run()`
- `lib.rs` — Tauri app setup, loads `.env` via `dotenvy`, registers all commands
- `clickup.rs` — ClickUp v2 tasks + v3 docs HTTP; `claude -p` shell-out for Now/Next AI pick; Today-doc logging; `extract_json()` helper; logical-day comparison; calendar-context injection into AI prompts (calls `calendar::fetch_events`, formats upcoming events + current time, injects into both `claude_pick_full` and `claude_pick_next`)
- `calendar.rs` — gcalcli shell-out (`--tsv --details=calendar`); parses 6-column TSV; `CALENDARS` const filter; all-day event detection; requires `PYTHONIOENCODING=utf-8`; public `fetch_events(start, end)` for inter-module use (used by `clickup.rs` for NowNext calendar context)
- `vault.rs` — Reads AryanOS vault (env `VAULT_PATH`, default `C:/Users/aryan/Desktop/AryanOS`); parses `wiki/hot.md` into `HotSection` list; `wiki/index.md` counts; `wiki/log.md` recent days; inbox = `raw/notes/` + `raw/clips/` not referenced in `wiki/sources/*.md`; builds graph from `wiki/{sources,topics,entities,queries}/*.md` + root `topics/*.md` + `raw/{notes,clips}/*.md` (node types: source/topic/entity/query/raw)
- `notepad.rs` — `get_notepad` / `save_notepad` commands; resolves `notepad.md` via `CARGO_MANIFEST_DIR/../notepad.md` (project root)
- `projects.rs` — `get_projects` / `add_project(path)` / `remove_project(id)` commands; persists to `projects.json` at project root; eagerly reads `context.md` from each project path

## Backend — Commands Registered

- `get_today_tasks` — GET Daily To-Do incomplete tasks — `clickup.rs`
- `get_now_next(preserve_now)` — Cache HIT if pool unchanged; MISS + preserve_now → pick Next only; otherwise full `claude -p` re-pick — `clickup.rs`
- `create_task(name, list_id, priority, tags)` — POST task with tags; appends `**add**` to Today doc — `clickup.rs`
- `complete_task(task_id, task_name)` — PUT status=complete; appends `**complete**` to Today doc — `clickup.rs`
- `uncheck_task(task_id, task_name)` — PUT status="in progress"; appends `**uncheck**` to Today doc — `clickup.rs`
- `get_calendar_events(start, end)` — gcalcli agenda shell-out; ISO dates or phrases ("today", "in 7 days") — `calendar.rs`
- `get_vault_pulse()` — Returns VaultPulse (hot_sections, counts, inbox, recent_log, graph) — `vault.rs`
- `get_notepad()` — Reads `notepad.md` at project root; returns `""` if not found — `notepad.rs`
- `save_notepad(content)` — Writes full content to `notepad.md` at project root — `notepad.rs`
- `get_projects()` — Returns all projects with eagerly loaded `context.md` content — `projects.rs`
- `add_project(path)` — Appends project to `projects.json`; derives name from folder; returns updated list — `projects.rs`
- `remove_project(id)` — Removes project by id (path) from `projects.json`; returns updated list — `projects.rs`

## Config

- `package.json` — JS deps + scripts (`dev`, `build`, `check`, `tauri`)
- `svelte.config.js` — adapter-static SPA mode (`fallback: index.html`)
- `vite.config.js` — Vite + `tailwindcss()` + `sveltekit()` plugins; dev server port 1420
- `tsconfig.json` — TypeScript config (extends SvelteKit default)
- `src-tauri/Cargo.toml` — Rust deps: tauri 2, serde, reqwest 0.12, dotenvy, tokio (process + io-util), chrono, dirs, tauri-plugin-opener
- `src-tauri/tauri.conf.json` — Window "Aryan OS" 1400×900 (min 1100×700); identifier `com.aryan.osdashboard`
- `src-tauri/capabilities/default.json` — Tauri permission scopes
- `.env.example` — Template for `CLICKUP_TOKEN`; no Anthropic key needed (uses `claude -p`)
- `.gitignore` — `.env` excluded; secrets never committed
- `notepad.md` — User quick-notes data file; created on first save; lives at project root (gitignored not required, no secrets)
- `projects.json` — Persisted project list `[{id, name, path}]`; created on first add; lives at project root

## Scripts (`scripts/`)

- `tag-tasks.sh` — Bulk-add ClickUp tags by ID pairs or by list ID; reads `CLICKUP_TOKEN` from `.env`

## Docs

- `CLAUDE.md` — Routing rules (read first every session)
- `INDEX.md` — This file
- `hot.md` — Session handoff / active work / what's next
- `BUILD_NOTES.md` — Build log (append only, newest on top)
- `IDEAS.md` — Layout critique, future widget direction, open design questions
- `README.md` — Tauri scaffold default (may rewrite later)
