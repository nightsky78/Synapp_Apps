# Mail Client UI Copy Reference

Stage: 3 - UI Designer  
App: `apps/first-party/mail-client`  
Date: 2026-05-08

## 1. Voice And Terminology

Use direct product language. Do not describe platform internals unless the user needs to understand ownership or recovery.

- Use "account" for a user's mail account.
- Use "host secure field" or "Synapp host secure storage" when explaining credentials.
- Use "receive-only" when IMAP works and SMTP/send does not.
- Use "agent permissions" and "agent activity" only for permission/audit surfaces. Do not imply the app contains AI logic.
- Avoid "admin settings" for account setup. Account setup is user-scoped.

## 2. Command Bar

| Element | Copy |
| --- | --- |
| App name | Synapp Mail |
| Compose button | Compose |
| Account selector label | Account scope |
| Search placeholder | Search mail, from, subject, attachment |
| Clear search | Clear search |
| Filter chips | All, Unread, Flagged, Attachments |
| Refresh | Refresh |
| Settings | Settings |
| Return from settings | Inbox |
| Permission badge labels | No Access, Read, Draft, Send, Organize |

Disabled/tooltips:

- Compose with no accounts: "Add an account before composing."
- Compose on receive-only account: "Sending is disabled until SMTP passes."
- Send permission missing: "Send permission required."
- Organize permission missing: "Organize permission required."

## 3. First-Run Account Setup

Headers:

- "Add your first mail account"
- "Add account"

Intro copy:

- "Accounts are stored for the signed-in user. Credentials are captured by Synapp host secure fields and passed to this app only as opaque references."

Sections and labels:

- Identity
- Account label
- Display name
- Email address
- Reply-to
- Setup path
- Manual IMAP/SMTP
- Incoming IMAP
- IMAP host
- Port
- Security
- Username
- Outgoing SMTP
- SMTP host
- Use incoming credential for SMTP
- SMTP username
- Authentication
- Method
- Secure credential reference
- Sync and sender defaults
- Sync interval
- Initial sync
- Attachments
- Keep offline cache when host supports it
- Make this the default sender
- Connection test

Hints:

- Reply-to: "Optional. Leave blank to reply from the account address."
- Secure credential reference: "The actual password is entered through a host secure field. This UI stores only an opaque secret_input_ref."
- OAuth placeholder: "Host-owned OAuth is a placeholder until the platform connector completes authorization and returns an opaque connection reference."

Actions:

- Start host authorization
- Test incoming only
- Test connection
- Save account
- Save receive-only

Success toasts:

- "Connection test planned"
- "Account saved"
- "Account saved as receive-only"
- "[provider] authorization started by host"

Failure toasts:

- "Provider capabilities unavailable: [reason]"
- "Enter an email address before OAuth setup"
- "OAuth setup failed: [reason]"
- "Connection test failed: [reason]"
- "Save failed: [reason]"

Connection state labels:

- not tested
- testing
- passed
- warning
- failed

## 4. Inbox And Message List

Mailbox header:

- "[folder name]"
- "[count] messages"
- "[count] messages - [unread] unread"

Bulk toolbar:

- Select
- "[count] selected"
- Read
- Unread
- Archive
- Delete
- Rule

Empty states:

- Title: "No messages"
- Body: "This view has no messages. Try another filter, folder, or search scope."

Loading states:

- "Loading emails..."
- "Loading folders..."

Pagination:

- "Page [current] of [total]"
- "Prev"
- "Next"

Row accessibility labels:

- "Email from [sender]: [subject]"
- "Select message [subject]"
- "Flag email"
- "Remove flag"

Success toasts:

- "Marked as read"
- "Marked as unread"
- "Archived selected messages"
- "Moved selected messages to Trash"
- "Archived"
- "Moved to Trash"

Failure toasts:

- "Failed to load emails: [reason]"
- "Mark failed: [reason]"
- "Archive failed: [reason]"
- "Delete failed: [reason]"
- "Flag failed: [reason]"

## 5. Reading Pane

Empty state:

- "Select a message to read"

Labels:

- From
- To
- CC
- Attachments ([count])
- High importance
- Low importance

Actions:

- Reply
- Reply All
- Forward
- Flag
- Unflag
- Archive
- Delete

Permission tooltips:

- "Send permission required"
- "Organize permission required"

Failure toasts:

- "Flag failed: [reason]"

## 6. Compose

Dialog labels:

- New Message
- Reply
- Reply All
- Forward

Fields:

- From
- To
- CC
- BCC
- Subj
- Include default signature

Placeholders:

- "Add recipients..."
- "Subject"
- "Write your message here..."

Actions:

- Send
- Schedule
- Tomorrow noon
- Save draft
- Discard

Status text:

- "Saving..."
- "Draft saved"
- "Save failed"
- "Sending..."

Success toasts:

- "Draft saved"
- "Message sent!"
- "Scheduled for tomorrow noon"

Validation/errors:

- "Add at least one recipient"
- "Subject is required"
- "Invalid email address"
- "You do not have Send permission"
- "This account is receive-only until SMTP passes"
- "Draft save failed: [reason]"
- "Send failed: [reason]"
- "Schedule failed: [reason]"

## 7. Settings Sections

### Accounts

Header:

- "Accounts"

Description:

- "User-scoped account setup, connection health, sync state, and secure credential references."

Actions:

- Add account
- Test connection
- Sync now
- Remove

Confirmation:

- "Remove this account from your mail workspace? Local cache and scheduled sends will be cleaned up by the host."

Toasts:

- "Account removal planned"
- "Remove failed: [reason]"
- "Host connection test planned from account card"
- "Sync requested"

### Identities And Signatures

Header:

- "Identities and signatures"

Description:

- "Aliases, send-as addresses, default sender rules, and reusable signatures."

Actions:

- Add identity

Toasts:

- "Failed to load identities: [reason]"
- "Identity save surface is ready for host wiring"

### Preferences

Header:

- "Preferences"

Description:

- "Reading, compose, notification, and sync defaults. These are user preferences, not admin settings."

Sections:

- Reading
- Compose
- Notifications
- Sync defaults

Actions:

- Save preferences

Toasts:

- "Preferences saved"
- "Preferences unavailable: [reason]"
- "Save failed: [reason]"

### Rules

Header:

- "Rules"

Description:

- "User-scoped automation for moving, flagging, marking, archiving, and deleting mail."

Empty state:

- "No rules yet. Create one below."

Actions:

- Create rule
- Delete rule

Validation/errors:

- "Rule name and condition value are required"
- "Failed to load rules: [reason]"
- "Failed to create rule: [reason]"
- "Delete failed: [reason]"

Success:

- "Rule created"
- "Rule deleted"

### Agent Permissions And Activity

Header:

- "Agent permissions and activity"

Description:

- "Review what mail capabilities the current principal can use. This view contains permission and audit information only; it contains no AI logic."

Empty activity:

- "No agent mail activity in this local session"
- "When the host supplies audit events, reviewed sends, draft changes, rules, and organize actions appear here."

### Legacy Settings Migration

Header:

- "Legacy settings migration"

Description:

- "Move old schema-form settings into user preferences and account reconnect prompts without preserving raw credentials."

Actions:

- Inspect legacy settings
- Migrate user settings

Toasts:

- "Legacy settings inspected"
- "Legacy settings migration planned"
- "Inspection failed: [reason]"
- "Migration failed: [reason]"

## 8. Accessibility Copy Rules

- Button labels must stay action-oriented and short.
- Inputs keep visible labels; placeholders are examples only.
- Error messages name the failed action and include recovery context when possible.
- Disabled action titles must explain the missing permission, account state, or setup requirement.
- Toasts should not be the only long-term record for destructive or security-sensitive actions; account/rule/activity surfaces should retain state.

## 9. Platform Constraint Copy

Use this only where needed in developer-facing or settings/migration surfaces:

- "The main Mail Client route uses the packaged React workspace. A host-rendered JSON settings schema is not used for the inbox."
- "Credentials and OAuth tokens are managed by Synapp host secure services. This app receives only opaque references."
- "Agent permissions are shown for review and audit. The Mail Client UI does not contain AI prompts, routing, or model logic."
