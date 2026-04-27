# IDEAS.md — Aryan OS Dashboard

> Living scratchpad for design direction, layout experiments, and future widgets.
> Not a spec — these are positions to argue with, not decisions.

---

## ~~Do first — typography scale~~ ✓ Done

Cheapest, highest-payoff change. The dashboard currently has every widget at the same text size, which is why nothing visually leads. Fix:

- **Now/Next current task title:** bump to `text-2xl` or `text-3xl`, semibold. The eye should land there involuntarily.
- **Now/Next "Next" items:** standard size, but more line-height so they read as secondary.
- **Today list items:** keep current size — it's a scan list, density is the point.
- **Widget headers (`text-xs uppercase tracking-wider`):** keep — they're already correctly muted.
- **Calendar event titles:** one notch up from current; they're the actionable content of that widget.
- **Vault / Projects:** keep small — reference tier, shouldn't compete.

Principle: type size = importance. Pick the one thing in each widget that matters most and let it dominate.

---

## Critique of current layout (2026-04-26)

**Working well**
- Today + Now/Next side-by-side = truth + focus, one glance
- Now/Next AI pick is the actual differentiator vs every other "dashboard"
- Quick capture (Ctrl+N) is the highest-frequency interaction and it's frictionless
- Vault graph + Hot tab is rare and genuinely useful for a markdown-native workflow
- Logical-day correctness — most personal tools silently break at midnight

**Weak**
- AmbientPanel takes col-span-5 of the bottom row to render a clock + date that's already in the header. Prime real estate, decorative content.
- Two "Coming soon" cards (Workouts, Claude Terminal) advertise emptiness. Better to not render until shipped.
- Notepad in row 1 is a priority mismatch — passive scratchpad sharing space with the two most attention-demanding widgets.
- Now/Next doesn't know about Calendar. AI can pick a 90-min task at 1:50pm with a 2pm meeting on the books.
- No "next event" glance anywhere above the fold — have to scan the calendar grid.
- Every widget is the same card. Nothing tells the eye that Now/Next is *the* focus widget.

---

## Proposed rearrangement

```
HEADER:  Aryan OS · Mon Apr 26 · 14:32 · ⏭ "Standup" in 28m  | + New Task | Theme
ROW 1:   [ Now/Next  (5) ]  [ Today  (4) ]  [ Calendar peek — next 4 events (3) ]
ROW 2:   [ Calendar full  (8) ]  [ Vault  (4) ]
ROW 3:   [ Projects  (12) ]
ROW 4:   [ Workouts heatmap  (12) ]   ← when shipped, becomes the visual anchor
DRAWER:  Notepad → slide-out on hotkey (Ctrl+J), not a permanent grid item
KILLED:  AmbientPanel as a standalone tile (move clock → header)
         Placeholder cards (don't render until shipped)
```

**Principle:** information density ramps *down* the page — focus → reference → workspace → ambient/visual. Above the fold should be 100% actionable. The visual breathing room comes from real visualizations (heatmap, graph), not empty cards.

---

## Future widget notes

### Workouts (planned)
Natural-language input → parser → SQLite via `rusqlite`. Detects day type (push/pull/legs), exercises, sets, reps. GitHub-tile heatmap for streak/consistency.

- Doubles as the dashboard's visual anchor — solves the "needs visuals, not just text" problem.
- Optional `claude -p` cleanup pass for ambiguous input ("did 4x10 bench, then some incline").
- Heatmap belongs at full width — it's the kind of thing you glance at, not click.
- Consider: per-day tooltip showing the parsed session.

### Claude Terminal (questioned)
Embedded terminal (xterm.js + portable_pty) bottom-left.

**Honest assessment:** marginal value as a generic terminal. You're always one Alt-Tab from Windows Terminal, and that terminal is more capable. Only worth building if it's *context-aware*:
- Preloaded with `cd` to a Projects-widget project
- Auto-runs `claude -p` with current widget context as system prompt
- Wired to "open this task in Claude" buttons elsewhere on the dashboard

If it's just `cmd.exe` in a div, skip it. If it's "Claude with my dashboard state already loaded", build it.

### Notepad (re-home)
Move out of the row-1 grid into a slide-out drawer triggered by hotkey (Ctrl+J or similar). Auto-save behavior stays. Removes the priority mismatch and frees a row-1 slot for something more action-oriented (Calendar peek, AI summary, etc).

### Ambient slot (re-purpose)
Instead of a clock-in-a-box, this slot becomes a real visual:
- Option A: Workouts heatmap (when shipped) — best fit
- Option B: 14-day task completion sparkline + completion rate
- Option C: Vault graph rendered larger as an ambient hero
- Option D: Bigger SpaceBackground reveal — actual ambient theming
- Option E: Weather + sunrise/sunset (genuine glanceable info)

Pick one. The job is "give the eye a place to rest that's not text", not "fill space".

---

## Logical gaps to close

1. ~~**Now/Next ↔ Calendar awareness.**~~ ✓ Done — `calendar::fetch_events` called on every cache miss; current time + upcoming timed events injected into both pick prompts.
2. ~~**Header next-event glance.**~~ ✓ Done — date + live clock; calendar glance skipped for now.
3. **Visual hierarchy.** Now/Next deserves an accent border, slightly different surface, or a subtle glow — currently it visually equals Notepad.

---

## Open questions

- Should the dashboard have a "morning mode" vs "working mode" view? Different widgets matter at different times of day.
- Is Vault really earning col-span-5? Day-to-day usage probably doesn't justify it. Might collapse to a rail.
- Projects widget: accordion is right, but with 5+ projects expanded it eats vertical space fast. Consider max-2-expanded-at-once.
- Heatmap color scheme should respect the theme (light/dark/space) — three palettes needed.
