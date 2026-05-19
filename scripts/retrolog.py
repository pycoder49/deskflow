#!/usr/bin/env python3
"""
retrolog.py — append a structured retro entry to the monthly ClickUp log doc.

Usage:
  python scripts/retrolog.py \
    --date 2026-05-18 \
    --focus "School prep + project work" \
    --tasks "Task 1, Task 2, Task 3" \
    --projects "Built X feature; refactored Y" \
    --notes "Some note about the day"

Appends under the date header:
  **Focus:** School prep + project work
  **ClickUp:** 3 completed — Task 1, Task 2, Task 3
  **Projects:** Built X feature; refactored Y
  **Notes:** Some note about the day

--projects and --notes are optional. --focus defaults to "No entry" if omitted.
"""
import argparse
import sys
from pathlib import Path
from datetime import datetime

sys.path.insert(0, str(Path(__file__).parent))
from log_action import (
    load_token,
    load_config,
    date_header,
    compute_delta,
    cu_resolve_month_doc,
    cu_get_page_content,
    cu_append_page,
    load_state,
    save_state,
    month_label,
)


def build_entry(focus: str, tasks: list[str], projects: str, notes: str) -> str:
    count = len(tasks)
    task_str = ", ".join(tasks) if tasks else "none"
    lines = [
        f"**Focus:** {focus}",
        f"**ClickUp:** {count} completed — {task_str}",
    ]
    if projects:
        lines.append(f"**Projects:** {projects}")
    if notes:
        lines.append(f"**Notes:** {notes}")
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description="Append a retro entry to the monthly ClickUp log doc.")
    parser.add_argument("--date", required=True, help="Date of the retro entry (YYYY-MM-DD)")
    parser.add_argument("--focus", default="No entry", help="What the day was focused on")
    parser.add_argument("--tasks", default="", help="Comma-separated completed task names")
    parser.add_argument("--projects", default="", help="Notable project work (optional)")
    parser.add_argument("--notes", default="", help="Any notes worth keeping (optional)")
    args = parser.parse_args()

    try:
        target_date = datetime.strptime(args.date, "%Y-%m-%d").date()
    except ValueError:
        sys.exit(f"Invalid --date: {args.date!r} — expected YYYY-MM-DD")

    task_names = [t.strip() for t in args.tasks.split(",") if t.strip()]
    entry = build_entry(
        focus=args.focus.strip() or "No entry",
        tasks=task_names,
        projects=args.projects.strip(),
        notes=args.notes.strip(),
    )

    cfg = load_config()
    mode = cfg.get("logging", {}).get("mode", "none")
    if mode == "none":
        print("[retrolog] logging mode is 'none' — skipping")
        return
    if mode != "clickup_doc":
        sys.exit(f"retrolog.py only supports clickup_doc mode (got {mode!r})")

    token = load_token()
    workspace_id = cfg["clickup"]["workspace_id"]
    logs_folder_id = cfg["logging"]["clickup_logs_folder_id"]
    month = month_label(target_date)

    state = load_state()
    if state and state.get("month") == month:
        doc_id, page_id = state["doc_id"], state["page_id"]
    else:
        doc_id, page_id = cu_resolve_month_doc(token, workspace_id, logs_folder_id, month)
        save_state(month, doc_id, page_id)

    header = date_header(target_date)
    content = cu_get_page_content(token, workspace_id, doc_id, page_id)
    delta = compute_delta(content, header, entry).rstrip("\n")
    cu_append_page(token, workspace_id, doc_id, page_id, delta)
    print(f"[retrolog] wrote retro for {args.date} ({len(task_names)} completed)")


if __name__ == "__main__":
    main()
