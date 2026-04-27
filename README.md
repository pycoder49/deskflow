# Aryan OS Dashboard

A personal desktop dashboard built with Tauri 2 + SvelteKit. Aggregates daily workflow — tasks, calendar, vault knowledge, and an AI focus-picker — into a single native window.

![Dashboard screenshot](docs/screenshot.png)

---

## Features

### Now / Next (AI Focus Picker)
Uses `claude -p` to pick the single most important task to work on right now, plus up to two queued next tasks. Automatically re-picks when tasks are completed or the pool changes. Calendar-aware: fetches today's upcoming events via gcalcli and avoids suggesting tasks whose estimates would run past the next meeting.

### Today Tasks
Live view of the Daily To-Do list from ClickUp. Check off tasks directly from the dashboard — completions sync back to ClickUp and trigger a Now/Next re-pick.

### Calendar
Four views (list, day, week, month) pulling from Google Calendar via gcalcli. Color-coded by calendar. Logical-day aware — days roll over at 4am, not midnight.

### Vault
Three-tab view into a local Obsidian vault: active notes (hot.md), unprocessed inbox files, and a live wikilink graph rendered with d3-force. File clicks open directly in Obsidian.

### Notepad
Quick scratchpad that auto-saves to a local `notepad.md` file. Accessible from the dashboard without opening a separate editor.

### Projects
Per-project context viewer. Add any local project by path; the dashboard reads and displays its `context.md`. Useful for keeping project notes visible without switching windows.

### Quick Capture (Ctrl+N)
Modal for adding ClickUp tasks without leaving the dashboard. Supports list selection, priority, and tags. Logs every add to a Today doc in ClickUp.

---

## Stack

| Layer | Tech |
|-------|------|
| Desktop shell | Tauri 2 (Rust) |
| Frontend | SvelteKit 2 (SPA, adapter-static) |
| Language | TypeScript + Rust |
| Styling | Tailwind CSS 4 + `@theme` |
| Tasks | ClickUp HTTP API (v2 + v3) |
| Calendar | gcalcli (shell-out) |
| AI | `claude -p` subprocess (uses your Claude subscription) |
| Vault | Local Obsidian markdown files |

---

## Prerequisites

- [Tauri prerequisites](https://tauri.app/start/prerequisites/) (Rust, WebView2 on Windows)
- [Node.js](https://nodejs.org/) 18+
- [Claude Code CLI](https://claude.ai/code) — logged in (`claude` on PATH)
- [gcalcli](https://github.com/insanum/gcalcli) — authenticated with your Google account
- A [ClickUp](https://clickup.com) account + API token

---

## Setup

1. **Clone the repo**
   ```bash
   git clone https://github.com/pycoder49/aryan-os-dashboard.git
   cd aryan-os-dashboard
   ```

2. **Install JS dependencies**
   ```bash
   npm install
   ```

3. **Configure environment**
   ```bash
   cp .env.example .env
   ```
   Fill in `.env`:
   ```
   CLICKUP_TOKEN=pk_your_token_here
   VAULT_PATH=C:/path/to/your/obsidian/vault
   ```

4. **Run in development**
   ```bash
   npm run tauri dev
   ```

5. **Build for production**
   ```bash
   npm run tauri build
   ```

---

## Configuration

### ClickUp
Set `CLICKUP_TOKEN` in `.env`. The dashboard targets a hardcoded Daily To-Do list ID — update `DAILY_LIST` in `src-tauri/src/clickup.rs` to match your own list.

### Calendar
gcalcli must be authenticated and working (`gcalcli agenda` should return events). Update the `CALENDARS` constant in `src-tauri/src/calendar.rs` with your calendar display names.

### Vault
Set `VAULT_PATH` in `.env` to your Obsidian vault root. The vault widget expects:
- `wiki/hot.md` — active notes
- `wiki/log.md` — daily log
- `raw/notes/` and `raw/clips/` — inbox files

These paths are configurable in `src-tauri/src/vault.rs`.

---

## Themes

Three built-in themes toggled via the button in the header: **Light**, **Dark**, and **Space** (dark with a starfield background). Theme persists across restarts via localStorage.

---

## Project Structure

```
src/
  routes/        # SvelteKit pages
  lib/
    widgets/     # One .svelte file per widget
    services/    # TypeScript invoke() wrappers
    stores/      # Svelte stores (theme, refresh signals)
src-tauri/
  src/           # Rust backend modules
```

See `INDEX.md` for a one-liner description of every source file.

---

## License

MIT
