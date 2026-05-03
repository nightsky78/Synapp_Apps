# 01 — Concepts: The App Platform Mental Model

> **Goal of this page:** Give you a solid mental model before you write any code.
> Understanding *why* things work this way will make all the subsequent steps obvious.

---

## The Core Problem This Architecture Solves

In a traditional platform, the host service would contain special-case branches for each installed app:

```rust
// ❌ What we deliberately avoid
if app_id == "calendar" { calendar::handle(action) }
else if app_id == "crm"  { crm::handle(action) }
```

This couples the host to every app, makes security audits difficult, and violates the
architectural rule that the host must remain app-agnostic.

**Synapp's solution:** The host only knows about *generic platform contracts* (`HostEffect`, `AppActionResult`).
Your app implements those contracts. The host never imports your domain types.

---

## The Effect Pattern (Most Important Concept)

Your `execute_action()` function is a **pure function**. It receives input, returns output, and
does **no I/O whatsoever**. It cannot write to a database, cannot call HTTP endpoints, and cannot
read files. Instead, it returns a list of *effects* — descriptions of I/O that the host will
perform on your behalf.

```
                    ┌─────────────────────────────────────┐
  Caller JWT ───▶   │  Transformer host                   │
                    │  1. Authenticates caller             │
                    │  2. Calls your execute_action()      │
                    │  3. Iterates returned effects        │
                    │  4. Calls gRPC storage services      │
                    │  5. Returns merged JSON response     │
                    └─────────────────────────────────────┘
                              │           ▲
                    pure fn   │           │ AppActionResult { payload, effects }
                    no I/O    ▼           │
                    ┌─────────────────────────────────────┐
                    │  Your app crate (execute_action)    │
                    │  — zero network access              │
                    │  — zero filesystem access           │
                    │  — zero async code                  │
                    └─────────────────────────────────────┘
```

### Why pure functions?

1. **Security** — Your code cannot exfiltrate data, forge identities, or make unauthorized calls.
2. **Testability** — Pure functions are trivially unit-testable. No mocks, no test databases.
3. **Auditability** — The host knows exactly what I/O happened and can log all of it.
4. **Isolation** — A bug in your app cannot corrupt another app's data.

---

## The CallerCtx: Who Is Calling?

The host extracts the caller's identity from the validated JWT and passes it as a `CallerCtx`:

```rust
pub struct CallerCtx {
    pub actor_id: String,    // The entity making the call (human user ID or agent ID)
    pub actor_type: String,  // "human" or "agent"
    pub principal_id: String, // The owning human (same as actor_id for humans; agent's owner for agents)
}
```

**You receive this read-only.** You cannot change these values. The host always re-derives them
from the JWT on every request — your code cannot forge a different identity.

`principal_id` is used as the storage namespace. If an agent calls your app on behalf of a user,
the data is stored under the user's namespace, not the agent's.

---

## The AppActionResult: What Your Function Returns

```rust
pub struct AppActionResult {
    pub ok: bool,          // Did the action succeed?
    pub payload: Value,    // JSON returned to the HTTP caller
    pub effects: Vec<HostEffect>, // I/O the host should perform
}
```

Effects are executed **in order**. If an effect fails, the host returns an HTTP 500 and
does not execute subsequent effects. Design your effect lists to be idempotent where possible.

---

## HostEffect: The Four Platform I/O Operations

Your app can request four types of host-executed I/O:

| Effect | What it does | Storage backend |
|---|---|---|
| `PutDocument` | Save/update a JSON document in your app's document store | PostgreSQL (meta-service) |
| `QueryDocuments` | Query JSON documents with optional filters | PostgreSQL (meta-service) |
| `WriteTextFile` | Upload a text file to your app's VFS namespace | S3/MinIO (storage-service) |
| `ListFiles` | List files under a path prefix in your app's VFS | PostgreSQL (meta-service) |

**Namespace isolation is automatic.** The host always prefixes storage calls with `app_id` and
`user_id` — you cannot read or write data belonging to another app or another user.

Full details and examples for each effect are in [03-capabilities-reference.md](03-capabilities-reference.md).

---

## App Identity

Every app has a unique `app_id` string. This is declared as a constant in your crate:

```rust
pub const APP_ID: &str = "your-app";  // e.g. "calendar", "crm", "todo"
```

Rules:
- Lowercase, alphanumeric, hyphens allowed.
- Must be unique across all installed apps.
- Must not start or end with a hyphen.
- This string appears in all HTTP URLs, storage namespaces, and audit logs.

---

## Dual Interface: Execution + Semantic

Every app exposes two surfaces:

1. **Execution Interface** — the `execute_action()` function, called by authenticated HTTP requests.
2. **Semantic Interface** — tool metadata (name, description, JSON schema, required scope) that
   allows the AI Copilot and external agents to *discover and invoke* your tools automatically.

These never diverge — there is exactly one action handler per semantic tool. The AI calls the
same HTTP endpoint as a human browser would.

---

## Scope-Based Authorization

Every action requires a scope. The host does not enforce scopes itself (the JWT already has
scopes from the IAM service), but you declare the required scope in your tool manifest so
the Copilot and agents know what permission a caller needs.

Standard scope naming convention:

```
{app_id}:read     — for read-only actions
{app_id}:write    — for data-mutating actions
orchestrator:prompt — for AI-invocable tools (no app-specific scope needed)
```

---

## What the Host Never Does

These rules protect you and your users:

- The host **never** reads your crate's source or knows your domain types.
- The host **never** trusts `app_id` or `user_id` values from your returned payload.
- The host **never** executes effects from an unauthenticated request.
- The host **never** stores data from two different `app_id` values in the same namespace.

---

## Summary: The App Developer Contract

As an app developer, you agree to:

1. Keep all business logic inside your `wasm-module` crate.
2. Express all I/O as `HostEffect` values — no direct database, network, or filesystem access.
3. Trust the `CallerCtx` as your identity source — never derive identity from request parameters.
4. Declare accurate semantic manifests so the AI can discover your tools.
5. Never circumvent the platform's namespacing by constructing storage keys that include other users' IDs.

In return, the platform guarantees:

1. Your app's data is always namespaced and isolated.
2. Your `app_id` and `principal_id` are always injected correctly by the host.
3. Your effects are executed in order with platform-level retries and error reporting.
4. Your tools are automatically discoverable by AI agents and the Copilot.
