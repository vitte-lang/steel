#!/usr/bin/env bash
set -euo pipefail

WORKFLOW_FILE="${1:-.github/workflows/steecleditor.yml}"
if [[ ! -f "$WORKFLOW_FILE" ]]; then
  echo "Missing workflow file: $WORKFLOW_FILE" >&2
  exit 1
fi

echo "site/quick jobs in $WORKFLOW_FILE:"
awk '
  /^[[:space:]]{2}[a-zA-Z0-9_-]+:[[:space:]]*$/ {
    job=$1
    sub(/:$/, "", job)
    if (job ~ /^site-/) {
      print "- " job
    }
  }
' "$WORKFLOW_FILE"
