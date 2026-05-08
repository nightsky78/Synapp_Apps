# Mail Client plugin.json Schema Plan

Stage: 2 - Architect  
App: `apps/first-party/mail-client`  
Date: 2026-05-08

## 1. Synchronization Rule

`plugin.json` is the semantic contract for the Wasm execution interface. Every export in `contracts.md` must have a matching entry in both:

- `executionInterface.exports[]` with exact `name`, `inputSchema`, and `outputSchema`.
- `semanticInterface.tools[]` with exact `name`, user-facing description, category, capability, and risk notes where supported.

Whenever a Rust request struct, response data shape, host-effect payload, permission requirement, or error enum changes, `plugin.json` must be updated in the same commit.

The current ABI remains:

```json
{
  "executionInterface": {
    "wasmTarget": "wasm32-wasip1",
    "wasmPath": "target/wasm32-wasip1/release/mail_client.wasm",
    "memoryManagement": {
      "returnedStringsAreNullTerminated": true,
      "deallocatorExport": "free_string",
      "inputEncoding": "utf-8-json"
    }
  }
}
```

## 2. Shared Definitions To Add Or Tighten

The current `plugin.json` has many `{ "type": "object" }` placeholders. Replace them with explicit definitions or `$ref`s:

- `RequestEnvelope`: require `context.user_id`, `context.permission`, and `context.available_effects`; allow `permission` enum values `none`, `read`, `draft`, `send`, `organize`, `accounts`, `admin` if the broker maps these directly.
- `ResultEnvelope`: preserve `ok.operation`, `ok.status`, `ok.data`, `ok.host_effects`, `ok.warnings`, and `err` shape.
- `HostEffect`: require `effect`, `intent`, `payload`, `idempotency_key`.
- `Error`: require `code`, `message`, optional `details`; list known error codes.
- `Warning`: require `code`, `message`.
- `MailAccountSummary`, `MailAccountStatus`, `UserMailAccountSetup`, `IncomingServerConfig`, `OutgoingServerConfig`, `OAuthAccountConfig`, `ConnectionTestResult`, `ConnectionTestStepResult`, `UserMailPreferences`, `MailIdentity`, `MailSignature`, `Mailbox`, `EmailAddress`, `Attachment`, `EmailMessage`, `DraftDocument`, `MailRule`, `RuleCondition`, `RuleAction`, `Pagination`, `SortSpec`, `SearchFilters`, `EmailPageSnapshot`, `SearchSnapshot`, `UnreadCount`.
- `ProviderCapabilities`: manual IMAP/SMTP support, OAuth providers, security modes, test modes, sync bounds, max recipients, attachment limits, and policy flags.
- `LegacySettings`: old `ui_schemas.main`, old `settings.accounts`, and old `settings.general` shapes for migration only.

Security-sensitive schema rule: no definition may contain a property named `password`, `app_password`, `access_token`, `refresh_token`, `client_secret`, or `smtp_password`. Use `secret_input_ref`, `secret_ref`, `oauth_connection_ref`, or `authorization_ref`.

## 3. synapp.app.json Schema Plan

`synapp.app.json` must be updated alongside `plugin.json` during implementation:

- Keep `ui.entrypoint: "ui/dist/index.html"`.
- Keep navigation to `app://mail-client/inbox`.
- Remove or deprecate `ui_schemas.main`; it must not render as the main route.
- Replace current account settings schema with user-scoped account setup surfaces or remove settings contribution if the React route owns setup.
- Split old `general` into `reading`, `compose`, `notifications`, and `sync_defaults` user preferences.
- Add a separate admin policy schema only if the host supports admin policy contributions. It must contain policy/capability flags only, never per-user account credentials or identity data.

Migration mapping:

| Old Location | New Location | Notes |
| --- | --- | --- |
| `ui_schemas.main.mailbox_id` | UI route state or reading preference seed | Do not use as host-rendered main page. |
| `ui_schemas.main.thread_view` | `UserMailPreferences.reading.thread_view` | User-scoped. |
| `ui_schemas.main.focused_inbox` | `UserMailPreferences.reading.focused_inbox` | User-scoped. |
| `ui_schemas.main.preview_pane` | `UserMailPreferences.reading.preview_pane` | User-scoped. |
| `ui_schemas.main.emails_per_page` | `UserMailPreferences.reading.page_size` | Cap at 100 unless contract changes. |
| `settings.accounts[]` | `UserMailAccountSetup` | Import metadata only; require reconnect/retest for secrets. |
| `settings.general.undo_send_delay_seconds` | `UserMailPreferences.compose.undo_send_delay_seconds` | User-scoped. |
| `settings.general.notifications_enabled` | `UserMailPreferences.notifications.enabled` | User-scoped. |
| `settings.general.sync_interval_minutes` | Account `sync.interval_minutes` or `sync_defaults.interval_minutes` | User-scoped. |

## 4. New Export Schema Map

Add these exports to `executionInterface.exports[]` and `semanticInterface.tools[]`.

| Export | Input Schema Definition | Output Schema | Semantic Category | Capability | Required Host Effects |
| --- | --- | --- | --- | --- | --- |
| `get_provider_capabilities` | `GetProviderCapabilitiesInput` | `ResultEnvelope<ProviderCapabilitiesData>` | query | `mail:accounts` | `ProviderCapabilityRead`, `MailStoreRead` |
| `validate_account_setup` | `ValidateAccountSetupInput` | `ResultEnvelope<AccountValidationData>` | validation | `mail:accounts` | optional `ProviderCapabilityRead` |
| `plan_connection_test` | `PlanConnectionTestInput` | `ResultEnvelope<ConnectionTestPlanData>` | mutation-plan | `mail:accounts` | `ProviderCapabilityRead`, `AccountCredentialRead`, `ImapSync`, `ImapFetch`, `SmtpSend` |
| `complete_account_setup` | `CompleteAccountSetupInput` | `ResultEnvelope<AccountSetupCompletionData>` | mutation | `mail:accounts` | `AccountCredentialWrite`, `StorePlatformSecret`, `MailStoreWrite`, optional `ImapSync` |
| `begin_oauth_account_setup` | `BeginOAuthAccountSetupInput` | `ResultEnvelope<OAuthBeginData>` | mutation-plan | `mail:accounts` | `ProviderCapabilityRead`, `OAuthConnect` |
| `complete_oauth_account_setup` | `CompleteOAuthAccountSetupInput` | `ResultEnvelope<OAuthCompletionData>` | mutation | `mail:accounts` | `OAuthConnect`, `AccountCredentialWrite`, `MailStoreWrite`, optional `ImapSync` |
| `disconnect_oauth_account` | `DisconnectOAuthAccountInput` | `ResultEnvelope<OAuthDisconnectData>` | mutation | `mail:accounts` | `OAuthDisconnect`, `AccountCredentialWrite`, `MailStoreWrite` |
| `remove_account` | `RemoveAccountInput` | `ResultEnvelope<AccountRemovalData>` | mutation | `mail:accounts` | `AccountCredentialWrite`, `MailStoreWrite`, `MailStoreDelete`, `CancelJob`, optional `OAuthDisconnect` |
| `list_identities` | `ListIdentitiesInput` | `ResultEnvelope<IdentityListData>` | query | `mail:accounts` | `AccountCredentialRead`, `MailStoreRead` |
| `save_identity` | `SaveIdentityInput` | `ResultEnvelope<IdentityMutationData>` | mutation | `mail:accounts` | `AccountCredentialWrite`, `MailStoreWrite` |
| `delete_identity` | `DeleteIdentityInput` | `ResultEnvelope<IdentityDeleteData>` | mutation | `mail:accounts` | `AccountCredentialWrite`, `MailStoreWrite` |
| `save_signature` | `SaveSignatureInput` | `ResultEnvelope<SignatureMutationData>` | mutation | `mail:accounts` | `MailStoreWrite` |
| `delete_signature` | `DeleteSignatureInput` | `ResultEnvelope<SignatureDeleteData>` | mutation | `mail:accounts` | `MailStoreWrite`, `MailStoreDelete` |
| `get_user_preferences` | `GetUserPreferencesInput` | `ResultEnvelope<UserPreferencesData>` | query | `mail:accounts` | `MailStoreRead` |
| `save_user_preferences` | `SaveUserPreferencesInput` | `ResultEnvelope<UserPreferencesMutationData>` | mutation | `mail:accounts` | `MailStoreWrite` |
| `inspect_legacy_settings` | `InspectLegacySettingsInput` | `ResultEnvelope<LegacyInspectionData>` | query | `mail:accounts` | optional `MailStoreRead` |
| `migrate_legacy_settings` | `MigrateLegacySettingsInput` | `ResultEnvelope<LegacyMigrationData>` | mutation | `mail:accounts` | `MailStoreRead`, `MailStoreWrite`, `MailStoreDelete`, optional `AccountCredentialWrite` |

Required schema details:

- `GetProviderCapabilitiesInput`: `context`, optional `provider_ids: string[]`.
- `ValidateAccountSetupInput`: `context`, `account: UserMailAccountSetup`, optional `admin_policy_snapshot: ProviderCapabilities`, optional `existing_account_snapshot: UserMailAccountSetup`.
- `PlanConnectionTestInput`: `context`, `account: UserMailAccountSetup`, `test_scope` enum `all|incoming|outgoing|folder_mapping`, optional `admin_policy_snapshot`.
- `CompleteAccountSetupInput`: `context`, `account: UserMailAccountSetup`, optional `connection_test_result: ConnectionTestResult`, optional `make_default: boolean`, optional `save_mode` enum `active|receive_only|disabled|incomplete`.
- `BeginOAuthAccountSetupInput`: `context`, `provider`, `email_address`, optional `account_label`, optional `requested_scopes: string[]`, optional `redirect_route`.
- `CompleteOAuthAccountSetupInput`: `context`, `provider`, `authorization_ref`, `oauth_connection_ref`, optional `account: UserMailAccountSetup`, optional `connection_test_result`.
- `DisconnectOAuthAccountInput`: `context`, `account_id`, optional `oauth_connection_ref`, `disable_account: boolean`.
- `RemoveAccountInput`: `context`, `account_id`, `remove_local_cache`, `cancel_scheduled_sends`, optional `delete_drafts`, optional `revision`.
- `ListIdentitiesInput`: `context`, optional `account_id`, optional `snapshot` with identities/signatures.
- `SaveIdentityInput`: `context`, `identity: MailIdentity`, optional `revision`.
- `DeleteIdentityInput`: `context`, `identity_id`, optional `account_id`, optional `force`.
- `SaveSignatureInput`: `context`, `signature: MailSignature`, optional `revision`.
- `DeleteSignatureInput`: `context`, `signature_id`, optional `account_id`, optional `force`.
- `GetUserPreferencesInput`: `context`, optional `snapshot: UserMailPreferences`, optional `include_defaults`.
- `SaveUserPreferencesInput`: `context`, `preferences: UserMailPreferences`, optional `revision`.
- `InspectLegacySettingsInput`: `context`, optional `legacy_settings: LegacySettings`.
- `MigrateLegacySettingsInput`: `context`, `legacy_settings: LegacySettings`, optional `migration_options`, optional `dry_run`.

## 5. Existing Export Schema Map

Preserve these current exports and tighten each schema. Do not rename unless a compatibility shim remains in `plugin.json`.

| Export | Input Schema Definition | Output Schema | Semantic Category | Capability | Required Host Effects |
| --- | --- | --- | --- | --- | --- |
| `list_accounts` | `ListAccountsInput` | `ResultEnvelope<QueryCollection<MailAccountSummary>>` | query | `mail:read` | `AccountCredentialRead` |
| `get_account_status` | `GetAccountStatusInput` | `ResultEnvelope<AccountStatusData>` | query | `mail:read` | `AccountCredentialRead`, `ImapSync` |
| `list_mailboxes` | `ListMailboxesInput` | `ResultEnvelope<QueryCollection<Mailbox>>` | query | `mail:read` | `AccountCredentialRead`, `ImapFetch` |
| `read_emails` | `ReadEmailsInput` | `ResultEnvelope<ReadEmailsData>` | query | `mail:read` | `AccountCredentialRead`, `ImapFetch` |
| `search_emails` | `SearchEmailsInput` | `ResultEnvelope<SearchEmailsData>` | query | `mail:read` | `AccountCredentialRead`, `ImapFetch` |
| `get_email` | `GetEmailInput` | `ResultEnvelope<EmailMessageData>` | query | `mail:read` | `AccountCredentialRead`, `ImapFetch`, optional `MailStoreWrite` |
| `get_thread` | `GetThreadInput` | `ResultEnvelope<ThreadData>` | query | `mail:read` | `AccountCredentialRead`, `ImapFetch` |
| `mark_read` | `MarkReadInput` | `ResultEnvelope<MutationAck>` | mutation | `mail:read` | `MailStoreWrite` |
| `flag_email` | `FlagEmailInput` | `ResultEnvelope<MutationAck>` | mutation | `mail:organize` | `MailStoreWrite` |
| `draft_email` | `DraftEmailInput` | `ResultEnvelope<DraftMutationData>` | mutation | `mail:draft` | `MailStoreWrite`, optional `ContactLookup` |
| `update_draft` | `UpdateDraftInput` | `ResultEnvelope<DraftMutationData>` | mutation | `mail:draft` | `MailStoreWrite` |
| `discard_draft` | `DiscardDraftInput` | `ResultEnvelope<MutationAck>` | mutation | `mail:draft` | `MailStoreDelete` |
| `send_email` | `SendEmailInput` | `ResultEnvelope<SendPlan>` | mutation | `mail:send` | `MailStoreWrite`, `SmtpSend`, optional `NotifyUser` |
| `reply_email` | `ReplyEmailInput` | `ResultEnvelope<ReplyPlan>` | mutation | `mail:send` | `MailStoreRead`, `MailStoreWrite`, `SmtpSend` |
| `forward_email` | `ForwardEmailInput` | `ResultEnvelope<ForwardPlan>` | mutation | `mail:send` | `MailStoreRead`, `MailStoreWrite`, `SmtpSend` |
| `move_email` | `MoveEmailInput` | `ResultEnvelope<MutationAck>` | mutation | `mail:organize` | `MailStoreWrite` |
| `delete_email` | `DeleteEmailInput` | `ResultEnvelope<MutationAck>` | mutation | `mail:organize` | `MailStoreWrite`, `MailStoreDelete` |
| `archive_email` | `ArchiveEmailInput` | `ResultEnvelope<MutationAck>` | mutation | `mail:organize` | `MailStoreWrite` |
| `create_folder` | `CreateFolderInput` | `ResultEnvelope<FolderMutationData>` | mutation | `mail:organize` | `MailStoreWrite` |
| `rename_folder` | `RenameFolderInput` | `ResultEnvelope<FolderMutationData>` | mutation | `mail:organize` | `MailStoreWrite` |
| `delete_folder` | `DeleteFolderInput` | `ResultEnvelope<MutationAck>` | mutation | `mail:organize` | `MailStoreDelete` |
| `get_unread_count` | `GetUnreadCountInput` | `ResultEnvelope<UnreadCountData>` | query | `mail:read` | `AccountCredentialRead`, `ImapFetch` |
| `create_rule` | `CreateRuleInput` | `ResultEnvelope<RuleMutationData>` | mutation | `mail:organize` | `MailStoreWrite` |
| `list_rules` | `ListRulesInput` | `ResultEnvelope<QueryCollection<MailRule>>` | query | `mail:organize` | `MailStoreRead` |
| `delete_rule` | `DeleteRuleInput` | `ResultEnvelope<MutationAck>` | mutation | `mail:organize` | `MailStoreDelete` |
| `schedule_send` | `ScheduleSendInput` | `ResultEnvelope<SchedulePlan>` | mutation | `mail:send` | `MailStoreWrite`, `ScheduleJob` |
| `cancel_scheduled_send` | `CancelScheduledSendInput` | `ResultEnvelope<MutationAck>` | mutation | `mail:send` | `MailStoreWrite`, `CancelJob` |

## 6. Schema Tightening Checklist

For each input definition:

- Add `additionalProperties: false` except where host metadata explicitly allows extension.
- Require `context` and every operation-specific required field from `contracts.md`.
- Use string enums for `permission`, provider, security, auth method, status, connection state, test state, undo send, sort direction, folder kind, rule action type, and preference values.
- Add `minItems`, `maxItems`, `minLength`, `maxLength`, `minimum`, and `maximum` constraints matching `contracts.md`.
- Use `format: "email"` for address strings in account setup and identities while preserving the app's planner validation.
- Use opaque refs for secrets and attachments.

For each output definition:

- Use `ResultEnvelope` consistently.
- Define `data` for each operation instead of leaving `{ "type": "object" }`.
- List host-effect payload fields expected by the host executor.
- Include warnings for stale snapshots, receive-only account setup, reconnect required, destructive action confirmation, and migration reconnect requirements.

## 7. Semantic Tool Descriptions

Descriptions must tell agents what the planner can do and what the host owns. Required wording pattern:

- Query tools: "Read host-managed mail/account data for the current user without exposing credentials."
- Mutation tools: "Validate and plan host-owned effects; does not perform protocol/network I/O inside Wasm."
- Account setup tools: "Plan user-scoped account setup. Credentials and OAuth are handled by Synapp host secure services."
- Migration tools: "Classify or migrate legacy user settings; never migrate credentials into admin policy."

Risk notes:

- `mail:read`: high, because message content may be sensitive.
- `mail:draft`: medium, because draft content may affect user communication.
- `mail:send`: critical, because mail is sent externally or scheduled.
- `mail:organize`: high, because messages/folders/rules can be changed or deleted.
- `mail:accounts`: high, because account availability, identity, sync, and secret references are managed, though raw secrets are not exposed.

## 8. Compatibility Plan

Implementation order:

1. Add shared definitions and schema-tighten existing exports without changing current export names.
2. Add new account/preference/migration exports to Rust and `plugin.json` together.
3. Update `synapp.app.json` so the main route uses `ui.entrypoint` and `/inbox`, not `ui_schemas.main`.
4. Move old account/general schemas into migration-only legacy definitions or remove them from user-facing settings.
5. Add catalog entry updates after manifest/plugin changes and regenerate package SHA through the existing packaging workflow.

Compatibility requirements:

- Existing agent integrations using current mail workflow names continue to work.
- The host may treat `list_accounts` as read-only account summary, while account configuration uses the new `mail:accounts` exports.
- Legacy settings migration is explicit and user-scoped. It must not silently promote old settings data into admin policy.
- `ui_schemas.main` can remain temporarily only if ignored by the host for route rendering and marked deprecated; preferred final state is removal.

## 9. Schema Gates

| Gate | Status | Required Action |
| --- | --- | --- |
| Every Wasm export has input/output schema | Pass by plan | Existing and new exports are mapped above. |
| Account setup contracts included | Pass by plan | New exports cover validation, tests, OAuth, save, remove, identities, signatures, preferences, migration. |
| Host effects identified | Pass by plan | Existing and new required effects are listed per export. |
| Main UI no longer schema-rendered | Pass by plan | Requires `synapp.app.json` implementation change. |
| No credentials in admin settings | Pass by plan | Schemas forbid raw secret fields and separate admin policy from user setup. |
| wasm32-wasip1 compatibility | Pass by plan | Schema and exports require only JSON planning and pure Rust validation. |
