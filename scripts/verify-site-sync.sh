#!/usr/bin/env bash
set -euo pipefail

DIFF_FILE=""
ALLOW_DIFF=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --diff-file)
      DIFF_FILE="$2"
      shift 2
      ;;
    --allow-diff)
      ALLOW_DIFF=true
      shift
      ;;
    *)
      echo "Unknown argument: $1" >&2
      echo "Usage: $0 [--diff-file <path>] [--allow-diff]" >&2
      exit 2
      ;;
  esac
done

if git diff --quiet -- docs/site; then
  if [[ -n "$DIFF_FILE" ]]; then
    printf 'docs/site is in sync\n' > "$DIFF_FILE"
  fi
  echo "docs/site is in sync"
  exit 0
fi

if [[ -n "$DIFF_FILE" ]]; then
  git --no-pager diff -- docs/site > "$DIFF_FILE"
else
  git --no-pager diff -- docs/site
fi

if [[ "$ALLOW_DIFF" == true ]]; then
  echo "docs/site is out of sync (allowed in preview mode)"
  exit 0
fi

echo "docs/site is out of sync with docs/angular sources. Run docs/angular build and commit docs/site changes." >&2
exit 1
