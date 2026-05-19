---
name: start-day
description: AI-driven start-of-day routine. Refreshes completion stats, considers today's calendar, and moves area-list tasks into Daily To-Do — deferring tasks that clearly won't fit the day's available time. Generic logic only; no personal heuristics. Override by setting `commands.start_day_skill` in os-config.json to a personal skill name.
---

# /start-day

The "Start Day" button in the dashboard invokes this skill. It's AI-driven:
look at the user's calendar + incomplete area-list tasks and decide which
tasks should actually move to Daily To-Do today. Tasks that clearly won't
fit the day's available work time get deferred (left in their area list).

This skill does *not* apply personal frameworks (Eisenhower, GTD, energy
pacing, etc.). Its only filter is "does the math fit?" — total task time
vs. available work time after subtracting meetings. Anything more
opinionated belongs in the user's own custom skill (point
`commands.start_day_skill` at it).

## Workflow

### 1. Audit current state

Run from the project root:

```
python scripts/start_day.py --audit
```

The output is a single JSON object with this shape:

```json
{
  "logical_today": "YYYY-MM-DD",
  "stats": { "yesterday_count": 7, "backfilled_days": 0 },
  "calendar": [
    { "start_time": "09:00", "title": "Standup", "all_day": false },
    ...
  ],
  "areas": [
    {
      "slug": "school",
      "label": "School",
      "list_id": "...",
      "tasks": [
        { "id": "abc", "name": "...", "priority": "high",
          "due_date_ms": 1715731200000, "time_estimate_min": 60 }
      ]
    }
  ]
}
```

This call always refreshes stats and pulls calendar; it does not move tasks.

### 2. Make the move decision

Reason about the audit:

- **Available work time today**: a rough 8-hour day minus the time blocked
  by non-all-day calendar events. (Be generous — don't try to over-optimize.)
- **Total task time**: sum `time_estimate_min` across all incomplete area
  tasks. Treat tasks with no estimate as ~30 min.
- If total ≤ available: move everything. No skips needed.
- If total > available: defer the lowest-leverage tasks until the math
  fits. Lower-leverage = lower `priority` and farther-out `due_date_ms`.
  Never defer a task whose `due_date_ms` is within ~2 days.

The decision should be quick — a few seconds of reasoning, not a full
strategy session. If you find yourself agonizing, default to moving
everything.

### 3. Execute the moves

```
python scripts/start_day.py --moves-only [--skip "id1,id2,id3"]
```

Pass `--skip` with the comma-separated IDs of any tasks you decided to
defer. Without `--skip`, every incomplete area task is moved.

The Rust `start_day` command's `clickup-state.json` check is the single
once-per-day gate — when invoked via the dashboard button, repeated clicks
never reach this skill. Running this skill manually will always execute
the moves; deduplicate via the `--skip` flag if needed.

### 4. Report

Two or three short lines for the user:

- How many tasks moved, how many deferred (if any)
- One-line reason for any deferrals (e.g. "deferred 3 lower-priority tasks
  — calendar shows ~5 hours of meetings, total task time was 11+ hours")
- The most notable calendar event of the day if there is one

Keep it brief. The dashboard surfaces the script's stdout to the user
already; you're just providing the framing.

## Notes

- Tag application happens automatically inside the move step (the script
  POSTs `/task/{id}/tag/{slug}` for each moved task). You don't need to
  invoke this separately.
- Action logging (every move appended to the configured log destination)
  happens automatically inside the script via `scripts/log_action.py`.
