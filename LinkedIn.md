# LinkedIn Post — DeskFlow

---

I built a personal desktop dashboard that kills task paralysis — here's what I learned.

Most productivity apps solve a different problem than the one I actually have.

The problem isn't organizing tasks. It's sitting down, staring at 40 of them, and not knowing which one to start.

So I built **DeskFlow** — a native desktop dashboard (Tauri 2 + SvelteKit) that pulls my entire daily workflow into one window and tells me exactly what to work on.

---

**The brain: Now / Next**

The centerpiece is a Claude-powered focus picker. It reads my ClickUp tasks, checks my Google Calendar for today's schedule, and surfaces exactly one task to work on *right now* — plus two queued up next. It re-picks automatically as I complete things.

No more decision paralysis. No more "let me just check Slack real quick" spirals.

Everything else lives in the same window: live task list, calendar (day/week/month), ambient sounds + Spotify, an embedded terminal, and an AI-driven morning routine that moves tasks from my backlog into my daily list based on what actually fits the calendar.

---

**Design decisions I'm proud of:**

**1. Tauri over Electron.**
Rust backend + WebView2 shell is about 10x smaller and faster to start. For a dashboard you open every morning, startup time is a real UX decision.

**2. Shell out to `claude -p`, don't call the Anthropic API directly.**
The AI picker runs through Claude Code instead of hitting an API endpoint. My subscription covers it — no API key, no per-call billing. The dashboard stays free to run.

**3. 4am logical day, not midnight.**
I work late. A "day" in DeskFlow runs 4am → 3:59am the next calendar day, so late-night work counts toward the day I actually started. Surprisingly few productivity tools respect actual human sleep schedules.

**4. Config-driven personalization over hardcoded structure.**
A setup wizard discovers your ClickUp workspace, picks your calendar, and writes a local config file. Daily list, area lists, logging destination, AI skill — all configurable. The goal: someone else can clone the repo and be running in under 10 minutes without touching source code.

**5. Shell out to gcalcli for calendar.**
gcalcli is a battle-tested CLI that handles OAuth and parsing. Three lines of Rust instead of a 200-line Google Calendar SDK integration. Boring is good.

---

**What I want to build next:**

The dashboard already collects a lot of behavioral signal — what you complete, when, what you defer. The natural next step is actually learning from it:

- **Task completion time modeling** — track how long tasks actually take vs. estimates, build per-category time models, use them for realistic daily capacity warnings before you overcommit
- **Behavioral pattern learning** — learn when you're most likely to complete certain types of tasks, and let the AI picker factor energy and time-of-day into its picks
- **Smart deferrals** — instead of moving every backlog task into the daily list, predict which ones you'll actually touch based on history
- **Focus session tracking** — automatic Pomodoro-style sessions tied to the Now task, with break suggestions based on actual session length
- **Weekly AI retrospective** — end-of-week summary: what got done, what kept getting deferred and why, suggested focus for next week
- **Natural language task capture** — "remind me to do X after my 3pm" parsed and scheduled without touching ClickUp
- **Cross-device sync** — right now it's a desktop-only app; a lightweight mobile companion for capturing tasks on the go would close the loop

---

It's open source — built to be personalized, not just cloned and forgotten.

→ github.com/pycoder49/deskflow

#buildinpublic #tauri #rust #svelte #productivity #ai #opensource
