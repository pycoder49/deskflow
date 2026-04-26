# CLAUDE.md — Aryan OS Dashboard

> Read this file FIRST every session. It is your routing table.
> Do not read files outside the task's Read column.

---

## Identity

Aryan's personal dashboard — Tauri 2 desktop app aggregating daily workflow (ClickUp tasks, Google Calendar, vault knowledge, AI focus-picking).

**Owner:** Aryan — CS student, UCSD 2026, Windows.  
**Related vault:** `C:/Users/aryan/Desktop/AryanOS/` — has its own CLAUDE.md. Cross-load only when the task explicitly touches vault files.

---

## Stack

| Layer | Tech | Notes |
|-------|------|-------|
| Desktop shell | Tauri 2 (Rust) | Native Windows; filesystem, subprocess spawning, HTTP |
| Frontend | SvelteKit 2 (SPA, adapter-static) | No SSR — Tauri is not a Node server |
| Language | TypeScript + Rust | TS frontend, Rust backend |
| Styling | Tailwind CSS 4 + `@theme` | CSS variables for light/dark/space themes; no `theme.extend` |
| Integrations | ClickUp HTTP API, gcalcli (shell-out), vault markdown | Direct HTTP (no SDK); calendar via shell |
| AI calls | `claude -p` subprocess | Uses Aryan's subscription — no API key needed |

---

## Folder Map

| Path | Purpose | Writable? |
|------|---------|-----------|
| `src/routes/` | SvelteKit pages + layout shell | YES |
| `src/lib/widgets/` | One `.svelte` per widget (self-contained islands) | YES |
| `src/lib/services/` | TS `invoke()` wrappers around Rust commands | YES |
| `src/lib/stores/` | Svelte stores (theme, cross-widget signals) | YES |
| `src/lib/types.ts` | Shared TypeScript types | YES |
| `src/app.css` | Tailwind entry + CSS variables | YES |
| `src-tauri/src/` | Rust backend modules + `lib.rs` (command registration) | YES |
| `src-tauri/tauri.conf.json` | Window / bundle config | YES (on request) |
| `src-tauri/capabilities/` | Tauri permission scopes | YES (on request) |
| `scripts/` | Reusable bash/python scripts for bulk ops | YES |
| `CLAUDE.md` | This file — routing rules | YES (on request) |
| `INDEX.md` | One-liner file index — update on every structural change | YES |
| `hot.md` | Session handoff: active work, what's next | YES (end of session) |
| `BUILD_NOTES.md` | Build log — append only, newest on top | YES (append only) |
| `node_modules/`, `src-tauri/target/`, `.svelte-kit/` | Build artifacts | NEVER READ |

---

## Routing Table

Read this before every task. Load ONLY files in the Read column.

`INDEX.md` is already read in step 3 of the Session Start Checklist — do not re-read it per task.

| Task | Read (beyond session baseline) | Write To |
|------|-------------------------------|----------|
| **Add a widget** | 1 sibling widget (pattern), matching service | new widget file, `+page.svelte` (mount it), `INDEX.md` |
| **Modify a widget** | target widget + its service only | target widget; `INDEX.md` if description changes |
| **Add a Rust command** | target `*.rs` module, `lib.rs` | target module, `lib.rs`, matching TS service, `INDEX.md` |
| **New integration (end-to-end)** | `clickup.rs` (reference pattern) | new Rust module, `lib.rs`, new service, new widget, `INDEX.md`, `BUILD_NOTES.md` |
| **Fix a bug** | only modules matching the symptom (use INDEX.md already read) | affected file(s) |
| **Theme / styling** | `src/app.css`, `stores/theme.ts`, `+layout.svelte`; target widget if scoped | same files |
| **Layout / grid** | `+page.svelte` | `+page.svelte` |
| **Refresh / state logic** | `stores/refresh.ts`; `TodayTasks.svelte` + `NowNext.svelte` if UI side involved | `stores/refresh.ts`, affected widgets |
| **AI picker (Now/Next)** | `clickup.rs`, `NowNext.svelte` | same files |
| **Config / deps** | target config file only | target config file |

**Rule: If a file is not in the Read column, do not open it.**

---

## Key Gotchas (Non-Obvious — Read Once, Never Re-Derive)

These are baked in here so you don't need to read hot.md or source files to remember them.

**Tauri / Frontend**
- Use `openUrl` not `open` from `@tauri-apps/plugin-opener`
- Never have both `onMount(load)` AND a `$effect(() => load())` — double-fires on mount
- `.env` lives at project root; Rust loads it via `dotenvy` using `CARGO_MANIFEST_DIR`
- `CARGO_MANIFEST_DIR` resolves to `src-tauri/` — to reach the project root (e.g. `notepad.md`, `.env`), join `../`. Example: `PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../notepad.md")`
- Tailwind 4: `@theme` directive makes CSS variables into utilities — no `theme.extend` in config
- `obsidian://` requires explicit `opener:allow-open-url` scope in `capabilities/default.json` — `opener:default` doesn't cover custom URL schemes
- Never write `<style>` literally inside Svelte `<script>` comments — parser mistakes it for unclosed script tag; write "style block" instead

**Svelte 5**
- Runes only: `$state`, `$derived`, `$effect`, `$props`, `$bindable` — no `$:`, no `writable()`

**ClickUp API**
- Auth header: `Authorization: pk_xxx` — no `Bearer` prefix
- Tasks = v2 API base URL; Docs = v3 API base URL — separate consts in `clickup.rs`
- Native doc append: `content_edit_mode: "append"` on the edit-page endpoint
- `clickup-state.json` date mismatch = silent no-op (logical day check fails, doc append skipped)

**Refresh / State Timing**
- Bump `clickupVersion` **after** the API call succeeds — never inside `markCompleted`/`unmarkCompleted` (races server)
- Bumping triggers NowNext's `claude -p` re-pick — only bump when the mutated task was in NowNext's display (`$nowNextIds`)

**AI Shell-Out (`claude -p`)**
- Windows: wrap in `cmd /C claude ...` — `CreateProcess` can't resolve npm `.cmd` shims
- Pass prompt via stdin, not args (avoids quoting hell on Windows)
- Use `extract_json()` in `clickup.rs` to tolerate stray prose or code fences in model output
- Default model: Sonnet — downgrade to Haiku only if latency or cost becomes a concern

**gcalcli (Calendar)**
- `end` date is exclusive — to include target day, pass `end = target + 1 day`
- Requires `PYTHONIOENCODING=utf-8` on Windows — Python defaults to cp1252, crashes on emoji
- Personal calendar name in gcalcli is `rn.ahuja04@gmail.com` — NOT "Personal (Aryan Ahuja)". `CALENDARS` const in `calendar.rs` must use the email form or events are silently excluded
- `--calendar` filter does substring match; unmatched names return zero events (not an error)

**Logical Day**
- A "day" runs 4:00am → 3:59am next calendar day. Compute: `(now - 4h).date()`
- Use for Today-doc names, date comparisons in `clickup.rs`, vault log headers
- **Frontend `logicalToday` must use `getFullYear/Month/Date()` — never `.toISOString().slice(0,10)`.** `toISOString()` is UTC; at 11pm PDT (UTC-7) it already reads the next calendar day, so the 4h shift still lands on the wrong date

---

## Conventions

### Widgets (`src/lib/widgets/`)
- PascalCase file name; one widget per file; self-contained (imports its own service, owns its own state).
- Cross-widget data: Svelte stores only. Widgets never import each other.

### Services (`src/lib/services/`)
- Lowercase file name; thin TS wrappers around `invoke()`. Types defined here or in `types.ts`.
- Services never import widgets.

### Rust Backend (`src-tauri/src/`)
- One module per integration; commands registered in `lib.rs` under `tauri::generate_handler![...]`.
- Naming: Rust = `snake_case`, TS = `camelCase` (`get_today_tasks` ↔ `getTodayTasks`).
- Commands return `Result<T, String>` — don't leak internal error types across the boundary.

### State
- Ephemeral UI state → Svelte stores.
- Secrets → `.env` (gitignored), loaded in Rust via `dotenvy`.

---

## Session Start Checklist

1. Read `CLAUDE.md` (this file) ✓
2. Read `hot.md` — pickup point: active work, what's next
3. Read `INDEX.md` — one-liner map of every source file
4. Identify task type in Routing Table above
5. Read ONLY the listed files → begin work

---

## Token Efficiency Rules

1. **Routing table is law.** Not in Read column = don't open it.
2. **Index before drilling.** Read `INDEX.md` first; pick 1–3 files; open only those.
3. **No speculative reads.** "Might need this to be safe" = you don't.
4. **Update `INDEX.md` on every structural change.** Add/delete/rename = update same turn.
5. **Widgets are islands.** Don't cross-read when modifying one widget.
6. **Scripts beat loops.** 3+ files touched the same way → write a script to `scripts/`.

---

## What You Never Do

- Load all files at session start
- Read `node_modules/`, `src-tauri/target/`, `.svelte-kit/` — build artifacts
- Read unrelated widgets when modifying one widget
- Skip updating `INDEX.md` after structural changes
- Rewrite `BUILD_NOTES.md` — append new entries only, newest on top
- Commit secrets — `.env` is gitignored for a reason
- Call Anthropic API directly — always shell out to `claude -p`
- Mix responsibilities: widgets don't call APIs, services don't render UI, Rust doesn't render UI
