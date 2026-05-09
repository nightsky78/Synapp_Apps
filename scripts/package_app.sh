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
UI_ENTRYPOINT="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1])).get("ui", {}).get("entrypoint", ""))' "$MANIFEST")"

mapfile -t SAFE_PACKAGE_PATHS < <(python3 - "$ENTRYPOINT" "$UI_ENTRYPOINT" <<'PY'
import sys
from pathlib import PurePosixPath

def package_safe(label, value):
  path = PurePosixPath(value)
  if path.is_absolute() or ".." in path.parts or str(path).startswith("~"):
    raise SystemExit(f"{label} path is not package-safe")
  return path

wasm_entrypoint = package_safe("entrypoint", sys.argv[1])
print(wasm_entrypoint)

ui_entrypoint = sys.argv[2]
if ui_entrypoint:
  ui_path = package_safe("ui.entrypoint", ui_entrypoint)
  ui_root = ui_path.parent
  print(ui_path if str(ui_root) == "." else ui_root)
PY
)

if [[ -f "$APP_DIR/Cargo.toml" ]]; then
  (cd "$APP_DIR" && cargo build --release --target wasm32-wasip1)
fi

if [[ ! -f "$APP_DIR/$ENTRYPOINT" ]]; then
  echo "Missing Wasm entrypoint: $APP_DIR/$ENTRYPOINT" >&2
  exit 1
fi

UI_PACKAGE_PATH=""
if [[ -n "$UI_ENTRYPOINT" ]]; then
  if [[ -f "$APP_DIR/$UI_ENTRYPOINT" ]]; then
    UI_PACKAGE_PATH="${SAFE_PACKAGE_PATHS[1]}"
  elif [[ -f "$APP_DIR/ui/dist/index.html" ]]; then
    UI_PACKAGE_PATH="ui/dist"
  else
    echo "Missing UI bundle for entrypoint: $UI_ENTRYPOINT" >&2
    exit 1
  fi
fi

OUT_DIR="$ROOT_DIR/packages"
mkdir -p "$OUT_DIR"
OUT="$OUT_DIR/$APP_ID-$VERSION.tar.gz"

PACKAGE_FILES=("synapp.app.json" "$ENTRYPOINT")
if [[ -n "$UI_ENTRYPOINT" ]]; then
  PACKAGE_FILES+=("$UI_PACKAGE_PATH")
fi
if [[ -f "$APP_DIR/plugin.json" ]]; then
  PACKAGE_FILES+=("plugin.json")
fi
if [[ -f "$APP_DIR/README.md" ]]; then
  PACKAGE_FILES+=("README.md")
fi

(cd "$APP_DIR" && tar --sort=name --mtime='UTC 2026-01-01' --owner=0 --group=0 --numeric-owner -czf "$OUT" "${PACKAGE_FILES[@]}")
python3 - "$OUT" "$ENTRYPOINT" "$UI_ENTRYPOINT" "$UI_PACKAGE_PATH" <<'PY'
import sys
import tarfile

archive, entrypoint, ui_entrypoint, ui_package_path = sys.argv[1:]
required = [entrypoint]
if ui_entrypoint and ui_package_path == ui_entrypoint:
  required.append(ui_entrypoint)
elif ui_entrypoint and ui_package_path:
  required.append(ui_package_path.rstrip('/') + '/index.html')

with tarfile.open(archive, "r:gz") as tar:
  names = set(tar.getnames())

missing = [path for path in required if path not in names]
if missing:
  raise SystemExit("Package archive is missing required files: " + ", ".join(missing))
PY
sha256sum "$OUT"