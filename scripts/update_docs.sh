#!/usr/bin/env bash
set -euo pipefail

# Synapp platform app-developer documentation updater
# This script keeps the local docs/app-developer reference in sync with the remote Synapp repo

SYNAPP_DOCS_REPO="https://github.com/nightsky78/Synapp.git"
LOCAL_DOCS_DIR="/home/johannes/projects/Synapp_apps/docs/app-developer"
TEMP_CLONE="/tmp/synapp_docs_sync_$$"

echo "Updating app-developer documentation from $SYNAPP_DOCS_REPO..."

# Clone the remote repo
git clone --depth 1 "$SYNAPP_DOCS_REPO" "$TEMP_CLONE" >/dev/null 2>&1

# Backup current docs
if [[ -d "$LOCAL_DOCS_DIR" ]]; then
  backup_dir="$LOCAL_DOCS_DIR.backup.$(date +%s)"
  cp -r "$LOCAL_DOCS_DIR" "$backup_dir"
  echo "Backed up current docs to $backup_dir"
fi

# Copy updated docs
mkdir -p "$(dirname "$LOCAL_DOCS_DIR")"
cp -r "$TEMP_CLONE/docs/app-developer" "$LOCAL_DOCS_DIR"

# Cleanup
rm -rf "$TEMP_CLONE"

echo "Documentation updated successfully."
echo "Location: $LOCAL_DOCS_DIR"
