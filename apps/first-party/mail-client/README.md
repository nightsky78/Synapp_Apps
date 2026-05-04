# Mail Client

The Mail Client is the first-party Synapp email application. It is packaged as a `wasm32-wasip1` app and follows the Synapp Dual-Interface model:

- The execution interface is the WebAssembly module in `target/wasm32-wasip1/release/mail_client.wasm`.
- The semantic interface is described in `plugin.json` and `synapp.app.json`.
- All protocol, credential, scheduling, notification, and storage operations are delegated to host effects.
- The same contract is usable from the human UI and from authenticated AI agents.

## What The App Provides

The app covers the full operational surface expected from a production mail client:

- Multi-account mailbox access through host-managed account settings
- Mailbox and folder navigation
- Paginated message reading and threaded conversation access
- Full-text and filtered search
- Draft creation, editing, and discard flows
- Send, reply, forward, and scheduled send planning
- Message organization: mark read, flag, move, archive, delete
- Folder management
- Rule management
- Unread-count queries and account health checks

## Architecture

This app is intentionally sandboxed.

- No direct IMAP, SMTP, OAuth, filesystem, or database access occurs inside Wasm.
- Account credentials live in the host settings panel exposed by the `mail-client.accounts` settings contribution.
- Each export validates request shape, permission level, and host-effect availability.
- Each successful export returns a normalized JSON result with one or more `host_effects` that the Synapp host must execute.

The Rust implementation is in `src/lib.rs`. All exported functions accept a single UTF-8 JSON request payload and return a UTF-8 JSON response. Returned strings are null-terminated and must be released by the host through the exported `free_string` deallocator.

## Permissions

The app expects the host to inject a `context.permission` value into each request.

Supported permission levels:

- `none`
- `read`
- `draft`
- `send`
- `organize`

Capability mapping:

- `mail:read`: account listing, mailbox listing, reading, search, threads, unread counts, mark read/unread
- `mail:draft`: draft creation, draft update, draft discard
- `mail:send`: send, reply, forward, schedule send, cancel scheduled send
- `mail:organize`: flagging, moving, archiving, deleting, folder management, rule management

## Export Surface

The Wasm module currently exports these operations:

- `list_accounts`
- `get_account_status`
- `list_mailboxes`
- `read_emails`
- `search_emails`
- `get_email`
- `get_thread`
- `mark_read`
- `flag_email`
- `draft_email`
- `update_draft`
- `discard_draft`
- `send_email`
- `reply_email`
- `forward_email`
- `move_email`
- `delete_email`
- `archive_email`
- `create_folder`
- `rename_folder`
- `delete_folder`
- `get_unread_count`
- `create_rule`
- `list_rules`
- `delete_rule`
- `schedule_send`
- `cancel_scheduled_send`
- `free_string`

See `plugin.json` for the execution schemas and semantic tool definitions.

## Settings Panel

Mail accounts are configured only through the host-rendered settings panel declared in `synapp.app.json`.

The settings schema includes:

- Account identity and enablement
- Password-based IMAP/SMTP configuration
- Google and Microsoft OAuth2 account modes
- Sync interval and default-account settings
- Per-account signatures
- General preferences such as preview pane position, paging size, conversation grouping, focused inbox, notifications, and undo-send delay

The app itself does not store secrets.

## Required Host Effects

The host must provide these effects for the app to work end-to-end:

- `AccountCredentialRead`
- `AccountCredentialWrite`
- `ImapFetch`
- `ImapSync`
- `SmtpSend`
- `ContactLookup`
- `NotifyUser`
- `ScheduleJob`
- `CancelJob`
- `StorePlatformSecret`
- `MailStoreRead`
- `MailStoreWrite`
- `MailStoreDelete`

If any required effect is unavailable, the app returns `EffectUnavailable` and reports the missing effect names in `err.details.missing_effects`.

See `../../../docs/reports/mail-client-v2-host-blockers.md` for the current blocker list.

## Build

Build the app locally:

```bash
cd apps/first-party/mail-client
cargo build --release --target wasm32-wasip1
```

Expected output:

- `target/wasm32-wasip1/release/mail_client.wasm`

## Package

Generate a distributable archive from the repository root:

```bash
scripts/package_app.sh apps/first-party/mail-client
```

Expected output:

- `packages/mail-client-2.1.0.tar.gz`

The package script currently bundles:

- `synapp.app.json`
- `plugin.json`
- `README.md`
- `target/wasm32-wasip1/release/mail_client.wasm`

## Install

### Catalog-based installation

1. Build and package the app.
2. Update `catalog/apps/mail-client.v1.json` with the new version, package URL, package digest, manifest digest, and source commit.
3. Update `catalog/index.v1.json` so `latest_version` points to the new release.
4. Run `scripts/validate_catalog.sh`.
5. Publish the updated catalog metadata and the package archive to the repository location referenced by `package_url`.
6. Let the Synapp host fetch the catalog entry, download the archive, verify the hash and signature metadata, and install the package.

### Direct package installation

If your host supports direct local package import:

1. Generate `packages/mail-client-2.1.0.tar.gz`.
2. Import that archive into the host installer.
3. Verify the host unpacks `synapp.app.json` and the Wasm entrypoint correctly.
4. Open the host settings panel and configure at least one mail account in `Mail Accounts`.
5. Ensure the host exposes the required effects listed above before invoking tools.

## Validation

Recommended validation steps after changes:

```bash
cd apps/first-party/mail-client
cargo build --release --target wasm32-wasip1
cd /home/johannes/projects/Synapp_apps
scripts/package_app.sh apps/first-party/mail-client
scripts/validate_catalog.sh
```

## File Layout

- `src/lib.rs`: Wasm implementation and export surface
- `synapp.app.json`: package manifest consumed by the Synapp host
- `plugin.json`: execution and semantic interface contract
- `Cargo.toml`: Rust crate configuration
- `README.md`: app-local build, package, and install guide

## Known External Dependencies

The local repository now contains the code, manifest, package recipe, and catalog metadata required for installation. Actual runtime usability still depends on the Synapp host implementing the required mail effects and publishing the generated package to the URL recorded in the catalog.
