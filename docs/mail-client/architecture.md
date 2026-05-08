# Mail Client Architecture

Stage: 2 - Architect  
App: `apps/first-party/mail-client`  
Source UX contract: `docs/mail-client/ux-spec.md`  
Date: 2026-05-08

## 1. Purpose, Boundaries, And Assumptions

The first-party Mail Client must become a real inbox-first Synapp mail workspace. The default route opens usable mail, not a schema-rendered configuration form. Human users get a full UI for account setup, reading, search, triage, compose, send, scheduling, folders, rules, identities, signatures, and preferences. AI agents get the same capability surface through Wasm exports and `plugin.json` tools.

The Wasm module remains a sandboxed planner and validator. It validates inputs, normalizes request models, checks permissions and required host effects, and returns host-effect plans. It never opens IMAP, SMTP, OAuth, TCP, database, attachment, notification, or scheduling connections directly. It contains no prompts, LLM SDKs, agent routing, or AI decision logic.

Core assumptions:

- Runtime target is `wasm32-wasip1`; the current Cargo dependencies are `serde` and `serde_json` with no native networking, async runtime, C bindings, threads, or OS-specific APIs.
- Host owns protocol I/O, OAuth redirect flows, credentials, persistence, attachments, contact lookup, scheduling, notifications, cache, audit, and account-provider capability discovery.
- User identity, permission grants, available host effects, request idempotency, and optional agent identity are injected by the Synapp Broker in `context`.
- Account setup is user-scoped. Admin settings govern policy and availability only; they must not contain another user's IMAP/SMTP hosts, usernames, credentials, signatures, aliases, or per-user preferences.
- Existing mail workflow exports are preserved where possible. New exports are additive planner/validator contracts for account setup, connection tests, preferences, identities, account removal, OAuth placeholders, and migration.

## 2. UI And Manifest Architecture

### Main Route

`synapp.app.json` must make the real UI entrypoint canonical for the main app experience:

- Keep `ui.entrypoint`: `ui/dist/index.html`.
- Keep the navigation contribution target as `app://mail-client/inbox` or equivalent inbox route.
- Remove or deprecate `ui_schemas.main` as a host-rendered form. The main schema with `mailbox_id`, `search_query`, `thread_view`, `focused_inbox`, `preview_pane`, and `emails_per_page` is the source of the unusable page and must not be the main route contract.
- If the host requires a main UI declaration, replace it with an app-shell contribution that references the entrypoint and default route, for example `{ "type": "app_shell", "entrypoint": "ui/dist/index.html", "default_route": "/inbox" }`, subject to catalog schema support.
- Deep links required by UX: `/inbox`, `/mailbox/:accountId/:mailboxId`, `/message/:accountId/:messageId`, `/search`, `/compose`, `/draft/:accountId/:draftId`, `/settings/accounts`, `/settings/identities`, `/settings/preferences`, `/settings/rules`, `/settings/agents`.

### User Settings Versus Admin Policy

The old settings contribution must be split by ownership:

| Area | Owner | Manifest Location | Contains | Must Not Contain |
| --- | --- | --- | --- | --- |
| Mail workspace | UI app | `ui.entrypoint` plus navigation route | Inbox, folders, message list, reading pane, compose, search, rules links | Host-rendered settings fields as the main page |
| User account setup | Signed-in user | In-app routes and/or user settings contribution | IMAP/SMTP config, OAuth connection placeholders, account health, secret handles, sync preferences, aliases, signatures, default sender | Admin/global settings or raw secret values |
| User mail preferences | Signed-in user | In-app routes and/or user settings contribution | Reading, compose, notification, sync, and display preferences | Vague `General Settings` grouping |
| Admin policy | Organization admin | Admin/policy contribution only | Allowed providers, manual IMAP/SMTP enablement, insecure transport policy, max recipients, attachment policy, agent send/delete/rule controls, host effect availability | Per-user credentials, account labels, usernames, aliases, signatures, default sender |

Recommended `synapp.app.json` changes for implementation:

- Rename the existing settings contribution from `Mail Accounts` to a user-scoped surface such as `Mail Setup` only if the host settings shell is user-specific. Otherwise move account setup entirely into `/settings/accounts` in the app UI.
- Replace `general` with explicit user preference groups: `reading`, `compose`, `notifications`, and `sync_defaults`.
- Add an admin policy contribution only for policy and capability flags. Example fields: `allow_manual_imap_smtp`, `allowed_oauth_providers`, `allow_insecure_transport`, `min_sync_interval_minutes`, `max_recipients`, `allow_agent_send`, `require_review_for_agent_send`, `allow_permanent_delete`, `allow_rule_creation`, `attachment_max_bytes`.
- Ensure stored account data uses secret references and OAuth connection ids, never cleartext passwords, app passwords, access tokens, or refresh tokens.

## 3. Runtime Components

| Component | Responsibility |
| --- | --- |
| React UI | Inbox-first workspace, account setup, preferences, optimistic states, recovery flows, compose buffer, accessibility, and all human workflows. |
| Wasm planner | Pure validation, normalization, permission checks, host-effect availability checks, idempotency keys, operation plans, and structured errors. |
| Synapp Host Broker | Tool routing, user/agent context injection, permission enforcement, audit metadata, and host-effect execution. |
| Mail host services | IMAP sync/fetch, SMTP send/test, OAuth connect/disconnect, mail store, credentials, folders, rules, drafts, contacts, notifications, scheduled jobs, attachment store. |
| Admin policy service | Organization policy and allowed capability/provider configuration. |

The current export ABI remains valid for all operations:

```rust
#[no_mangle]
pub extern "C" fn export_name(input_ptr: *const u8, input_len: usize) -> *mut u8
```

Input is UTF-8 JSON. Output is a null-terminated JSON string released via `free_string`. Every request contains `context`. Every successful planner response has `operation`, `status: "accepted"`, `data`, `host_effects`, and optional `warnings`. Every failure has `err.code`, `err.message`, and optional `err.details`.

## 4. Host Effects

Existing host effects remain in use:

- `AccountCredentialRead`: read account summaries, health, secret reference metadata, and identity metadata without exposing secret values.
- `AccountCredentialWrite`: create/update account config, secret references, identity metadata, OAuth connection references, and credential lifecycle state.
- `ImapFetch`: fetch folders, messages, threads, unread counts, and search snapshots.
- `ImapSync`: sync or account-health refresh.
- `SmtpSend`: send, SMTP test, and send capability checks through host mail services.
- `ContactLookup`: recipient autocomplete and contact metadata.
- `NotifyUser`: send, schedule, sync, and account-health notifications.
- `ScheduleJob`: scheduled send dispatch owned by host.
- `CancelJob`: scheduled send cancellation owned by host.
- `StorePlatformSecret`: secure password/app-password write or replacement flow.
- `MailStoreRead`: read host-stored mail, drafts, rules, preferences, identities, scheduled metadata, and migration data.
- `MailStoreWrite`: write drafts, mutations, preferences, identities, account metadata, rules, folder changes, and migration results.
- `MailStoreDelete`: delete drafts, local cache, rules, folders, removed-account data, and obsolete settings keys.
- New required effect: `OAuthConnect` for host-owned provider authorization start/complete planning.
- New required effect: `OAuthDisconnect` for host-owned provider disconnect and token revocation planning.
- New required effect: `ProviderCapabilityRead` for OAuth/manual provider support, transport security policy, folder mapping support, and connection test capabilities.

No host effect payload may contain raw passwords, app passwords, OAuth access tokens, OAuth refresh tokens, or attachment bytes. Secret entry happens through host-controlled secure fields; Wasm receives only opaque `secret_input_ref` during setup and persistent `secret_ref` after save.

## 5. Data Ownership Model

### User Account Setup Model

`UserMailAccountConfig` is user-scoped and host-persisted:

- `account_id`: stable opaque id.
- `account_label`, `display_name`, `email_address`, optional `reply_to`.
- `provider`: `manual_imap_smtp`, `google_oauth`, `microsoft_oauth`, or host-defined provider id.
- `auth_method`: `password`, `app_password`, `oauth2`, or `host_connector`.
- `incoming`: IMAP host, port, security (`ssl_tls`, `starttls`, `none` if policy allows), username, auth method, optional path prefix.
- `outgoing`: SMTP host, port, security, username, auth method, same-credential flag, optional separate credential reference.
- `credential_state`: `missing`, `pending_entry`, `stored`, `oauth_connected`, `expired`, `revoked`.
- `secret_refs`: opaque incoming/outgoing secret refs; never populated with raw values.
- `oauth_connection_ref`: opaque provider connection id where applicable.
- `folder_mapping`: inbox, sent, drafts, trash, archive, junk ids or host defaults.
- `sync`: interval, initial range, attachment behavior, offline cache preference if host supports it.
- `status`: `active`, `disabled`, `incomplete`, `receive_only`, `auth_required`, `degraded`, `removed_pending_cleanup`.
- `created_at`, `updated_at`, `last_test_at`, `last_sync_at`.

### User Preferences Model

`UserMailPreferences` is user-scoped and separate from account credentials:

- `reading`: preview pane, conversation/thread view, focused inbox, page size or list density, remote content behavior if host exposes it.
- `compose`: undo send seconds, default signature behavior, reply/forward quote defaults, default format, default sender strategy.
- `notifications`: enabled, per-account toggles, quiet hours if host supports them.
- `sync_defaults`: default sync interval, initial sync range, attachment download behavior, offline cache preference.

### Identities, Aliases, And Signatures

`MailIdentity` is user-scoped and usually account-scoped:

- `identity_id`, `account_id`, `display_name`, `email_address`, optional `reply_to`.
- `kind`: `primary`, `alias`, `send_as`, or host-defined value.
- `enabled`, `is_default_for_account`, `is_global_default`.
- `signature_id` and compose-context defaults for new, reply, and forward.
- Host/provider verification state: `allowed`, `pending_verification`, `blocked_by_provider`, `blocked_by_policy`.

`MailSignature` stores `signature_id`, account/identity scope, label, `body_html` or `body_text`, enabled state, and usage defaults. The Wasm planner validates size and required fields; the host sanitizes/render-checks rich content.

### Admin Policy Model

`MailAdminPolicy` is organization-scoped:

- Provider availability and labels.
- Manual IMAP/SMTP enablement.
- Allowed security modes; `none` is disabled by default and requires explicit policy.
- Min/max sync intervals.
- Max recipients and attachment limits.
- Agent capability policy, review requirements, audit retention, permanent delete permission, broad-rule confirmation policy.

## 6. Validation And Error Semantics

Common error codes across all exports:

- `InvalidInput`: malformed JSON, missing required field, empty id, invalid enum, invalid range, invalid route or unsupported combination.
- `PermissionDenied`: injected permission does not satisfy the operation requirement.
- `EffectUnavailable`: required host effect is not listed in `context.available_effects`.
- `PolicyDenied`: admin policy disallows the provider, security mode, send behavior, permanent delete, rule, OAuth provider, or account action.
- `AccountNotFound`: account id is missing or not owned by the current user.
- `AccountIncomplete`: setup lacks required identity, incoming configuration, credential, OAuth connection, or passed connection test.
- `CredentialRequired`: secret entry or OAuth reconnection is required before proceeding.
- `ConnectionTestRequired`: setup changed and must be retested before enabling.
- `ConnectionTestFailed`: incoming, outgoing, or folder mapping test failed; details include field-level results.
- `IdentityNotFound`: requested alias/signature/default sender is absent or disabled.
- `MailboxNotFound`, `EmailNotFound`, `ThreadNotFound`, `DraftNotFound`, `RuleNotFound`, `ScheduledJobNotFound`.
- `Conflict`: supplied revision/version is stale for draft, account, identity, preferences, rule, or folder.
- `TooManyRecipients`: recipient count exceeds policy or module cap.
- `HostRejected`: host effect executor rejected the plan after Wasm validation.
- `InternalError`: serialization or unexpected planner failure.

Validation rules:

- Email addresses must be syntactically valid enough for client-side/planner gating: contain one `@`, no spaces, non-empty local and domain parts, max 320 characters. Host may perform stricter provider validation.
- Message batches are capped at 250 ids.
- Page size defaults to 50 and caps at 100 unless the contract is changed in both Rust and `plugin.json`.
- Recipients cap at 100 unless admin policy lowers it.
- Subject length caps at 998 characters.
- Draft body combined HTML/text caps at 512,000 bytes.
- Folder names cap at 128 characters and system folders cannot be renamed or deleted.
- Rule names cap at 120 characters; destructive or broad rules must be flagged for confirmation where policy requires it.
- IMAP default port is 993 with `ssl_tls`; SMTP default port is 587 with `starttls`; SMTP `ssl_tls` port 465 is allowed. Security `none` is invalid unless admin policy explicitly allows it.
- Manual account setup cannot become `active` unless incoming IMAP authentication and mailbox discovery pass. If SMTP fails but IMAP passes, the account may be saved as `receive_only` with send disabled. If IMAP fails, active setup is blocked.

## 7. Account Setup Flow

1. UI calls `get_provider_capabilities` to render OAuth providers, manual setup availability, security modes, and policy constraints.
2. UI collects identity, incoming, outgoing, sync, and default sender data. Secret fields are host-controlled and yield `secret_input_ref`, not a password.
3. UI calls `validate_account_setup` for local/planner validation.
4. UI calls `plan_connection_test`. Wasm returns host effects for IMAP auth, mailbox discovery, SMTP auth/capability, optional non-delivery send test, and folder mapping validation.
5. Host executes effects and returns a test snapshot. UI calls `complete_account_setup` with the sanitized test result and desired status.
6. Wasm allows `active` only when required tests passed. It allows `receive_only` when incoming passed and outgoing failed or is disabled by policy. It allows `incomplete`/`disabled` save without enabling mail workflows.
7. Host persists account metadata, secret refs, OAuth refs, identity defaults, folder mapping, and sync settings.
8. Main route opens `/inbox` or `/mailbox/:accountId/inbox` after setup.

## 8. Compatibility And Migration

Migration must preserve user-owned useful data while removing the settings-form architecture.

Old data sources:

- `ui_schemas.main`: ignore for main route rendering. Its values may seed user reading preferences during migration, but it must no longer define the main page.
- `settings.accounts`: treat as user-scoped account setup data if saved by the signed-in user. Convert clear account metadata into `UserMailAccountConfig`. Any raw secret-like data must be discarded and replaced with `credential_state: "missing"` plus a reconnect prompt.
- `settings.general`: split into `reading`, `compose`, `notifications`, and `sync_defaults` user preferences. Do not keep a visible `General Settings` label.

Migration exports:

- `inspect_legacy_settings`: classify old data, identify credential risk, and produce a migration preview.
- `migrate_legacy_settings`: write user-scoped account/preference records, remove obsolete keys, and mark accounts requiring reconnect/test.

Migration safety:

- Never migrate per-user accounts into admin policy.
- Never persist raw passwords or OAuth token material from legacy settings.
- Preserve `account_id` where safe; otherwise map old ids to new opaque ids and return `migration_map`.
- Mark imported manual accounts `incomplete` or `auth_required` until the user re-enters secrets and passes connection testing.

## 9. Security Boundaries

- Wasm memory may temporarily hold JSON request strings, message bodies, draft bodies, and opaque secret refs; it must not receive raw credential values.
- Agents never receive raw credentials, OAuth tokens, or attachment bytes.
- `mail:read` gates message bodies, thread content, account summaries, folder lists, unread counts, search, and account status.
- `mail:draft` gates draft create/update/discard and compose validation.
- `mail:send` gates send, reply-send, forward-send, schedule, and cancel scheduled send.
- `mail:organize` gates flag, move, archive, delete, folders, and rules. `mark_read` currently requires `mail:read`; policy may later split it under organize if needed.
- Account setup exports require a new user-account-management capability such as `mail:accounts` or equivalent host user-settings permission. They are not admin capabilities.
- Admin policy changes require admin permission outside this app's user mail contracts.

## 10. Implementation Constraints For Wasm

- Target: `wasm32-wasip1`.
- ABI: `input_ptr`, `input_len` JSON in; allocated null-terminated JSON out; `free_string` deallocator.
- Dependencies: `serde`, `serde_json`, and `alloc`-compatible pure Rust crates only.
- No `std::net`, TCP/UDP, IMAP/SMTP libraries, OAuth clients, database clients, filesystem persistence, background threads, async runtimes, native TLS, C bindings, or LLM/AI SDKs.
- Host-effect names and JSON payloads are plain data contracts and must be mirrored in `plugin.json`.

## 11. Architecture Gates

| Gate | Status | Notes |
| --- | --- | --- |
| Main route opens real UI workspace | Pass by design | Requires implementation manifest change: remove/deprecate `ui_schemas.main` form and use `ui.entrypoint` for `/inbox`. |
| User account setup is not admin settings | Pass by design | Account setup/preference models are user-scoped; admin policy is separate and credential-free. |
| Wasm exports are planner/validator only | Pass by design | Host effects own IMAP/SMTP/OAuth/persistence/secrets/scheduling. |
| `plugin.json` synchronization explicit | Pass by design | See `plugin-schema-plan.md`; every export must map to an input/output schema. |
| `wasm32-wasip1` compatibility | Pass by design | Dependencies and constraints avoid native network/DB/AI/thread APIs. |
| Migration from old settings | Pass by design | Legacy settings are inspected, split, and migrated to user records with reconnect requirements. |
