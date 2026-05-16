#!/usr/bin/env python3
"""
setup.py — interactive wizard for first-time Aryan OS Dashboard configuration.

Reads CLICKUP_TOKEN from .env, queries ClickUp for your workspace structure,
runs `gcalcli list` for your calendar, and writes `os-config.json` at
the project root. Re-run anytime to refresh your selections.

Usage:
  python scripts/setup.py
"""
import json
import os
import random
import re
import shutil
import subprocess
import sys
from datetime import datetime, timedelta
from pathlib import Path

import requests

PROJECT_ROOT = Path(__file__).resolve().parent.parent
ENV_FILE = PROJECT_ROOT / ".env"
CONFIG_FILE = PROJECT_ROOT / "os-config.json"
API = "https://api.clickup.com/api/v2"


# ─── Token loading ───────────────────────────────────────────────────────────
def load_token() -> str:
    tok = os.environ.get("CLICKUP_TOKEN", "")
    if tok:
        return tok
    if ENV_FILE.exists():
        for line in ENV_FILE.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if line.startswith("CLICKUP_TOKEN="):
                return line.split("=", 1)[1].strip().strip('"').strip("'")
    sys.exit("CLICKUP_TOKEN not found. Add it to .env and try again.")


# ─── ClickUp helpers ─────────────────────────────────────────────────────────
def get(path: str, token: str) -> dict:
    r = requests.get(f"{API}{path}", headers={"Authorization": token})
    r.raise_for_status()
    return r.json()


def fetch_workspaces(token: str) -> list:
    return get("/team", token).get("teams", [])


def fetch_spaces(token: str, workspace_id: str) -> list:
    return get(f"/team/{workspace_id}/space", token).get("spaces", [])


def fetch_folders(token: str, space_id: str) -> list:
    return get(f"/space/{space_id}/folder", token).get("folders", [])


def fetch_folderless_lists(token: str, space_id: str) -> list:
    return get(f"/space/{space_id}/list", token).get("lists", [])


def fetch_folder_lists(token: str, folder_id: str) -> list:
    return get(f"/folder/{folder_id}/list", token).get("lists", [])


def fetch_all_lists(token: str, workspace_id: str) -> list:
    """Flatten every list in the workspace into [{id, name, path}]."""
    rows = []
    for space in fetch_spaces(token, workspace_id):
        space_name = space["name"]
        for lst in fetch_folderless_lists(token, space["id"]):
            rows.append({"id": lst["id"], "name": lst["name"], "path": f"{space_name} › {lst['name']}"})
        for folder in fetch_folders(token, space["id"]):
            for lst in fetch_folder_lists(token, folder["id"]):
                rows.append({
                    "id": lst["id"],
                    "name": lst["name"],
                    "path": f"{space_name} › {folder['name']} › {lst['name']}",
                })
    return rows


def fetch_all_folders(token: str, workspace_id: str) -> list:
    rows = []
    for space in fetch_spaces(token, workspace_id):
        for folder in fetch_folders(token, space["id"]):
            rows.append({
                "id": folder["id"],
                "name": folder["name"],
                "path": f"{space['name']} › {folder['name']}",
            })
    return rows


# ─── Prompt helpers ──────────────────────────────────────────────────────────
def prompt_pick_one(label: str, options: list, format_fn) -> dict:
    print(f"\n{label}")
    for i, opt in enumerate(options, 1):
        print(f"  {i:>2}) {format_fn(opt)}")
    while True:
        raw = input(f"  Pick [1-{len(options)}]: ").strip()
        if raw.isdigit() and 1 <= int(raw) <= len(options):
            return options[int(raw) - 1]
        print("  Invalid choice — try again.")


def prompt_pick_many(label: str, options: list, format_fn) -> list:
    print(f"\n{label}")
    for i, opt in enumerate(options, 1):
        print(f"  {i:>2}) {format_fn(opt)}")
    while True:
        raw = input(f"  Pick (comma-separated, e.g. 1,3,4): ").strip()
        try:
            idxs = [int(x.strip()) for x in raw.split(",") if x.strip()]
            if all(1 <= i <= len(options) for i in idxs):
                return [options[i - 1] for i in idxs]
        except ValueError:
            pass
        print("  Invalid choice — try again.")


# ─── Slugify ─────────────────────────────────────────────────────────────────
def slugify(name: str) -> str:
    s = name.lower().replace("&", "and")
    s = re.sub(r"[^a-z0-9]+", "-", s).strip("-")
    return s or "list"


# ─── Calendar ────────────────────────────────────────────────────────────────
def list_calendars() -> list:
    if not shutil.which("gcalcli"):
        print("\n[calendar] gcalcli not on PATH — skipping calendar setup.")
        return []
    try:
        env = os.environ.copy()
        env["PYTHONIOENCODING"] = "utf-8"
        out = subprocess.check_output(["gcalcli", "list"], env=env, text=True, encoding="utf-8")
    except subprocess.CalledProcessError:
        print("\n[calendar] gcalcli list failed — skipping calendar setup.")
        return []
    cals = []
    for line in out.splitlines():
        line = line.strip()
        if not line or line.startswith("-") or line.startswith("Access"):
            continue
        # gcalcli list rows are "<access>  <calendar name>"; split on whitespace runs
        parts = re.split(r"\s{2,}", line)
        if len(parts) >= 2:
            cals.append(parts[-1])
    return cals


# ─── Main ────────────────────────────────────────────────────────────────────
def main():
    print("Aryan OS Dashboard — Setup wizard")
    print("=" * 50)

    token = load_token()

    # 1. Workspace
    workspaces = fetch_workspaces(token)
    if not workspaces:
        sys.exit("No ClickUp workspaces found for this token.")
    ws = (
        workspaces[0]
        if len(workspaces) == 1
        else prompt_pick_one(
            "[1/4] Choose your ClickUp workspace:",
            workspaces,
            lambda w: f"{w['name']}  ({w['id']})",
        )
    )
    if len(workspaces) == 1:
        print(f"\n[1/4] Workspace: {ws['name']} ({ws['id']})")
    workspace_id = ws["id"]

    # 2. All lists across workspace
    print("\n[2/4] Fetching lists across all spaces…")
    all_lists = fetch_all_lists(token, workspace_id)
    if not all_lists:
        sys.exit("No lists found in this workspace.")

    # 3. Daily list
    daily = prompt_pick_one(
        "[3/4] Choose your DAILY truth list (drives TodayTasks + NowNext):",
        all_lists,
        lambda l: f"{l['path']}  ({l['id']})",
    )

    # 4. Area lists
    remaining = [l for l in all_lists if l["id"] != daily["id"]]
    areas_picked = prompt_pick_many(
        "[4/4] Choose AREA lists for the New Task dropdown (multi-select):",
        remaining,
        lambda l: f"{l['path']}  ({l['id']})",
    ) if remaining else []

    # 5. Logging mode
    LOG_MODES = [
        ("clickup_doc", "Append to a monthly doc in a ClickUp Logs folder (requires ClickUp Docs)"),
        ("local_file",  "Append to a local Markdown/text file"),
        ("none",        "Don't log task actions (the chart still tracks daily completion counts)"),
    ]
    log_choice = prompt_pick_one(
        "\n[+]  How should task actions be logged?",
        LOG_MODES,
        lambda m: m[1],
    )
    log_mode = log_choice[0]

    log_folder_id = ""
    log_file_path = ""

    if log_mode == "clickup_doc":
        folders = fetch_all_folders(token, workspace_id)
        if not folders:
            print("  No folders found in workspace — falling back to local_file mode.")
            log_mode = "local_file"
        else:
            logs_folder = prompt_pick_one(
                "Choose the LOGS folder (monthly action-log docs created here):",
                folders,
                lambda f: f"{f['path']}  ({f['id']})",
            )
            log_folder_id = logs_folder["id"]

    if log_mode == "local_file":
        raw = input("Path for the log file (relative to project root or absolute) [logs/actions.md]: ").strip()
        log_file_path = raw or "logs/actions.md"

    # 6. Calendar (optional)
    cals = list_calendars()
    personal_email = ""
    if cals:
        cal_pick = prompt_pick_one(
            "[+]  Choose your PERSONAL Google calendar:",
            cals,
            lambda c: c,
        )
        personal_email = cal_pick

    # Build config
    config = {
        "clickup": {
            "workspace_id": workspace_id,
            "daily_list_id": daily["id"],
            "areas": [
                {"list_id": a["id"], "label": a["name"], "slug": slugify(a["name"])}
                for a in areas_picked
            ],
        },
        "calendar": {
            "personal_email": personal_email,
        },
        "commands": {
            "start_day_skill": "start-day",
        },
        "logging": {
            "mode": log_mode,
            "clickup_logs_folder_id": log_folder_id,
            "local_file_path": log_file_path,
        },
    }

    CONFIG_FILE.write_text(json.dumps(config, indent=2), encoding="utf-8")
    print(f"\nWrote {CONFIG_FILE.name}")

    # Download ambient sounds (skips files already present / unchanged).
    print("\n[+] Downloading ambient sounds (10-min snippets from YouTube)…")
    download_script = PROJECT_ROOT / "scripts" / "download_sounds.py"
    rc = subprocess.run([sys.executable, str(download_script)]).returncode
    if rc != 0:
        print("  (sound download had errors — re-run `python scripts/download_sounds.py` later)")

    # Seed demo task stats so the TaskStats chart isn't empty on first launch.
    maybe_seed_demo_stats()

    print("\nDone. Launch the dashboard with `npm run tauri dev`.")


def maybe_seed_demo_stats() -> None:
    """Write 14 days of random completion counts to ~/.claude/task-stats.json
    if RANDOM_COMPLETIONS_ON_DOWNLOAD is True AND the file doesn't exist.
    Never overwrites existing data."""
    flag = env_value("RANDOM_COMPLETIONS_ON_DOWNLOAD", "True").strip().lower()
    if flag not in ("true", "1", "yes", "y"):
        return

    stats_path = Path.home() / ".claude" / "task-stats.json"
    if stats_path.exists():
        print(f"\n[+] {stats_path.name} already exists — skipping demo seed.")
        return

    stats_path.parent.mkdir(parents=True, exist_ok=True)
    today = (datetime.now() - timedelta(hours=4)).date()
    stats = {}
    # Seed 14 days of HISTORY (yesterday and back). Today starts empty — the
    # dashboard's TaskStats chart will populate it tomorrow morning when the
    # user clicks Start Day, which writes the real count for the prior day.
    for i in range(1, 15):
        day = today - timedelta(days=i)
        # 70% productive days (4-12 completions), 30% lighter days (0-3)
        count = random.randint(4, 12) if random.random() < 0.7 else random.randint(0, 3)
        stats[day.strftime("%Y-%m-%d")] = count

    stats_path.write_text(
        json.dumps(stats, indent=2, sort_keys=True),
        encoding="utf-8",
    )
    print(f"\n[+] Seeded 14 days of demo task stats → {stats_path}")


def env_value(name: str, default: str = "") -> str:
    """Read an env var from the process env, falling back to .env parsing."""
    val = os.environ.get(name, "")
    if val:
        return val
    if ENV_FILE.exists():
        for line in ENV_FILE.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if line.startswith(f"{name}="):
                return line.split("=", 1)[1].strip().strip('"').strip("'")
    return default


if __name__ == "__main__":
    main()
