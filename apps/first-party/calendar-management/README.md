# Calendar Management

Calendar Management is a first-party Synapp app inspired by Outlook Calendar. It follows the Synapp dual-interface model:

- Human users use the React UI in `ui/`.
- AI agents and host automations call the same calendar capability surface through the Wasm exports described in `plugin.json`.
- The app runtime contains no AI logic and performs no direct persistence, provider, network, invite, free/busy, reminder, sharing, subscription, offline, or audit I/O.

## Host-Effect Boundary

The Wasm module is a deterministic planner and validator. It validates and normalizes input, applies permission checks, previews recurrence, scores suggested meeting times from supplied snapshots, and returns host-effect plans with stable idempotency keys.

Host services own all side effects:

- Calendar event and calendar persistence: `CalendarStoreRead`, `CalendarStoreWrite`, `CalendarStoreDelete`
- Invites and RSVP transport: `CalendarInviteSend`, `CalendarInviteRespond`
- Free/busy, room, and contact lookups: `FreeBusyLookup`, `RoomResourceLookup`, `ContactLookup`
- Conference links, reminders, sharing, subscriptions, offline cache, audit, and user notifications

No provider credentials, OAuth tokens, raw TCP/UDP connections, Graph/Exchange/CalDAV/SMTP calls, database access, OS threads, C bindings, prompts, OpenAI SDKs, LangChain code, or autonomous scheduling logic are used inside the app runtime.

## Rust Build And Test

Run unit tests:

```bash
cd apps/first-party/calendar-management
cargo test
```

Build the Wasm artifact when the local toolchain has the target installed:

```bash
cd apps/first-party/calendar-management
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

The expected Wasm entrypoint is:

```text
target/wasm32-wasip1/release/calendar_management.wasm
```

## Frontend Build

The UI scaffold lives in `ui/` and is registered from `synapp.app.json` as `ui/dist/index.html`.

```bash
cd apps/first-party/calendar-management/ui
npm install
npm run build
```

## Export Surface

All exports use the Synapp JSON ABI:

```rust
#[no_mangle]
pub extern "C" fn export_name(input_ptr: *const u8, input_len: usize) -> *mut u8
```

Inputs are UTF-8 JSON objects that include `context`. Outputs are null-terminated UTF-8 JSON strings released with `free_string(ptr)`. Successful responses use `{ "ok": { ... } }`; errors use `{ "err": { "code", "message", "details" } }`.

The Rust tests cover every exported function for successful input, missing context, missing permission, plus targeted recurrence validation/preview, event validation, suggested meeting times, effect availability, and host-effect planning behavior.