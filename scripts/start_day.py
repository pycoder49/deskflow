#!/usr/bin/env python3
"""
start_day.py — bookkeeping for the dashboard's "Start Day" button.

Three job groups:
  1. Stats refresh + backfill — re-records yesterday's completion count and
     fills any missing days in the last 14 by querying ClickUp.
  2. Calendar pull — today's events via `gcalcli` for the personal calendar.
  3. Task moves — incomplete area-list tasks → daily list, tagged + logged.

The Rust `start_day` command's `clickup-state.json` check is the single gate
that prevents repeated runs in a logical day; this script runs unconditionally
whenever invoked.

Four operating modes:
  --bootstrap   Stats refresh only — no calendar, no moves, no JSON output.
                Deterministic safety net the Rust `start_day` command runs
                before invoking the AI skill.
  --audit       Emits a JSON snapshot (stats summary + calendar + per-area
                task inventory) to stdout. Refreshes stats; does NOT move.
                Intended for the /start-day skill to consume.
  --moves-only  Skips stats refresh + calendar display. Runs the move step.
                Pair with --skip "id1,id2" to defer specific tasks.
  (default)     All three groups, with terminal-friendly output.

Reads os-config.json for ClickUp IDs + calendar email, and .env for the token.
"""
import argparse
import json
import os
import shutil
import subprocess
import sys
from datetime import datetime, timedelta
from pathlib import Path

import requests

PROJECT_ROOT = Path(__file__).resolve().parent.parent
ENV_FILE = PROJECT_ROOT / ".env"
CONFIG_FILE = PROJECT_ROOT / "os-config.json"
LOG_ACTION = PROJECT_ROOT / "scripts" / "log_action.py"
STATS_FILE = PROJECT_ROOT / "task-stats.json"
BASE = "https://api.clickup.com/api/v2"


# ─── Env + config ────────────────────────────────────────────────────────────
def load_token() -> str:
    tok = os.environ.get("CLICKUP_TOKEN", "")
    if tok:
        return tok
    if ENV_FILE.exists():
        for line in ENV_FILE.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if line.startswith("CLICKUP_TOKEN="):
                return line.split("=", 1)[1].strip().strip('"').strip("'")
    sys.exit("CLICKUP_TOKEN not found in environment or .env")


def load_config() -> dict:
    if not CONFIG_FILE.exists():
        sys.exit(f"{CONFIG_FILE.name} not found — run `python scripts/setup.py` first")
    return json.loads(CONFIG_FILE.read_text(encoding="utf-8"))


def hdrs(token: str) -> dict:
    return {"Authorization": token, "Content-Type": "application/json"}


def logical_today_date():
    return (datetime.now() - timedelta(hours=4)).date()


# ─── Stats (refresh yesterday + backfill missing) ────────────────────────────
def update_completion_stats(token: str, cfg: dict) -> dict:
    """Re-record yesterday's count and backfill missing days in last 14.
    Doesn't overwrite existing entries beyond yesterday.
    Returns {"yesterday_count": int, "backfilled_days": int}."""
    logical_today = logical_today_date()

    stats = {}
    if STATS_FILE.exists():
        try:
            stats = json.loads(STATS_FILE.read_text(encoding="utf-8"))
        except Exception:
            pass

    target_days = []
    for i in range(1, 15):
        day = logical_today - timedelta(days=i)
        key = day.strftime("%Y-%m-%d")
        if i == 1 or key not in stats:
            target_days.append(day)

    earliest = min(target_days)
    latest = max(target_days)
    range_start = datetime.combine(earliest, datetime.min.time()) + timedelta(hours=4)
    range_end = datetime.combine(latest, datetime.min.time()) + timedelta(hours=4, days=1)
    start_ms = int(range_start.timestamp() * 1000)
    end_ms = int(range_end.timestamp() * 1000)

    list_ids = [cfg["clickup"]["daily_list_id"]]
    for area in cfg["clickup"].get("areas", []):
        if area.get("list_id"):
            list_ids.append(area["list_id"])

    day_counts = {d.strftime("%Y-%m-%d"): 0 for d in target_days}
    for list_id in list_ids:
        if not list_id:
            continue
        r = requests.get(
            f"{BASE}/list/{list_id}/task",
            headers=hdrs(token),
            params={
                "include_closed": "true",
                "subtasks": "false",
                "statuses[]": "complete",
                "date_updated_gt": str(start_ms),
                "date_updated_lt": str(end_ms),
            },
        )
        if not r.ok:
            print(f"  ! stats fetch failed for list {list_id}: {r.status_code}", file=sys.stderr)
            continue
        for t in r.json().get("tasks", []):
            dc = t.get("date_closed")
            if not dc:
                continue
            dc_ms = int(dc)
            if dc_ms < start_ms or dc_ms >= end_ms:
                continue
            dc_dt = datetime.fromtimestamp(dc_ms / 1000)
            logical_dc = (dc_dt - timedelta(hours=4)).date()
            key = logical_dc.strftime("%Y-%m-%d")
            if key in day_counts:
                day_counts[key] += 1

    for key, count in day_counts.items():
        stats[key] = count

    STATS_FILE.parent.mkdir(parents=True, exist_ok=True)
    STATS_FILE.write_text(json.dumps(stats, indent=2, sort_keys=True), encoding="utf-8")

    yesterday_key = (logical_today - timedelta(days=1)).strftime("%Y-%m-%d")
    return {
        "yesterday_count": day_counts.get(yesterday_key, 0),
        "backfilled_days": len(target_days) - 1,
    }


# ─── Calendar context ────────────────────────────────────────────────────────
def fetch_today_calendar(personal_email: str) -> list:
    """Today's events via gcalcli. Empty list on any failure."""
    if not personal_email or not shutil.which("gcalcli"):
        return []

    today = logical_today_date()
    tomorrow = today + timedelta(days=1)
    env = os.environ.copy()
    env["PYTHONIOENCODING"] = "utf-8"

    try:
        out = subprocess.check_output(
            [
                "gcalcli",
                "--calendar", personal_email,
                "agenda",
                today.strftime("%Y-%m-%d"),
                tomorrow.strftime("%Y-%m-%d"),
                "--tsv",
                "--details", "calendar",
            ],
            env=env,
            text=True,
            encoding="utf-8",
        )
    except (subprocess.CalledProcessError, FileNotFoundError):
        return []

    events = []
    for line in out.splitlines():
        parts = line.split("\t")
        if len(parts) < 5:
            continue
        start_date, start_time, end_date, end_time, title = parts[:5]
        all_day = not start_time or start_time == "00:00"
        events.append({
            "start_date": start_date,
            "start_time": start_time,
            "end_date": end_date,
            "end_time": end_time,
            "title": title.strip(),
            "all_day": all_day,
        })
    return events


def print_calendar(events: list) -> None:
    if not events:
        print("\nNo calendar events for today.")
        return
    print("\nToday's calendar:")
    for e in events:
        if e["all_day"]:
            print(f"  (all-day)  {e['title']}")
        else:
            print(f"  {e['start_time']}  {e['title']}")


# ─── Task inventory + moves ──────────────────────────────────────────────────
def fetch_incomplete(token: str, list_id: str) -> list:
    r = requests.get(
        f"{BASE}/list/{list_id}/task",
        headers=hdrs(token),
        params={"include_closed": "false", "subtasks": "false"},
    )
    if not r.ok:
        print(f"  ! fetch failed for list {list_id}: {r.status_code} {r.text[:200]}", file=sys.stderr)
        return []
    tasks = r.json().get("tasks", [])
    return [
        t for t in tasks
        if (t.get("status", {}).get("type") or "").lower() not in ("done", "closed")
    ]


def task_summary(t: dict) -> dict:
    """Slim task representation for the audit JSON. Drops fields the skill
    doesn't need to reason about (custom_fields, watchers, etc.)."""
    te = t.get("time_estimate")
    pri = t.get("priority") or {}
    return {
        "id": t["id"],
        "name": t.get("name", ""),
        "priority": pri.get("priority") if isinstance(pri, dict) else None,
        "due_date_ms": int(t["due_date"]) if t.get("due_date") else None,
        "time_estimate_min": (int(te) // 60000) if te else None,
    }


def move_task(token: str, task_id: str, daily_list_id: str) -> bool:
    r = requests.put(
        f"{BASE}/task/{task_id}",
        headers=hdrs(token),
        json={"list": daily_list_id},
    )
    if r.ok:
        return True
    print(f"  ! move failed for {task_id}: {r.status_code} {r.text[:200]}", file=sys.stderr)
    return False


def tag_task(token: str, task_id: str, slug: str) -> None:
    if not slug:
        return
    r = requests.post(
        f"{BASE}/task/{task_id}/tag/{slug}",
        headers=hdrs(token),
    )
    if not r.ok:
        print(f"  ! tag {slug!r} failed for {task_id}: {r.status_code}", file=sys.stderr)


def log_move(task_name: str, area_slug: str) -> None:
    subprocess.run(
        [
            sys.executable, str(LOG_ACTION),
            "--action", "move",
            "--task-name", task_name,
            "--tags", area_slug,
            "--details", "→ Daily To-Do",
        ],
        check=False,
    )


def do_moves(token: str, cfg: dict, skip_ids: set) -> tuple:
    """Move incomplete area tasks → daily, except those in skip_ids.
    Returns (moved_count, skipped_count, touched_areas_count)."""
    daily_list_id = cfg["clickup"]["daily_list_id"]
    areas = cfg["clickup"].get("areas", [])

    moved = 0
    skipped = 0
    touched_areas = 0
    for area in areas:
        tasks = fetch_incomplete(token, area["list_id"])
        if not tasks:
            continue
        touched_areas += 1
        slug = area.get("slug", "")
        for task in tasks:
            if task["id"] in skip_ids:
                skipped += 1
                continue
            if move_task(token, task["id"], daily_list_id):
                existing = {t.get("name", "").lower() for t in task.get("tags", [])}
                if slug and slug.lower() not in existing:
                    tag_task(token, task["id"], slug)
                moved += 1

    return moved, skipped, touched_areas


# ─── Modes ───────────────────────────────────────────────────────────────────
def do_bootstrap(token: str, cfg: dict) -> int:
    """Deterministic safety net for the dashboard's Start Day button.
    Refreshes task-stats.json (the chart's source of truth) via direct ClickUp
    HTTP. No calendar, no moves, no JSON output — just stats. Runs before the
    AI skill so the chart stays correct even if the skill flakes."""
    s = update_completion_stats(token, cfg)
    print(f"[bootstrap] yesterday={s['yesterday_count']} backfilled={s['backfilled_days']}")
    return 0


def do_audit(token: str, cfg: dict) -> None:
    """Refresh stats + pull calendar + build task inventory; emit JSON."""
    stats_result = update_completion_stats(token, cfg)
    events = fetch_today_calendar(cfg.get("calendar", {}).get("personal_email", ""))

    areas_inventory = []
    for area in cfg["clickup"].get("areas", []):
        tasks = fetch_incomplete(token, area["list_id"])
        areas_inventory.append({
            "slug": area.get("slug", ""),
            "label": area.get("label", ""),
            "list_id": area["list_id"],
            "tasks": [task_summary(t) for t in tasks],
        })

    audit = {
        "logical_today": logical_today_date().strftime("%Y-%m-%d"),
        "stats": stats_result,
        "calendar": events,
        "areas": areas_inventory,
    }
    print(json.dumps(audit, indent=2))


def do_default(token: str, cfg: dict, skip_ids: set, moves_only: bool) -> int:
    if not moves_only:
        print("Updating completion stats…")
        s = update_completion_stats(token, cfg)
        msg = f"  Recorded {s['yesterday_count']} completions for yesterday"
        if s["backfilled_days"] > 0:
            msg += f" (+ backfilled {s['backfilled_days']} earlier missing day{'s' if s['backfilled_days'] != 1 else ''})"
        print(msg + ".")

        print_calendar(fetch_today_calendar(cfg.get("calendar", {}).get("personal_email", "")))

    moved, skipped, areas = do_moves(token, cfg, skip_ids)
    line = f"\nMoved {moved} tasks from {areas} area lists to Daily To-Do"
    if skipped:
        line += f" ({skipped} deferred)"
    print(line + ".")
    return 0


# ─── Main ────────────────────────────────────────────────────────────────────
def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--bootstrap", action="store_true",
                        help="Refresh task-stats.json only — no calendar, no moves, no JSON. "
                             "Deterministic safety net for the dashboard's Start Day button.")
    parser.add_argument("--audit", action="store_true",
                        help="Emit JSON snapshot (stats, calendar, task inventory) and exit. No moves.")
    parser.add_argument("--moves-only", action="store_true",
                        help="Skip stats refresh + calendar; just do the move step.")
    parser.add_argument("--skip", default="",
                        help="Comma-separated task IDs to NOT move (defer to their area list).")
    args = parser.parse_args()

    token = load_token()
    cfg = load_config()

    if not cfg["clickup"].get("daily_list_id"):
        sys.exit("clickup.daily_list_id is empty in os-config.json")

    if args.bootstrap:
        return do_bootstrap(token, cfg)

    if args.audit:
        do_audit(token, cfg)
        return 0

    skip_ids = {s.strip() for s in args.skip.split(",") if s.strip()}
    return do_default(token, cfg, skip_ids, args.moves_only)


if __name__ == "__main__":
    sys.exit(main())
