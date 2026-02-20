#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [[ -f "$HOME/export-esp.sh" ]]; then
  # shellcheck disable=SC1090
  source "$HOME/export-esp.sh"
else
  echo "Fehler: $HOME/export-esp.sh nicht gefunden. Bitte zuerst espup ausfuehren."
  exit 1
fi

MODE="${1:-build}"

case "$MODE" in
  build)
    cargo +esp build --release
    ;;
  run)
    cargo +esp run --release
    ;;
  check)
    cargo +esp check --release
    ;;
  *)
    echo "Usage: $0 [build|run|check]"
    exit 2
    ;;
esac
