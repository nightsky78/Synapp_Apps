# Mail Client — Developer Guide

**App:** Mail Client v1.0.0  
**Source:** `apps/MailClient/src/lib.rs`  
**Schema:** `apps/MailClient/plugin.json`  
**Audience:** Engineers maintaining or extending the Mail Client  

---

## Architecture Overview

### The Dual-Interface Constraint

Every function in this app exists twice: once as a Rust Wasm export (the **Execution Interface**) and once as a JSON Schema tool description in `plugin.json` (the **Semantic Interface**). These two representations are a synchronized contract. If you change one, you must change the other. The platform uses `plugin.json` to route AI agent calls to the Wasm binary. A mismatch will cause silent routing failures or schema validation errors at the broker.

### Stateless Executor Pattern

The Mail Client is a pure input-output machine. It holds no state between invocations. There are no global variables, no `static mut`, no `Mutex`, and no async runtimes. Every function:

1. Receives raw pointer + length pairs for all string/JSON inputs (standard Wasm FFI)
2. Parses inputs from raw memory into Rust types
3. Validates permission at the function boundary
4. Validates business inputs (nulls, format, bounds)
5. Returns a JSON-encoded effect payload to the host

The host broker executes the effects (reading from email store, sending via SMTP, persisting drafts). The Wasm module never touches a network socket or a database directly.

### Effect Pattern

Instead of performing I/O, functions return a description of the I/O they need. These are **platform effects**, serialised as JSON in the response. Example:

```rust
// send_email does NOT call SMTP. It returns this to the host:
let effects = json!({
    "effects": [
        { "type": "SendEmail", "params": { "draft_id": draft_id, "user_id": user_id } },
        { "type": "MoveDocument", "params": { "user_id": user_id, "from_collection": "drafts", ... } }
    ]
});
```

The host broker interprets and executes effects in sequence. This keeps the Wasm sandbox completely isolated from external systems.

### Permission Model

Permissions flow in from the host as a string parameter. The Rust `Permission` enum maps the string:

```
"none"  → Permission::None
"read"  → Permission::Read
"draft" → Permission::Draft
"send"  → Permission::Send
any other string → Permission::None  (conservative default)
```

The `satisfies()` method enforces the hierarchy:

```
Send  > Draft > Read > None
```

A `Send` permission satisfies any required level. `Read` permission satisfies only `Read` and `None`. The hierarchy is checked at the first line of every exported function via `check_permission(actual, required)`. If the check fails, the function immediately returns a `PermissionDenied` error without executing any other logic.

### Memory Management (FFI Contract)

String inputs arrive as `(*const u8, usize)` pointer-length pairs. The `parse_string_from_raw` helper validates non-null pointer, non-zero length, and valid UTF-8 before producing a `String`. The single unsafe block inside this helper is the standard Wasm FFI pattern for reading host-provided memory:

```rust
unsafe { std::slice::from_raw_parts(ptr, len) }
```

Return values are heap-allocated JSON strings. Ownership is transferred to the caller via `std::mem::forget` — the host runtime is responsible for deallocation. This is the standard WASI memory handoff pattern.

---

## Function-by-Function Guide

### `read_emails`
Fetches a paginated list of emails from a folder. Pure query — no side effects.

- **Permission:** `read`
- **Key behaviours:** Results sorted newest-first. Limit is clamped to 500 (never panics on large inputs). Returns `total_count` and `has_more` for UI pagination.
- **Platform effect emitted:** `QueryEmailStore(user_id, mailbox, offset, limit)`

### `get_email`
Fetches a single email by ID including full body. Pure query from the Wasm side. The host marks the email as read after a successful call — this is a host-side side effect, not Wasm logic.

- **Permission:** `read`
- **Platform effect emitted:** `QueryEmailStore(user_id, email_id)`

### `draft_email`
Validates recipient format and count, then emits a `CreateDocument` effect to store the draft in the platform ephemeral store. Drafts expire after 7 days.

- **Permission:** `draft`
- **Key validation:** At least one `to` recipient required. Combined to + cc + bcc ≤ 100. All addresses validated via `is_valid_email()` (basic format; not full RFC 5322).
- **Platform effect emitted:** `CreateDocument` in the `drafts` collection.

### `send_email`
The only function that requires `send` permission (alongside `move_email` for non-DRAFTS targets). Emits two effects: `SendEmail` (dispatches via platform SMTP) and `MoveDocument` (moves the draft to SENT folder with `sent_at` timestamp). Idempotent — a second call with the same `draft_id` returns `DraftAlreadySent`.

- **Permission:** `send`
- **Platform effects emitted:** `SendEmail`, `MoveDocument` (drafts → sent)

### `list_folders`
Enumerates all system and custom folders with unread and total counts. Pure query.

- **Permission:** `read`
- **Platform effect emitted:** `QueryFolderList(user_id)`
- **Note:** System folders always present; custom folders listed alphabetically after.

### `move_email`
Permission required depends on the target folder. Moving to or from DRAFTS requires `draft`. Moving to any other folder (TRASH, ARCHIVE, INBOX, SENT, custom) requires `send`. The function determines the required permission at runtime based on `target_folder_id`:

```rust
let required_perm = if target_folder_id == "drafts" {
    Permission::Draft
} else {
    Permission::Send
};
```

- **Platform effect emitted:** `UpdateDocument` — updates the email's `folder_id` field.

### `search_emails`
Full-text search with operator support (`from:`, `to:`, `subject:`, `before:`, `after:`, `is:unread`, `has:attachment`). Limit clamped to 500. Empty query is accepted and passed to the platform. Returns `matched_count` (total matches across all pages) and `has_more`.

- **Permission:** `read`
- **Platform effect emitted:** `SearchEmailIndex(user_id, query, limit)`

### `get_agent_permissions`
Reflects back the permission context injected by the host. No platform effect required — the data is already present in the function parameters. Always succeeds; no permission gate. Useful for agents to introspect their capabilities before attempting gated operations.

- **Permission:** none
- **Note:** Returns a flat `PermissionSet` object, not wrapped in `{ "ok": ... }` — this is a known v1.0 inconsistency to be addressed in v1.1.

---

## Adding New Functions

Follow this checklist in order for every new exported function. Do not skip steps — the Dual-Interface Rule requires schema and code to move together.

### Step 1: Define the contract

Before writing code, add the function specification to `docs/mail-client/contracts.md`. Document:
- Purpose
- Permission required
- Input parameters (name, type, description)
- Return type (success shape and error codes)
- Side effects
- Platform effects emitted

### Step 2: Update plugin.json (Semantic Interface)

Add the function to `plugin.json` in three places:

1. **`executionInterface.exports`** — Add the function name as an export entry with `inputSchema` and `outputSchema` using JSON Schema.
2. **`semanticInterface.tools`** — Add the tool description in plain English. Include `description`, `category` (`query` or `mutation`), `permissionLevel`, and an `example`.
3. **`definitions`** — If the function introduces new shared types, add them here and reference them with `$ref`.

### Step 3: Implement in Rust

Add the exported function to `src/lib.rs`. Follow the established pattern exactly:

```rust
#[no_mangle]
pub extern "C" fn my_new_function(
    user_id: *const u8, user_id_len: usize,
    // ... other parameters
    permission: *const u8, permission_len: usize,
) -> *mut u8 {
    // 1. Parse permission
    let perm = parse_permission(permission, permission_len);

    // 2. Check permission FIRST — no business logic before this
    if let Err(e) = check_permission(perm, Permission::RequiredLevel) {
        return return_json_response(&e);
    }

    // 3. Parse and validate other inputs
    let user_id = match parse_string_from_raw(user_id, user_id_len) {
        Ok(s) => s,
        Err(e) => return return_json_response(&e),
    };

    // 4. Business logic — build effects, validate business rules

    // 5. Return result
    return_json_response(&result)
}
```

**Critical rules:**
- `#[no_mangle]` is required. Without it, the export is optimised away or renamed.
- The permission check must be the first substantive operation. No parsing, no logging, no state access before it.
- Use `unwrap_or_else` and `unwrap_or_default` on fallible paths rather than `unwrap()`. Panics in Wasm produce trap signals that the host cannot meaningfully handle.
- Never use `std::net`, `std::fs`, database crates, or async runtimes. If a new dependency is needed, verify it compiles for `wasm32-wasip1` before adding it to `Cargo.toml`.

### Step 4: Update documentation

- Add the new function to `docs/mail-client/contracts.md` (already done in Step 1).
- Update the permission matrix table in `docs/mail-client/architecture.md`.
- Add the function to the summary table at the bottom of `contracts.md`.
- If the function adds a new UI surface, update `docs/mail-client/ui-spec.md` and `docs/mail-client/ui-copy.md`.

### Step 5: Build and verify

```bash
cargo build --release --target wasm32-wasip1
strings target/wasm32-wasip1/release/mail_client.wasm | grep "my_new_function"
```

The new function name must appear in the binary. If it does not, verify the `#[no_mangle]` attribute and that the function is `pub extern "C"`.

---

## Permission Model: Extending or Modifying

### Adding a New Permission Level

The permission model is defined in two places: the `Permission` enum and its `satisfies()` method in `src/lib.rs`, and the `permissionLevel` enum in `plugin.json`. Both must be updated together.

In `src/lib.rs`:

```rust
enum Permission {
    None,
    Read,
    Draft,
    Send,
    // Add new level here
    Admin,
}
```

Update `satisfies()` to include the new level in the hierarchy. Update `from_str()` to parse the new string.

In `plugin.json`, add the new level to the `permissionLevel` enum in `semanticInterface.tools[*].permissionLevel`.

### Changing a Function's Permission Requirement

Change the `check_permission(perm, Permission::X)` call in the function implementation. Then update:
- `plugin.json` → `semanticInterface.tools[function].permissionLevel`
- `docs/mail-client/architecture.md` → permission matrix table
- `docs/mail-client/contracts.md` → the function's contract entry

---

## Error Handling Patterns

All functions return a JSON-encoded result using the `AppResult` envelope:

```rust
// Success
{ "ok": <value> }

// Error
{ "err": { "code": "<ErrorCode>", "message": "<human-readable detail>" } }
```

The exception is `get_agent_permissions`, which returns a flat `PermissionSet` (no envelope). This inconsistency is a known v1.0 limitation.

Use the defined `ErrorResponse` struct for all errors. Do not return free-form error strings. Error codes must match the `Error` definition in `plugin.json`. If you add a new error code:

1. Add it to the `MailClientError` enum in `src/lib.rs`.
2. Add it to the `code` enum in `plugin.json` definitions.
3. Update `contracts.md` for each function that can produce the new code.

Error messages should be human-readable but must not include:
- Stack traces
- System file paths
- Credentials or tokens
- Raw database error messages
- Internal state not visible to the caller

---

## Testing Locally

### Building

```bash
cd apps/MailClient
cargo build --release --target wasm32-wasip1
```

### Checking for Compilation Errors and Warnings

```bash
cargo check --target wasm32-wasip1
```

This is faster than a full build and sufficient to catch type errors and missing imports without producing a binary.

### Running cargo clippy

```bash
cargo clippy --target wasm32-wasip1 -- -D warnings
```

### Unit Tests (Current Limitation)

The app currently has no `#[test]` blocks. Unit tests for `wasm32-wasip1` binaries cannot be run with standard `cargo test` — they require a Wasm runtime host.

To run unit tests once a test suite is added:

1. Install `wasmtime`:
   ```bash
   curl https://wasmtime.dev/install.sh -sSf | bash
   ```

2. Use `cargo test` with the `wasm32-wasip1` target and `wasmtime` as the test runner:
   ```bash
   cargo test --target wasm32-wasip1
   ```
   This requires configuring `.cargo/config.toml` with a runner:
   ```toml
   [target.wasm32-wasip1]
   runner = "wasmtime"
   ```

### Debugging a Function

Because the module is pure Wasm with no debug output, the recommended approach is:

1. **Test pure logic natively first.** Extract the business logic (permission checks, validation, effect construction) into functions that take `String` inputs. Write a small `main.rs` or a `#[test]` block that exercises these with the native target (`x86_64-unknown-linux-gnu`). Validate behaviour before compiling to Wasm.

2. **Inspect effect payloads.** The functions return JSON-encoded effects. Use the broker's development mode to capture and print the raw JSON returned by the Wasm module before effects are executed.

3. **Check error codes.** If a function returns an unexpected error, match the `code` field against `contracts.md` to confirm the exact code path that was triggered.

4. **Binary inspection.** If a function is not callable, confirm it appears in the binary exports:
   ```bash
   strings target/wasm32-wasip1/release/mail_client.wasm | grep "function_name"
   ```

### Verifying plugin.json Synchronization

After any change to function signatures or new function additions, manually compare:

- Every export in `executionInterface.exports` has a matching `#[no_mangle] pub extern "C"` function in `src/lib.rs`.
- Every parameter in `inputSchema.properties` corresponds to a real parameter in the Rust function signature.
- Every error code in `outputSchema` matches an arm in the `MailClientError` enum.
- The `permissionLevel` in `semanticInterface.tools` matches the `check_permission(perm, Permission::X)` call in the implementation.

There is no automated schema-sync tool. This verification is a manual step in the development checklist.
