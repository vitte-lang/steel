#!/usr/bin/env bash
set -euo pipefail

cargo test --bin steecleditor regenerate_doc_snapshot_blocks -- --ignored

if ! git diff --quiet -- docs/editor-setup.md; then
  echo "docs/editor-setup.md snapshots updated. Commit the changes."
  git --no-pager diff -- docs/editor-setup.md
  exit 1
fi

echo "Documentation snapshots already synchronized."
