#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
AGENTS_DIR="$ROOT_DIR/.github/agents"

required_files=(
  "$AGENTS_DIR/pipeline.yaml"
  "$AGENTS_DIR/architect.agent.md"
  "$AGENTS_DIR/ui-designer.agent.md"
  "$AGENTS_DIR/developer.agent.md"
  "$AGENTS_DIR/qa-security-tester.agent.md"
  "$AGENTS_DIR/testing.agent.md"
  "$AGENTS_DIR/techwriter.agent.md"
)

missing=0
for file in "${required_files[@]}"; do
  if [[ ! -f "$file" ]]; then
    echo "Missing required pipeline file: $file"
    missing=1
  fi
done

if [[ "$missing" -ne 0 ]]; then
  echo "Pipeline check failed."
  exit 1
fi

echo "Pipeline check passed."
echo "Execution order: architect -> ui-designer -> developer -> qa-security-tester -> testing -> techwriter"
