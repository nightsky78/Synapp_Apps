# Mail Client UX Acceptance Criteria - HostEffect Contract Migration

Stage: 1 - UX-Spec  
App: apps/first-party/mail-client  
Date: 2026-05-19  
Scope: Logic-only migration to platform mail HostEffect contracts

## 1. Objective

Fix three confirmed user-visible bugs by migrating mail-client effect names and payload contracts to current platform host effects:

- Wrong To field in inbox
- Sent mails not stored on IMAP Sent folder
- Drafts folder shows zero messages

No visual redesign is in scope.

## 2. Current UX-Relevant Findings (from app source)

- The app still emits legacy effect names: ImapFetch, ImapSync, SmtpSend.
- Read/list flows currently emit IMAP fetch payloads shaped around user_id, account_id, mailbox_id, offset/limit, not credential_key mailbox contracts.
- Send flow currently emits smtp send payload using account_id + draft_id and separately writes a local SENT marker, which does not guarantee append to IMAP Sent.
- Folder refresh behavior does not enforce mailbox-required sync contract for non-INBOX folders.

## 3. Liaison-Validated Platform Assumptions

These assumptions are treated as platform contract truth for this UX slice:

- imap_fetch must be used with effect name imap_fetch.
- imap_fetch requires credential_key; mailbox optional (default INBOX); max_count optional.
- imap_fetch returns email payload including to_addrs string array (source of To field fix).
- imap_sync must be used with effect name imap_sync.
- imap_sync requires credential_key and mailbox (required), max_count optional.
- smtp_send must be used with effect name smtp_send.
- smtp_send requires smtp_credential_key, imap_credential_key, to; optional sent_mailbox (default Sent).
- smtp_send behavior includes append to Sent mailbox on IMAP server.
- Credentials are stored via store_credential as:
  - key mail:<account_id>:imap
  - key mail:<account_id>:smtp

## 4. UX Acceptance Criteria

### 4.1 Bug 1 - Wrong To field in inbox

Acceptance:

- Given a mailbox read in Inbox, when the app refreshes via imap_fetch, then each message row and message detail uses server-returned recipient data from to_addrs (or mapped recipient list derived from to_addrs), not account owner email fallback.
- Given a message with multiple recipients, when viewed in list/detail, then all intended recipients are represented correctly according to existing UI display rules.
- Given messages where to_addrs is empty, the UI shows existing safe fallback state (for example blank/unknown recipient label) without substituting owner address incorrectly.

### 4.2 Bug 2 - Sent mail missing on IMAP Sent

Acceptance:

- Given a valid compose/reply/forward send, when user confirms send, then app emits smtp_send with smtp_credential_key and imap_credential_key for the selected account.
- Given smtp_send success, then sent mail appears in Sent mailbox after next Sent refresh (imap_sync/imap_fetch sequence) without requiring local-only synthetic Sent write.
- Given optional sent_mailbox override is absent, default Sent behavior is used.
- Given smtp_send failure, user sees existing send failure UX and draft remains recoverable; no false success confirmation is shown.

### 4.3 Bug 3 - Drafts folder empty

Acceptance:

- Given user opens Drafts folder, app uses imap_sync with mailbox explicitly set to Drafts and account credential_key before rendering refreshed results.
- Given sync completes, Drafts list shows server draft messages instead of persistent zero-state when drafts exist remotely.
- Given Drafts sync fails, user gets a recoverable sync error state (existing error pattern), not silent empty-folder success.

### 4.4 Folder-Specific Sync Contract (cross-cutting)

Acceptance:

- INBOX reads may use imap_fetch default mailbox behavior.
- Non-INBOX system folders (Drafts, Sent, Trash, Spam/Junk) use imap_sync with explicit mailbox, then fetch/read pipeline as currently defined by app architecture.
- Account/folder switching keeps existing navigation behavior; only backend effect contract changes.

### 4.5 Credential Key Lifecycle UX

Acceptance:

- On account setup completion, app stores IMAP and SMTP credentials via store_credential under keys:
  - mail:<account_id>:imap
  - mail:<account_id>:smtp
- Read/sync/send actions derive and pass those credential keys consistently.
- If credential keys are missing or stale, user gets explicit reconnect/account-fix guidance through existing account health/error UX.

## 5. Happy-Path Journeys Required After Fix

### 5.1 Open inbox and verify recipients

- User opens Inbox.
- Messages load through imap_fetch using credential_key.
- To display reflects actual recipients from to_addrs.

### 5.2 Send and verify Sent

- User sends a draft/reply/forward.
- App emits smtp_send with both credential keys.
- User navigates to Sent.
- Sent refresh shows the newly sent message from server mailbox.

### 5.3 Open Drafts with server-backed content

- User opens Drafts.
- App emits imap_sync with mailbox Drafts, then reads updated data.
- Draft list shows remote drafts count and rows.

## 6. Negative Paths That Must Stay Guarded

- Missing credentials: show actionable account-reconnect/setup error, do not crash.
- Effect unavailable from host: return structured effect-unavailable error path and preserve prior snapshot where applicable.
- Invalid mailbox mapping: block request with safe validation error rather than silently falling back to Inbox.
- Send partial failure (SMTP success but append failure surfaced by host as failure): show failed send state and keep draft recoverable.
- Account without send capability (receive-only): send actions remain disabled/guarded as today.

## 7. UI Surface Impact

- No new UI components are expected.
- No layout, copy, or interaction redesign is required for this migration.
- Existing loading/error/snapshot UI states are reused.

## 8. data-testid Impact

- No new UI controls are introduced.
- No existing control semantics are intended to change.
- Therefore no new data-testid additions are required by this slice.

## 9. Non-Goals

- No platform service implementation details beyond contract usage from Wasm app.
- No mailbox visual redesign.
- No new account settings information architecture.

## 10. Done Definition for UX Gate

This UX slice passes when all three bug outcomes are reproducibly resolved in app behavior and the migration remains logic-only with unchanged UI structure.

Evidence expected from architect/implementation stage:

- Effect name migration proofs in app action outputs (imap_fetch, imap_sync, smtp_send).
- Credential key usage proofs (mail:<account_id>:imap and mail:<account_id>:smtp).
- Scenario validation for Inbox recipient correctness, Sent append visibility, and Drafts population.
