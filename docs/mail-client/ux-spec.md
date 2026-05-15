# Mail Client UX Specification

Stage: 1 - UX-Spec  
App: `apps/first-party/mail-client`  
Date: 2026-05-15

## 1. Product Scope

The Mail Client must be a full human-usable email application for Synapp, not a settings form or tool demo. The first screen must open to an inbox-first work surface where a user can read, search, organize, compose, reply, forward, send, schedule, and manage mail across one or more configured accounts. The same product capabilities must be available to AI agents through the existing Wasm/plugin tool surface, with user-visible permissions and control points.

The app follows familiar Outlook, Gmail, and Apple Mail patterns: account and folder navigation on the left, a dense message list, a reading pane, fast search, a persistent compose affordance, robust account setup, and clear recovery when sync or send fails. It is tailored to Synapp by keeping protocol access, credentials, OAuth state, persistence, scheduling, notifications, and network I/O owned by the host platform. The Wasm app remains a sandbox-compatible planner and validator with no AI logic and no direct IMAP/SMTP connections.

### Target Users

- Individual Synapp users who need a reliable mailbox client for daily work.
- Power users who manage multiple personal or work accounts, aliases, signatures, folders, rules, and scheduled sends.
- Administrators who grant app capabilities and agent permissions, but do not configure another user's private mail accounts in an admin settings page.
- AI agents that act on behalf of a user within explicit Synapp permissions such as `mail:read`, `mail:draft`, `mail:send`, and `mail:organize`.

### Goals

- Make the main route immediately useful as a mail client: show mail, folders, actions, and compose, not a generic settings schema.
- Move account configuration into a user-scoped account setup surface owned by the signed-in user.
- Support manual IMAP/SMTP configuration, OAuth provider placeholders where host support exists, connection testing, credential lifecycle messaging, aliases, signatures, default sender, and sync preferences.
- Preserve Synapp's dual-interface model by mapping every human workflow to Wasm exports and `plugin.json` semantic tools.
- Make permission, trust, and recovery behavior visible enough that humans understand what agents can do and what the app cannot do locally.

### Success Outcomes

- A new user can add an account, test incoming and outgoing connectivity, complete setup, and land in a usable inbox without needing admin settings.
- A returning user can triage mail from the unified inbox, open a message, reply, archive/delete/move, search, manage folders, and compose/send in one session.
- A user with multiple accounts can understand which account a message belongs to, choose the sender identity, set a default, and manage signatures and aliases.
- An agent can read, draft, send, organize, and schedule mail only within granted permissions, with user-visible audit and review points for sensitive operations.
- Failure states are recoverable without data loss, especially for account connection, sync, compose autosave, send, and conflicting changes.

### May 15 Folder And Persistence Correction

This correction addresses the current mail-client reliability goal: entries must appear in all folders, the Sent folder must not break the workspace when live IMAP fetch returns `502` / `BAD FETCH Invalid messageset`, mail snapshots must persist across app sessions, and moving messages between folders must be supported and testable.

Liaison-validated platform assumptions:

- Available now: the app can persist app-owned mail metadata and message snapshots through generic `PutDocument` and `QueryDocuments` effects. These documents are scoped by `app_id` and `principal_id` by the host.
- Available now: the app can implement a metadata-local move by updating its persisted message record, source/target folder ids, folder counts, and `sync_state` without requiring an upstream IMAP move to succeed immediately.
- Required caveat: true upstream IMAP move/sync is platform-required. The current mail proxy supports `ImapFetch` and `SmtpSend`, but does not expose a validated move/copy/delete operation. Live Sent fetch can return `BAD FETCH Invalid messageset`, so the app must not treat a Sent fetch failure as an empty or fatal mailbox state.
- Required caveat: until the platform exposes upstream IMAP move/sync, moved messages must carry `sync_state: "pending_remote_move"` and be shown as locally organized with pending remote reconciliation.

App-side UX behavior for this slice:

- Every folder view uses persisted app-side mail documents as the durable baseline and overlays fresh host fetch results when available.
- Folder navigation includes Inbox, Sent, Drafts, Archive, Junk, Trash, and custom folders when those folders exist in persisted metadata, even if a live fetch for one folder fails.
- Sent folder `502` / `BAD FETCH Invalid messageset` errors are shown as a recoverable sync warning for Sent only; the workspace, other folders, and persisted Sent entries remain visible.
- A successful local move immediately removes the message from the source folder, adds it to the target folder, updates counts, and records `sync_state: "pending_remote_move"` until platform reconciliation is available.
- The UI distinguishes locally persisted data from live sync freshness using quiet status text such as last synced time, pending remote move, or folder sync warning.
- Agents with organize permission can perform the same local move workflow and receive structured pending-sync status in the tool response.

## 2. Current UX Problem To Replace

The current main view is effectively a host-rendered settings/configuration form using fields such as mailbox, search query, thread view, focused inbox, preview pane, and emails per page. Saving those settings does not create a mail workflow, so the page does not function as a mail client.

The current settings contribution mixes per-user account configuration into a settings panel that reads like an admin surface and includes a vague `General Settings` group. This must be replaced with clear product areas:

- Mail workspace: the main inbox-first app surface.
- User account setup: per-user account and identity configuration.
- Mail preferences: user-level display, compose, notification, and sync preferences with explicit labels.
- Admin/agent controls: platform permission and policy surfaces only, not individual account credentials.

## 3. Information Architecture

### Primary App Routes

- `/inbox`: Default route. Opens the unified inbox or the user's last selected inbox view.
- `/mailbox/:accountId/:mailboxId`: Opens a folder for one account.
- `/message/:accountId/:messageId`: Opens a message in full view; on desktop this can also be represented by reading-pane state.
- `/search`: Shows structured search results across selected accounts and folders.
- `/compose`: Opens a new compose window or panel.
- `/draft/:accountId/:draftId`: Reopens a saved draft.
- `/settings/accounts`: User-scoped mail account setup and account health.
- `/settings/identities`: Sender aliases, signatures, default sender, reply-to behavior.
- `/settings/preferences`: Reading, conversation, notification, sync, undo-send, and compose preferences.
- `/settings/rules`: User-created mail rules for folders and triage.
- `/settings/agents`: Human review of granted agent mail permissions and recent agent mail activity, if exposed by the host.

### Main Workspace Layout

Desktop layout:

- Top command bar: app name, account/scope switcher, search, refresh/sync status, settings, permission indicator.
- Left navigation: Compose button, favorites, unified folders, account sections, custom folders, rules shortcut.
- Message list: focused/other or all messages, filters, sort, selection checkboxes, row actions, pagination or infinite cursor loading.
- Reading pane: selected message/thread, attachments, inline reply affordance, message actions.
- Compose surface: modal, docked panel, or separate route with autosave and send controls.

Mobile/tablet layout:

- Bottom or top navigation exposes folders, inbox, search, compose, and account switcher.
- Message list and reading pane become separate screens with a reliable back path.
- Bulk actions use a selection toolbar after long press or checkbox selection.

## 4. Human User Journeys

### 4.1 First Run: Add A Mail Account

1. User opens Mail Client with no configured accounts.
2. The app shows an empty account setup state, not an empty inbox.
3. User selects `Add account`.
4. User chooses a setup path:
   - Automatic/OAuth provider option if host supports Google, Microsoft, or other providers.
   - Manual IMAP/SMTP setup for any provider.
5. User enters display name and email address.
6. For manual setup, user enters incoming IMAP settings and outgoing SMTP settings.
7. User enters a password/app password or starts host-owned OAuth authorization.
8. User clicks `Test connection`.
9. The app shows separate incoming and outgoing test results.
10. User fixes any errors and retests.
11. User confirms sync preferences and default identity.
12. The app completes setup and opens the account inbox.

Required behavior:

- Passwords, app passwords, OAuth tokens, and refresh tokens are never displayed after entry and are never returned to Wasm.
- The app explains that credentials are stored by Synapp host secret storage, not by the mail app module.
- Setup cannot finish until required account identity fields are valid and either host OAuth authorization succeeds or manual IMAP and SMTP tests pass, unless the user explicitly saves as disabled/incomplete.
- If SMTP fails but IMAP succeeds, the user may save the account as receive-only with clear disabled send behavior.
- If IMAP fails but SMTP succeeds, the app blocks active account setup because the inbox cannot function.

Manual IMAP fields:

- Account label.
- Display name.
- Email address.
- Username, defaulting to email address but editable.
- IMAP server host.
- IMAP port, default 993.
- Security: SSL/TLS, STARTTLS where supported, or none only if host policy allows.
- Authentication method: password/app password initially; OAuth placeholders may be provider-specific.

Manual SMTP fields:

- SMTP server host.
- SMTP port, default 587.
- Security: STARTTLS by default, SSL/TLS option for port 465 where supported.
- SMTP username, default same as incoming.
- SMTP authentication: same credential as incoming or separate credential.
- Optional reply-to address.

### 4.2 Returning User: Daily Inbox Triage

1. User opens Mail Client.
2. App loads unified inbox with folder navigation, message list, and reading pane.
3. User sees sync freshness, account health, unread counts, and whether the current view is unified or account-specific.
4. User selects a message and reads it in the pane.
5. User marks unread/read, flags, archives, deletes, moves, or replies.
6. The message list updates immediately with optimistic feedback and reconciles with host confirmation.
7. If an action fails, the app restores or explains the previous state and offers retry.

Required behavior:

- The default route is mail-first. It must not render configuration fields as the main experience.
- Row density supports scanning sender, subject, snippet, timestamp, attachments, flags, unread state, account badge, and folder.
- The app preserves user context when switching folders or returning from a message.
- Bulk selection supports archive, delete, move, mark read/unread, flag/unflag, and rule creation from selected messages.

### 4.3 Compose And Send

1. User clicks `Compose`.
2. App opens compose with selected default sender.
3. User can change From account/alias if permitted.
4. User enters To, Cc, Bcc, subject, and rich or plain text body.
5. Contacts autocomplete is host-assisted when available.
6. Signature is inserted based on selected identity and compose context.
7. Draft autosaves during editing.
8. User sends now or schedules send.
9. On send now, app shows undo-send for the configured window.
10. On success, draft moves to Sent and message list counts update.

Required behavior:

- Send is disabled until at least one valid recipient exists and the account has working SMTP or equivalent host send capability.
- The user sees which account and alias will send the message.
- Attachments are represented with host-owned upload/attachment references; the Wasm app validates metadata and intent, not file bytes.
- Closing compose with unsaved changes prompts to save draft or discard.
- If a draft was modified elsewhere, the app shows conflict recovery before overwriting.

### 4.4 Reply, Reply All, Forward

1. User opens a message or thread.
2. User chooses Reply, Reply all, or Forward.
3. App chooses the correct sending identity based on the receiving account and alias rules.
4. User edits content and sends, schedules, saves, or discards.

Required behavior:

- Reply all clearly shows all recipients and excludes the user's own matching identities where appropriate.
- Forward includes attachments only when the user explicitly keeps them or host policy defaults to include them.
- Original message quoting is collapsible.
- Permission errors explain whether the user/agent lacks draft or send capability.

### 4.5 Search And Filter

1. User enters a query from the command bar.
2. App searches current mailbox by default and offers scope chips for all accounts, current account, current folder, unread, flagged, has attachment, from, to, date range, and folder.
3. Search results show matched messages in the same message-list pattern.
4. User can open, bulk act, or save a rule/search from results.

Required behavior:

- Search input supports plain text and structured filters.
- Active filters are visible and removable.
- Empty search results suggest broadening filters or searching all accounts.
- Search failures preserve the previous result set until a retry or new search succeeds.

### 4.6 Folder Management

1. User opens folder menu for an account.
2. User creates, renames, favorites, moves, or deletes a custom folder.
3. App updates navigation and affected messages.

Required behavior:

- System folders such as Inbox, Sent, Drafts, Archive, Junk, and Trash cannot be renamed or deleted.
- Deleting a non-empty custom folder requires confirmation and explains whether messages move to Archive, parent folder, or Trash according to host behavior.
- Folder actions are account-scoped.

### 4.7 Rules

1. User opens Rules from settings or from a message action.
2. User creates a rule using conditions such as from, to, subject contains, has attachment, account, folder, unread, or importance.
3. User selects actions such as move to folder, mark read, flag, archive, delete, or stop processing.
4. User tests the rule against recent mail where host support exists.
5. User saves, disables, reorders, or deletes rules.

Required behavior:

- Rule creation is user-scoped and account-scoped unless the user explicitly chooses all accounts.
- Potentially destructive rule actions require confirmation.
- Rules created by an agent are labeled as agent-created and visible to the user.

### 4.8 Scheduled Send

1. User composes or replies.
2. User chooses `Schedule send`.
3. App offers common times and custom date/time.
4. User confirms.
5. Draft appears in a Scheduled folder/view.
6. User can cancel scheduled send before host dispatch.

Required behavior:

- Scheduled send requires a valid account, draft, recipients, and send permission.
- Time zone is visible.
- Cancel restores draft editability.
- If host scheduling fails, the draft remains saved and unsent.

## 5. Account Configuration UX

Account configuration is a user-scoped workflow, not an admin settings workflow. Admins may govern whether mail features, providers, OAuth connectors, or agent capabilities are available, but users configure their own accounts and identities.

### Account List

Each configured account shows:

- Account label and email address.
- Provider or manual IMAP/SMTP badge.
- Connection health: connected, degraded, offline, authentication required, receive-only, disabled.
- Last successful sync time.
- Unread count.
- Default sender indicator.
- Actions: sync now, edit, test connection, disable/enable, remove.

### Add/Edit Account Sections

- Identity: account label, display name, email address, reply-to.
- Incoming mail: IMAP host, port, security, username, authentication method.
- Outgoing mail: SMTP host, port, security, username, authentication method, separate credential option.
- Authentication: password/app password entry or provider OAuth connection placeholder.
- Sync: interval, initial sync range, download attachments behavior, offline cache preference if host supports it.
- Sender defaults: default account, default alias, default signature.
- Advanced: IMAP path prefix, sent/drafts/trash/archive folder mapping, server timeout where host exposes it.

### Connection Test

The connection test must show independent checks:

- Incoming IMAP authentication.
- Incoming mailbox discovery.
- Outgoing SMTP authentication.
- Outgoing send capability using a non-delivery test where supported, or host-approved test mode.
- Folder mapping validation.

Connection test states:

- Not tested.
- Testing.
- Passed.
- Warning, such as SMTP unavailable or folder mapping incomplete.
- Failed, with field-specific guidance and retry.

### Credential Handling

- Secrets are entered into host-controlled secure fields.
- Saved secrets are represented as `Stored by Synapp` or `OAuth connected`; they are never echoed back.
- Replacing credentials requires re-entry and retest.
- Removing an account asks whether to remove local synced mail cache and scheduled sends associated with the account.
- OAuth disconnect and password removal are distinct user actions if the host exposes both.

### Identities, Aliases, And Signatures

- A user can configure aliases per account when the host/provider allows them.
- Each alias has display name, email address, optional reply-to, default signature, and enabled state.
- Signatures are user-authored rich text or plain text snippets associated with accounts, aliases, new messages, replies, and forwards.
- The compose From selector displays account and alias clearly.
- Default sender can be global or based on the mailbox/message being replied to.

### Migration From Current Settings

- Existing `accounts` data under the old settings panel should be treated as user account setup data if it was saved by the signed-in user.
- Existing generic `general` fields must be renamed and split into specific user preference groups:
  - Reading preferences: preview pane, conversation view, focused inbox, page size or list density.
  - Compose preferences: undo send window, default signature, reply behavior.
  - Notification preferences: new mail notifications, quiet hours if host supports them.
  - Sync preferences: sync interval defaults and attachment behavior.
- The main app route must ignore `ui_schemas.main` as a form-rendered configuration page and instead load the real UI entrypoint for the mail workspace.
- The migration must not put per-user accounts into admin settings. If admin policy currently hosts the panel, the product requirement is to relocate the surface to user settings or in-app account setup.

## 6. Agent-Assisted And Headless Workflows

Agents use the same app capabilities through the Synapp Broker and the Wasm/plugin semantic interface. The app must not include prompts, LLM routing, or autonomous decision logic. It validates requests, plans host effects, and returns outcomes.

### Agent Working Models

- Read-only assistant: can list accounts, inspect account status, list folders, read/search mail, get messages, and inspect threads.
- Draft assistant: can perform read actions and create/update/discard drafts, but cannot send.
- Send-capable assistant: can draft, reply, forward, send, and schedule send when granted `mail:send`.
- Organizer assistant: can move, archive, delete, flag, mark read/unread, manage folders, and manage rules when granted `mail:organize`.

### Human Control Points

- A draft created by an agent is visible in Drafts with an agent-created label, timestamp, and originating agent where host metadata is available.
- Agent send requires the host's explicit `mail:send` grant. The UI must expose that grant state and recent sends.
- Destructive or high-risk agent actions such as permanent delete, broad rule creation, or scheduled send cancellation must be auditable and may require host-level confirmation depending on policy.
- Users can revoke or reduce agent mail permissions from a permissions surface provided by the host or linked from Mail settings.

### Headless Outcomes

- Agent read/search results are scoped to the authenticated user and permission context.
- Agent draft/send operations use host-managed account identities and cannot access raw credentials.
- Agent organize operations produce the same visible mailbox changes a human action would produce.
- Errors returned to agents are structured and map to user-visible recovery guidance.

## 7. Permissions And Trust Boundaries

### User-Visible Permissions

The UI must show relevant capability state without overwhelming normal mail use:

- Account unavailable because mail read permission, host effect, or credential state is missing.
- Compose unavailable because draft permission or sender account capability is missing.
- Send unavailable because send permission, SMTP capability, OAuth grant, or host policy is missing.
- Organize unavailable because organize permission or folder capability is missing.

### Trust Boundaries

- Wasm owns validation, normalization, and host-effect planning.
- Host owns IMAP/SMTP/OAuth network calls, account credentials, mail persistence, attachment storage, contact lookup, notifications, and scheduled jobs.
- The UI owns human interaction state, optimistic feedback, local unsaved compose state, and clear recovery flows.
- The Synapp Broker owns AI routing, permission injection, audit, and agent identity.
- Admin surfaces own organization policy and capability grants, not user credentials.

### Sensitive Data Requirements

- Message bodies and attachments are mail data and require `mail:read` for access.
- Draft bodies require `mail:draft` for mutation.
- Sending requires `mail:send` and an enabled sender identity.
- Folder/rule mutation requires `mail:organize` unless the action is intrinsic to send/draft behavior.
- Credentials and OAuth tokens are never displayed, exported, passed to agents, or stored in Wasm memory beyond the immediate secure host input flow.

## 8. States And Recovery

### Global App States

- Initializing: app shell loads account summaries, permissions, and last selected mailbox.
- Ready: account/folder navigation, message list, and reading pane are interactive.
- No accounts: show account setup CTA and explain user-scoped setup.
- Permission limited: show readable app shell with disabled actions and explanations.
- Offline/degraded: show cached mail if host supports it, last sync time, and retry controls.
- Host effect unavailable: show that required Synapp host services are missing and identify impacted features.

### Loading States

- Account list loading.
- Folder tree loading.
- Message list loading with skeleton rows.
- Message body loading in reading pane.
- Search in progress with previous results preserved where possible.
- Draft autosave in progress.
- Send or schedule operation in progress with duplicate-submit protection.
- Connection test in progress with step-level progress.

### Empty States

- No accounts configured: `Add account` primary action.
- Inbox empty: explain no messages and offer sync now or compose.
- Folder empty: offer move/create rule only where useful.
- Search empty: show query, active filters, and broadening actions.
- Drafts empty: offer compose.
- Scheduled empty: explain no scheduled messages.
- Rules empty: offer create rule.

### Error States

- Account authentication failed: prompt to reconnect or update credentials.
- IMAP connection failed: show server/port/security guidance and retry.
- SMTP connection failed: show send disabled or receive-only status.
- Sync failed: preserve cached view, show last successful sync, retry.
- Message load failed: keep list selection and allow retry.
- Search failed: keep prior view and allow retry/clear.
- Draft save failed: keep local compose buffer and warn before close.
- Send failed: keep message as draft, show reason, offer retry/edit account.
- Schedule failed: keep draft unscheduled and offer retry.
- Folder/rule mutation failed: restore previous UI state and explain.

### Conflict States

- Draft changed elsewhere: show local and remote updated timestamps, allow keep local, load remote, duplicate draft, or discard local.
- Message moved/deleted elsewhere: remove or mark stale in list and show recovery message.
- Folder renamed/deleted elsewhere: refresh navigation and redirect to nearest valid mailbox.
- Rule changed elsewhere: ask user to reload before saving over remote changes.
- Account disabled/revoked during session: stop sync/send actions and show reconnect or choose another account.

### Recovery Behavior

- Never lose unsent compose content without an explicit discard confirmation.
- Failed optimistic actions must either roll back or show a pending state until reconciled.
- Retry actions must be available where the user can reasonably recover.
- Account and send errors should offer the shortest path to the relevant account setup section.
- The app must distinguish user-correctable errors from host/admin policy errors.

## 9. Feature Requirements

### Inbox And Message List

- Unified inbox and per-account inbox views.
- Favorite folders and account folder sections.
- Unread counts by folder.
- Focused inbox optional, with a visible way to switch to all mail or other messages.
- Conversation/thread view optional, with visible grouping state.
- Message rows include unread, flagged, attachment, sender, subject, snippet, date, account badge, and selection.
- Bulk action toolbar appears after selection.
- Refresh/sync action shows last sync time and current progress.

### Reading Pane

- Shows message header, recipients, date, subject, body, attachments, security warnings where host supplies them, and thread context.
- Supports reply, reply all, forward, archive, delete, move, mark read/unread, flag, print/download where host supports it.
- Threaded view collapses older messages but keeps expansion accessible.
- External images and remote content follow host privacy policy and must show user-visible blocked/loaded state if applicable.

### Compose

- New, reply, reply all, and forward compose modes.
- To, Cc, Bcc, From, subject, body, attachments, signature, formatting, discard, save draft, send, and schedule send.
- Recipient validation and duplicate warning.
- Undo send window from user compose preferences.
- Autosave state visible but quiet.
- Keyboard shortcuts may be supported but cannot be the only path.

### Search And Filters

- Full-text search with filters for account, folder, from, to, date, unread, flagged, attachments, and importance if available.
- Filter chips are visible and removable.
- Search results support opening, bulk actions, and save-as-rule entry point.

### Rules And Organization

- Create/list/delete rules at minimum, matching current plugin tools.
- Rule UI must explain conditions and actions in human language before save.
- Folder create/rename/delete are available from account folder menus and settings.
- Archive, delete, move, mark read/unread, and flag are available from list and reading pane.

### Account And Preference Settings

- Account setup, identity/signature management, preferences, rules, and agent activity are separate navigable settings areas.
- Preference labels must be specific; do not use vague grouping names such as `General Settings` as a user-facing section title.
- Account health must be visible both in settings and in the main nav when degraded.

## 10. Observable Acceptance Criteria

### Main App

- Given a user with at least one configured account, when they open Mail Client, then the first visible screen is an inbox workspace with folder navigation, message list, reading pane or message preview state, search, compose, and refresh controls.
- Given the app opens with no accounts, when the user lands on Mail Client, then they see a user-scoped `Add account` workflow and no admin-settings language.
- Given a user saves mail preferences, when they return to the inbox, then the preference changes affect visible behavior such as preview pane position, conversation grouping, focused inbox, or notifications.

### Account Setup

- Given a user manually adds an account, when required IMAP and SMTP fields are missing, then the app identifies the missing fields before testing.
- Given the user clicks `Test connection`, when IMAP succeeds and SMTP fails, then the app shows incoming passed, outgoing failed, and send disabled/receive-only options.
- Given credentials are saved, when the user edits the account later, then secrets are masked as host-stored and cannot be revealed.
- Given multiple accounts exist, when the user marks one default, then compose defaults to that account except when replying from another receiving account where reply identity rules apply.

### Mail Workflows

- Given messages are loading, when the mailbox request is in progress, then skeleton/loading state appears and actions that need selected messages are disabled.
- Given a message is selected, when the user archives it, then the list updates, unread/folder counts reconcile, and failure rolls back or shows pending recovery.
- Given a user composes a message, when autosave fails, then the compose buffer remains visible and the user is warned before closing.
- Given the user sends a message, when host send succeeds within the undo window, then the UI shows undo until the window expires and then shows Sent state.
- Given the user schedules a message, when they cancel before dispatch, then the scheduled job is canceled and the draft becomes editable.

### Search, Folders, Rules

- Given the user searches with filters, when results return, then active query and filters are visible and removable.
- Given search fails, when the host returns an error, then prior list/search state is preserved and retry is available.
- Given a user deletes a non-empty custom folder, when they confirm, then the app explains where contained messages will go before applying the action.
- Given a user creates a rule from selected messages, when the rule is saved, then the rule appears in the rules list with account scope and enabled state.

### Folder Persistence And Local Moves

- Given persisted mail documents exist for Inbox, Sent, Drafts, Archive, Junk, Trash, and custom folders, when the user opens the app after a fresh session, then every folder with persisted entries appears in navigation and its entries are visible without requiring a successful live IMAP fetch first.
- Given a live folder fetch succeeds, when the app receives fresher message metadata, then the persisted folder documents are updated through app-side persistence and the user sees refreshed rows, counts, and last-sync state.
- Given the Sent folder live fetch returns `502` / `BAD FETCH Invalid messageset`, when the user opens Sent, then persisted Sent entries remain visible, the Sent folder shows a recoverable sync warning, and other folders remain usable.
- Given one folder returns a live fetch error, when the user switches to another folder, then the failing folder state does not clear or block the other folder's persisted entries.
- Given the user moves one or more messages from a source folder to a target folder, when the local move is accepted, then the source list removes the messages, the target list includes them, both folder counts update, and each moved message records `sync_state: "pending_remote_move"`.
- Given a locally moved message has `sync_state: "pending_remote_move"`, when the user views the message row or message details, then the UI indicates the message is organized locally and waiting for remote sync instead of claiming final upstream IMAP completion.
- Given the app is reopened after a local move, when the persisted documents are queried, then the moved message remains in the target folder with `sync_state: "pending_remote_move"` until a future platform remote-move reconciliation clears or changes that state.
- Given local move persistence fails, when the user attempts to move messages, then the UI leaves the messages in their original folder, shows a recoverable error, and does not display a false pending-remote state.
- Given an agent with `mail:organize` moves messages folder-to-folder, when the action is accepted, then the headless result includes source folder, target folder, moved message ids, updated local state, and `sync_state: "pending_remote_move"`.
- Given an agent without `mail:organize` attempts a folder-to-folder move, when the tool is invoked, then the operation is denied and no persisted message documents or folder counts change.

### Agents And Permissions

- Given an agent has `mail:read`, when it tries to send, then the Wasm/tool response denies the operation and the UI can show that send permission is not granted.
- Given an agent creates a draft, when the user opens Drafts, then the draft is visible with available agent attribution metadata.
- Given an agent sends or schedules mail with permission, when the user reviews recent activity, then the action is auditable by account, identity, recipient, timestamp, and originating agent where host metadata exists.
- Given send capability is revoked during a session, when the user or agent attempts to send, then the UI disables send and surfaces the revocation reason.

### States And Recovery

- Given account authentication fails, when the user opens the app, then affected account nav shows degraded status and links to reconnect.
- Given a draft conflict occurs, when the user attempts to save, then the app offers keep local, load remote, duplicate, or discard.
- Given host mail effects are unavailable, when the user opens the app, then the app identifies impacted features and does not present impossible actions as available.
- Given a destructive action is requested, when it is permanent or broad in scope, then the app requires confirmation and describes the outcome.

## 11. Non-Goals For Stage 1

- This spec does not define pixel-perfect visual design, component styling, or final copy strings.
- This spec does not require the Wasm module to open IMAP, SMTP, OAuth, or attachment network connections.
- This spec does not add AI reasoning or prompts inside the app.
- This spec does not mandate offline mail storage unless the host platform provides it.
- This spec does not require full Exchange/Graph/EWS support beyond OAuth/provider placeholders and host-owned connectors.

## 12. Open Questions For Later Pipeline Stages

- Which OAuth providers are supported by the Synapp host at launch, and what provider labels should appear in account setup?
- Does the host support true SMTP non-delivery testing, or should outgoing connection test be limited to authentication and capability validation?
- What attachment upload/download limits and MIME safety policies should the UI expose?
- Should focused inbox be implemented in host mail classification, user rules, or postponed until a host classifier exists?
- What audit metadata does the Broker expose for agent-created drafts, sends, moves, deletes, and rules?
- Does the host provide cached/offline mail snapshots, or should offline state always be treated as degraded online-only access?
- When will the platform expose true IMAP move/copy/delete and reconciliation effects beyond `ImapFetch` and `SmtpSend`, and what success/failure states should clear `pending_remote_move`?
- Should account setup live entirely inside the app route, the Synapp user settings shell, or both as deep-linked surfaces?

## 13. Pipeline Handoff Notes

Architect should translate this UX spec into updated contracts for:

- Real UI entrypoint behavior for the main route instead of host-rendered configuration schemas.
- User-scoped account setup and preference schemas with explicit section names.
- Host effects for account connection tests, credential writes, OAuth connect/disconnect, sync status, and provider capability discovery.
- App-side persistence using `PutDocument`/`QueryDocuments` for durable folder/message snapshots, plus local folder-to-folder move semantics with `sync_state: "pending_remote_move"`.
- Platform-required upstream IMAP move/sync reconciliation, because current mail proxy capability is limited to `ImapFetch` and `SmtpSend` and live Sent can return `BAD FETCH Invalid messageset`.
- Existing mail tool coverage alignment for read, search, get thread, mark/flag, draft/update/discard, send/reply/forward, move/delete/archive, folders, unread counts, rules, schedule/cancel send.

UI-Designer should produce an inbox-first layout and account setup flow that covers all loading, empty, error, conflict, and recovery states above.

Developer should keep the Wasm app sandboxed and contract-driven, update `plugin.json` and `synapp.app.json` together whenever signatures or exposed UI schemas change, and ensure every Wasm operation has a corresponding human UI path.