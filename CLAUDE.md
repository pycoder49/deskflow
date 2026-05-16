# CLAUDE.md — Aryan OS Dashboard

> Read this file FIRST every session. It is your routing table.
> Do not read files outside the task's Read column. Do not grep to "explore" — the codebase is mapped.

---

## Identity

Aryan's personal dashboard — Tauri 2 desktop app aggregating daily workflow (ClickUp tasks, Google Calendar, vault knowledge, AI focus-picking).

**Owner:** Aryan — CS student, UCSD 2026, Windows.
**Related vault:** `C:/Users/aryan/Desktop/AryanOS/` — has its own CLAUDE.md. Cross-load only when the task explicitly touches vault files.

---

## Stack

- **Desktop shell:** Tauri 2 (Rust) — Native Windows; FS, subprocess, HTTP
- **Frontend:** SvelteKit 2 SPA (adapter-static, no SSR — Tauri is not a Node server)
- **Languages:** TypeScript (frontend) + Rust (backend)
- **Styling:** Tailwind CSS 4 + `@theme` (CSS variables; no `theme.extend`)
- **Integrations:** ClickUp HTTP API, gcalcli (shell-out), vault markdown
- **AI calls:** `claude -p` subprocess — uses Aryan's subscription, no API key

---

## Path Conventions (Names → Paths — Construct, Don't Search)

Build any file path from its name. Never `Glob` to "find" a known archetype.

| Archetype | Path | Example |
|---|---|---|
| Widget `Foo` | `src/lib/widgets/Foo.svelte` | `TodayTasks.svelte` |
| Service `foo` | `src/lib/services/foo.ts` | `clickup.ts` |
| Store `foo` | `src/lib/stores/foo.ts` | `refresh.ts` |
| Rust module `foo` | `src-tauri/src/foo.rs` | `clickup.rs` |
| Rust command `foo_bar` ↔ TS `fooBar` | registered in `src-tauri/src/lib.rs` | `get_today_tasks` ↔ `getTodayTasks` |

---

## Symbol Lookup (Where Things Live — Check Before Grepping)

| Symbol | File |
|---|---|
| `Task`, `TaskTag`, `NowNextResult`, `StatEntry` types | `src/lib/services/clickup.ts` |
| `LISTS`, `AREAS`, `PRIORITY_META` constants | `src/lib/services/clickup.ts` |
| ClickUp invoke wrappers (`getTodayTasks`, `completeTask`, etc.) | `src/lib/services/clickup.ts` |
| `CalendarEvent`, `eventColor`, `getCalendarEvents` | `src/lib/services/calendar.ts` |
| `logicalToday()` (4am-shifted date) | `src/lib/stores/refresh.ts` |
| `clickupVersion`, `nowNextIds`, `todayIds`, `completedTasks` | `src/lib/stores/refresh.ts` |
| `startTaskPolling()` (30s loop) | `src/lib/stores/refresh.ts` |
| `backgroundEffect` + `sectionEffect` (canvas overlay + section FX) | `src/lib/stores/ambience.ts` |
| `theme` store + `cycle()` | `src/lib/stores/theme.ts` |
| Theme CSS variables, `@theme`, themed scrollbars | `src/app.css` |
| Tauri command registration (`generate_handler!`), `.env` loading | `src-tauri/src/lib.rs` |
| ClickUp HTTP, AI picker (`claude_pick_*`), `extract_json()` | `src-tauri/src/clickup.rs` |
| Config loading (`config::get()`, `get_config` cmd) | `src-tauri/src/config.rs` |
| Frontend config store (`config`, `lists`, `areas`, `loadConfig()`) | `src/lib/stores/config.ts` |
| gcalcli shell-out, `CALENDARS`, `fetch_events()` | `src-tauri/src/calendar.rs` |
| `notepad.md` I/O | `src-tauri/src/notepad.rs` |
| `cmd /C` shell runner | `src-tauri/src/terminal.rs` |
| Spotify OAuth + `Playlist` (incl. `last_played_at`), MRU sort via `/me/player/recently-played`, `SCOPES_ENC` | `src-tauri/src/spotify.rs` |
| ClickUp log doc append (4am-aware) | `scripts/log_action.py` |
| Task stats writer | `scripts/record_stats.py` |
| Tauri permission scopes | `src-tauri/capabilities/default.json` |
| Window size, app identifier | `src-tauri/tauri.conf.json` |

---

## Reference Patterns (Copy These — Don't Browse Siblings)

| Archetype | Reference |
|---|---|
| Simplest widget (load + save) | `src/lib/widgets/Notepad.svelte` |
| Multi-tab widget | `src/lib/widgets/Ambience.svelte` |
| Chart / SVG widget | `src/lib/widgets/TaskStats.svelte` |
| Modal widget | `src/lib/widgets/QuickCaptureModal.svelte` |
| Store-driven widget | `src/lib/widgets/NowNext.svelte` |
| Simplest service | `src/lib/services/notepad.ts` |
| Service with types + constants | `src/lib/services/clickup.ts` |
| Simplest Rust module | `src-tauri/src/notepad.rs` |
| Rust shell-out / PTY | `src-tauri/src/terminal.rs` |
| Complex integration (HTTP + AI + logging) | `src-tauri/src/clickup.rs` |
| OAuth + token persistence | `src-tauri/src/spotify.rs` |

---

## Routing Table

`INDEX.md` is read once per session (Session Start step 3). Never re-read it per task. **If a file is not in the Read column, do not open it.**

| Task | Read | Write To |
|---|---|---|
| **Add a widget** | reference pattern from table above + matching service | new widget file, `src/routes/+page.svelte` (mount it), `INDEX.md` |
| **Modify a widget** | target widget + its service | target widget; `INDEX.md` if description changes |
| **Add a Rust command** | target `*.rs`, `src-tauri/src/lib.rs` | target module, `lib.rs` (`generate_handler!`), matching service, `INDEX.md` |
| **New integration (end-to-end)** | `src-tauri/src/clickup.rs` (reference) + Reference Patterns row | new Rust module, `lib.rs`, new service, new widget, `+page.svelte`, `INDEX.md`, `BUILD_NOTES.md` |
| **Add a task field** | `services/clickup.ts` (Task type), `src-tauri/src/clickup.rs` (struct), `widgets/TodayTasks.svelte` (render) | those three |
| **Cross-widget refresh / signal** | `stores/refresh.ts` + every widget reading the changed store | those files |
| **AI picker (Now/Next)** | `src-tauri/src/clickup.rs::claude_pick_*`, `widgets/NowNext.svelte` | same files |
| **Calendar feature** | `src-tauri/src/calendar.rs`, `services/calendar.ts`, `widgets/Calendar.svelte` | those three |
| **Spotify / music feature** | `src-tauri/src/spotify.rs`, `widgets/Ambience.svelte` (Spotify section, no separate service — embedded in Ambience) | those two |
| **Theme / styling** | `src/app.css`, `stores/theme.ts`, `routes/+layout.svelte`; target widget if scoped | same |
| **Layout / grid** | `src/routes/+page.svelte` | `src/routes/+page.svelte` |
| **Logging an action** | `scripts/log_action.py`, `src-tauri/src/clickup.rs` (caller) | those two |
| **Fix a bug** | match symptom against Bug Map below | affected file(s) |
| **Config / deps** | target config file only | target config file |

---

## Bug Map (Symptom → Likely File)

Match symptom keywords here BEFORE grepping. If no match here AND no match in `INDEX.md` Tags, *then* grep — for a specific identifier, never a concept.

| Symptom | File(s) |
|---|---|
| Task complete / uncheck / delete / update / create | `src-tauri/src/clickup.rs` + `widgets/TodayTasks.svelte` |
| AI picker latency / wrong picks / cache | `src-tauri/src/clickup.rs::claude_pick_*` + `widgets/NowNext.svelte` |
| Calendar event missing / duplicated | `src-tauri/src/calendar.rs::CALENDARS` (calendar name match) |
| Logical day off by 1 at night | `stores/refresh.ts::logicalToday()`, `src-tauri/src/clickup.rs`, `scripts/log_action.py::logical_today()` |
| Tasks not refreshing after action | `stores/refresh.ts::clickupVersion` bump site (must be **after** API ack) |
| 30s poll not detecting changes | `stores/refresh.ts::startTaskPolling`, `todayIds` diff |
| Theme / color / dark mode broken | `src/app.css` + `stores/theme.ts` |
| Tag pills not wrapping / overflow | `widgets/TodayTasks.svelte` (canvas `measureText` + ResizeObserver) |
| `obsidian://` link not opening | `src-tauri/capabilities/default.json` (`opener:allow-open-url` scope) |
| Spotify auth fails / "No auth code in callback" | Spotify Developer Dashboard → App Settings → add `http://127.0.0.1:8888/callback` as redirect URI |
| Spotify "port 8888 unavailable" on retry | Previous auth attempt timed out; wait ~1s — timeout handler sends a fake connection to unblock the listener thread |
| Spotify shows "No active playback" | HTTP 204 = no active Spotify session; open the app on any device first |
| Notepad not saving | `src-tauri/src/notepad.rs` (path: `CARGO_MANIFEST_DIR/../`) |
| Terminal widget hangs / wrong dir | `src-tauri/src/terminal.rs` (`cmd /C`, home dir) |
| Today doc not appending | `clickup-state.json` date check (logical-day mismatch = silent no-op) |
| Start Day button broken / runs wrong skill | `os-config.json::commands.start_day_skill` + `src-tauri/src/clickup.rs::start_day` (cwd is project root so project-local `.claude/skills/<name>/` resolves) |
| Start Day moves nothing on re-click | Gate hit. State at `~/.claude/dashboard-state.json::last_start_day` matches today's logical day. Stats + calendar still refresh; move skipped. Run `python scripts/start_day.py --force` to bypass. |
| `claude -p` shell-out fails on Windows | `src-tauri/src/clickup.rs` — must be `cmd /C claude`, stdin not args |
| Emoji / unicode crash in calendar | `src-tauri/src/calendar.rs` (`PYTHONIOENCODING=utf-8`) |

---

## When to Grep (Last Resort)

Grep ONLY if (a) the term isn't in Symbol Lookup, (b) the symptom isn't in Bug Map or `INDEX.md` Tags, AND (c) you're searching for a specific identifier — not a concept. If you catch yourself grepping to "see how X is done," stop and use Reference Patterns.

---

## Key Gotchas (Non-Obvious — Read Once, Never Re-Derive)

**Tauri / Frontend**
- Use `openUrl` not `open` from `@tauri-apps/plugin-opener`
- Never combine `onMount(load)` AND `$effect(() => load())` — double-fires on mount
- `CARGO_MANIFEST_DIR` resolves to `src-tauri/`. To reach project root, join `../`. Example: `PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../notepad.md")`
- Tailwind 4: `@theme` makes CSS vars into utilities — no `theme.extend`

**Svelte 5**
- Runes only: `$state`, `$derived`, `$effect`, `$props`, `$bindable`. No `$:`, no `writable()` inside components.

**ClickUp API**
- Auth header: `Authorization: pk_xxx` (no `Bearer`)
- Tasks = v2 base; Docs = v3 base — separate consts in `clickup.rs`
- Native doc append: `content_edit_mode: "append"` on edit-page endpoint

**Refresh / State Timing**
- Bump `clickupVersion` **after** API ack — never inside `markCompleted`/`unmarkCompleted` (races server)
- Bump triggers NowNext re-pick — only bump when mutated task is in `$nowNextIds`

**AI Shell-Out (`claude -p`)**
- Windows: `cmd /C claude ...` (CreateProcess can't resolve npm `.cmd` shims)
- Pass prompt via stdin, not args (avoids quoting hell)
- Use `extract_json()` to tolerate stray prose / code fences from model
- Default model: Sonnet — downgrade to Haiku only on latency/cost pressure

**Spotify OAuth**
- Redirect URI in Spotify Developer Dashboard must be `http://127.0.0.1:8888/callback` exactly — Spotify rejects mismatches silently (callback has `error=` param instead of `code=`)
- `cmd /C start "" <url>` is needed on Windows (`start` requires an empty title arg when the URL contains `=` characters)
- Tokens stored at `~/.claude/spotify-tokens.json` — delete this file to force re-auth
- On timeout, the listener thread is unblocked via a self-connection to 127.0.0.1:8888 so the port is freed for the next attempt
- Adding a new scope to `SCOPES_ENC` does NOT invalidate existing tokens — refresh succeeds with old scopes, and any endpoint requiring the new scope returns 403. Delete `~/.claude/spotify-tokens.json` to force re-consent.

**gcalcli (Calendar)**
- `end` date is exclusive — to include target day, pass `end = target + 1 day`
- `PYTHONIOENCODING=utf-8` required on Windows (Python defaults to cp1252, crashes on emoji)
- Personal calendar = `rn.ahuja04@gmail.com` in gcalcli, NOT "Personal (Aryan Ahuja)". `--calendar` is substring match — wrong name silently returns zero events.

**Logical Day**
- "Day" runs 4:00am → 3:59am next day. Compute: `(now - 4h).date()`
- Frontend `logicalToday` MUST use `getFullYear/Month/Date()` — never `.toISOString().slice(0,10)`. `toISOString()` is UTC; at 11pm PDT the 4h shift lands wrong.

---

## Layering Rules (Non-Obvious Only)

- Widgets never import each other. Cross-widget data → Svelte stores only.
- Services never import widgets. Services are thin `invoke()` wrappers.
- Tauri commands return `Result<T, String>` — don't leak internal error types across the boundary.
- Secrets → `.env` (gitignored), loaded in Rust via `dotenvy`. Never call Anthropic API directly — always shell out to `claude -p`.

---

## Self-Maintenance (Keep the Routing System Alive)

This file is only useful if it stays accurate. Update routing data in the **same turn** as the code change — never defer. Stale routing = the regex storms this whole structure exists to prevent.

| Trigger | Update |
|---|---|
| Add a widget | `INDEX.md` (Widgets, with `Tags:`). New archetype → also Reference Patterns. |
| Add a service | `INDEX.md` (Services). New types/constants/wrappers → Symbol Lookup. |
| Add a store | `INDEX.md` (Stores). New exports → Symbol Lookup. |
| Add a Rust module | `INDEX.md` (Rust Modules + Commands Registered). Key items → Symbol Lookup. New feature area → Routing Table row. |
| Add a Rust command | `INDEX.md` (Commands Registered). Add invoke wrapper to matching service same turn. |
| Discover a new bug class | Add row to Bug Map. |
| Discover a new gotcha | Add to Key Gotchas — what surprised you, what was the fix. |
| Rename / delete a file | Find/replace across Symbol Lookup, Reference Patterns, Routing Table, Bug Map, `INDEX.md`. No dangling pointers. |
| New path convention emerges | Add row to Path Conventions. |

**End-of-turn self-check:** Did I create / rename / delete a source file, or add a new symbol? If yes, did I update `INDEX.md` and the relevant CLAUDE.md tables? If not, do it now.

**When in doubt, add to Symbol Lookup.** Better to over-list than send a future Claude grepping. Every `INDEX.md` entry ends with `Tags:` keywords matching likely symptom phrases — that's what makes the index searchable without grep.

---

## Session Start Checklist

1. Read `CLAUDE.md` (this file) ✓
2. Read `hot.md` — pickup point: active work
3. Read `INDEX.md` — one-liner map of every source file
4. Identify task type in Routing Table
5. Read ONLY the listed files → begin work

**Reminders:** Never read `node_modules/`, `src-tauri/target/`, `.svelte-kit/`. Append to `BUILD_NOTES.md` (newest on top), never rewrite. 3+ files touched the same way → write a script to `scripts/`.
