# Synapp Apps

This repository contains applications built for the **Synapp Omni-Channel, AI-Native Application Platform**. Each app exposes its logic via a compiled WebAssembly binary (Execution Interface) and a `plugin.json` manifest (Semantic Interface), making it callable by both human users in a React UI and autonomous AI agents through the Synapp Host Broker.

## Apps

| App | Description | Docs |
|---|---|---|
| **Mail Client** | AI-native email management: read, search, draft, send, organize. 8 Wasm exports with hierarchical permission model. | [Release Notes](docs/mail-client/RELEASE.md) |
| **DummyPlugin** | Minimal reference implementation for the Dual-Interface pattern. | — |

## Repository Layout

```
apps/          # Wasm application source (Rust)
docs/          # Platform and app-level documentation
scripts/       # Build and tooling scripts
```

## Building

Individual apps:

```bash
cd apps/<AppName>
cargo build --release --target wasm32-wasip1
```

See each app's documentation for build prerequisites and deployment steps.

## Architecture

All apps in this repository follow the [Synapp Dual-Interface architecture](.github/copilot-instructions.md):

- **Execution Interface:** Stateless `wasm32-wasip1` Wasm binary containing pure business logic.
- **Semantic Interface:** `plugin.json` manifest describing each function as an AI tool in JSON Schema.
- **Zero AI logic in apps:** Apps are dumb executors. The platform handles all AI routing and RAG.
- **State and network isolation:** No direct TCP/DB connections. All I/O delegated to the host via platform effects.