#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APPS_DIR="$ROOT_DIR/apps/first-party"
TARGET="wasm32-wasip1"

if ! command -v cargo >/dev/null 2>&1; then
  echo "Error: cargo is not installed or not in PATH." >&2
  exit 1
fi

if ! command -v rustup >/dev/null 2>&1; then
  echo "Error: rustup is required to manage/install Rust targets." >&2
  exit 1
fi

if ! rustup target list --installed | grep -Fxq "$TARGET"; then
  echo "Rust target $TARGET is not installed. Attempting to install..."
  if ! rustup target add "$TARGET"; then
    echo "Error: unable to install required target $TARGET." >&2
    echo "This repository enforces the wasm32-wasip1 (WASI Preview 1) target by design." >&2
    exit 1
  fi
fi

if [[ ! -d "$APPS_DIR" ]]; then
  echo "No apps directory found at $APPS_DIR"
  exit 0
fi

found_any=false

for app_dir in "$APPS_DIR"/* "$ROOT_DIR/apps/community"/*; do
  [[ -d "$app_dir" ]] || continue
  found_any=true
  app_name="$(basename "$app_dir")"

  if [[ -f "$app_dir/Cargo.toml" ]]; then
    echo "Building $app_name for target $TARGET..."
    (cd "$app_dir" && cargo build --release --target "$TARGET")
  else
    echo "Skipping $app_name (no Cargo.toml found)."
  fi
done

if [[ "$found_any" = false ]]; then
  echo "No app directories found under $APPS_DIR"
  exit 0
fi

echo "Build complete."
