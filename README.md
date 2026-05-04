# Synapp Apps

This repository is the GitHub-backed curated app catalog for the **Synapp Omni-Channel, AI-Native Application Platform**. Each app exposes its logic via a compiled WebAssembly binary and a `synapp.app.json` manifest, making it callable by both human users in the native UI and autonomous AI agents through generic host contracts.

## Apps

| App | Description | Docs |
|---|---|---|
| **Mail Client** | Full mail workflow app with account-aware reading, search, compose, send, scheduling, organization, and automation tools for humans and agents. | [App README](apps/first-party/mail-client/README.md) |
| **DummyPlugin** | Minimal reference implementation for the Dual-Interface pattern. | — |

## Repository Layout

```
catalog/              # Versioned catalog index, app entries, and JSON schemas
apps/first-party/     # Synapp-maintained Wasm applications
apps/community/       # Reviewed community applications
packages/             # Generated release packages (not committed)
docs/                 # Publishing and app developer documentation
scripts/              # Validation, packaging, and build tooling
```

## Building

Individual apps:

```bash
cd apps/first-party/<app>
cargo build --release --target wasm32-wasip1
```

Validate the catalog before opening a pull request:

```bash
scripts/validate_catalog.sh
```

See each app's documentation for build prerequisites and deployment steps.

## Architecture

All apps in this repository follow the [Synapp Dual-Interface architecture](.github/copilot-instructions.md):

- **Execution Interface:** Stateless `wasm32-wasip1` Wasm binary containing pure business logic.
- **Semantic Interface:** `synapp.app.json` manifest describing each function as an AI tool in JSON Schema.
- **Zero AI logic in apps:** Apps are dumb executors. The platform handles all AI routing and RAG.
- **State and network isolation:** No direct TCP/DB connections. All I/O delegated to the host via platform effects.