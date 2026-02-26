#!/usr/bin/env bash
set -euo pipefail

RUN_NPM_CI=true
while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-npm-ci)
      RUN_NPM_CI=false
      shift
      ;;
    *)
      echo "Unknown argument: $1" >&2
      echo "Usage: $0 [--no-npm-ci]" >&2
      exit 2
      ;;
  esac
done

./scripts/verify-editorconfig.sh
(
  cd docs/angular
  if [[ "$RUN_NPM_CI" == true ]]; then
    npm run qa:local
  else
    npm run build:verify
  fi
)
./scripts/verify-site-sync.sh
