#!/bin/bash
set -e

MUFFIN_CONFIG=$1
MFG_FILES=$(find . -name "*.mfg" -type f)

for mfg_file in $MFG_FILES; do
    echo "Compiling: $mfg_file"
    # ...existing compilation logic...
done

echo "✓ Compilation finished"
