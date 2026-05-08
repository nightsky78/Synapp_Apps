# Mail Client UI Specification

Stage: 3 - UI Designer  
App: `apps/first-party/mail-client`  
Date: 2026-05-08

## 1. UI Contract Decision

The Mail Client main experience is the packaged React workspace at `ui/dist/index.html`, entered through `app://mail-client/inbox`. The app must not publish a settings-style `ui_schemas.main` JSON Schema for the main route. That schema is interpreted by the host as `schema_type=json_schema` and routes to `SchemaFormRenderer`, which saves user app settings instead of rendering a mailbox.

Immediate app-owned contract:

- `synapp.app.json` keeps `ui.entrypoint: "ui/dist/index.html"`.
- Navigation targets `app://mail-client/inbox`.
- `ui_schemas.main` is removed/deprecated and must not contain mailbox, preview pane, focused inbox, page size, or search fields.
- User account setup, identities, signatures, preferences, rules, migration, and agent permission review live inside the React UI under settings sections.
- Admin/policy schemas, if added later, may contain provider/policy flags only. They must not contain per-user IMAP/SMTP hosts, usernames, aliases, signatures, OAuth connection ids, or credential references.

Platform constraints remain visible in the UI design:

- The host `page_layout_schema` renderer currently previews components and cannot render a functional mailbox.
- The generic app action endpoint is not fully wired for a native host-rendered mailbox.
- The React bridge calls Wasm tools through `window.__synapp.invoke`; local development uses stubs only.

## 2. Primary Journeys

### First Run: No Accounts

Entry state: user opens Mail Client and `list_accounts` returns an empty list.

Screen: Add your first mail account.

Required UI:

- Identity fields: account label, display name, email address, optional reply-to.
- Setup path segmented control: Manual IMAP/SMTP plus OAuth placeholders returned by `get_provider_capabilities`.
- Manual incoming IMAP fields: host, port, security, username, authentication method, secure credential reference.
- Manual outgoing SMTP fields: host, port, security, username, same-credential toggle, secure credential reference when separate.
- Sync/defaults: interval, initial range, attachment behavior, offline cache, default sender.
- Connection test panel with separate incoming, outgoing, and folder mapping states.
- Save active and save receive-only actions.

States:

| State | Behavior |
| --- | --- |
| Idle | Shows empty setup form with manual IMAP/SMTP selected. |
| Loading | Shows provider capability loading state. |
| Success | `complete_account_setup` succeeds, account list refreshes, route returns to inbox. |
| Failure | Field/action error appears in toast and connection test panel remains visible. |
| Empty | No-account state is the account setup screen, not an empty inbox. |

Receive-only behavior: if incoming succeeds and outgoing fails or is intentionally disabled, the user may save as receive-only. Compose and send are disabled for that account with an explicit message.

Credential behavior: no password fields are owned by this app. The UI collects only `secret_input_ref` or OAuth placeholder refs from host-owned secure controls.

### Returning User: Inbox Triage

Default screen: inbox-first mail workspace.

Layout:

- Top command bar: app name, Compose, account switcher, search, filter chips, current folder, health dot, Refresh, Settings, permission badge.
- Left navigation: account health, receive-only notice, favorites, system folders, custom folders, folder creation, settings shortcuts.
- Message list: mailbox title, counts, selection toolbar, filterable list rows, pagination.
- Reading pane: message headers, body, attachments, reply/reply all/forward, flag, archive, delete.
- Compose: docked dialog with From account, recipients, Cc/Bcc, subject, body, signature toggle, save draft, schedule, send, discard.

Message list states:

| State | Behavior |
| --- | --- |
| Idle | Rows show sender, subject, preview, date, unread state, importance, attachments, flag, selection checkbox. |
| Loading | Centered spinner while `read_emails` resolves. |
| Success | Rows update from snapshot; unread counts and active selection remain coherent. |
| Failure | Toast explains the failed operation; destructive optimistic actions reload the mailbox. |
| Empty | Empty state suggests changing filter, folder, or search scope. |

Bulk action toolbar:

- Select all visible.
- Mark read/unread via `mark_read`.
- Archive via `archive_email`.
- Delete via `delete_email`.
- Create rule shortcut into user-scoped Rules section.
- Organize actions are disabled unless permission allows them.

Search/filter:

- Search box calls `search_emails` with a debounced query.
- Filter chips: All, Unread, Flagged, Attachments.
- Results use the same row pattern and preserve selectable/openable messages.
- Empty search tells the user to broaden filters or scope.

### Compose, Reply, Forward, Schedule

Compose surfaces `draft_email`, `update_draft`, `discard_draft`, `send_email`, `reply_email`, `forward_email`, `schedule_send`, and `cancel_scheduled_send` concepts.

Required behavior:

- From selector lists configured accounts and labels receive-only accounts.
- Send and schedule are blocked when permission is insufficient or the selected account is receive-only.
- Recipients are chip inputs with basic email validation.
- Draft autosaves every 10 seconds when draft permission exists.
- Closing with discard calls `discard_draft` when a draft id exists.
- Schedule offers a starter option and uses the contract field `scheduled_for`.
- Signature toggle maps to the selected identity/signature surface; host/plugin implementation may later insert exact signature bodies.

States:

| State | Behavior |
| --- | --- |
| Idle | Form is editable and focused in message body. |
| Loading | Send/schedule button shows progress and is disabled. |
| Success | Toast confirms sent, scheduled, saved, or discarded; compose closes when appropriate. |
| Failure | Toast explains validation, permission, receive-only, draft, send, or schedule failure. |
| Empty | New compose starts with empty recipients/subject/body and selected default account. |

### User-Scoped Settings

Settings sections inside React UI:

- Accounts: account cards, health, sync now, test connection, remove, add account.
- Identities: sender aliases/send-as records, verification state, default identity, signature list/editor.
- Preferences: reading, compose, notifications, sync defaults.
- Rules: conditions/actions for user-scoped mail automation.
- Agents: current permission, available host effects, activity/audit placeholder.
- Migration: inspect and migrate legacy settings into user records without migrating raw secrets.

These are not admin settings. The signed-in user owns them.

## 3. Wasm Function To UI Surface Map

| Contract Area / Export | Human UI Surface |
| --- | --- |
| `list_accounts` | Account switcher, sidebar account summary, Accounts settings list, first-run no-account gate. |
| `get_account_status` | Health dot, account card status, receive-only/degraded notices, refresh/sync actions. |
| `list_mailboxes` | Sidebar favorites/system/custom folders and folder counts. |
| `read_emails` | Message list for selected folder, pagination, loading/empty/error states. |
| `search_emails` | Top search and SearchView results. |
| `get_email` | Reading pane message detail. |
| `get_thread` | Reading pane/thread-ready contract; replies/forwards preserve thread context. |
| `mark_read` | Message open auto-mark, bulk read/unread. |
| `flag_email` | Row flag and reading pane flag action. |
| `draft_email`, `update_draft`, `discard_draft` | Compose autosave, Save draft, Discard. |
| `send_email`, `reply_email`, `forward_email` | Compose Send, Reply, Reply all, Forward flows. |
| `move_email` | Bulk toolbar/folder move design surface; folder menu reserved for implementation expansion. |
| `delete_email`, `archive_email` | Reading pane and bulk actions. |
| `create_folder`, `rename_folder`, `delete_folder` | Folder manager and custom folder inline actions. |
| `get_unread_count` | Folder unread counts and mailbox header counts. |
| `create_rule`, `list_rules`, `delete_rule` | Rules settings section and create-rule-from-selection shortcut. |
| `schedule_send`, `cancel_scheduled_send` | Compose schedule menu and Scheduled/restore design contract. |
| `get_provider_capabilities` | Account setup provider/OAuth/manual options and policy hints. |
| `validate_account_setup` | Account setup validation before test/save. |
| `plan_connection_test` | Connection test panel for IMAP, SMTP, folder mapping. |
| `complete_account_setup` | Save active/receive-only/disabled/incomplete account. |
| `begin_oauth_account_setup`, `complete_oauth_account_setup`, `disconnect_oauth_account` | OAuth placeholder controls and account disconnect lifecycle. |
| `remove_account` | Account removal card action with cleanup explanation. |
| `list_identities`, `save_identity`, `delete_identity` | Identities settings section. |
| `save_signature`, `delete_signature` | Signature list/editor. |
| `get_user_preferences`, `save_user_preferences` | Preferences settings section. |
| `inspect_legacy_settings`, `migrate_legacy_settings` | Migration settings section. |

Gate note: this Stage 3 UI exposes a human surface for every planned contract area. Some new account/preference/migration exports still require Stage 4+ Wasm/plugin implementation and host action wiring.

## 4. Accessibility Requirements

- Use semantic landmarks: command bar, navigation, main, section, dialog, toolbar.
- Every form control has a visible label or accessible label.
- All icon-only or compact controls expose `aria-label`/`title`.
- Keyboard access: Tab reaches every control; Enter/Space activates buttons; Enter opens selected message rows; Escape should close modal/dialog surfaces in implementation follow-up.
- Focus ring remains visible via `:focus-visible`.
- Dialogs use `role="dialog"` and `aria-modal="true"`.
- Toasts remain visible long enough to read and should be announced through live-region behavior in follow-up hardening.
- Text and controls maintain readable contrast against white, gray, and accent surfaces.
- Bulk selection checkboxes have message-specific labels.
- Disabled actions include a reason in title/toast copy when the user attempts a related flow.

## 5. Component Behavior Notes

### `Toolbar`

- Controls route between inbox and settings.
- Search is debounced by app context.
- Account selector changes active account and resets mailbox state.
- Health dot reflects account connection state.
- Compose is disabled when no account exists or the active account is receive-only.

### `Sidebar`

- Shows account identity, health, receive-only warning, folder groups, and user settings shortcuts.
- Custom folder actions are available only with organize permission.
- Folder selection returns to inbox route and resets search/selection.

### `EmailList`

- Applies local quick filters to the loaded snapshot.
- Bulk actions call Wasm contracts and reload mailbox on success/failure recovery.
- Empty state is filter-aware.

### `ReadingPane`

- Shows selected message details, attachments, and permission-gated actions.
- Reply/reply all/forward open compose with context.
- Archive/delete are optimistic and recover by reloading on failure.

### `ComposePane`

- Supports From account, To/Cc/Bcc chips, subject/body, signature toggle, autosave, send, schedule, discard.
- Uses host/Wasm contracts only; no direct SMTP/OAuth/network logic.

### `SettingsCenter`

- Owns user-scoped account setup, identities, preferences, rules, agent review, and migration.
- Keeps credentials as opaque references.
- Shows connection test states: not tested, testing, passed, warning, failed.

## 6. Stage 3 Gates

| Gate | Status | Notes |
| --- | --- | --- |
| Main route is inbox-first, not settings form | Pass | `ui_schemas.main` removed from app manifest; React entrypoint owns `/inbox`. |
| Per-user account setup is clear | Pass | Account setup lives in React settings/no-account flow, with manual IMAP/SMTP and OAuth placeholders. |
| Every Wasm function has a UI surface | Pass by UI design | Existing and planned contract areas are mapped above. New exports need implementation in later stages. |
| Accessibility requirements documented | Pass | Baseline states and semantic requirements included. |
| UI components implement architect contracts | Pass for UI layer | Bridge invokes contract names and local stubs expose planned data. Host/platform wiring remains constrained. |
| No AI logic in UI | Pass | Agent section is permission/activity review only. |
