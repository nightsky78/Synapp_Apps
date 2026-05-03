# Mail Client App — Function Contracts

> **Purpose:** Explicit contracts for every exported function, including signatures, parameters, return types, error semantics, and permission requirements.
>
> **Golden Rule:** These contracts are the unbreakable interface. Any deviation in function signature or return type must be reflected immediately in `plugin.json`.

---

## Data Structures (Input/Output Types)

### Email
```rust
{
  "id": String,                    // Unique email identifier
  "message_id": String,            // RFC 5322 Message-ID header
  "user_id": String,               // Owner (always scoped to current user)
  "from": String,                  // Sender email address
  "to": Vec<String>,               // Recipients
  "cc": Vec<String>,               // Carbon copy recipients
  "bcc": Vec<String>,              // Blind carbon copy recipients
  "subject": String,               // Email subject
  "body": String,                  // Email body (HTML or plain text)
  "received_at": i64,              // Unix timestamp (seconds)
  "sent_at": i64,                  // Unix timestamp (seconds), 0 if draft
  "is_read": bool,                 // Read/unread status
  "folder_id": String,             // Current folder (INBOX, DRAFTS, SENT, etc.)
  "thread_id": String,             // Thread grouping identifier
  "has_attachments": bool,         // Whether email has attachments
  "importance": String             // "low", "normal", or "high"
}
```

### Folder
```rust
{
  "folder_id": String,             // Unique identifier (e.g., "inbox", "custom_1")
  "name": String,                  // Display name (e.g., "Inbox", "Project Alpha")
  "type": String,                  // "system" or "custom"
  "unread_count": u32,             // Number of unread emails
  "total_count": u32               // Total emails in folder
}
```

### Draft
```rust
{
  "draft_id": String,              // Unique draft identifier
  "to": Vec<String>,               // Recipients
  "cc": Vec<String>,               // Carbon copy
  "bcc": Vec<String>,              // Blind carbon copy
  "subject": String,               // Subject line
  "body": String,                  // Email body
  "created_at": i64,               // Unix timestamp when draft was created
  "updated_at": i64,               // Unix timestamp of last modification
  "scheduled_send_at": i64         // Optional: when to send (0 = send immediately)
}
```

### PermissionSet
```rust
{
  "permission_level": String,      // "none", "read", "draft", or "send"
  "user_id": String,               // The authenticated user
  "agent_id": String,              // The requesting agent
  "granted_at": i64,               // Unix timestamp when permission was granted
  "expires_at": i64                // Unix timestamp when permission expires (0 = never)
}
```

### Error Codes
```rust
enum MailClientError {
  PermissionDenied,                // Agent lacks required permission
  EmailNotFound,                   // Email id does not exist
  DraftNotFound,                   // Draft id does not exist
  FolderNotFound,                  // Folder id does not exist
  InvalidRecipient,                // Email address format invalid
  InvalidInput,                    // Malformed parameter (e.g., negative offset)
  DraftAlreadySent,                // Attempt to send a draft twice
  TooManyRecipients,               // Exceeds platform limit (e.g., >100)
  InternalError                    // Unexpected platform error
}
```

---

## Function Contracts

### 1. `read_emails`

**Purpose:** Fetch a paginated list of emails from a specified mailbox.

**Permission Required:** `read`

**Function Signature (Rust):**
```rust
#[no_mangle]
pub extern "C" fn read_emails(
    user_id: *const u8,            // UTF-8 JSON string
    user_id_len: usize,
    mailbox: *const u8,            // UTF-8 JSON string ("INBOX", "SENT", "DRAFTS", etc.)
    mailbox_len: usize,
    offset: u32,                   // Pagination offset (0-indexed)
    limit: u32,                    // Max results (1-500)
    permission: *const u8,         // UTF-8 JSON string ("none", "read", "draft", "send")
    permission_len: usize
) -> *mut u8                       // JSON-encoded Vec<Email>
```

**Input Parameters:**
- `user_id`: Authenticated user identifier (provided by host, validated JWT claim)
- `mailbox`: Folder identifier (e.g., "INBOX", "DRAFTS", custom folder ID)
- `offset`: Pagination starting index (0 = first email)
- `limit`: Max emails to return (clamped to 500)
- `permission`: Current agent permission level

**Return Value:**
```json
{
  "ok": {
    "emails": [
      { Email object },
      { Email object }
    ],
    "total_count": 1234,            // Total emails in mailbox (for pagination UI)
    "has_more": true
  }
}
```
OR
```json
{
  "err": {
    "code": "PermissionDenied",
    "message": "Agent lacks 'read' permission"
  }
}
```

**Error Semantics:**
- `PermissionDenied`: Agent does not have `read` or higher permission
- `FolderNotFound`: Mailbox does not exist for this user
- `InvalidInput`: Offset or limit out of bounds (offset >= total_count or limit > 500)
- `InternalError`: Platform storage unreachable

**Side Effects:** None (pure query)

**Platform Effects Required:** `QueryEmailStore(user_id, mailbox, offset, limit)`

**Notes:**
- Emails are sorted by `received_at` descending (newest first)
- Only unread emails are returned if mailbox is a search result (optimized for triage)
- Large offsets may incur latency; UI should implement cursor-based pagination

---

### 2. `get_email`

**Purpose:** Fetch a single email by ID, including full body and metadata.

**Permission Required:** `read`

**Function Signature (Rust):**
```rust
#[no_mangle]
pub extern "C" fn get_email(
    user_id: *const u8,
    user_id_len: usize,
    email_id: *const u8,           // UTF-8 JSON string
    email_id_len: usize,
    permission: *const u8,
    permission_len: usize
) -> *mut u8                       // JSON-encoded Result<Email, Error>
```

**Input Parameters:**
- `user_id`: Authenticated user
- `email_id`: Unique email identifier
- `permission`: Agent permission level

**Return Value:**
```json
{
  "ok": { Email object }
}
```
OR
```json
{
  "err": {
    "code": "EmailNotFound",
    "message": "Email id=abc123 does not exist or belongs to different user"
  }
}
```

**Error Semantics:**
- `PermissionDenied`: Agent lacks `read` permission
- `EmailNotFound`: Email does not exist or belongs to different user
- `InternalError`: Storage error

**Side Effects:** None (pure query)

**Platform Effects Required:** `QueryEmailStore(user_id, email_id)`

**Notes:**
- Mark email as read in side-effect (host responsibility after successful call)
- Return full HTML body, not truncated

---

### 3. `draft_email`

**Purpose:** Create a new email draft (not sent).

**Permission Required:** `draft`

**Function Signature (Rust):**
```rust
#[no_mangle]
pub extern "C" fn draft_email(
    user_id: *const u8,
    user_id_len: usize,
    to: *const u8,                 // UTF-8 JSON array of strings
    to_len: usize,
    cc: *const u8,                 // UTF-8 JSON array of strings (can be empty)
    cc_len: usize,
    bcc: *const u8,                // UTF-8 JSON array of strings (can be empty)
    bcc_len: usize,
    subject: *const u8,            // UTF-8 JSON string
    subject_len: usize,
    body: *const u8,               // UTF-8 JSON string (HTML or plain text)
    body_len: usize,
    permission: *const u8,
    permission_len: usize
) -> *mut u8                       // JSON-encoded Result<Draft, Error>
```

**Input Parameters:**
- `user_id`: Authenticated user
- `to`: Array of recipient email addresses (at least 1 required)
- `cc`: Array of CC recipients (optional, can be empty)
- `bcc`: Array of BCC recipients (optional, can be empty)
- `subject`: Email subject (can be empty)
- `body`: Email body (can be empty)
- `permission`: Agent permission level

**Return Value:**
```json
{
  "ok": {
    "draft_id": "draft_xyz789",
    "to": ["recipient@company.com"],
    "cc": [],
    "bcc": [],
    "subject": "Hello",
    "body": "...",
    "created_at": 1704067200,
    "updated_at": 1704067200,
    "scheduled_send_at": 0
  }
}
```
OR
```json
{
  "err": {
    "code": "InvalidRecipient",
    "message": "Invalid email format: 'not-an-email'"
  }
}
```

**Error Semantics:**
- `PermissionDenied`: Agent lacks `draft` or higher permission
- `InvalidRecipient`: Email address format invalid in `to`, `cc`, or `bcc`
- `InvalidInput`: `to` array is empty
- `TooManyRecipients`: More than 100 recipients combined (to + cc + bcc)
- `InternalError`: Draft storage failed

**Side Effects:** Creates ephemeral draft in platform store (auto-expires after 7 days if unsent)

**Platform Effects Required:** `CreateDraft(user_id, to, cc, bcc, subject, body)`

**Notes:**
- Draft is stored in DRAFTS folder (not visible in INBOX)
- `draft_id` is valid for 7 days; after expiration, `send_email()` will fail with `DraftNotFound`
- Multiple versions of a draft can coexist if caller loses the `draft_id`; platform should garbage-collect

---

### 4. `send_email`

**Purpose:** Send a draft email to recipients. Transitions draft from DRAFTS to SENT folder.

**Permission Required:** `send`

**Function Signature (Rust):**
```rust
#[no_mangle]
pub extern "C" fn send_email(
    user_id: *const u8,
    user_id_len: usize,
    draft_id: *const u8,           // UTF-8 JSON string
    draft_id_len: usize,
    permission: *const u8,
    permission_len: usize
) -> *mut u8                       // JSON-encoded Result<EmailId, Error>
```

**Input Parameters:**
- `user_id`: Authenticated user (must own the draft)
- `draft_id`: Identifier from `draft_email()`
- `permission`: Agent permission level

**Return Value:**
```json
{
  "ok": {
    "email_id": "email_abc123",
    "sent_at": 1704067260
  }
}
```
OR
```json
{
  "err": {
    "code": "DraftNotFound",
    "message": "Draft draft_xyz789 not found or has expired"
  }
}
```

**Error Semantics:**
- `PermissionDenied`: Agent lacks `send` permission
- `DraftNotFound`: Draft does not exist, has expired, or belongs to different user
- `DraftAlreadySent`: Draft was already sent (idempotency check)
- `InvalidRecipient`: Recipient list is now invalid (user deleted all recipients between draft and send)
- `InternalError`: SMTP service unreachable or other platform error

**Side Effects:** 
- Sends email via platform mail service
- Moves draft to SENT folder with updated `sent_at` timestamp
- Commits email record to database

**Platform Effects Required:** 
- `SendEmail(draft_id, to, cc, bcc, subject, body)`
- `CommitEmail(email_id, user_id, sent_at)`

**Notes:**
- Idempotent: Calling `send_email()` twice with same draft_id should succeed once, fail with `DraftAlreadySent` on second attempt
- No retry logic in Wasm; host handles retries
- Recipient validation is re-checked before sending (may reject if addresses are now invalid)

---

### 5. `list_folders`

**Purpose:** Enumerate all folders for the user.

**Permission Required:** `read` (minimum)

**Function Signature (Rust):**
```rust
#[no_mangle]
pub extern "C" fn list_folders(
    user_id: *const u8,
    user_id_len: usize,
    permission: *const u8,
    permission_len: usize
) -> *mut u8                       // JSON-encoded Result<Vec<Folder>, Error>
```

**Input Parameters:**
- `user_id`: Authenticated user
- `permission`: Agent permission level

**Return Value:**
```json
{
  "ok": {
    "folders": [
      {
        "folder_id": "inbox",
        "name": "Inbox",
        "type": "system",
        "unread_count": 42,
        "total_count": 1234
      },
      {
        "folder_id": "sent",
        "name": "Sent",
        "type": "system",
        "unread_count": 0,
        "total_count": 567
      },
      {
        "folder_id": "custom_1",
        "name": "Project Alpha",
        "type": "custom",
        "unread_count": 5,
        "total_count": 89
      }
    ]
  }
}
```

**Error Semantics:**
- `PermissionDenied`: Agent lacks `read` permission
- `InternalError`: Folder enumeration failed

**Side Effects:** None (pure query)

**Platform Effects Required:** `QueryFolderList(user_id)`

**Notes:**
- System folders (INBOX, SENT, DRAFTS, TRASH, JUNK, ARCHIVE) are always present
- Unread count includes only direct emails, not threaded replies
- Folder order: system folders first, then custom folders alphabetically

---

### 6. `move_email`

**Purpose:** Move an email from one folder to another.

**Permission Required:** `draft` (for moving to/from DRAFTS) or `send` (for all other moves)

**Function Signature (Rust):**
```rust
#[no_mangle]
pub extern "C" fn move_email(
    user_id: *const u8,
    user_id_len: usize,
    email_id: *const u8,
    email_id_len: usize,
    target_folder_id: *const u8,  // UTF-8 JSON string (e.g., "trash", "custom_1")
    target_folder_id_len: usize,
    permission: *const u8,
    permission_len: usize
) -> *mut u8                       // JSON-encoded Result<(), Error>
```

**Input Parameters:**
- `user_id`: Authenticated user
- `email_id`: Email to move
- `target_folder_id`: Destination folder
- `permission`: Agent permission level

**Return Value:**
```json
{
  "ok": null
}
```
OR
```json
{
  "err": {
    "code": "FolderNotFound",
    "message": "Target folder custom_1 does not exist"
  }
}
```

**Error Semantics:**
- `PermissionDenied`: Agent lacks required permission for target folder
- `EmailNotFound`: Email does not exist or belongs to different user
- `FolderNotFound`: Target folder does not exist or belongs to different user
- `InternalError`: Move operation failed

**Side Effects:** Email record updated with new `folder_id`

**Platform Effects Required:** `UpdateEmail(email_id, { folder_id: target_folder_id })`

**Notes:**
- Moving to TRASH soft-deletes email (kept for 30 days, then auto-purged)
- Moving from DRAFTS to SENT requires the email to have `sent_at` set (only sent drafts can move to SENT)
- Recursive folder moves (e.g., moving INBOX itself) are not supported

---

### 7. `search_emails`

**Purpose:** Full-text search across emails matching query string.

**Permission Required:** `read`

**Function Signature (Rust):**
```rust
#[no_mangle]
pub extern "C" fn search_emails(
    user_id: *const u8,
    user_id_len: usize,
    query: *const u8,              // UTF-8 JSON string (search expression)
    query_len: usize,
    limit: u32,                    // Max results (1-500)
    permission: *const u8,
    permission_len: usize
) -> *mut u8                       // JSON-encoded Result<Vec<Email>, Error>
```

**Input Parameters:**
- `user_id`: Authenticated user
- `query`: Search expression (free-form text or operators like `from:alice@company.com`, `subject:"urgent"`, `before:2024-01-01`)
- `limit`: Max results to return (clamped to 500)
- `permission`: Agent permission level

**Return Value:**
```json
{
  "ok": {
    "emails": [
      { Email object },
      { Email object }
    ],
    "matched_count": 1234,         // Total matches (may exceed limit)
    "has_more": true
  }
}
```

**Error Semantics:**
- `PermissionDenied`: Agent lacks `read` permission
- `InvalidInput`: Query syntax invalid or limit out of bounds
- `InternalError`: Search index unreachable

**Side Effects:** None (pure query)

**Platform Effects Required:** `SearchEmailIndex(user_id, query, limit)`

**Notes:**
- Search is scoped to authenticated user automatically
- Operators: `from:`, `to:`, `subject:`, `before:`, `after:`, `is:unread`, `is:read`, `has:attachment`
- Free-form text searches across subject and body (case-insensitive)
- Results sorted by relevance (platform responsibility)
- Large queries may timeout; platform should enforce reasonable timeout (5-10s)

---

### 8. `get_agent_permissions`

**Purpose:** Query the current agent's permission level.

**Permission Required:** None (always callable)

**Function Signature (Rust):**
```rust
#[no_mangle]
pub extern "C" fn get_agent_permissions(
    user_id: *const u8,
    user_id_len: usize,
    permission: *const u8,
    permission_len: usize
) -> *mut u8                       // JSON-encoded PermissionSet
```

**Input Parameters:**
- `user_id`: Authenticated user
- `permission`: Current agent permission (passed by host)

**Return Value:**
```json
{
  "permission_level": "send",
  "user_id": "user_abc123",
  "agent_id": "agent_xyz789",
  "granted_at": 1704000000,
  "expires_at": 0
}
```

**Error Semantics:** None (always succeeds)

**Side Effects:** None (pure query)

**Platform Effects Required:** None

**Notes:**
- Utility function for agents to introspect their own capabilities
- Useful for conditional behavior: "if permission == 'send' then send_email() else draft_email()"
- No platform round-trip required; permission is already provided to the function
- Always returns exactly one PermissionSet (for the current user-agent pair)

---

## Summary: Signature Quick Reference

| Function | Permission | Input | Output | Key Effect |
|----------|-----------|-------|--------|------------|
| `read_emails` | read | user_id, mailbox, offset, limit | Vec<Email> | Query |
| `get_email` | read | user_id, email_id | Email | Query + mark read |
| `draft_email` | draft | user_id, to, cc, bcc, subject, body | Draft | Create draft |
| `send_email` | send | user_id, draft_id | EmailId | Send + move to SENT |
| `list_folders` | read | user_id | Vec<Folder> | Query |
| `move_email` | draft/send | user_id, email_id, target_folder_id | () | Update folder |
| `search_emails` | read | user_id, query, limit | Vec<Email> | Query |
| `get_agent_permissions` | none | user_id, permission | PermissionSet | Introspect |

---

## Error Handling Philosophy

- **Permission errors** are always returned as `PermissionDenied`, never silently ignored
- **Not-found errors** are explicit (`EmailNotFound`, `DraftNotFound`, etc.), enabling agent retry logic
- **Validation errors** are descriptive (`InvalidRecipient`, `InvalidInput`) with message hints
- **Internal errors** are rare but possible (`InternalError`); platform should log and alert
- **All errors include a message field** for debugging and user feedback
