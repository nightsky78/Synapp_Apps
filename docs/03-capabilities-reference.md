# 03 — Capabilities Reference

> **Goal:** Comprehensive reference for every capability your app can use.
> Read this when you need the exact fields, types, and behaviour for a specific feature.

---

## Table of Contents

1. [HostEffect: PutDocument](#1-hosteffect-putdocument)
2. [HostEffect: QueryDocuments](#2-hosteffect-querydocuments)
3. [HostEffect: WriteTextFile](#3-hosteffect-writetextfile)
4. [HostEffect: ListFiles](#4-hosteffect-listfiles)
5. [Settings Schema](#5-settings-schema)
6. [Semantic Tool Manifest](#6-semantic-tool-manifest)
7. [HTTP API Reference](#7-http-api-reference)
8. [AppActionResult Reference](#8-appactionresult-reference)
9. [CallerCtx Reference](#9-callercxt-reference)

---

## 1. HostEffect: PutDocument

**Purpose:** Save or update a JSON document in your app's document store.

### Rust definition

```rust
HostEffect::PutDocument {
    collection: String,  // Logical name of the collection (like a table name)
    doc_id:     String,  // Document ID. Pass "" to have the host generate a UUID.
    data:       Value,   // Any valid JSON object
}
```

### Behaviour

- If `doc_id` is empty (`""`), the host generates a UUIDv4 and returns it in `host_results`.
- If `doc_id` is non-empty, the document is upserted (created or fully replaced).
- The host automatically adds `app_id` and `user_id` (from `CallerCtx.principal_id`) to the
  storage key. **You cannot read or write documents belonging to another user or app.**
- `data` must be a JSON **object** (`{}`). Arrays and primitives at the top level are rejected.

### What `host_results` contains after execution

```json
{ "doc_id": "550e8400-e29b-41d4-a716-446655440000" }
```

### Collection naming guidelines

| ✅ Good | ❌ Bad |
|---|---|
| `"todos"`, `"events"`, `"messages"` | `"todos/user-123"` (host scopes by user automatically) |
| Lowercase, alphanumeric, hyphens | `"my Collection!"` (special characters) |

### Full example

```rust
fn action_save_note(arguments: &Value, _caller: &CallerCtx) -> AppActionResult {
    let title = arguments["title"].as_str().unwrap_or("Untitled").to_string();
    let body  = arguments["body"].as_str().unwrap_or("").to_string();

    AppActionResult {
        ok: true,
        payload: json!({ "queued": true }),
        effects: vec![HostEffect::PutDocument {
            collection: "notes".to_string(),
            doc_id:     "".to_string(),     // auto-generate ID
            data:       json!({
                "title": title,
                "body":  body,
                "pinned": false,
            }),
        }],
    }
}
```

### Update an existing document

```rust
effects: vec![HostEffect::PutDocument {
    collection: "notes".to_string(),
    doc_id:     "the-known-uuid".to_string(),  // replaces the existing document
    data:       json!({ "title": "Updated title", "body": "...", "pinned": true }),
}],
```

---

## 2. HostEffect: QueryDocuments

**Purpose:** Retrieve a list of JSON documents from your app's document store.

### Rust definition

```rust
HostEffect::QueryDocuments {
    collection: String,                       // Must match the collection used in PutDocument
    filters:    HashMap<String, String>,      // Optional key=value filters on the JSON data
    limit:      i32,                          // Maximum number of documents to return (1–1000)
}
```

### Behaviour

- Returns all documents in the collection that belong to the current `principal_id`.
- `filters` performs simple equality matching on top-level JSON fields.
  Example: `{"done": "false"}` returns only documents where `data.done == false`.
- Results are returned newest-first by default.
- The host scopes the query to `app_id + user_id` automatically — no cross-user data leakage.

### What `host_results` contains after execution

```json
{
  "documents": [
    {
      "doc_id": "550e8400-e29b-41d4-a716-446655440000",
      "data": { "title": "Buy milk", "done": false }
    },
    {
      "doc_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
      "data": { "title": "Walk the dog", "done": true }
    }
  ]
}
```

### Limit guidelines

| Use case | Suggested limit |
|---|---|
| Dashboard widget (most recent N) | 5–20 |
| User list page | 50–100 |
| Export / bulk action | 500–1000 |

### Full example

```rust
fn action_list_pending(arguments: &Value) -> AppActionResult {
    let limit = arguments.get("limit")
        .and_then(Value::as_i64)
        .map(|v| v.clamp(1, 100) as i32)
        .unwrap_or(25);

    let mut filters = HashMap::new();
    filters.insert("done".to_string(), "false".to_string()); // only incomplete todos

    AppActionResult {
        ok: true,
        payload: json!({ "queued": true }),
        effects: vec![HostEffect::QueryDocuments {
            collection: "todos".to_string(),
            filters,
            limit,
        }],
    }
}
```

---

## 3. HostEffect: WriteTextFile

**Purpose:** Upload a UTF-8 text file to your app's virtual filesystem (VFS) namespace.

### Rust definition

```rust
HostEffect::WriteTextFile {
    file_path:    String,  // Relative path (e.g. "reports/2026-01.csv")
    content:      String,  // UTF-8 file content
    content_type: String,  // MIME type (e.g. "text/csv", "text/plain", "application/json")
}
```

### Behaviour

- `file_path` must be a **relative** path. Absolute paths (`/etc/passwd`) and traversal
  sequences (`../`) are rejected with HTTP 400.
- The host namespaces the storage path as `apps/{app_id}/users/{principal_id}/{file_path}`.
  You cannot write outside your app's namespace.
- Files are stored encrypted (AES-256-GCM) in S3/MinIO.
- If a file at the same path already exists, it is overwritten.
- Binary files are not supported via this effect. Use `PutDocument` to store binary metadata
  and manage binary uploads through the core Files API.

### What `host_results` contains after execution

```json
{
  "file_path":    "reports/2026-01.csv",
  "storage_key":  "apps/todo-app/users/user-123/reports/2026-01.csv",
  "size":         1048
}
```

### Valid file path examples

| ✅ Allowed | ❌ Rejected |
|---|---|
| `"report.txt"` | `"/etc/passwd"` — absolute path |
| `"exports/data.csv"` | `"../other-app/secret.txt"` — traversal |
| `"2026/january/summary.json"` | `""` — empty path |

### Full example

```rust
fn action_export_csv(arguments: &Value, _caller: &CallerCtx) -> AppActionResult {
    // Build a CSV string from the arguments
    let items = arguments["items"].as_array().cloned().unwrap_or_default();
    let mut csv = "id,title,done\n".to_string();
    for item in &items {
        let id    = item["id"].as_str().unwrap_or("unknown");
        let title = item["title"].as_str().unwrap_or("");
        let done  = item["done"].as_bool().unwrap_or(false);
        csv.push_str(&format!("{id},{title},{done}\n"));
    }

    AppActionResult {
        ok: true,
        payload: json!({ "queued": true }),
        effects: vec![HostEffect::WriteTextFile {
            file_path:    "exports/todos.csv".to_string(),
            content:      csv,
            content_type: "text/csv".to_string(),
        }],
    }
}
```

---

## 4. HostEffect: ListFiles

**Purpose:** List files stored in your app's VFS namespace under a given path prefix.

### Rust definition

```rust
HostEffect::ListFiles {
    prefix: String,  // Path prefix to list (e.g. "exports/" or "" for root)
}
```

### Behaviour

- Returns all files whose path starts with `prefix` within `apps/{app_id}/users/{principal_id}/`.
- Pass `""` (empty string) to list all files in the user's namespace for this app.
- Results are not paginated. If you expect many files, use a specific prefix to narrow results.

### What `host_results` contains after execution

```json
{
  "entries": [
    {
      "file_path":    "exports/todos.csv",
      "size":         1048,
      "content_type": "text/csv",
      "updated_at":   "2026-01-15T10:30:00Z"
    }
  ]
}
```

### Full example

```rust
fn action_list_exports(_arguments: &Value) -> AppActionResult {
    AppActionResult {
        ok: true,
        payload: json!({ "queued": true }),
        effects: vec![HostEffect::ListFiles {
            prefix: "exports/".to_string(),
        }],
    }
}
```

---

## 5. Settings Schema

Your app can expose a settings panel in the platform's Admin UI without writing any frontend code.
Return a schema from `settings_schema_payload()` in your crate and the platform renders the form.

### Schema structure

```rust
pub fn settings_schema_payload() -> Value {
    json!({
        "app_id":       APP_ID,           // Must match your app's APP_ID constant
        "title":        "My App",         // Display name in the Admin sidebar
        "description":  "Settings for My App.",  // Shown below the title
        "fields": [
            // ... field definitions (see below)
        ]
    })
}
```

### Field types

| `field_type` | Renders as | Value type |
|---|---|---|
| `"string"` | Text input | String |
| `"integer"` | Number input | Integer |
| `"boolean"` | Toggle switch | Boolean |
| `"url"` | URL input with validation | String (valid URL) |
| `"secret"` | Password input, never displayed again | String |

### Complete field definition

```json
{
    "key":        "max_items",        // The settings key (used in API reads/writes)
    "label":      "Max Items",        // Human-readable label shown in the UI
    "field_type": "integer",          // One of the types in the table above
    "required":   true,               // Whether the field must be set
    "default":    100,                // Default value (displayed when not yet set)
    "help":       "Tooltip text."     // Optional tooltip shown next to the field
}
```

### Reading and writing settings via HTTP

**Read current settings (for your app):**
```bash
GET /api/admin/apps/settings/{app_id}
Authorization: Bearer <admin-jwt>
```

Response:
```json
{
  "max_items": 100,
  "allow_due_dates": true,
  "webhook_url": "https://example.com/hook"
}
```

**Update settings:**
```bash
PUT /api/admin/apps/settings/{app_id}
Authorization: Bearer <admin-jwt>
Content-Type: application/json

{ "max_items": 250, "allow_due_dates": false }
```

**Read settings from within an action** (to enforce your configured limits):
```rust
// Within execute_action, you receive settings via a dedicated effect.
// For now, pass settings values in action arguments from the frontend,
// or design your app to read defaults from tool_registration_payload().
// Full settings-as-effect support is planned.
```

### Headless/agent settings access

Agents with the `admin:apps:write` scope can manage settings programmatically:

```bash
# Read
GET /api/agents/v1/apps/{app_id}/settings

# Write
PUT /api/agents/v1/apps/{app_id}/settings
```

---

## 6. Semantic Tool Manifest

The semantic manifest is how your app tells the AI Copilot and external agents:
- What tools are available
- What parameters each tool accepts
- What permission (scope) is required

### Manifest structure

```rust
pub fn tool_registration_payload() -> Value {
    json!({
        "app_id": APP_ID,
        "tools": [
            {
                // Internal action name — must match a case in your execute_action() match
                "action_name": "todo.create",

                // Human-readable description for the AI to understand when to use this tool
                "description": "Creates a new to-do item for the authenticated user.",

                // JSON Schema for the arguments object.
                // The AI uses this to construct valid arguments.
                "json_schema": {
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "The title of the to-do item.",
                            "minLength": 1,
                            "maxLength": 200
                        },
                        "description": {
                            "type": "string",
                            "description": "Optional details about the to-do item."
                        }
                    },
                    "required": ["title"]
                },

                // Scope required to call this action.
                // The caller's JWT must include this scope.
                "required_scope": "todo-app:write"
            }
        ]
    })
}
```

### Writing good tool descriptions

The AI selects tools based on the `description` field. Be explicit about:
- What the tool does (not just what it is named)
- Any important constraints (e.g., "maximum 100 items")
- When NOT to use it if ambiguous

| ✅ Good description | ❌ Vague description |
|---|---|
| `"Creates a new to-do item with a required title and optional due date for the current user."` | `"Creates a todo."` |
| `"Returns a list of the user's incomplete to-do items, newest first, up to the specified limit."` | `"Lists todos."` |

### How tools are discovered by AI agents

External agents call:

```bash
GET /api/v1/tools?channel=external_agent&q=todo&limit=10
Authorization: Bearer <agent-jwt>
```

Response excerpt:
```json
{
  "tools": [
    {
      "tool_id":       "todo-app:todo.create",
      "app_id":        "todo-app",
      "display_name":  "todo.create",
      "description":   "Creates a new to-do item...",
      "input_schema":  { ... },
      "required_scope": "todo-app:write",
      "channels":      ["ui", "agent", "orchestrator"]
    }
  ]
}
```

The agent then calls:
```bash
POST /api/v1/apps/todo-app/actions/todo.create
Authorization: Bearer <agent-jwt>
Content-Type: application/json

{ "arguments": { "title": "Schedule dentist appointment" } }
```

---

## 7. HTTP API Reference

### Execute an action

```
POST /api/v1/apps/{app_id}/actions/{action_name}
Authorization: Bearer <jwt>
Content-Type: application/json

{
  "arguments": { ... }   // JSON object matching your action's json_schema
}
```

Response (200 OK):
```json
{
  "ok":          true,
  "app_id":      "todo-app",
  "action_name": "todo.create",
  "payload":     { "queued": true },
  "host_results": [
    { "doc_id": "550e8400-..." }
  ]
}
```

| Field | Meaning |
|---|---|
| `ok` | Whether your `execute_action()` returned `ok: true` |
| `payload` | The JSON you put in `AppActionResult.payload` |
| `host_results` | One entry per effect, in order of declaration |

### Error responses

| HTTP Status | Cause |
|---|---|
| 401 Unauthorized | Missing or expired JWT |
| 403 Forbidden | Token is valid but does not have required scope |
| 404 Not Found | `app_id` is not registered in `AppRegistry::new()` |
| 400 Bad Request | Invalid JSON body, or effect validation failure (e.g. path traversal attempt) |
| 500 Internal Server Error | gRPC failure to a backend storage service |

### Discover available tools

```
GET /api/v1/tools
Authorization: Bearer <jwt>

Query parameters:
  channel   — "ui" | "agent" | "orchestrator" | "external_agent"
  app_id    — filter to a specific app
  q         — text search on tool name/description
  limit     — page size (default 50, max 200)
  cursor    — opaque cursor for pagination
```

---

## 8. AppActionResult Reference

```rust
pub struct AppActionResult {
    pub ok:      bool,
    pub payload: Value,
    pub effects: Vec<HostEffect>,
}
```

| Field | Rules |
|---|---|
| `ok` | Set to `false` for all validation/business logic errors. The HTTP response is still 200 — the caller reads `ok` to detect failures. |
| `payload` | Any JSON value. Returned to the HTTP caller as-is. If `{"queued": true}` is the payload **and** effects are present, the host replaces it with the first effect's result. |
| `effects` | Executed in declaration order. An effect failure stops execution and returns HTTP 500. |

### The `queued: true` sentinel

When you want the response to reflect an effect's result (e.g. the document that was stored),
return `payload: json!({"queued": true})` and the host replaces it with `host_results[0]`.

When you want a custom response alongside effect results, return a real payload value:
```rust
payload: json!({ "message": "Created successfully", "title": title }),
// host_results will still contain the doc_id from the PutDocument effect
```

---

## 9. CallerCtx Reference

```rust
pub struct CallerCtx {
    pub actor_id:     String,  // The entity that made the HTTP request
    pub actor_type:   String,  // "human" or "agent"
    pub principal_id: String,  // The owning human — used as the storage namespace
}
```

| Scenario | `actor_id` | `actor_type` | `principal_id` |
|---|---|---|---|
| Human user calls directly | user's UUID | `"human"` | user's UUID |
| Agent calls on user's behalf | agent's UUID | `"agent"` | human owner's UUID |
| Admin calls | admin's UUID | `"human"` | admin's UUID |

**Always use `principal_id` for data namespacing.** When an agent calls your app on behalf of
a user, the data should live in the user's namespace, not the agent's.

```rust
// ✅ Correct: store under the human owner
data["owner_note"] = json!(format!("Created by {} for {}", 
    caller.actor_id, caller.principal_id));

// ❌ Wrong: using actor_id would use the agent's ID, breaking user data isolation
data["owner_note"] = json!(caller.actor_id);
```
