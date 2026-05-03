# 02 — Building Your First App: Step-by-Step Tutorial

> **Goal:** Go from nothing to a fully working app that stores data, exposes an AI tool,
> and appears in the Admin settings panel. Every file you need to create or edit is listed
> with the exact content.

---

## Before You Start

Make sure the platform is running locally:

```bash
cd /path/to/NexusCore
docker compose up -d
# Wait until transformer-service is healthy
docker compose ps
```

You should be able to reach `http://localhost:8080` and log in.

---

## What We Are Building

A minimal **TodoApp** that lets authenticated users:
- Create a to-do item (stores a JSON document)
- List all their to-do items (queries documents)

It will also:
- Register itself as an AI tool so the Copilot can create to-dos via natural language
- Appear in the Admin settings panel with a configurable "max items" limit

---

## Step 1 — Create the App Crate

```bash
mkdir -p apps/TodoApp/backend/wasm-module/src
```

Create `apps/TodoApp/backend/wasm-module/Cargo.toml`:

```toml
[package]
name = "todo-app-wasm-module"
version = "0.1.0"
edition = "2024"

# cdylib  → shared library the host links against
# rlib    → test harness for `cargo test`
[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
```

> **Why no `async` dependencies?** Your code is a pure synchronous library.
> The host provides all async I/O. You should not add `tokio`, `reqwest`, or any
> networking crate — they will not work inside the isolation boundary.

---

## Step 2 — Write the App Logic

Create `apps/TodoApp/backend/wasm-module/src/lib.rs`:

```rust
//! TodoApp — a minimal demonstration app for the Synapp platform.
//!
//! All business logic lives here. The host (transformer-service) is completely
//! unaware of the concept of a "todo" — it only sees generic HostEffect values.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;

// ── Identity ──────────────────────────────────────────────────────────────────

/// Your app's unique identifier across the entire platform.
/// This string appears in URLs, storage namespaces, and audit logs.
pub const APP_ID: &str = "todo-app";

// ── Action name constants ─────────────────────────────────────────────────────

/// Always define action names as constants. They are your public API.
/// Breaking changes to these strings require a migration plan.
pub const ACTION_CREATE_TODO: &str = "todo.create";
pub const ACTION_LIST_TODOS:  &str = "todo.list";

// ── Platform contracts (re-exported from the host, copied here for isolation) ─

/// An effect the host will execute on your behalf.
/// The host injects app_id and user_id — you never set them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HostEffect {
    PutDocument {
        collection: String,
        doc_id:     String,  // empty string → host generates a UUID
        data:       Value,
    },
    QueryDocuments {
        collection: String,
        filters:    HashMap<String, String>,
        limit:      i32,
    },
    WriteTextFile {
        file_path:    String,
        content:      String,
        content_type: String,
    },
    ListFiles {
        prefix: String,
    },
}

/// Result of an action: what to return to the caller, plus what I/O to run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppActionResult {
    pub ok:      bool,
    pub payload: Value,
    pub effects: Vec<HostEffect>,
}

/// Caller identity — derived from the validated JWT by the host.
/// Read-only. You cannot modify or forge these values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallerCtx {
    pub actor_id:     String,
    pub actor_type:   String,
    pub principal_id: String,
}

// ── Semantic manifest ─────────────────────────────────────────────────────────

/// Returns the JSON that tells the AI Copilot and external agents:
///   - What tools your app provides
///   - What parameters each tool accepts
///   - What scope is required to call each tool
///
/// This is the "Semantic Interface" of your app.
pub fn tool_registration_payload() -> Value {
    json!({
        "app_id": APP_ID,
        "tools": [
            {
                "action_name": ACTION_CREATE_TODO,
                "description": "Creates a new to-do item for the current user.",
                "json_schema": {
                    "type": "object",
                    "properties": {
                        "title":       { "type": "string", "minLength": 1, "maxLength": 200 },
                        "description": { "type": "string", "maxLength": 2000 },
                        "due_date":    { "type": "string", "format": "date" }
                    },
                    "required": ["title"]
                },
                "required_scope": "todo-app:write"
            },
            {
                "action_name": ACTION_LIST_TODOS,
                "description": "Returns all to-do items for the current user.",
                "json_schema": {
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer", "minimum": 1, "maximum": 100 }
                    }
                },
                "required_scope": "todo-app:read"
            }
        ]
    })
}

// ── Admin settings schema ─────────────────────────────────────────────────────

/// Returns the schema that drives the settings form in the Admin UI.
/// The platform renders these fields dynamically — no frontend code needed.
pub fn settings_schema_payload() -> Value {
    json!({
        "app_id": APP_ID,
        "title": "TodoApp",
        "description": "Settings for the TodoApp extension.",
        "fields": [
            {
                "key":        "max_items_per_user",
                "label":      "Max Items Per User",
                "field_type": "integer",
                "required":   true,
                "default":    100,
                "help":       "Maximum number of to-do items a single user can create."
            },
            {
                "key":        "allow_due_dates",
                "label":      "Allow Due Dates",
                "field_type": "boolean",
                "required":   true,
                "default":    true,
                "help":       "Whether users can set due dates on to-do items."
            }
        ]
    })
}

// ── Action dispatcher ─────────────────────────────────────────────────────────

/// The platform calls this function for every HTTP action request.
///
/// Rules:
///   - This function MUST be pure (no I/O, no async, no side effects).
///   - Return HostEffect values for all I/O you need.
///   - The host executes effects in order and injects app_id + user_id.
pub fn execute_action(
    action_name: &str,
    arguments:   &Value,
    caller:      &CallerCtx,
) -> AppActionResult {
    match action_name {
        ACTION_CREATE_TODO => action_create_todo(arguments, caller),
        ACTION_LIST_TODOS  => action_list_todos(arguments),
        _ => AppActionResult {
            ok:      false,
            payload: json!({ "error": format!("Unknown action: {action_name}") }),
            effects: vec![],
        },
    }
}

// ── Individual action implementations ────────────────────────────────────────

fn action_create_todo(arguments: &Value, _caller: &CallerCtx) -> AppActionResult {
    // 1. Validate input
    let title = match arguments.get("title").and_then(Value::as_str) {
        Some(t) if !t.is_empty() => t.to_string(),
        _ => {
            return AppActionResult {
                ok:      false,
                payload: json!({ "error": "title is required and must be non-empty" }),
                effects: vec![],
            };
        }
    };

    let description = arguments
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let due_date = arguments
        .get("due_date")
        .and_then(Value::as_str)
        .map(String::from);

    // 2. Build the document data
    let mut data = json!({
        "title":       title,
        "description": description,
        "done":        false,
        "created_at":  "2026-01-01T00:00:00Z"  // In a real app, get this from arguments or a clock effect
    });
    if let Some(dd) = due_date {
        data["due_date"] = json!(dd);
    }

    // 3. Return a PutDocument effect — the host will store this under your app_id + user_id
    AppActionResult {
        ok: true,
        // payload is returned to the HTTP caller immediately; host_results are added after effects execute
        payload: json!({ "queued": true }),
        effects: vec![HostEffect::PutDocument {
            collection: "todos".to_string(),
            doc_id:     "".to_string(),  // empty = host generates a new UUID
            data,
        }],
    }
}

fn action_list_todos(arguments: &Value) -> AppActionResult {
    let limit = arguments
        .get("limit")
        .and_then(Value::as_i64)
        .map(|v| v.clamp(1, 100) as i32)
        .unwrap_or(50);

    // Return a QueryDocuments effect — the host queries only THIS user's todos
    AppActionResult {
        ok: true,
        payload: json!({ "queued": true }),
        effects: vec![HostEffect::QueryDocuments {
            collection: "todos".to_string(),
            filters:    HashMap::new(),  // no additional filters — host already scopes by user_id
            limit,
        }],
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_caller() -> CallerCtx {
        CallerCtx {
            actor_id:     "user-123".to_string(),
            actor_type:   "human".to_string(),
            principal_id: "user-123".to_string(),
        }
    }

    #[test]
    fn create_todo_returns_put_document_effect() {
        let args = json!({ "title": "Buy milk", "description": "Whole milk" });
        let result = execute_action(ACTION_CREATE_TODO, &args, &test_caller());

        assert!(result.ok);
        assert_eq!(result.effects.len(), 1);

        match &result.effects[0] {
            HostEffect::PutDocument { collection, doc_id, data } => {
                assert_eq!(collection, "todos");
                assert_eq!(doc_id, "");  // host generates UUID
                assert_eq!(data["title"], "Buy milk");
                assert_eq!(data["done"], false);
            }
            _ => panic!("Expected PutDocument effect"),
        }
    }

    #[test]
    fn create_todo_rejects_missing_title() {
        let args = json!({ "description": "No title here" });
        let result = execute_action(ACTION_CREATE_TODO, &args, &test_caller());

        assert!(!result.ok);
        assert!(result.effects.is_empty());
        assert!(result.payload["error"].as_str().unwrap().contains("title is required"));
    }

    #[test]
    fn list_todos_returns_query_effect() {
        let args = json!({ "limit": 10 });
        let result = execute_action(ACTION_LIST_TODOS, &args, &test_caller());

        assert!(result.ok);
        assert_eq!(result.effects.len(), 1);

        match &result.effects[0] {
            HostEffect::QueryDocuments { collection, limit, .. } => {
                assert_eq!(collection, "todos");
                assert_eq!(*limit, 10);
            }
            _ => panic!("Expected QueryDocuments effect"),
        }
    }

    #[test]
    fn list_todos_clamps_limit_to_100() {
        let args = json!({ "limit": 9999 });
        let result = execute_action(ACTION_LIST_TODOS, &args, &test_caller());
        match &result.effects[0] {
            HostEffect::QueryDocuments { limit, .. } => assert_eq!(*limit, 100),
            _ => panic!(),
        }
    }

    #[test]
    fn unknown_action_returns_error() {
        let result = execute_action("todo.doesnotexist", &json!({}), &test_caller());
        assert!(!result.ok);
        assert!(result.effects.is_empty());
    }

    #[test]
    fn tool_registration_payload_is_valid() {
        let payload = tool_registration_payload();
        assert_eq!(payload["app_id"], APP_ID);
        let tools = payload["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 2);
        // Both tools must have an action_name
        for tool in tools {
            assert!(tool["action_name"].is_string());
            assert!(tool["json_schema"].is_object());
            assert!(tool["required_scope"].is_string());
        }
    }
}
```

> **Run these tests now** before proceeding:
> ```bash
> cd apps/TodoApp/backend/wasm-module
> cargo test
> ```
> All 6 tests should pass in under a second. No Docker, no database.

---

## Step 3 — Add the Crate to the Workspace

Open `services/transformer-service/Cargo.toml` and add your crate to the workspace
members and as a dependency:

```toml
# In [workspace] members array — add:
"../../apps/TodoApp/backend/wasm-module",

# In [dependencies] section — add:
todo-app-wasm-module = { path = "../../apps/TodoApp/backend/wasm-module" }
```

---

## Step 4 — Register Your App in the AppRegistry

Open `services/transformer-service/src/app_dispatch.rs`.

Add the import at the top (next to the existing BlueprintApp import):

```rust
use todo_app_wasm_module::{
    APP_ID as TODO_APP_ID, AppActionResult, CallerCtx, HostEffect,
    execute_action as todo_execute_action,
};
```

In `AppRegistry::new()`, add your app next to the existing BlueprintApp entry:

```rust
// TodoApp — all logic lives in the wasm module crate.
handlers.insert(
    TODO_APP_ID.to_string(),
    Arc::new(|action_name, arguments, caller| {
        todo_execute_action(action_name, arguments, caller)
    }),
);
```

That is the **only change needed in the core platform** for a new app.

---

## Step 5 — Build and Verify Compilation

```bash
cd services/transformer-service
cargo build
```

If the build succeeds, your app is integrated. If you see a compilation error, check that
the `APP_ID`, `execute_action`, `AppActionResult`, `CallerCtx`, and `HostEffect` types are
all exported from your crate (all are `pub` in the lib.rs above).

---

## Step 6 — Restart the Platform

Rebuild and restart the transformer container with your new code:

```bash
cd /path/to/NexusCore

# Build the frontend first (needed for the Docker image)
cd frontend && npm install && npm run build && cd ..

# Rebuild and restart only the transformer service
docker compose build transformer-service
docker compose up -d transformer-service
```

Wait for the container to become healthy:
```bash
docker compose ps
```

---

## Step 7 — Call Your App

First, log in to get a token:

```bash
TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123!"}' \
  | jq -r .access_token)
```

Create a to-do:

```bash
curl -s -X POST http://localhost:8080/api/v1/apps/todo-app/actions/todo.create \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"title": "Buy groceries", "description": "Milk and eggs"}}' \
  | jq
```

Expected response:

```json
{
  "ok": true,
  "app_id": "todo-app",
  "action_name": "todo.create",
  "payload": { "queued": true },
  "host_results": [
    { "doc_id": "550e8400-e29b-41d4-a716-446655440000" }
  ]
}
```

List your to-dos:

```bash
curl -s -X POST http://localhost:8080/api/v1/apps/todo-app/actions/todo.list \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {"limit": 10}}' \
  | jq
```

Expected response:

```json
{
  "ok": true,
  "app_id": "todo-app",
  "action_name": "todo.list",
  "payload": { "queued": true },
  "host_results": [
    {
      "documents": [
        {
          "doc_id": "550e8400-e29b-41d4-a716-446655440000",
          "data": { "title": "Buy groceries", "description": "Milk and eggs", "done": false }
        }
      ]
    }
  ]
}
```

---

## Step 8 — Verify Data Isolation

Create a second user and verify they cannot see the first user's to-dos:

```bash
# Create a second user (as admin)
curl -s -X POST http://localhost:8080/api/admin/users \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"alice123!","email":"alice@example.com"}' \
  | jq

# Log in as alice
ALICE_TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"alice123!"}' \
  | jq -r .access_token)

# Alice should see an empty list
curl -s -X POST http://localhost:8080/api/v1/apps/todo-app/actions/todo.list \
  -H "Authorization: Bearer $ALICE_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"arguments": {}}' \
  | jq '.host_results[0].documents | length'
# Output: 0
```

The platform enforces this. You did not write any data-scoping code.

---

## What You Just Built

| What | How |
|---|---|
| App business logic | `apps/TodoApp/backend/wasm-module/src/lib.rs` |
| Data persistence | `PutDocument` / `QueryDocuments` HostEffects |
| AI tool discovery | `tool_registration_payload()` in your crate |
| Admin settings | `settings_schema_payload()` in your crate |
| Platform registration | One entry in `AppRegistry::new()` |
| Unit tests | Pure functions, no mocks, run in < 1 second |

---

## Next Steps

- **[03-capabilities-reference.md](03-capabilities-reference.md)** — Full reference for all HostEffects,
  the complete settings schema format, and the semantic tool manifest specification.
- **[04-security-and-testing.md](04-security-and-testing.md)** — Mandatory security checklist and
  integration test patterns.
