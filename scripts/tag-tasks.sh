#!/usr/bin/env bash
# tag-tasks.sh — Add ClickUp tags to tasks in bulk.
#
# Modes:
#   tag-tasks.sh <task_id>:<tag> [<task_id>:<tag> ...]   # individual mappings
#   tag-tasks.sh --list <list_id> <tag>                  # tag every task in a list (incl. closed)
#
# Examples:
#   ./scripts/tag-tasks.sh 86b9jffwv:projects 86b9j8duy:health-habits
#   ./scripts/tag-tasks.sh --list 901415158864 school
#
# Reads CLICKUP_TOKEN from .env at the project root (../.env relative to this script).

set -euo pipefail

SCRIPT_DIR=$(dirname "$(realpath "$0")")
ENV_FILE="$SCRIPT_DIR/../.env"

if [ ! -f "$ENV_FILE" ]; then
  echo "error: .env not found at $ENV_FILE" >&2
  exit 1
fi

set -a
. "$ENV_FILE"
set +a

if [ -z "${CLICKUP_TOKEN:-}" ]; then
  echo "error: CLICKUP_TOKEN not set in .env" >&2
  exit 1
fi

usage() {
  cat <<EOF >&2
usage:
  $0 <task_id>:<tag> [<task_id>:<tag> ...]
  $0 --list <list_id> <tag>
EOF
  exit 1
}

[ "$#" -eq 0 ] && usage

add_tag() {
  local tid="$1" slug="$2" status
  status=$(curl -s -o /dev/null -w "%{http_code}" \
    -X POST "https://api.clickup.com/api/v2/task/$tid/tag/$slug" \
    -H "Authorization: $CLICKUP_TOKEN")
  printf '[%s] %s -> %s\n' "$status" "$tid" "$slug"
}

if [ "$1" = "--list" ]; then
  [ "$#" -ne 3 ] && usage
  list_id="$2"
  tag="$3"
  page=0
  total=0
  while [ "$page" -lt 20 ]; do
    response=$(curl -s "https://api.clickup.com/api/v2/list/$list_id/task?subtasks=false&include_closed=true&page=$page" \
      -H "Authorization: $CLICKUP_TOKEN")
    ids=$(echo "$response" | python -c "import sys,json; d=json.load(sys.stdin); [print(t['id']) for t in d.get('tasks', [])]" 2>/dev/null || true)
    [ -z "$ids" ] && break
    while IFS= read -r tid; do
      [ -z "$tid" ] && continue
      add_tag "$tid" "$tag"
      total=$((total + 1))
    done <<< "$ids"
    page=$((page + 1))
  done
  printf '\ntotal: %d tasks tagged with %s\n' "$total" "$tag"
  exit 0
fi

# Individual mappings mode
for entry in "$@"; do
  tid="${entry%:*}"
  slug="${entry#*:}"
  if [ "$tid" = "$entry" ] || [ -z "$slug" ]; then
    echo "skip: invalid entry '$entry' (expected task_id:tag)" >&2
    continue
  fi
  add_tag "$tid" "$slug"
done
