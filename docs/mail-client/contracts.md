# Mail Client Contracts

Stage: 2 - Architect  
App: `apps/first-party/mail-client`  
Date: 2026-05-08

## 1. ABI And Envelope Contract

All exports use the same Wasm ABI:

```rust
#[no_mangle]
pub extern "C" fn export_name(input_ptr: *const u8, input_len: usize) -> *mut u8
```

Input is a UTF-8 JSON object. Output is a null-terminated UTF-8 JSON string released by `free_string(ptr)`. Function names below are the exact Wasm export names and the exact `plugin.json` tool names unless marked as a manifest-only/admin policy item.

Every input includes:

```json
{
  "context": {
    "user_id": "usr_123",
    "permission": "read|draft|send|organize|accounts|admin|none",
    "available_effects": ["MailStoreRead"],
    "agent_id": "optional_agent_id",
    "request_id": "optional_idempotency_seed"
  }
}
```

Every successful response uses:

```json
{
  "ok": {
    "operation": "export_name",
    "status": "accepted",
    "data": {},
    "host_effects": [],
    "warnings": []
  }
}
```

Every error response uses:

```json
{
  "err": {
    "code": "InvalidInput",
    "message": "Human-readable recovery guidance",
    "details": {}
  }
}
```

## 2. Shared Data Models

### HostEffect

```json
{
  "effect": "MailStoreRead",
  "intent": "Read a mailbox page snapshot",
  "payload": {},
  "idempotency_key": "read_emails:usr_123:req_456"
}
```

### MailAccountSummary

Fields: `account_id`, `account_label`, `display_name`, `email_address`, `provider`, `enabled`, `is_default`, `status`, `connection_state`, `last_sync_at`, `unread_count`, `color`, `capabilities`.

Secrets are represented only as `credential_state`, `incoming_secret_ref`, `outgoing_secret_ref`, or `oauth_connection_ref`. Raw secret values are forbidden.

### UserMailAccountSetup

Fields:

- `account_id` optional for create, required for update.
- `revision` optional optimistic concurrency token.
- `account_label`, `display_name`, `email_address`, optional `reply_to`.
- `provider`: `manual_imap_smtp`, `google_oauth`, `microsoft_oauth`, or host provider id.
- `auth_method`: `password`, `app_password`, `oauth2`, `host_connector`.
- `incoming`: `{ "protocol": "imap", "host", "port", "security", "username", "auth_method", "secret_input_ref", "secret_ref", "path_prefix" }`.
- `outgoing`: `{ "protocol": "smtp", "host", "port", "security", "username", "auth_method", "use_incoming_secret", "secret_input_ref", "secret_ref" }`.
- `oauth`: `{ "provider", "connection_ref", "redirect_state_ref", "scopes" }`.
- `sync`: `{ "interval_minutes", "initial_range", "download_attachments", "offline_cache" }`.
- `folder_mapping`: `{ "inbox", "sent", "drafts", "trash", "archive", "junk" }`.
- `desired_status`: `active`, `disabled`, `incomplete`, or `receive_only`.

### ConnectionTestPlan And Result

`ConnectionTestPlan.data` includes `account_id`, `test_id`, `steps`, and `requires_host_execution: true`.

Step names: `imap_auth`, `imap_mailbox_discovery`, `smtp_auth`, `smtp_send_capability`, `folder_mapping`.

Host returns sanitized `ConnectionTestResult` with per-step `state`: `not_tested`, `testing`, `passed`, `warning`, `failed`, plus `code`, `message`, and `field_paths`. No credentials or tokens are returned.

### UserMailPreferences

Fields:

- `reading`: `preview_pane` (`right`, `bottom`, `off`), `thread_view`, `focused_inbox`, `page_size`, `list_density`, optional `remote_content`.
- `compose`: `undo_send_delay_seconds` (`0`, `5`, `10`, `30`), `default_format`, `default_sender_strategy`, `reply_quote_mode`, `forward_attachments_default`.
- `notifications`: `enabled`, `per_account`, optional `quiet_hours`.
- `sync_defaults`: `interval_minutes`, `initial_range`, `download_attachments`, `offline_cache`.

### MailIdentity And Signature

`MailIdentity`: `identity_id`, `account_id`, `kind`, `display_name`, `email_address`, optional `reply_to`, `signature_id`, `enabled`, `is_default_for_account`, `is_global_default`, `verification_state`.

`MailSignature`: `signature_id`, `account_id`, optional `identity_id`, `label`, `body_html`, `body_text`, `enabled`, `use_for_new`, `use_for_replies`, `use_for_forwards`.

### Mail Message, Draft, Rule, And Folder

Existing models remain in force:

- `Mailbox`: `mailbox_id`, optional `account_id`, `name`, `kind`, `unread_count`, `total_count`, optional `parent_id`, `favorite`, optional `sync_state`.
- `EmailAddress`: `email`, optional `display_name`.
- `EmailMessage`: `email_id`, `account_id`, `mailbox_id`, optional `thread_id`, `from`, `to`, `cc`, `bcc`, `subject`, `preview`, `body_html`, `body_text`, timestamps, `is_read`, `is_flagged`, `has_attachments`, `importance`, categories, attachments.
- `DraftDocument`: optional `draft_id`, `account_id`, `to`, `cc`, `bcc`, `subject`, body, attachment references, optional `signature`, optional `importance`, read receipt flag, optional `scheduled_send_at`.
- `MailRule`: `rule_id`, `name`, `enabled`, `stop_processing`, `priority`, `conditions`, `actions`.

## 3. Common Validation And Errors

Common errors: `InvalidInput`, `PermissionDenied`, `EffectUnavailable`, `PolicyDenied`, `AccountNotFound`, `AccountIncomplete`, `CredentialRequired`, `ConnectionTestRequired`, `ConnectionTestFailed`, `IdentityNotFound`, `MailboxNotFound`, `EmailNotFound`, `ThreadNotFound`, `DraftNotFound`, `RuleNotFound`, `ScheduledJobNotFound`, `Conflict`, `TooManyRecipients`, `HostRejected`, `InternalError`.

Validation constants:

- Message id batches: 1 to 250 ids.
- Page size: default 50, max 100.
- Recipients: max 100.
- Subject: max 998 characters.
- Body: combined HTML/text max 512,000 bytes.
- Folder name: max 128 characters.
- Rule name: max 120 characters.
- Undo send: `0`, `5`, `10`, or `30` seconds.
- IMAP defaults: port 993, `ssl_tls`.
- SMTP defaults: port 587, `starttls`; port 465 with `ssl_tls` allowed.
- `security: none` is invalid unless admin policy explicitly allows it.

## 4. Account Setup And Preference Exports

These exports are required to implement the UX gap. Permission should be represented by a user-scoped capability such as `mail:accounts`; the exact permission string may be `accounts` in Wasm context or mapped by the broker.

### `get_provider_capabilities`

Purpose: Read host/admin policy for manual IMAP/SMTP, OAuth providers, security modes, test capabilities, sync bounds, max recipients, and attachment limits.

Input: `context`, optional `provider_ids: string[]`.

Return data: `{ "providers": [], "manual_imap_smtp": {}, "policy": {}, "oauth_placeholders": [] }`.

Host effects: `ProviderCapabilityRead`, `MailStoreRead`.

Errors: `PermissionDenied`, `EffectUnavailable`, `PolicyDenied`, `InvalidInput`.

### `validate_account_setup`

Purpose: Validate a create/update account setup draft before testing or saving.

Input: `context`, `account: UserMailAccountSetup`, optional `admin_policy_snapshot`, optional `existing_account_snapshot`.

Return data: `{ "valid": boolean, "normalized_account": UserMailAccountSetup, "field_errors": [], "warnings": [], "next_required_action": "enter_secret|connect_oauth|test_connection|save_disabled|complete" }`.

Host effects: none when policy snapshot is supplied; otherwise `ProviderCapabilityRead`.

Errors: `InvalidInput`, `PolicyDenied`, `CredentialRequired`, `Conflict`, `EffectUnavailable`.

### `plan_connection_test`

Purpose: Plan host-owned incoming/outgoing/folder test steps.

Input: `context`, `account: UserMailAccountSetup`, `test_scope: "all|incoming|outgoing|folder_mapping"`, optional `admin_policy_snapshot`.

Return data: `ConnectionTestPlan`.

Host effects: `ProviderCapabilityRead`, `AccountCredentialRead`, `ImapSync`, `ImapFetch`, `SmtpSend`.

Errors: `InvalidInput`, `PolicyDenied`, `CredentialRequired`, `EffectUnavailable`, `AccountIncomplete`.

### `complete_account_setup`

Purpose: Save or update a user account after validation and optional connection tests.

Input: `context`, `account: UserMailAccountSetup`, optional `connection_test_result`, optional `make_default: boolean`, optional `save_mode: "active|receive_only|disabled|incomplete"`.

Return data: `{ "account": MailAccountSummary, "status", "requires_reconnect": boolean, "requires_sync": boolean }`.

Host effects: `AccountCredentialWrite`, `StorePlatformSecret`, `MailStoreWrite`, optional `ImapSync`.

Errors: `InvalidInput`, `PolicyDenied`, `CredentialRequired`, `ConnectionTestRequired`, `ConnectionTestFailed`, `Conflict`, `EffectUnavailable`.

### `begin_oauth_account_setup`

Purpose: Plan a host-owned OAuth authorization start for a supported provider.

Input: `context`, `provider`, `email_address`, optional `account_label`, optional `requested_scopes`, optional `redirect_route`.

Return data: `{ "provider", "authorization_ref", "status": "host_action_required", "next_route" }`.

Host effects: `ProviderCapabilityRead`, `OAuthConnect`.

Errors: `InvalidInput`, `PolicyDenied`, `EffectUnavailable`.

### `complete_oauth_account_setup`

Purpose: Convert a host-completed OAuth connection into a user mail account setup plan.

Input: `context`, `provider`, `authorization_ref`, `oauth_connection_ref`, optional `account: UserMailAccountSetup`, optional `connection_test_result`.

Return data: `{ "account": MailAccountSummary, "identity": MailIdentity, "requires_sync": boolean }`.

Host effects: `OAuthConnect`, `AccountCredentialWrite`, `MailStoreWrite`, optional `ImapSync`.

Errors: `InvalidInput`, `PolicyDenied`, `CredentialRequired`, `ConnectionTestRequired`, `ConnectionTestFailed`, `EffectUnavailable`.

### `disconnect_oauth_account`

Purpose: Plan OAuth disconnect without removing local account metadata unless requested.

Input: `context`, `account_id`, optional `oauth_connection_ref`, `disable_account: boolean`.

Return data: `{ "account_id", "credential_state": "revoked", "disabled": boolean }`.

Host effects: `OAuthDisconnect`, `AccountCredentialWrite`, `MailStoreWrite`.

Errors: `AccountNotFound`, `PermissionDenied`, `EffectUnavailable`, `PolicyDenied`.

### `remove_account`

Purpose: Remove or disable a user account and plan cleanup of local cache and scheduled sends.

Input: `context`, `account_id`, `remove_local_cache: boolean`, `cancel_scheduled_sends: boolean`, optional `delete_drafts: boolean`, optional `revision`.

Return data: `{ "account_id", "status": "removed_pending_cleanup|disabled", "cleanup_planned": [] }`.

Host effects: `AccountCredentialWrite`, `MailStoreWrite`, `MailStoreDelete`, `CancelJob`, optional `OAuthDisconnect`.

Errors: `AccountNotFound`, `Conflict`, `EffectUnavailable`, `PolicyDenied`.

### `list_identities`

Purpose: Read sender identities, aliases, and signatures for account settings and compose From selector.

Input: `context`, optional `account_id`, optional `snapshot`.

Return data: `{ "identities": MailIdentity[], "signatures": MailSignature[], "needs_host_refresh": boolean }`.

Host effects: `AccountCredentialRead`, `MailStoreRead`.

Errors: `PermissionDenied`, `AccountNotFound`, `EffectUnavailable`.

### `save_identity`

Purpose: Create or update an alias/send-as identity.

Input: `context`, `identity: MailIdentity`, optional `revision`.

Return data: `{ "identity": MailIdentity, "verification_required": boolean }`.

Host effects: `AccountCredentialWrite`, `MailStoreWrite`.

Errors: `InvalidInput`, `AccountNotFound`, `PolicyDenied`, `Conflict`, `IdentityNotFound`, `EffectUnavailable`.

### `delete_identity`

Purpose: Disable or remove an alias/send-as identity.

Input: `context`, `identity_id`, optional `account_id`, optional `force: boolean`.

Return data: `{ "identity_id", "removed": boolean, "default_reassigned_to": string|null }`.

Host effects: `AccountCredentialWrite`, `MailStoreWrite`.

Errors: `IdentityNotFound`, `PolicyDenied`, `Conflict`, `InvalidInput`, `EffectUnavailable`.

### `save_signature`

Purpose: Create or update a signature.

Input: `context`, `signature: MailSignature`, optional `revision`.

Return data: `{ "signature": MailSignature }`.

Host effects: `MailStoreWrite`.

Errors: `InvalidInput`, `AccountNotFound`, `IdentityNotFound`, `PolicyDenied`, `Conflict`, `EffectUnavailable`.

### `delete_signature`

Purpose: Remove a signature and clear identity defaults that reference it.

Input: `context`, `signature_id`, optional `account_id`, optional `force: boolean`.

Return data: `{ "signature_id", "removed": boolean, "identity_updates": [] }`.

Host effects: `MailStoreWrite`, `MailStoreDelete`.

Errors: `IdentityNotFound`, `InvalidInput`, `Conflict`, `EffectUnavailable`.

### `get_user_preferences`

Purpose: Read user mail preferences, optionally with defaults.

Input: `context`, optional `snapshot`, optional `include_defaults: boolean`.

Return data: `{ "preferences": UserMailPreferences, "defaults_applied": boolean, "needs_host_refresh": boolean }`.

Host effects: `MailStoreRead`.

Errors: `PermissionDenied`, `EffectUnavailable`.

### `save_user_preferences`

Purpose: Validate and save user mail preferences.

Input: `context`, `preferences: UserMailPreferences`, optional `revision`.

Return data: `{ "preferences": UserMailPreferences, "changed_groups": [] }`.

Host effects: `MailStoreWrite`.

Errors: `InvalidInput`, `PolicyDenied`, `Conflict`, `EffectUnavailable`.

### `inspect_legacy_settings`

Purpose: Classify old `ui_schemas.main`, `settings.accounts`, and `settings.general` data for safe migration.

Input: `context`, `legacy_settings`.

Return data: `{ "accounts_detected": number, "preferences_detected": [], "credential_risks": [], "migration_preview": {}, "requires_user_reconnect": [] }`.

Host effects: none if legacy data is supplied; otherwise `MailStoreRead`.

Errors: `InvalidInput`, `PermissionDenied`, `EffectUnavailable`.

### `migrate_legacy_settings`

Purpose: Write user-scoped account/preference records from old settings and remove obsolete settings keys.

Input: `context`, `legacy_settings`, optional `migration_options`, optional `dry_run: boolean`.

Return data: `{ "migrated_accounts": [], "migrated_preferences": {}, "migration_map": {}, "requires_reconnect": [], "obsolete_keys_removed": [] }`.

Host effects: `MailStoreRead`, `MailStoreWrite`, `MailStoreDelete`, optional `AccountCredentialWrite`.

Errors: `InvalidInput`, `Conflict`, `PermissionDenied`, `EffectUnavailable`, `PolicyDenied`.

## 5. Existing Mail Workflow Exports

These existing exports should be preserved. Where implementation details are currently broad in `plugin.json`, schemas must be tightened to the models above.

| Export | Permission | Parameters | Return Data | Host Effects | Errors |
| --- | --- | --- | --- | --- | --- |
| `list_accounts` | `mail:read` | `context`, optional `snapshot: MailAccountSummary[]`, `refresh: boolean` | `QueryCollection<MailAccountSummary>` | `AccountCredentialRead` | `PermissionDenied`, `EffectUnavailable`, `InvalidInput` |
| `get_account_status` | `mail:read` | `context`, `account_id`, optional `snapshot: MailAccountStatus`, `refresh` | `MailAccountStatus` with `needs_host_refresh` | `AccountCredentialRead`, `ImapSync` | `AccountNotFound`, `PermissionDenied`, `EffectUnavailable` |
| `list_mailboxes` | `mail:read` | `context`, optional `account_id`, optional `snapshot: Mailbox[]`, `include_favorites`, `refresh` | `QueryCollection<Mailbox>` | `AccountCredentialRead`, `ImapFetch` | `AccountNotFound`, `PermissionDenied`, `EffectUnavailable` |
| `read_emails` | `mail:read` | `context`, optional `account_id`, `mailbox_id`, optional `pagination`, `sort`, `filters`, `thread_view`, `focused_inbox`, `snapshot`, `refresh` | `ReadEmailsData` | `AccountCredentialRead`, `ImapFetch` | `MailboxNotFound`, `InvalidInput`, `PermissionDenied`, `EffectUnavailable` |
| `search_emails` | `mail:read` | `context`, `query`, optional `pagination`, `filters`, `snapshot`, `refresh` | `SearchEmailsData` | `AccountCredentialRead`, `ImapFetch` | `InvalidInput`, `PermissionDenied`, `EffectUnavailable` |
| `get_email` | `mail:read` | `context`, `email_id`, optional `account_id`, optional `snapshot`, `mark_read`, `refresh` | `EmailMessage` plus mark-read plan when requested | `AccountCredentialRead`, `ImapFetch`, optional `MailStoreWrite` | `EmailNotFound`, `PermissionDenied`, `EffectUnavailable` |
| `get_thread` | `mail:read` | `context`, `thread_id`, optional `account_id`, optional `snapshot`, `refresh` | `ThreadData` | `AccountCredentialRead`, `ImapFetch` | `ThreadNotFound`, `PermissionDenied`, `EffectUnavailable` |
| `mark_read` | `mail:read` | `context`, `email_ids`, `read`, optional `account_id`, `mailbox_id` | `MutationAck` | `MailStoreWrite` | `InvalidInput`, `EmailNotFound`, `PermissionDenied`, `EffectUnavailable` |
| `flag_email` | `mail:organize` | `context`, `email_ids`, `flagged`, optional `account_id` | `MutationAck` | `MailStoreWrite` | `InvalidInput`, `EmailNotFound`, `PermissionDenied`, `EffectUnavailable` |
| `draft_email` | `mail:draft` | `context`, `draft: DraftDocument`, `autocomplete_recipients` | `DraftMutationData` | `MailStoreWrite`, optional `ContactLookup` | `InvalidInput`, `TooManyRecipients`, `AccountIncomplete`, `IdentityNotFound`, `PermissionDenied`, `EffectUnavailable` |
| `update_draft` | `mail:draft` | `context`, `draft: DraftDocument` | `DraftMutationData` | `MailStoreWrite` | `DraftNotFound`, `Conflict`, `InvalidInput`, `PermissionDenied`, `EffectUnavailable` |
| `discard_draft` | `mail:draft` | `context`, `draft_id`, optional `account_id` | `MutationAck` | `MailStoreDelete` | `DraftNotFound`, `PermissionDenied`, `EffectUnavailable` |
| `send_email` | `mail:send` | `context`, `draft_id`, `account_id`, optional `undo_window_seconds`, `notify` | `SendPlan` | `MailStoreWrite`, `SmtpSend`, optional `NotifyUser` | `DraftNotFound`, `AccountIncomplete`, `CredentialRequired`, `InvalidInput`, `PermissionDenied`, `EffectUnavailable` |
| `reply_email` | `mail:send` | `context`, `email_id`, `account_id`, optional `body_html`, `body_text`, `include_original`, `reply_all`, `send_now`, `undo_window_seconds` | `ReplyPlan` | `MailStoreRead`, `MailStoreWrite`, `SmtpSend` | `EmailNotFound`, `AccountIncomplete`, `InvalidInput`, `PermissionDenied`, `EffectUnavailable` |
| `forward_email` | `mail:send` | `context`, `email_id`, `account_id`, `to`, optional `cc`, `bcc`, `note_html`, `note_text`, `send_now`, `undo_window_seconds` | `ForwardPlan` | `MailStoreRead`, `MailStoreWrite`, `SmtpSend` | `EmailNotFound`, `TooManyRecipients`, `InvalidInput`, `PermissionDenied`, `EffectUnavailable` |
| `move_email` | `mail:organize` | `context`, `email_ids`, `destination_mailbox_id`, optional `account_id` | `MutationAck` | `MailStoreWrite` | `MailboxNotFound`, `EmailNotFound`, `InvalidInput`, `PermissionDenied`, `EffectUnavailable` |
| `delete_email` | `mail:organize` | `context`, `email_ids`, optional `account_id`, `permanent` | `MutationAck` | `MailStoreWrite`, `MailStoreDelete` | `PolicyDenied`, `EmailNotFound`, `InvalidInput`, `PermissionDenied`, `EffectUnavailable` |
| `archive_email` | `mail:organize` | `context`, `email_ids`, optional `account_id` | `MutationAck` | `MailStoreWrite` | `EmailNotFound`, `InvalidInput`, `PermissionDenied`, `EffectUnavailable` |
| `create_folder` | `mail:organize` | `context`, `account_id`, `name`, optional `parent_mailbox_id`, `favorite` | `FolderMutationData` | `MailStoreWrite` | `AccountNotFound`, `InvalidInput`, `Conflict`, `PermissionDenied`, `EffectUnavailable` |
| `rename_folder` | `mail:organize` | `context`, `account_id`, `mailbox_id`, `new_name` | `FolderMutationData` | `MailStoreWrite` | `MailboxNotFound`, `PolicyDenied`, `InvalidInput`, `Conflict`, `PermissionDenied`, `EffectUnavailable` |
| `delete_folder` | `mail:organize` | `context`, `account_id`, `mailbox_id`, `force` | `MutationAck` | `MailStoreDelete` | `MailboxNotFound`, `PolicyDenied`, `Conflict`, `PermissionDenied`, `EffectUnavailable` |
| `get_unread_count` | `mail:read` | `context`, optional `account_id`, optional `snapshot`, `refresh` | `UnreadCountData` | `AccountCredentialRead`, `ImapFetch` | `AccountNotFound`, `PermissionDenied`, `EffectUnavailable` |
| `create_rule` | `mail:organize` | `context`, `account_id`, `rule: MailRule` | `MailRule` or `MutationAck` | `MailStoreWrite` | `AccountNotFound`, `PolicyDenied`, `InvalidInput`, `Conflict`, `PermissionDenied`, `EffectUnavailable` |
| `list_rules` | `mail:organize` | `context`, `account_id`, optional `snapshot`, `refresh` | `QueryCollection<MailRule>` | `MailStoreRead` | `AccountNotFound`, `PermissionDenied`, `EffectUnavailable` |
| `delete_rule` | `mail:organize` | `context`, `account_id`, `rule_id` | `MutationAck` | `MailStoreDelete` | `RuleNotFound`, `PermissionDenied`, `EffectUnavailable` |
| `schedule_send` | `mail:send` | `context`, `draft_id`, `account_id`, `scheduled_for`, optional `undo_window_seconds` | `SchedulePlan` | `MailStoreWrite`, `ScheduleJob` | `DraftNotFound`, `InvalidInput`, `PolicyDenied`, `PermissionDenied`, `EffectUnavailable` |
| `cancel_scheduled_send` | `mail:send` | `context`, `job_id`, `account_id`, optional `draft_id` | `MutationAck` | `MailStoreWrite`, `CancelJob` | `ScheduledJobNotFound`, `PermissionDenied`, `EffectUnavailable` |

## 6. Manifest-Only Admin Policy Contract

Admin policy is not a user credential export. It belongs in `synapp.app.json` or a host admin policy registry, not in account setup tools.

Fields: `allow_manual_imap_smtp`, `allowed_oauth_providers`, `allow_insecure_transport`, `min_sync_interval_minutes`, `max_recipients`, `allow_agent_send`, `require_review_for_agent_send`, `allow_permanent_delete`, `allow_rule_creation`, `attachment_max_bytes`, `connection_test_modes`.

No per-user `account_id`, `email_address`, `username`, secret reference, alias, signature, or default sender is valid in admin policy.
