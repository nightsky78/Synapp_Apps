#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

python3 - "$ROOT_DIR" <<'PY'
import hashlib
import json
import re
import sys
from pathlib import Path
from urllib.parse import urlparse

root = Path(sys.argv[1])
index_path = root / "catalog" / "index.v1.json"
hex64 = re.compile(r"^[a-f0-9]{64}$")
app_id_re = re.compile(r"^[a-z0-9][a-z0-9-]{1,62}$")
allowed_hosts = {"github.com", "raw.githubusercontent.com"}
allowed_prefixes = (
    "https://github.com/nightsky78/Synapp_Apps/",
    "https://raw.githubusercontent.com/nightsky78/Synapp_Apps/",
)
bad_ui_keys = {"html", "css", "javascript", "script", "iframe", "srcdoc", "dangerouslySetInnerHTML"}

def fail(message):
    raise SystemExit(f"catalog validation failed: {message}")

def read_json(path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        fail(f"{path.relative_to(root)} is not valid JSON: {exc}")

def validate_url(value, field):
    parsed = urlparse(value)
    if parsed.scheme != "https":
        fail(f"{field} must use HTTPS")
    if parsed.netloc not in allowed_hosts:
        fail(f"{field} host is not allowlisted: {parsed.netloc}")
    if not value.startswith(allowed_prefixes):
        fail(f"{field} must point at nightsky78/Synapp_Apps")

def scan_ui(value, path="ui"):
    if isinstance(value, dict):
        for key, child in value.items():
            if key in bad_ui_keys:
                fail(f"forbidden UI field {path}.{key}")
            scan_ui(child, f"{path}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            scan_ui(child, f"{path}[{index}]")

index = read_json(index_path)
if index.get("schema_version") != "1.0.0":
    fail("catalog/index.v1.json schema_version must be 1.0.0")
seen = set()
for item in index.get("apps", []):
    app_id = item.get("app_id", "")
    if not app_id_re.match(app_id):
        fail(f"invalid app_id in index: {app_id}")
    if app_id in seen:
        fail(f"duplicate app_id in index: {app_id}")
    seen.add(app_id)
    entry_rel = item.get("catalog_entry", "")
    if entry_rel != f"catalog/apps/{app_id}.v1.json":
        fail(f"catalog_entry mismatch for {app_id}")
    entry_path = root / entry_rel
    entry = read_json(entry_path)
    manifest = entry.get("manifest", {})
    if entry.get("app_id") != app_id or manifest.get("app_id") != app_id:
        fail(f"app_id mismatch in {entry_rel}")
    if manifest.get("runtime") != "wasm32-wasip1" or manifest.get("platform_api") != "v1":
        fail(f"{app_id} must target wasm32-wasip1 platform_api v1")
    if manifest.get("resource_limits", {}).get("memory_bytes", 0) > 134217728:
        fail(f"{app_id} memory limit exceeds host maximum")
    if manifest.get("resource_limits", {}).get("timeout_ms", 0) > 5000:
        fail(f"{app_id} timeout exceeds host maximum")
    for field in ("repository_url", "package_url"):
        validate_url(entry.get(field, ""), f"{app_id}.{field}")
    validate_url(entry.get("source", {}).get("repository", ""), f"{app_id}.source.repository")
    for field in ("package_sha256", "manifest_sha256"):
        if not hex64.match(entry.get(field, "")):
            fail(f"{app_id}.{field} must be lowercase sha256 hex")
    manifest_dir = root / entry.get("source", {}).get("directory", "")
    manifest_path = manifest_dir / "synapp.app.json"
    if manifest_path.exists():
        source_manifest = read_json(manifest_path)
        actual_manifest_hash = hashlib.sha256(
            json.dumps(source_manifest, sort_keys=True, separators=(",", ":")).encode("utf-8")
        ).hexdigest()
        if actual_manifest_hash != entry["manifest_sha256"]:
            fail(f"{app_id}.manifest_sha256 does not match canonical {manifest_path.relative_to(root)}")
    signature = entry.get("signature", {})
    if signature.get("verification_status") not in {"verified", "metadata_verified"}:
        fail(f"{app_id} signature verification_status must be verified or metadata_verified")
    scan_ui(manifest.get("ui", {}))

print(f"Validated {len(seen)} catalog apps")
PY