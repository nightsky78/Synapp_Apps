# Publishing Flow

Synapp_Apps is a GitHub-backed curated catalog. Apps are reviewed as pull requests; Synapp does not run a custom app-store backend.

1. Add or update `apps/first-party/<app>/synapp.app.json` or `apps/community/<app>/synapp.app.json`.
2. Build the Wasm app with `scripts/build_all.sh` or `scripts/package_app.sh <app-dir>`.
3. Update `catalog/apps/<app_id>.v1.json` and `catalog/index.v1.json` with the release URL, SHA-256 digests, signature metadata, and exact source commit.
4. Run `scripts/validate_catalog.sh` locally.
5. Open a pull request. CI validates schemas, allowlisted GitHub URLs, digest shape, UI contribution safety, and resource limits.

The Synapp host treats catalog metadata as an install descriptor. Package-byte verification is a separate installer boundary and must not be reported as successful until the archive has been downloaded, hashed, signature-checked, and traversal-checked.