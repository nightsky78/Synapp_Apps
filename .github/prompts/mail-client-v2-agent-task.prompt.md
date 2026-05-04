---
mode: agent
description: App Dev Pipeline — Mail Client v2 full implementation prompt
---

# Task: Implement Mail Client v2 — Full MS Outlook-Parity Feature Set

## Context

You are the Synapp App Dev pipeline agent. You are implementing **version 2.0.0** of the `mail-client` first-party app located at `apps/first-party/mail-client/` in the `Synapp_apps` repository.

The full feature specification is in `docs/mail-client-v2-feature-spec.md`. Read it completely before writing any code.

Read the following files for architectural context before starting:
- `.github/copilot-instructions.md` — platform rules and dual-interface mandate
- `.github/AGENTS.md` — pipeline agent roles and execution order
- `apps/first-party/mail-client/synapp.app.json` — current manifest (v1.0.0)
- `apps/first-party/mail-client/plugin.json` — current semantic interface (v1.0.0)
- `apps/first-party/mail-client/src/lib.rs` — current Wasm implementation (v1.0.0)
- `docs/mail-client-v2-feature-spec.md` — this task's full feature specification

---

## What You Must Deliver

Run the full pipeline in order:

### Stage 1 — Architect: Design the dual-interface contract

1. Update `apps/first-party/mail-client/synapp.app.json` to version `2.0.0`:
   - Declare ALL 25+ agent tools listed in the spec section 2.11
   - Add ALL capabilities from spec section 6
   - Add ALL host effect dependencies from spec section 3
   - Declare UI contributions: `settings_panel` (account management, spec section 4) and `navigation_pages` for InboxView, ComposeView, SearchResultsView, FolderManagementView, and RulesManagementView (spec section 5)
   - Declare ALL events published (spec section 8) and subscribed (spec section 9)
   - Set resource limits per spec section 7

2. Update `apps/first-party/mail-client/plugin.json` to version `2.0.0`:
   - Add JSON Schema entries for EVERY new exported Wasm function
   - Keep every existing schema entry, upgrade to v2 where needed
   - Schemas must be strict: `additionalProperties: false`, all required fields listed, all enums exhaustive
   - Each export must have: `name`, `description`, `requiredCapability`, `inputSchema`, `outputSchema`
   - Add `definitions` section for shared types: `Email`, `Draft`, `Folder`, `Account`, `Rule`, `Category`, `SearchFilter`, `Error`, `ScheduledSend`

### Stage 2 — Developer: Implement the Wasm execution layer

Expand `apps/first-party/mail-client/src/lib.rs` to implement ALL of the following exported Wasm functions:

**Account tools:**
- `list_accounts(user_id, permission)` — returns account metadata list (no credentials, never expose passwords/tokens)
- `get_account_status(user_id, account_id, permission)` — returns sync status, last sync time, error state

**Folder tools:**
- `list_mailboxes(user_id, account_id, permission)` — returns full folder tree with unread counts
- `create_folder(user_id, account_id, name, parent_folder_id, permission)` — creates folder, returns new folder metadata
- `rename_folder(user_id, account_id, folder_id, new_name, permission)` — renames folder
- `delete_folder(user_id, account_id, folder_id, permission)` — deletes empty folder, returns error if non-empty unless `force=true`

**Email read tools:**
- `read_emails(user_id, account_id, folder_id, offset, limit, sort_by, sort_order, filter_json, permission)` — paginated list with full filter support
- `search_emails(user_id, search_filter_json, permission)` — full-text + filter search across all accounts/folders
- `get_email(user_id, email_id, permission)` — full email with body, headers, attachments list
- `get_thread(user_id, thread_id, permission)` — full conversation thread ordered chronologically
- `get_unread_count(user_id, account_id, permission)` — unread count per folder

**Email state mutation tools:**
- `mark_read(user_id, email_ids_json, is_read, permission)` — batch mark read/unread
- `flag_email(user_id, email_id, is_flagged, permission)` — flag or unflag
- `move_email(user_id, email_ids_json, target_folder_id, permission)` — move emails
- `delete_email(user_id, email_ids_json, permanent, permission)` — move to trash or permanent delete
- `archive_email(user_id, email_ids_json, permission)` — move to archive folder
- `apply_category(user_id, email_id, category_ids_json, permission)` — assign categories

**Compose & send tools:**
- `draft_email(user_id, account_id, to_json, cc_json, bcc_json, subject, body_html, body_plain, importance, request_read_receipt, attachment_ids_json, permission)` — create draft, returns `draft_id`
- `update_draft(user_id, draft_id, to_json, cc_json, bcc_json, subject, body_html, body_plain, importance, request_read_receipt, attachment_ids_json, permission)` — update existing draft
- `discard_draft(user_id, draft_id, permission)` — permanently delete draft
- `send_email(user_id, draft_id, permission)` — send the draft immediately, returns `message_id`
- `reply_email(user_id, original_email_id, reply_all, body_html, body_plain, attachment_ids_json, permission)` — reply inline thread
- `forward_email(user_id, original_email_id, to_json, cc_json, body_prepend_html, permission)` — forward with optional note
- `schedule_send(user_id, draft_id, send_at_unix, permission)` — queue for future delivery, returns `job_id`
- `cancel_scheduled_send(user_id, job_id, permission)` — cancel queued send

**Rules tools:**
- `list_rules(user_id, account_id, permission)` — list all rules in priority order
- `create_rule(user_id, account_id, rule_json, permission)` — create new rule with conditions and actions
- `update_rule(user_id, rule_id, rule_json, permission)` — update rule
- `delete_rule(user_id, rule_id, permission)` — delete rule
- `reorder_rules(user_id, account_id, ordered_rule_ids_json, permission)` — set rule priority order

**Category tools:**
- `list_categories(user_id, permission)` — list user-defined categories with colors
- `create_category(user_id, name, color_hex, permission)` — create category
- `delete_category(user_id, category_id, permission)` — delete category

**Implementation rules for all functions:**
- NEVER expose passwords, OAuth tokens, or raw credentials in any output — return metadata only
- ALWAYS validate permission level before proceeding; return `PermissionDenied` error if insufficient
- ALWAYS validate all string inputs: length limits, format checks (email addresses), enum values
- All platform I/O (IMAP/SMTP/contact lookup) must be expressed as `HostEffect` JSON descriptors in the response `effects` array — the Wasm binary never opens sockets
- Return consistent `{ "ok": <data> }` or `{ "err": { "code": "...", "message": "..." } }` envelope
- Cap all paginated results: max `limit=500` for list operations
- Use `std::mem::forget` for returned string pointers (C ABI memory ownership model)
- Target: `wasm32-wasip1` — no std threads, no direct I/O, no OS signals

### Stage 3 — QA & Security: Security audit

Verify:
- No credential leakage in any function output (accounts, drafts, error messages)
- No path traversal via folder_id parameters (UUIDs only, no path strings)
- No SSRF via account configuration fields (host effects are broker-proxied; validate format only)
- Permission model is strictly additive: `send` implies `draft` implies `read` implies no mutation
- All batch operations (mark_read, move_email, delete_email) enforce a max batch size of 1000 items
- `body_html` in compose inputs must be passed through as-is — sanitization is the host's responsibility, but the Wasm layer must reject payloads > 10MB (10_485_760 bytes)
- Attachment references are UUIDs only — never file paths
- No `unwrap()`, `expect()`, or `panic!()` in production code paths

### Stage 4 — Testing: Build and unit test

1. Run: `cargo build --target wasm32-wasip1 --release` in `apps/first-party/mail-client/`
2. Run: `cargo test` in `apps/first-party/mail-client/`
3. Verify all unit tests pass with zero warnings

Write unit tests covering:
- Permission enforcement: each tool called with insufficient permission returns `PermissionDenied`
- Input validation: empty user_id, empty email_id, oversized body, invalid email format in to/cc/bcc
- Pagination bounds: limit=0 returns error, limit=501 is clamped to 500
- Batch size limit: passing 1001 IDs to batch operations returns `BatchTooLarge` error
- Host effect structure: verify effect descriptors contain all required fields for at least one happy-path per tool

### Stage 5 — Tech-Writer: Update package and catalog

1. Update `apps/first-party/mail-client/Cargo.toml` version to `2.0.0`
2. Rebuild the package tarball: `tar -czf packages/mail-client-2.0.0.tar.gz -C apps/first-party/mail-client synapp.app.json target/wasm32-wasip1/release/mail_client.wasm`
3. Update `catalog/apps/mail-client.v1.json` to reference version `2.0.0` and the new package URL
4. Update `catalog/index.v1.json` to set `latest_version: "2.0.0"` for `mail-client`
5. Update `apps/first-party/mail-client/README.md` with the full feature list

---

## Blocker Report Requirement

If any of the following host effects are NOT yet exposed by the platform ABI, you MUST write a blocker report at `docs/reports/mail-client-v2-host-blockers.md` listing:
- The exact effect name
- Which tools depend on it
- Suggested host implementation contract (request/response JSON shape)
- Priority: P0 (blocks core read/send) vs P1 (degrades gracefully)

Effects that are P0 blockers if missing:
- `ImapFetch` (blocks all email reading)
- `SmtpSend` (blocks all sending)
- `AccountCredentialRead` (blocks all account operations)
- `AccountCredentialWrite` (blocks account setup)

Effects that are P1 (graceful degradation if missing):
- `OAuthTokenExchange` (OAuth2 accounts unavailable; password auth still works)
- `ContactLookup` (autocomplete unavailable; manual address entry still works)
- `NotifyUser` (no in-app notifications; polling still works)
- `ScheduleJob` / `CancelJob` (scheduled send unavailable)
- `StorePlatformSecret` (OAuth refresh token persistence unavailable)

---

## Acceptance Criteria

- [ ] `synapp.app.json` version is `2.0.0` with all 25+ tools, all capabilities, all host effects, all UI contributions, all events
- [ ] `plugin.json` version is `2.0.0` with strict schemas for all exported functions and all shared type definitions
- [ ] All exported Wasm functions listed in Stage 2 are implemented in `lib.rs`
- [ ] `cargo build --target wasm32-wasip1 --release` exits 0 with no warnings treated as errors
- [ ] `cargo test` exits 0 with all tests passing
- [ ] Unit tests cover: permission enforcement, input validation, pagination bounds, batch size limits, host effect structure
- [ ] No `unwrap()`, `expect()`, `panic!()` in non-test code
- [ ] `packages/mail-client-2.0.0.tar.gz` exists and contains `synapp.app.json` + the Wasm binary
- [ ] Catalog updated with `latest_version: "2.0.0"`
- [ ] If any P0 host effect is missing: blocker report written and submitted before stopping
- [ ] `plugin.json` and `lib.rs` exports are fully synchronized (no function in one missing from the other)
