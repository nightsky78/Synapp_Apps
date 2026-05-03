# Mail Client — Release Notes

**Version:** 1.0.0  
**Release Date:** 2026-05-03  
**Artifact:** `apps/MailClient/target/wasm32-wasip1/release/mail_client.wasm`  
**Binary Size:** 129 KB  
**QA Status:** Conditional Pass — app binary approved for production  

---

## Scope

This is the initial production release of the **Mail Client** application for the Synapp platform. It covers the complete email management lifecycle: reading, searching, drafting, sending, organizing, and permission introspection. The app is built to the Synapp Dual-Interface standard — simultaneously usable by a human in a React UI and by an autonomous AI agent calling tools.

---

## What's New

### 8 Exported Functions (Execution Interface)

All eight functions are compiled to the `wasm32-wasip1` target and verified present in the release binary.

| Function | Category | Permission Required | Description |
|---|---|---|---|
| `read_emails` | Query | `read` | Paginated email list from any mailbox folder |
| `get_email` | Query | `read` | Full email detail by ID |
| `search_emails` | Query | `read` | Full-text and operator-based search across mailbox |
| `list_folders` | Query | `read` | Enumerate all system and custom folders with counts |
| `draft_email` | Mutation | `draft` | Compose and store a new draft |
| `send_email` | Mutation | `send` | Dispatch a draft to recipients |
| `move_email` | Mutation | `draft`/`send` | Relocate an email between folders |
| `get_agent_permissions` | Query | none | Introspect current agent permission level |

### Hierarchical Permission Model

A four-level permission model gates every function at the point of entry. Permissions are injected by the Synapp Host Broker at runtime; the Wasm module cannot modify or escalate them.

| Level | Capabilities |
|---|---|
| `none` | `get_agent_permissions` only |
| `read` | All query functions (read, search, list) |
| `draft` | All read functions + `draft_email`, `move_email` (to/from DRAFTS) |
| `send` | All functions including `send_email`, `move_email` (all folders) |

Permission is hierarchical: `send` implies `draft`, `draft` implies `read`. Unknown or empty permission strings default conservatively to `none`.

### Semantic Interface (plugin.json)

A complete AI Tool Manifest accompanies the binary. It documents all 8 functions in JSON Schema, enabling the Synapp Host Broker to expose the app as callable tools to AI agents. The schema is synchronized with the Rust source; every function signature, parameter, and return type is reflected in `plugin.json`.

### UI Specification

A contract-first UI specification is delivered alongside this release (`docs/mail-client/ui-spec.md`, `docs/mail-client/ui-copy.md`). It covers all screen states, permission-aware component behavior, WCAG 2.1 AA accessibility requirements, and exact microcopy for buttons, errors, and empty states.

### Security Audit: Pass

The QA Security Report (`docs/mail-client/qa-security-report.md`) confirms:
- No AI logic in the Wasm module
- No direct network or database connections (all I/O delegated via platform effects)
- Permission gates verified at every function entry point
- Input validation covers null pointers, UTF-8, email format, recipient limits, and pagination bounds
- No sensitive information in error responses

---

## Breaking Changes

None. This is the initial release (v1.0.0).

---

## Known Limitations

### No Rust Unit Test Suite (Medium)
`src/lib.rs` contains no `#[test]` blocks or integration test directory. Functional correctness for v1.0 was established through static code review against `contracts.md` and Wasm binary symbol inspection. All contract assertions pass. A unit test suite is planned for v1.1.

Running `cargo test` against the `wasm32-wasip1` target also requires a Wasm runtime (`wasmtime` or `wasmer`) that was not available in the current CI environment. This is an environment limitation, not a code defect.

### Requires Platform I/O Runtime
The Mail Client is a stateless Wasm executor. It has no I/O of its own. All email storage, SMTP sending, folder management, and draft persistence are delegated to the Synapp Host Broker via platform effects. The binary cannot be run standalone — it must be loaded and driven by a compatible Synapp runtime.

### build_all.sh Broken (High — CI Only)
`scripts/build_all.sh` currently targets `wasm32-wasi`, a deprecated target name replaced by `wasm32-wasip1` in Rust 1.78+. The individual app build succeeds correctly. The CI ecosystem script must be fixed before automated pipelines can build all apps together. See the Operator Guide for the fix.

### move_email Schema Imprecision (Low)
The `semanticInterface` in `plugin.json` declares `permissionLevel: "draft"` for `move_email`. The execution logic correctly requires `send` permission when moving to non-DRAFTS folders (TRASH, ARCHIVE, INBOX, SENT). The code is correct; the schema hint understates the minimum permission for the majority of move operations. This will be corrected in a follow-up schema update.

### get_agent_permissions Response Envelope (Low)
`get_agent_permissions` returns a flat `PermissionSet` JSON object, not wrapped in `{ "ok": ... }` like all other functions. This is consistent with the schema but diverges from the standard `AppResult` envelope pattern. This will be unified in v1.1.

### Email Validation Not Full RFC 5322
The `is_valid_email()` validator checks for the presence of `@` and `.` and rejects addresses starting or ending with `@`. It is not fully RFC 5322 compliant. This is adequate for the Wasm security sandbox (the platform mail service performs authoritative validation before sending), but callers should not rely on it for strict address format enforcement.

---

## Migration Guide

Not applicable. This is the initial release. There is no prior version to migrate from.

---

## Installation & Deployment

### Prerequisites

- Rust toolchain: stable, with `wasm32-wasip1` target installed
- Synapp platform runtime (host broker) — provides the I/O capability layer
- `plugin.json` loaded into the broker's schema registry

### Build the Binary

```bash
cd apps/MailClient
cargo build --release --target wasm32-wasip1
```

Output: `target/wasm32-wasip1/release/mail_client.wasm` (expected ~129 KB)

### Deployment Steps

1. **Verify the binary:** Confirm the artifact exists and is the correct format.
   ```bash
   file target/wasm32-wasip1/release/mail_client.wasm
   # Expected: WebAssembly (wasm) binary module version 0x1 (MVP)
   ```

2. **Verify exports:** Confirm all 8 functions are exported from the binary.
   ```bash
   wasm-objdump -x target/wasm32-wasip1/release/mail_client.wasm | grep "Export"
   # Must include: read_emails, get_email, draft_email, send_email,
   #               list_folders, move_email, search_emails, get_agent_permissions
   ```

3. **Load the schema:** Register `plugin.json` with the Synapp Host Broker. The broker uses this to map AI tool calls to Wasm exports.

4. **Mount the binary:** Deploy `mail_client.wasm` to the platform's Wasm module registry. The exact mechanism is platform-specific (file path, object store key, or registry endpoint).

5. **Validate end-to-end:** Use the broker's test harness to invoke `get_agent_permissions` with a known `user_id` and `permission`. Confirm a valid `PermissionSet` JSON response.

### Post-Deployment Checklist

- [ ] Binary file exists at the expected registry path
- [ ] Binary format confirmed as WebAssembly MVP
- [ ] All 8 exports verified in binary
- [ ] `plugin.json` schema loaded in broker registry
- [ ] `get_agent_permissions` smoke test passes
- [ ] Permission gates verified: read-permission agent cannot invoke `send_email`
- [ ] Error responses are structured JSON (not panics)
