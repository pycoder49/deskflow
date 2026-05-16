#!/usr/bin/env python3
"""
log_action.py — append a task action entry to the configured log destination.

Reads `logging.mode` from os-config.json and dispatches:
  - "clickup_doc"  → append to the monthly doc in ClickUp Logs folder
  - "local_file"   → append to a local Markdown / text file
  - "none"         → silently succeed (logging disabled)

Usage:
  python scripts/log_action.py --action complete --task-name "Task Name"
  python scripts/log_action.py --action update  --task-name "Task" --details "priority: normal → high" --tags "school,cse101"
  python scripts/log_action.py --ensure        # touch this month's destination (creates doc / file as needed)
"""
import argparse
import json
import os
import sys
from datetime import datetime, timedelta
from pathlib import Path
from typing import Optional

import requests

# ─── Constants ───────────────────────────────────────────────────────────────
DOCS_BASE = "https://api.clickup.com/api/v3"

PROJECT_ROOT = Path(__file__).resolve().parent.parent
STATE_FILE = Path.home() / ".claude" / "clickup-log-state.json"
ENV_FILE = PROJECT_ROOT / ".env"
CONFIG_FILE = PROJECT_ROOT / "os-config.json"


# ─── Auth (only needed for clickup_doc mode) ─────────────────────────────────
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


def hdrs(token: str) -> dict:
    return {"Authorization": token, "Content-Type": "application/json"}


# ─── Config ──────────────────────────────────────────────────────────────────
def load_config() -> dict:
    if not CONFIG_FILE.exists():
        sys.exit(f"{CONFIG_FILE.name} not found — run `python scripts/setup.py` first")
    return json.loads(CONFIG_FILE.read_text(encoding="utf-8"))


# ─── Shared formatting ──────────────────────────────────────────────────────
def logical_today():
    """Logical day rolls over at 4am — see ~/.claude/CLAUDE.md → Logical Day."""
    return (datetime.now() - timedelta(hours=4)).date()


def month_label(d) -> str:
    return d.strftime("%B %Y")


def date_header(d) -> str:
    day = d.day
    suffix = "th" if 11 <= day <= 13 else {1: "st", 2: "nd", 3: "rd"}.get(day % 10, "th")
    return f"## {d.strftime('%m-%d-%Y')} ({d.strftime('%B')} {day}{suffix})"


def format_entry(action: str, task_name: str, tags: list, details: Optional[str]) -> str:
    detail_str = f" ({details})" if details else ""
    tag_str = f" [{', '.join(tags)}]" if tags else ""
    return f"{action} -- {task_name}{detail_str}{tag_str}"


def compute_delta(content: str, header: str, entry: str) -> str:
    """Returns only the text to append (never rewrites full content)."""
    if any(line.strip() == header for line in content.splitlines()):
        return entry + "\n"
    prefix = "\n\n" if content.strip() else ""
    return f"{prefix}{header}\n{entry}\n"


# ─── Backend: ClickUp doc ─────────────────────────────────────────────────────
def load_state() -> Optional[dict]:
    if STATE_FILE.exists():
        try:
            return json.loads(STATE_FILE.read_text(encoding="utf-8"))
        except Exception:
            pass
    return None


def save_state(month: str, doc_id: str, page_id: str):
    STATE_FILE.parent.mkdir(parents=True, exist_ok=True)
    STATE_FILE.write_text(
        json.dumps({"month": month, "doc_id": doc_id, "page_id": page_id}, indent=2),
        encoding="utf-8",
    )


def cu_list_folder_docs(token: str, workspace_id: str, logs_folder_id: str) -> list:
    resp = requests.get(
        f"{DOCS_BASE}/workspaces/{workspace_id}/docs",
        headers=hdrs(token),
        params={"parent_id": logs_folder_id, "parent_type": 5},
    )
    if not resp.ok:
        return []
    return resp.json().get("docs", [])


def cu_get_doc_pages(token: str, workspace_id: str, doc_id: str) -> list:
    resp = requests.get(
        f"{DOCS_BASE}/workspaces/{workspace_id}/docs/{doc_id}/pages",
        headers=hdrs(token),
    )
    resp.raise_for_status()
    data = resp.json()
    return data if isinstance(data, list) else data.get("pages", [])


def cu_create_month_doc(token: str, workspace_id: str, logs_folder_id: str, month: str) -> tuple:
    resp = requests.post(
        f"{DOCS_BASE}/workspaces/{workspace_id}/docs",
        headers=hdrs(token),
        json={"name": month, "parent": {"id": logs_folder_id, "type": 5}},
    )
    resp.raise_for_status()
    doc_id = resp.json()["id"]
    pages = cu_get_doc_pages(token, workspace_id, doc_id)
    if not pages:
        sys.exit(f"No pages in newly created doc {doc_id}")
    return doc_id, pages[0]["id"]


def cu_resolve_month_doc(token: str, workspace_id: str, logs_folder_id: str, month: str) -> tuple:
    for doc in cu_list_folder_docs(token, workspace_id, logs_folder_id):
        if doc.get("name") == month:
            pages = cu_get_doc_pages(token, workspace_id, doc["id"])
            if pages:
                return doc["id"], pages[0]["id"]
    return cu_create_month_doc(token, workspace_id, logs_folder_id, month)


def cu_get_page_content(token: str, workspace_id: str, doc_id: str, page_id: str) -> str:
    resp = requests.get(
        f"{DOCS_BASE}/workspaces/{workspace_id}/docs/{doc_id}/pages/{page_id}",
        headers=hdrs(token),
        params={"content_format": "text/md"},
    )
    if not resp.ok:
        return ""
    return resp.json().get("content", "")


def cu_append_page(token: str, workspace_id: str, doc_id: str, page_id: str, text: str):
    resp = requests.put(
        f"{DOCS_BASE}/workspaces/{workspace_id}/docs/{doc_id}/pages/{page_id}",
        headers=hdrs(token),
        json={"content": text, "content_edit_mode": "append", "content_format": "text/md"},
    )
    if not resp.ok:
        sys.exit(f"Failed to append to page: {resp.status_code} {resp.text[:300]}")


def run_clickup_doc(cfg: dict, args, today, header: str, entry: str, ensure_only: bool):
    workspace_id = cfg["clickup"]["workspace_id"]
    logs_folder_id = cfg["logging"]["clickup_logs_folder_id"]
    if not workspace_id or not logs_folder_id:
        sys.exit("clickup_doc mode needs both clickup.workspace_id and logging.clickup_logs_folder_id")

    token = load_token()
    month = month_label(today)

    state = load_state()
    if state and state.get("month") == month:
        doc_id, page_id = state["doc_id"], state["page_id"]
    else:
        doc_id, page_id = cu_resolve_month_doc(token, workspace_id, logs_folder_id, month)
        save_state(month, doc_id, page_id)

    if ensure_only:
        print(f"[log] doc ready: {month} ({doc_id})")
        return

    content = cu_get_page_content(token, workspace_id, doc_id, page_id)
    delta = compute_delta(content, header, entry).rstrip("\n")
    cu_append_page(token, workspace_id, doc_id, page_id, delta)
    print(f"[log] {entry}")


# ─── Backend: local file ─────────────────────────────────────────────────────
def resolve_local_path(cfg: dict) -> Path:
    raw = cfg["logging"]["local_file_path"]
    if not raw:
        sys.exit("local_file mode needs logging.local_file_path")
    p = Path(raw)
    if not p.is_absolute():
        p = PROJECT_ROOT / p
    return p


def run_local_file(cfg: dict, header: str, entry: str, ensure_only: bool):
    path = resolve_local_path(cfg)
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.exists():
        path.touch()
    if ensure_only:
        print(f"[log] file ready: {path}")
        return

    content = path.read_text(encoding="utf-8") if path.exists() else ""
    delta = compute_delta(content, header, entry)
    with path.open("a", encoding="utf-8") as f:
        f.write(delta)
    print(f"[log] {entry}")


# ─── Main ────────────────────────────────────────────────────────────────────
def main():
    parser = argparse.ArgumentParser(description="Log a task action.")
    parser.add_argument("--ensure", action="store_true", help="Create/verify destination and exit (no entry)")
    parser.add_argument("--action", default="", help="Action verb: complete, create, delete, update, uncheck, move")
    parser.add_argument("--task-name", default="", dest="task_name")
    parser.add_argument("--tags", default="", help="Comma-separated tag names")
    parser.add_argument("--details", default="", help="Extra detail (e.g. 'priority: normal → high')")
    args = parser.parse_args()

    cfg = load_config()
    mode = cfg.get("logging", {}).get("mode", "none")

    if mode == "none":
        return

    today = logical_today()
    header = date_header(today)
    tags = [t.strip() for t in args.tags.split(",") if t.strip()]
    entry = format_entry(args.action, args.task_name, tags, args.details or None) if args.action else ""

    if not args.ensure and (not args.action or not args.task_name):
        sys.exit("--action and --task-name are required unless --ensure is passed")

    if mode == "clickup_doc":
        run_clickup_doc(cfg, args, today, header, entry, args.ensure)
    elif mode == "local_file":
        run_local_file(cfg, header, entry, args.ensure)
    else:
        sys.exit(f"Unknown logging mode: {mode!r}")


if __name__ == "__main__":
    main()
