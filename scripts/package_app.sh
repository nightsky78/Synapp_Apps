#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="${1:-}"

if [[ -z "$APP_DIR" ]]; then
  echo "Usage: scripts/package_app.sh <app-dir>" >&2
  exit 2
fi

if [[ ! -d "$APP_DIR" ]]; then
  APP_DIR="$ROOT_DIR/$APP_DIR"
fi

MANIFEST="$APP_DIR/synapp.app.json"
if [[ ! -f "$MANIFEST" ]]; then
  echo "Missing synapp.app.json in $APP_DIR" >&2
  exit 1
fi

APP_ID="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["app_id"])' "$MANIFEST")"
VERSION="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' "$MANIFEST")"
ENTRYPOINT="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["entrypoint"])' "$MANIFEST")"

if [[ -f "$APP_DIR/Cargo.toml" ]]; then
  (cd "$APP_DIR" && cargo build --release --target wasm32-wasip1)
fi

if [[ ! -f "$APP_DIR/$ENTRYPOINT" ]]; then
  echo "Missing Wasm entrypoint: $APP_DIR/$ENTRYPOINT" >&2
  exit 1
fi

OUT_DIR="$ROOT_DIR/packages"
mkdir -p "$OUT_DIR"
OUT="$OUT_DIR/$APP_ID-$VERSION.tar.gz"

python3 - "$APP_DIR" "$ENTRYPOINT" <<'PY'
import sys
from pathlib import PurePosixPath

entry = PurePosixPath(sys.argv[2])
if entry.is_absolute() or ".." in entry.parts or str(entry).startswith("~"):
    raise SystemExit("entrypoint path is not package-safe")
PY

PACKAGE_FILES=("synapp.app.json" "$ENTRYPOINT")
if [[ -f "$APP_DIR/plugin.json" ]]; then
  PACKAGE_FILES+=("plugin.json")
fi
if [[ -f "$APP_DIR/README.md" ]]; then
  PACKAGE_FILES+=("README.md")
fi

(cd "$APP_DIR" && tar --sort=name --mtime='UTC 2026-01-01' --owner=0 --group=0 --numeric-owner -czf "$OUT" "${PACKAGE_FILES[@]}")
sha256sum "$OUT"