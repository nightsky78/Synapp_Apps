# 04 — Security & Testing

> **Goal:** Know exactly what to check and test before shipping an app. This guide covers the
> mandatory security checklist, unit test patterns, integration test commands, and the complete
> list of required exports.

---

## Table of Contents

1. [Mandatory Security Checklist](#1-mandatory-security-checklist)
2. [Required Exports Checklist](#2-required-exports-checklist)
3. [Unit Test Patterns](#3-unit-test-patterns)
4. [Integration Test Commands](#4-integration-test-commands)
5. [File Path Safety Rules](#5-file-path-safety-rules)
6. [Scope & Authorization Enforcement](#6-scope--authorization-enforcement)
7. [Common Vulnerabilities to Avoid](#7-common-vulnerabilities-to-avoid)

---

## 1. Mandatory Security Checklist

Before submitting your app to be registered on a production platform, verify every item below.

### Identity and Data Isolation

- [ ] **Use `principal_id` for data namespacing**, not `actor_id`.
      An agent's `actor_id` is the agent's UUID; `principal_id` is the owning human.
      Using `actor_id` for storage would create a separate data silo per agent, breaking
      the user's ability to see their own data.

- [ ] **Never include `actor_id` or `principal_id` in collection names or `doc_id`**.
      The host injects namespacing automatically. Embedding these in keys creates redundant
      (and potentially spoofable) identifiers.

- [ ] **Never accept `app_id`, `user_id`, `actor_id`, or `principal_id` from request arguments.**
      These values come exclusively from `CallerCtx`, which is derived from the validated JWT.
      A caller who constructs `arguments: {"user_id": "admin"}` must not get admin's data.

### File Path Safety

- [ ] **All `WriteTextFile.file_path` values must be relative paths.**
      Reject or sanitize any argument that the caller provides as a file path.
      The platform rejects absolute paths and `..` sequences with HTTP 400, but your code
      should fail fast with a clear error message before reaching the effect layer.

- [ ] **Never construct file paths by concatenating untrusted input without validation.**

### App Isolation

- [ ] **Add `#![forbid(unsafe_code)]` to the top of your `lib.rs`.**
      App crates run inside the platform's trust boundary; unsafe code is never needed.

- [ ] **No networking crates in `Cargo.toml`.**
      `tokio`, `reqwest`, `hyper`, `aws-sdk-*`, etc. are not available in the app isolation layer.
      Using them will cause a compile error. All external I/O must go through `HostEffect` values.

- [ ] **No async in your crate.** `execute_action()` is a synchronous function.
      If you need async logic, it means the task belongs in a HostEffect, not in your app.

- [ ] **No filesystem access (`std::fs`, `std::path::Path` pointing to local disk).**
      App crates must not assume the presence of a POSIX filesystem.

### Output Safety

- [ ] **Do not include sensitive data in `payload`** (passwords, raw tokens, private keys).
      `payload` is returned verbatim in the HTTP response body to the caller.

- [ ] **Do not log secrets to stderr.**
      The platform captures stderr from app crates in production logs.

---

## 2. Required Exports Checklist

Your crate **must** export all of the following from `lib.rs`. The host will fail to compile
if any are missing:

| Export | Type | Notes |
|---|---|---|
| `APP_ID` | `pub const &str` | Must be globally unique across all installed apps |
| `execute_action` | `pub fn` | Signature: `(action_name: &str, arguments: &Value, caller: &CallerCtx) -> AppActionResult` |
| `tool_registration_payload` | `pub fn` | Signature: `() -> Value` |
| `settings_schema_payload` | `pub fn` | Signature: `() -> Value` |
| `HostEffect` | `pub enum` | Must match the host's expected variants exactly |
| `AppActionResult` | `pub struct` | Must have `ok: bool`, `payload: Value`, `effects: Vec<HostEffect>` |
| `CallerCtx` | `pub struct` | Must have `actor_id`, `actor_type`, `principal_id` as `String` |

### Quick compilation check

After adding your crate to `Cargo.toml`, run:

```bash
cd services/transformer-service
cargo build 2>&1 | head -50
```

A clean build output confirms all required types and exports are present and compatible.

---

## 3. Unit Test Patterns

Unit tests for app crates are pure function tests. No database, no Docker, no network.
Run in under a second with `cargo test`.

### Test module boilerplate

Add this module at the bottom of your `lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Use this helper throughout your tests. Never hardcode IDs.
    fn make_caller(actor_type: &str) -> CallerCtx {
        CallerCtx {
            actor_id:     "test-actor-001".to_string(),
            actor_type:   actor_type.to_string(),
            principal_id: "test-principal-001".to_string(),
        }
    }
```

### Pattern: Test the happy path

Every action must have at least one test for valid input:

```rust
    #[test]
    fn create_todo_succeeds_with_valid_title() {
        let args = json!({ "title": "Buy groceries" });
        let result = execute_action(ACTION_CREATE_TODO, &args, &make_caller("human"));

        assert!(result.ok, "Expected ok=true, got: {:?}", result.payload);
        assert_eq!(result.effects.len(), 1);
    }
```

### Pattern: Test that the correct HostEffect is emitted

Assert both that the right effect type is returned **and** that its fields are correct:

```rust
    #[test]
    fn create_todo_emits_put_document_effect() {
        let args = json!({ "title": "Call doctor" });
        let result = execute_action(ACTION_CREATE_TODO, &args, &make_caller("human"));

        match &result.effects[0] {
            HostEffect::PutDocument { collection, doc_id, data } => {
                assert_eq!(collection, "todos");
                assert_eq!(doc_id, "", "doc_id must be empty so host generates UUID");
                assert_eq!(data["title"], "Call doctor");
                assert_eq!(data["done"], false, "New todos must be incomplete");
            }
            other => panic!("Expected PutDocument, got: {:?}", other),
        }
    }
```

### Pattern: Test input validation (negative path)

Every validation rule must have a dedicated failing test:

```rust
    #[test]
    fn create_todo_rejects_empty_title() {
        let args = json!({ "title": "" });
        let result = execute_action(ACTION_CREATE_TODO, &args, &make_caller("human"));

        assert!(!result.ok);
        assert!(result.effects.is_empty(), "No effects must be emitted on failure");
        let err = result.payload["error"].as_str().expect("Error field missing");
        assert!(err.contains("title"), "Error message must mention the failing field");
    }

    #[test]
    fn create_todo_rejects_missing_title() {
        let args = json!({ "description": "A todo without a title" });
        let result = execute_action(ACTION_CREATE_TODO, &args, &make_caller("human"));
        assert!(!result.ok);
        assert!(result.effects.is_empty());
    }
```

### Pattern: Test the unknown action fallback

```rust
    #[test]
    fn unknown_action_returns_error_without_effects() {
        let result = execute_action("nonexistent.action", &json!({}), &make_caller("human"));
        assert!(!result.ok);
        assert!(result.effects.is_empty());
        let err = result.payload["error"].as_str().unwrap();
        assert!(err.contains("Unknown action"));
    }
```

### Pattern: Test file path validation

If your app takes file path arguments, always test the rejection of dangerous inputs:

```rust
    #[test]
    fn export_rejects_absolute_file_path() {
        let args = json!({ "file_path": "/etc/passwd" });
        let result = execute_action(ACTION_EXPORT, &args, &make_caller("human"));
        assert!(!result.ok);
        assert!(result.effects.is_empty());
    }

    #[test]
    fn export_rejects_path_traversal() {
        let args = json!({ "file_path": "../other-app/config.json" });
        let result = execute_action(ACTION_EXPORT, &args, &make_caller("human"));
        assert!(!result.ok);
        assert!(result.effects.is_empty());
    }
```

### Pattern: Test tool manifest structure

Validate the manifest at test time so regressions are caught early:

```rust
    #[test]
    fn tool_manifest_is_well_formed() {
        let manifest = tool_registration_payload();
        assert_eq!(manifest["app_id"], APP_ID);

        let tools = manifest["tools"].as_array().expect("tools must be an array");
        assert!(!tools.is_empty(), "Must register at least one tool");

        for tool in tools {
            assert!(tool["action_name"].is_string(), "action_name required");
            assert!(tool["description"].is_string(), "description required");
            assert!(tool["json_schema"].is_object(), "json_schema required");
            assert!(tool["required_scope"].is_string(), "required_scope required");
        }
    }
```

### Pattern: Test settings schema structure

```rust
    #[test]
    fn settings_schema_is_well_formed() {
        let schema = settings_schema_payload();
        assert_eq!(schema["app_id"], APP_ID);
        assert!(schema["title"].is_string());

        let fields = schema["fields"].as_array().expect("fields must be an array");
        for field in fields {
            assert!(field["key"].is_string());
            assert!(field["label"].is_string());
            assert!(field["field_type"].is_string());
            assert!(field["required"].is_boolean());
        }
    }
```

### Running tests

```bash
# From your wasm-module directory (fast, no platform needed)
cd apps/TodoApp/backend/wasm-module
cargo test

# With verbose output
cargo test -- --nocapture

# Run a specific test
cargo test create_todo_rejects_empty_title
```

---

## 4. Integration Test Commands

These tests require a running platform stack. Run them after `docker compose up -d`.

### Environment setup

```bash
# Start the stack
docker compose up -d
docker compose ps  # wait until all services are healthy

# Authenticate as admin
TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123!"}' \
  | jq -r .access_token)

echo "Token acquired: ${TOKEN:0:20}..."
```

### Test 1: App is reachable

```bash
# An unknown app should return 404
curl -s -X POST http://localhost:8080/api/v1/apps/nonexistent-app/actions/test \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"arguments":{}}' \
  | jq .

# Expected: HTTP 404 or {"ok":false,"error":"app not found"}
```

### Test 2: Valid action call

```bash
RESULT=$(curl -s -X POST http://localhost:8080/api/v1/apps/todo-app/actions/todo.create \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"arguments":{"title":"Integration test todo"}}')

echo $RESULT | jq .

# Assertions (run with jq)
echo $RESULT | jq -e '.ok == true'            # must pass
echo $RESULT | jq -e '.host_results | length == 1'  # one effect result
echo $RESULT | jq -e '.host_results[0].doc_id | length > 0'  # UUID was generated
```

### Test 3: Input validation

```bash
RESULT=$(curl -s -X POST http://localhost:8080/api/v1/apps/todo-app/actions/todo.create \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"arguments":{}}')  # missing required "title"

echo $RESULT | jq .
echo $RESULT | jq -e '.ok == false'  # must fail
```

### Test 4: User data isolation

```bash
# Create a second user (as admin)
curl -s -X POST http://localhost:8080/api/admin/users \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser2","password":"Test1234!","email":"t2@example.com"}' \
  | jq .

# Authenticate as testuser2
T2=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser2","password":"Test1234!"}' \
  | jq -r .access_token)

# testuser2 must see an empty list
COUNT=$(curl -s -X POST http://localhost:8080/api/v1/apps/todo-app/actions/todo.list \
  -H "Authorization: Bearer $T2" \
  -H "Content-Type: application/json" \
  -d '{"arguments":{}}' \
  | jq '.host_results[0].documents | length')

[ "$COUNT" = "0" ] && echo "✅ Data isolation: PASS" || echo "❌ Data isolation: FAIL (got $COUNT)"
```

### Test 5: Authentication is enforced

```bash
# Call without any token — must get 401
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
  -X POST http://localhost:8080/api/v1/apps/todo-app/actions/todo.list \
  -H "Content-Type: application/json" \
  -d '{"arguments":{}}')

[ "$HTTP_CODE" = "401" ] && echo "✅ Auth enforcement: PASS" || echo "❌ Auth enforcement: FAIL (got $HTTP_CODE)"
```

### Test 6: Tool discovery

```bash
# The tool registry should list your app's tools
TOOLS=$(curl -s "http://localhost:8080/api/v1/tools?app_id=todo-app" \
  -H "Authorization: Bearer $TOKEN" \
  | jq '.tools | length')

[ "$TOOLS" -ge "1" ] && echo "✅ Tool discovery: PASS" || echo "❌ Tool discovery: FAIL"
```

---

## 5. File Path Safety Rules

The platform enforces these rules in `validate_file_path()` inside `app_dispatch.rs`.
Even so, your app code should reject bad paths early to return a clear error message.

| Rule | Implementation in your action function |
|---|---|
| No absolute paths | Check `path.starts_with('/')` → return error |
| No parent traversal | Check `path.contains("..")` → return error |
| No empty path | Check `path.is_empty()` → return error |
| Max path length | Check `path.len() > 1024` → return error |

### Reference implementation for path validation

Copy this helper into your `lib.rs` if your app accepts file path arguments:

```rust
/// Validates that a file path provided by a caller is safe to use in a WriteTextFile effect.
///
/// Returns `Ok(())` if the path is valid, or `Err(String)` with a human-readable message.
fn validate_caller_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("file_path must not be empty".to_string());
    }
    if path.len() > 1024 {
        return Err("file_path must be under 1024 characters".to_string());
    }
    if path.starts_with('/') {
        return Err("file_path must be a relative path (no leading '/')".to_string());
    }
    if path.contains("..") {
        return Err("file_path must not contain '..' (path traversal is not allowed)".to_string());
    }
    Ok(())
}
```

Usage in an action:

```rust
fn action_write_report(arguments: &Value, _caller: &CallerCtx) -> AppActionResult {
    let file_path = match arguments.get("file_path").and_then(Value::as_str) {
        Some(p) => p.to_string(),
        None => return AppActionResult {
            ok: false,
            payload: json!({ "error": "file_path is required" }),
            effects: vec![],
        },
    };

    if let Err(e) = validate_caller_path(&file_path) {
        return AppActionResult {
            ok: false,
            payload: json!({ "error": e }),
            effects: vec![],
        };
    }

    // Safe to use file_path here
    AppActionResult {
        ok: true,
        payload: json!({ "queued": true }),
        effects: vec![HostEffect::WriteTextFile {
            file_path,
            content:      "report content".to_string(),
            content_type: "text/plain".to_string(),
        }],
    }
}
```

---

## 6. Scope & Authorization Enforcement

The platform validates scopes **before** calling `execute_action()`. Your app does not need to
check scopes in application code. However, you must declare them correctly in your manifest.

### Scope naming convention

```
{app_id}:{permission}
```

| Standard permission | Meaning |
|---|---|
| `{app_id}:read` | Read-only access to the app's data |
| `{app_id}:write` | Create and modify data |
| `{app_id}:admin` | Administrative actions (delete all, configure) |

### How scopes flow

```
                  ┌──────────────────────────────────────────────────┐
 HTTP request  →  │  JWT validated by IAM service                    │
 Bearer <jwt>     │  Scopes extracted from token                     │
                  │  Required scope looked up in tool_registration   │
                  │                                                  │
                  │  If scope missing → HTTP 403 (before your code)  │
                  │  If scope present → execute_action() is called   │
                  └──────────────────────────────────────────────────┘
```

Your `execute_action()` is only called when the caller has the required scope.
You do not need to re-check authorization inside the function.

---

## 7. Common Vulnerabilities to Avoid

These mistakes have been observed in real app implementations. Avoid all of them.

### ❌ Using arguments as a storage key

```rust
// WRONG: a caller can pass any doc_id they want, including another user's UUID
let doc_id = arguments["id"].as_str().unwrap_or("").to_string();
effects: vec![HostEffect::PutDocument { doc_id, ... }]
```

**Why:** An attacker could overwrite another user's document if the platform namespacing
were misconfigured. Your app should generate or derive doc_id values deterministically
from controlled inputs (e.g. a hash of app_id + principal_id + user-visible name),
or pass empty string to get a host-generated UUID.

### ❌ Trusting arguments for identity

```rust
// WRONG: allows any caller to claim to be any user
let user_id = arguments["user_id"].as_str().unwrap_or("");
data["owner"] = json!(user_id);
```

**Correct:**

```rust
// RIGHT: identity comes from the validated JWT, not the request body
data["owner"] = json!(caller.principal_id);
```

### ❌ Constructing paths from arguments without validation

```rust
// WRONG: caller can pass "../../../etc/passwd"
let path = format!("exports/{}", arguments["filename"].as_str().unwrap_or(""));
```

**Correct:**

```rust
let filename = arguments["filename"].as_str().unwrap_or("export");
// Sanitize: take only alphanumeric + hyphens + dots
let safe_name: String = filename.chars()
    .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '.')
    .take(100)
    .collect();
let path = format!("exports/{safe_name}");
```

### ❌ Returning sensitive data in payload

```rust
// WRONG: payload is returned in the HTTP response body
payload: json!({ "token": some_secret_token, "result": "ok" }),
```

**Correct:**

```rust
payload: json!({ "result": "ok" }),
```

### ❌ Panicking on invalid input

```rust
// WRONG: panics if "title" is missing — crashes the host thread
let title = arguments["title"].as_str().unwrap();
```

**Correct:**

```rust
// RIGHT: return an error result
let title = match arguments.get("title").and_then(Value::as_str) {
    Some(t) if !t.is_empty() => t.to_string(),
    _ => return AppActionResult {
        ok: false,
        payload: json!({ "error": "title is required" }),
        effects: vec![],
    },
};
```

---

## Quick Reference: App Security Summary

```
DO                                        DON'T
──────────────────────────────────────    ──────────────────────────────────────
Use CallerCtx.principal_id for data       Trust arguments for user identity
Pass "" as doc_id for new documents       Accept doc_id from caller arguments
Validate file paths before effects        Construct paths from raw arguments
Return errors as ok:false payloads        Use unwrap()/expect()/panic!()
Test negative paths as rigorously         Skip tests for error cases
  as happy paths
Use #![forbid(unsafe_code)]              Add network deps (tokio, reqwest, etc.)
Keep execute_action() synchronous         Add async fn or spawn threads
```
