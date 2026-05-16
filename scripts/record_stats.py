#!/usr/bin/env python3
"""Write today's completion count to ~/.claude/task-stats.json (logical day)."""
import argparse, json, os
from datetime import datetime, timedelta

parser = argparse.ArgumentParser()
parser.add_argument('--count', type=int, required=True)
parser.add_argument('--date', default=None, help='Override logical date (YYYY-MM-DD)')
args = parser.parse_args()

logical_day = args.date if args.date else (datetime.now() - timedelta(hours=4)).strftime('%Y-%m-%d')

stats_path = os.path.expanduser('~/.claude/task-stats.json')
stats: dict = {}
if os.path.exists(stats_path):
    with open(stats_path) as f:
        stats = json.load(f)

stats[logical_day] = args.count

with open(stats_path, 'w') as f:
    json.dump(stats, f, indent=2, sort_keys=True)

print(f"Recorded {args.count} completions for {logical_day}")
