# Mail Client - Security Audit Report

Date: 2026-05-08  
Auditor: Security-Auditor agent, report recorded by pipeline coordinator  
Scope: `apps/first-party/mail-client`, `catalog/apps/mail-client.v1.json`, `scripts/package_app.sh`

## Executive Summary

The Mail Client re-audit found no security blocker in the Wasm source, UI source, manifests, catalog entry, or packaging script changes. The app remains a sandboxed host-effect planner with no direct IMAP, SMTP, OAuth, TCP, database, filesystem credential, or AI logic inside the Wasm module.

The packaging rework is security-acceptable: `scripts/package_app.sh` validates relative Wasm and UI entrypoints, rejects absolute paths and parent traversal, includes the declared `ui.entrypoint` bundle directory, and validates that required archive members exist after package generation. It packages the UI bundle needed for the human interface without including unrelated source, dependency, VCS, or build-cache directories.

## Findings

No Critical, High, Medium, or Low security findings were identified.

## Architecture Compliance

| Rule | Status | Evidence |
| --- | --- | --- |
| Zero AI logic | PASS | No OpenAI, Anthropic, LangChain, LLM routing, prompt orchestration, or model SDK use found in Rust/UI source. |
| Network/database isolation | PASS | No `std::net`, TCP/UDP, HTTP clients, DB drivers, or direct IMAP/SMTP/OAuth clients found in Wasm source. |
| Host-effect boundary | PASS | Mail I/O, OAuth, persistence, scheduling, notifications, and secret handling are represented as host-effect plans. |
| Wasm target safety | PASS | `Cargo.toml` uses `cdylib`, `serde`, and `serde_json`; no native-thread, TLS, C binding, or OS-specific dependencies found. |
| Secret handling | PASS | Schemas and source use opaque refs such as `secret_input_ref`, `secret_ref`, `oauth_connection_ref`, and `authorization_ref`; raw secret fields are rejected or absent. |
| Permission lattice | PASS | `accounts`, `organize`, `draft`, and `send` permissions do not implicitly escalate into unrelated capability domains. |
| Dual interface | PASS | Rust exports, `plugin.json`, `synapp.app.json`, and catalog tool entries remain aligned. |
| Human UI entrypoint | PASS | Source and catalog manifests declare `ui.entrypoint: "ui/dist/index.html"`; the package script now includes and validates the UI bundle. |

## OWASP Review

| Area | Status | Notes |
| --- | --- | --- |
| Broken access control | PASS | Exported operations flow through permission checks; capability domains remain separated. |
| Cryptographic failures | PASS | Raw credentials/tokens are not returned; validation redacts secret refs from normalized output. |
| Injection | PASS | Host-bound strings, recipient arrays, message IDs, rules, filters, pagination, subjects, and bodies are bounded and validated. |
| Insecure design | PASS | Side effects are explicit host-effect plans; permanent delete, folder force, and broad actions are visible in payloads. |
| Security misconfiguration | PASS | No `ui_schemas.main` or `settings_panel` remains in source/catalog manifest for the Mail Client main route. |
| Vulnerable components | PASS with observation | Rust dependencies are minimal. UI `npm audit` risk is tracked in the test matrix as a dev dependency issue. |
| Identification/auth failures | PASS | `context.user_id` is required and validated; agent identity is metadata, not an authorization source. |
| Data integrity | PASS | Catalog has non-empty package hash, manifest hash, and signature metadata. |
| Logging/monitoring | PASS | Errors are structured and avoid credential disclosure. |
| SSRF | PASS | User-entered mail server data is passed to host mail services as metadata; Wasm performs no direct network calls. |

## Packaging Security

| Check | Status | Evidence |
| --- | --- | --- |
| Wasm entrypoint validation | PASS | Packager fails if the manifest entrypoint is missing. |
| UI entrypoint validation | PASS | Packager fails if declared `ui.entrypoint` is missing. |
| Path traversal defense | PASS | Manifest-derived paths reject absolute paths, `..`, and `~` prefixes. |
| UI bundle inclusion | PASS | For `ui/dist/index.html`, the packager adds `ui/dist`, covering `index.html` and Vite assets. |
| Required archive member validation | PASS | The generated tarball is opened and checked for declared Wasm/UI entrypoints. |
| Sensitive directory inclusion | PASS | The package inputs are limited to manifest, Wasm entrypoint, UI bundle root, optional `plugin.json`, and optional `README.md`. |
| Shell injection review | PASS | Manifest paths are validated and passed as quoted tar arguments; no `eval` or unquoted command interpolation was introduced. |

## Verification Notes

Security-Auditor verification was read-only and reviewed source, manifests, catalog metadata, UI source, UI dist presence, and packaging script logic. Developer and Testing stages separately executed shell validation for build, package generation, tar contents, and catalog validation.

## Residual Observations

- Browser smoke and package validation still need final Testing re-run after this report update.
- `npm audit` reported moderate Vite/esbuild development dependency advisories; this is tracked as a release risk in the test matrix, not a Wasm/app sandbox violation.

FINAL VERDICT: PASSED
