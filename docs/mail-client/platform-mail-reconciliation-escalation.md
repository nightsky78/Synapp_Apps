# Mail Client Platform Escalation - IMAP Reconciliation

Date: 2026-05-15
Classification: `platform_required`
Source: App Platform Liaison handoff from Synapp Apps Pipeline

## Context

The Mail Client app-side work is complete for the current app-platform contract:

- Mailbox, folder, message, and pending-operation state is persisted through platform document storage when authenticated.
- Sent folder `502` / `IMAP BAD FETCH Invalid messageset` is degraded in the app UI to saved messages with a warning.
- Folder-to-folder move works locally and records `remote_sync_state: pending_remote_move` plus a `mail_operations` pending move.
- App validation passed, package metadata was pushed, and local Dev installed Mail Client version `2.1.1` with package hash `f76185b61f8c852283e41af3dc753b90376c8d3c6da5e0d1ea8e917049bfb3c1`.

The remaining user requirement cannot be completed app-side because apps cannot perform direct IMAP networking, cannot access mail credentials, and cannot reconcile remote mailbox state themselves.

## Problem Statement

The platform mail proxy currently supports live IMAP fetch and SMTP send only. It lacks host-owned contracts for:

- Folder discovery/list/status.
- Empty/special folder fetch safety.
- Persistent mail sync/store.
- IMAP MOVE/COPY/STORE/EXPUNGE/delete.
- Reconciliation of Mail Client pending operations back to the remote IMAP server.

As a result, app-local moves are persisted locally but remote mailbox state is not reconciled. Empty or special folders such as Sent can still return `502` from the live platform fetch path if the app uses it directly.

## Known Platform Gaps

- `services/mail-proxy/proto/mail_proxy.proto` defines only `ImapFetch` and `SmtpSend`.
- `services/mail-proxy/internal/server/server.go` handles fetch and send only.
- `services/mail-proxy/internal/imap/client.go` fetches selected mailbox with `1:*`, which can fail on empty/special folders.
- `services/transformer-service/src/mail_accounts.rs` exposes account CRUD, `/messages`, and `/send`; `/messages` delegates to `ImapFetch` and maps proxy errors to `502`.
- No endpoint/gRPC method exists for folder discovery/list/status.
- No endpoint/gRPC method exists for message move/copy/delete/store/expunge.
- No reconciliation process consumes Mail Client `mail_operations` pending app documents.

## Required Platform Contracts

Implement host-owned, generic mail platform contracts rather than app-specific UI logic:

1. Empty/special folder fetch safety
   - `GET /api/v1/mail/messages?mailbox=SENT` must return `200` with `messages: []` for empty Sent folders.
   - Normal empty folder conditions must not leak raw IMAP errors to clients.

2. Folder discovery/list/status
   - Add REST and/or gRPC folder list/status contract.
   - Include Inbox, Sent, Drafts, Trash, Archive, Junk, and custom folders where available.
   - Preserve stable normalized folder identity useful for the app UI.

3. Persistent mail sync/store
   - Persist synced mailbox/message/folder state in `app_documents` or a host-owned mail store under validated principal scope.
   - Forward actor/principal metadata to audit-compatible storage paths.

4. IMAP move/copy/delete reconciliation
   - Add host-owned IMAP move/copy/delete capability.
   - Prefer IMAP MOVE when supported.
   - Fall back to COPY + STORE `\\Deleted` + EXPUNGE when MOVE is unavailable.
   - Return structured failures for unsupported or rejected operations.

5. App pending operation reconciliation
   - Process Mail Client `mail_operations` pending moves, or expose a callable reconciliation endpoint/job.
   - On success, update app-local message state from `pending_remote_move` to synced/confirmed and mark operation complete.
   - On remote failure, mark operation failed with non-secret structured error.
   - Make reconciliation idempotent.

6. Security and audit
   - Keep mail credentials only in host/platform secret storage.
   - Enforce human/agent scopes; agent calls must use the principal owner and explicit mail/app scopes.
   - Reject cross-principal access.
   - Audit state-mutating operations with actor ID, actor type, and principal ID.
   - Do not write raw passwords/OAuth tokens to logs, responses, app documents, or snapshots.

## Acceptance Checks For Platform Pipeline

1. Empty Sent folder
   - Configure or mock an account where Sent is empty.
   - Call `GET /api/v1/mail/messages?mailbox=SENT`.
   - Assert HTTP `200`, normalized mailbox identity, and `messages: []`.
   - Assert no `Invalid messageset` leaks to the client.

2. Folder discovery/list/status
   - Assert platform returns server-discovered folders, delimiter/attributes where available, special-use mapping, unread/total/status where supported, and stable IDs/names.
   - Assert empty/special folders are represented, not omitted.

3. Persistent sync
   - Fetch/sync messages into `app_documents` or host mail store under the validated principal.
   - Assert subsequent app queries can render persisted folder entries without only relying on live IMAP fetch.
   - Assert actor/principal metadata is available for audit.

4. IMAP move/copy/delete reconciliation
   - Start with a message in folder A.
   - Move it to folder B through the platform contract.
   - Assert remote IMAP state changes: absent from A, present in B.
   - Cover MOVE-capable server behavior and COPY+STORE+EXPUNGE fallback.
   - Add negative-path test for unsupported/rejected operation.

5. App pending operation reconciliation
   - Seed `remote_sync_state: pending_remote_move` and corresponding `mail_operations` document.
   - Run reconciliation.
   - On success, assert message state is synced/confirmed and operation complete.
   - On failure, assert operation failed with non-secret structured error.
   - Assert retry is idempotent.

6. RBAC, identity, audit
   - Human caller can operate on own configured mail account.
   - Agent caller requires explicit mail/app scope and uses owner principal.
   - Agent without scope receives forbidden.
   - Cross-principal access is rejected.
   - Mutating operations append audit events with actor ID, actor type, principal ID.

7. Local Dev handback
   - Re-run Mail Client E2E against local platform Dev:
     - all folders visible,
     - messages persist after reload,
     - folder-to-folder move updates UI,
     - remote reconciliation completes,
     - pending move is cleared or failed deterministically.

## Required Platform Handback Evidence

Platform Pipeline must return:

- NexusCore commit/revision hash.
- New or changed REST/gRPC endpoint/schema summary.
- Evidence empty Sent returns `200 []`.
- Evidence folder discovery/list/status works.
- Evidence persistent synced mail state is stored.
- Evidence IMAP move/copy/delete reconciliation updates remote state.
- Evidence app-local pending move state clears on success or marks failed on remote failure.
- RBAC/audit/security test output.
- Exact commands run and pass/fail results.
- Any blockers requiring a follow-up Liaison loop.

## Routing Status

Attempted to invoke platform `Pipeline` agent on 2026-05-15, but the Copilot session hit the agent rate limit before the handoff could run. This file preserves the Liaison-validated handoff payload for immediate retry when the agent limit resets.
