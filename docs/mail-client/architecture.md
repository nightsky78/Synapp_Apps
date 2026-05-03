# Mail Client App — Architecture & Design

## 1. App Purpose & Vision

The **Mail Client** is an AI-native email management application built for the Synapp platform. Its core design principle: **One app, two interfaces** (React UI for humans, semantic Tool interface for autonomous AI agents).

Unlike traditional email clients that require separate integrations for each use-case (human interaction vs. AI automation), the Mail Client exposes its logic uniformly via:
- **Execution Interface:** Stateless, Wasm-compiled functions that contain pure business logic
- **Semantic Interface:** Machine-readable JSON Schema describing each function as an AI tool

This allows AI agents to autonomously:
- Read emails and search mailboxes
- Draft messages with full composition capabilities
- Send emails (with explicit permission gating)
- Organize messages into folders
- Execute intelligent triage workflows

---

## 2. Boundaries & Non-Goals

### What This App Does
- **Email state queries:** Read, search, filter, and list messages
- **Composition & drafting:** Create email drafts with recipients, subject, body
- **Sending:** Dispatch emails to recipients (permission-controlled)
- **Folder management:** Move messages, list folders, organize mailbox structure
- **Permission enforcement:** Gate send/draft/read capabilities at function entry based on agent credentials

### What This App Does NOT Do
- **Direct SMTP/IMAP connections:** All email transport is handled by the Synapp platform via capability effects
- **Credential storage:** Platform IAM service manages OAuth tokens and email authentication
- **Full email parsing:** Rich attachments, MIME, and complex headers are platform concerns
- **Real-time sync:** Polling or event-driven sync is coordinated by the host; the app is stateless and pure
- **AI decision logic:** The app never contains LLM prompts, routing rules, or AI orchestration

---

## 3. Core Assumptions

1. **Stateless Execution**
   - Every function is pure: same inputs → same output, no side effects
   - The Wasm module holds zero state across invocations
   - User context (mailbox, folders, drafts) is provided as input parameters or fetched via platform effects

2. **Platform-Managed State**
   - Emails are stored in platform persistence (database, object store, or equivalent)
   - Drafts are temporary and stored in platform ephemeral store
   - Sent messages are committed by platform effects, not by the app logic

3. **User Scoping via JWT**
   - Every request includes a JWT with `user_id` claim
   - The app receives `user_id` as an implicit context in every function
   - All queries/mutations are automatically scoped to the authenticated user

4. **Agent Permissions as Runtime Constraints**
   - Permissions (read, draft, send) are passed to each function invocation
   - The app validates permissions at function entry; invalid access attempts return explicit errors
   - Permissions cannot be overridden from within the Wasm logic; they are checked before execution

5. **Mailbox Structure**
   - Users have one primary mailbox (INBOX)
   - Standard folders: INBOX, DRAFTS, SENT, TRASH, JUNK, ARCHIVE
   - Custom user-created folders are supported
   - Folder names are unique per user and identified by folder_id (not path)

6. **Email Model Simplicity**
   - Emails have core metadata: id, from, to, cc, bcc, subject, body, timestamp, read_status
   - Attachments are referenced by id but not processed/stored by this app
   - Rich HTML body is supported but treated as opaque string
   - Threading/conversation grouping is handled by the host, not this app

---

## 4. Agent Use-Cases

### Use-Case 1: Email Triage Agent (Permission: Read-Only)
**Scenario:** An autonomous system monitors incoming emails and categorizes them for routing.

- Calls `read_emails(mailbox="INBOX", offset=0, limit=100)` → returns latest 100 unread emails
- Calls `search_emails(query="urgent", limit=50)` → finds high-priority messages
- Returns categorization results to the platform (e.g., "route to support@company.com")
- **Permission:** Read-only; cannot draft or send

### Use-Case 2: Draft-Only Assistant (Permission: Draft)
**Scenario:** An AI assistant helps compose professional emails but requires human review before sending.

- Calls `get_agent_permissions()` → confirms "draft" permission
- Calls `draft_email(to=["recipient@company.com"], subject="...", body="...")` → creates draft in DRAFTS folder
- Returns draft_id for human review
- **Permission:** Can read and draft; cannot send
- Human must manually review and click "Send" in the UI

### Use-Case 3: Autonomous Email Agent (Permission: Send)
**Scenario:** A high-trust workflow agent autonomously handles routine communications.

- Calls `read_emails()` → scans for incoming requests
- Calls `draft_email()` → composes response
- Calls `send_email(draft_id)` → dispatches message
- Calls `move_email()` → archives original message
- **Permission:** Full access (read, draft, send)
- Used only for high-confidence, templated workflows (e.g., automated booking confirmations)

### Use-Case 4: Email Search & Summarization (Permission: Read-Only)
**Scenario:** A research bot extracts information from a user's email archive.

- Calls `search_emails(query="invoice AND (2024 OR 2025)", limit=100)` → finds invoices
- Calls `get_email(email_id)` → fetches full details for each result
- Extracts structured data (vendor, amount, date) and returns to platform
- **Permission:** Read-only; no mutation

---

## 5. Permission Model

### Permission Levels

| Permission | read_emails | get_email | search_emails | draft_email | send_email | move_email | list_folders | get_agent_permissions |
|-----------|-----------|-----------|-----------|-----------|-----------|-----------|-----------|-----------|
| **none** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| **read** | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ |
| **draft** | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ |
| **send** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

### Permission Enforcement
- **Granular per-function:** Each function checks its required permission at entry
- **Explicit error response:** If permission is insufficient, function returns `PermissionDenied` error
- **No fallback logic:** Functions do not attempt to execute with reduced capability
- **Immutable by Wasm:** The app cannot modify or escalate permissions; they are enforced by the host before function invocation

### Permission Assignment Workflow
1. **Admin UI:** Administrator selects agent and assigns permission level (read, draft, send, or none)
2. **Platform stores:** Permission level is recorded in agent profile
3. **Runtime injection:** When agent invokes a function, platform includes permission in function context
4. **App validation:** Wasm function reads permission, validates, or returns error

---

## 6. Data Flow & Platform Integration

### Query Flow (e.g., `read_emails`)
```
Agent
  │
  ├─→ Synapp Host Broker (interprets as tool call)
  │
  ├─→ Wasm Function Invocation (with permission context)
  │   ├─ Input: mailbox_id, offset, limit, user_id, permission
  │   ├─ Logic: Validate permission, build query, return effects
  │   └─ Output: Vec<Effect>
  │
  ├─→ Host Effect Executor
  │   ├─ Query email store: SELECT * FROM emails WHERE user_id=? LIMIT ?
  │   └─ Return: Vec<Email>
  │
  └─→ Response to Agent
```

### Mutation Flow (e.g., `send_email`)
```
Agent
  │
  ├─→ Synapp Host Broker
  │
  ├─→ Wasm Function Invocation
  │   ├─ Input: draft_id, user_id, permission
  │   ├─ Logic: Check send permission, validate draft exists, emit SendEmailEffect
  │   └─ Output: [SendEmailEffect { draft_id, recipient_list, subject, body }]
  │
  ├─→ Host Effect Executor
  │   ├─ Query: Get draft details from ephemeral store
  │   ├─ Validate: Draft belongs to user, hasn't been sent
  │   ├─ Action: Call SMTP service via platform mail interface
  │   ├─ Commit: Move draft to SENT folder, record sent_at timestamp
  │   └─ Return: EmailId of sent message
  │
  └─→ Response to Agent: Success(EmailId) or Error(SendFailed)
```

---

## 7. Security Constraints

### Enforcement Points

1. **Function-Level Permission Check**
   - First line of every export: Check `permission` against required level
   - Return `PermissionDenied` error if insufficient
   - No exception for user_id, no special cases

2. **User Scoping**
   - Every query includes WHERE clause for `user_id`
   - Wasm never constructs queries without user_id filter
   - Cross-user access always returns error or empty result

3. **Credential Isolation**
   - Wasm has no access to email passwords, OAuth tokens, or SMTP credentials
   - Platform IAM service manages all authentication
   - Wasm only receives `user_id` and `permission` context

4. **Effect Validation**
   - Platform validates all effects before execution
   - If Wasm returns malformed effect (e.g., invalid recipient), platform rejects
   - No unchecked delegation from Wasm to external services

---

## 8. Testing Strategy

### Unit Tests (Rust + wasm-bindgen-test)
- Test each function with valid/invalid permission combinations
- Test error paths: nonexistent email, malformed draft, invalid folder
- Test boundary conditions: zero emails, max limit, empty search results

### Integration Tests (Platform Host)
- End-to-end flows: read → draft → send
- Multi-folder workflows: move_email across DRAFTS, SENT, TRASH
- Permission gating: Verify read-only agent cannot call send_email

### Security Tests
- Cross-user access attempts (should fail silently or error)
- Permission escalation attempts (should always fail)
- Malformed input (should return validation error)

---

## 9. Roadmap & Future Enhancements

- **Phase 1:** Core CRUD operations (read, draft, send, list)
- **Phase 2:** Full-text search, folder management
- **Phase 3:** Attachment handling, email threading
- **Phase 4:** Template engine for standardized email composition
- **Phase 5:** Webhook-based notifications for real-time events

---

## 10. Build & Deployment Target

- **Compilation Target:** `wasm32-wasi`
- **Wasm Output:** `target/wasm32-wasi/release/mail_client.wasm`
- **Std Library:** wasm32-wasi provides POSIX-like syscalls; async I/O is prohibited (pure sync logic)
- **Dependencies:** serde, serde_json (no async runtimes, no native OS calls)
