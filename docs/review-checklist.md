# Review Checklist

- Manifest uses `schema_version: 1.0.0`, `runtime: wasm32-wasip1`, and `platform_api: v1`.
- Capabilities are minimal, risk-labelled, and grantable by administrators.
- UI contributions are schema-driven only. No HTML, CSS, JavaScript, remote scripts, or DOM injection fields are present.
- Resource limits do not exceed 128 MiB memory or 5000 ms execution time.
- Catalog URLs use HTTPS and point to `github.com/nightsky78/Synapp_Apps` or raw GitHub content for this repository.
- Package and manifest SHA-256 fields are present and lowercase hex.
- Signature metadata is present. `metadata_verified` is acceptable only for metadata-only development catalog entries.
- Source repository, directory, and commit are exact.