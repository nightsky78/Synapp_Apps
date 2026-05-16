# Contacts Management

Contacts Management is a first-party Synapp app for managing caller-scoped contacts, group filters, and directory synchronization planning.

The package follows the first-party app layout used elsewhere in this repository:

- `Cargo.toml` and `src/lib.rs` contain the pure Wasm action handlers.
- `synapp.app.json` declares the packaged semantic contract.
- `plugin.json` mirrors the execution and semantic interface metadata.
- `ui/` contains the schema-driven React workspace shell.

## Build

```bash
cd apps/first-party/contacts-management
cargo build --release --target wasm32-wasip1
cd ui
npm run build
```

## Actions

The Wasm module exposes five pure actions:

- `contacts.list`
- `contacts.create`
- `contacts.update`
- `contacts.delete`
- `contacts.sync-directory`

The first four actions return host-managed document effects. `contacts.sync-directory` is intentionally blocked until the platform user-enumeration contract is available, so it never emits side effects on failure.

## Layout Contract

The workspace still uses the three stable regions defined by the UI scaffold:

- Groups rail
- Contacts list
- Contact detail pane

On narrow screens, the layout collapses into a single active pane with a compact switcher.
