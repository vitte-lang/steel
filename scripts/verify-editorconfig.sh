#!/usr/bin/env bash
set -euo pipefail

if [[ ! -f ".editorconfig" ]]; then
  echo "Missing .editorconfig at repository root" >&2
  exit 1
fi

if ! grep -q "^\[steelconf\]$" .editorconfig; then
  echo "Missing [steelconf] section in .editorconfig" >&2
  exit 1
fi

if ! grep -q "^\[\*\\.muf\]$" .editorconfig; then
  echo "Missing [*.muf] section in .editorconfig" >&2
  exit 1
fi

check_section_indent_size() {
  local section="$1"
  if ! awk -v section="$section" '
    BEGIN { in_section=0; found=0; ok=0 }
    $0 == section { in_section=1; found=1; next }
    /^\[.*\]$/ && in_section { in_section=0 }
    in_section && $0 ~ /^indent_size[[:space:]]*=[[:space:]]*2$/ { ok=1 }
    END {
      if (!found) exit 2;
      if (!ok) exit 1;
    }
  ' .editorconfig; then
    if ! grep -Fxq "$section" .editorconfig; then
      echo "Missing ${section} section in .editorconfig" >&2
    else
      echo "${section} must define indent_size = 2" >&2
    fi
    exit 1
  fi
}

check_section_indent_size "[steelconf]"
check_section_indent_size "[*.muf]"

echo ".editorconfig verification passed"
