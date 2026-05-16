# Synapp App Developer Guide

> **Who this is for:** Developers building extension apps for the Synapp platform.
> You do **not** need to touch any core service (IAM, Meta, Storage, Transformer) to build a working app.

---

## Five-Minute Overview

An **app** in Synapp is a self-contained Rust crate that gets compiled to a `cdylib + rlib`.
The app crate lives entirely in `apps/<YourApp>/backend/wasm-module/` and is linked into the
platform at compile time. The host never contains any of your business logic — it only calls
your pure functions and executes the effects they return.

```
Your app crate
    └── execute_action(action_name, arguments, caller_ctx) → AppActionResult
                                                                 ├── payload   ← JSON returned to caller
                                                                 └── effects   ← I/O the host will do for you
                                                                       ├── PutDocument
                                                                       ├── QueryDocuments
                                                                       ├── WriteTextFile
                                                                       └── ListFiles
```

The host executes every effect on your behalf, injecting `app_id` and `user_id` from the
authenticated JWT — **your code can never forge those values**.

---

## What You Can Build

| Capability | Mechanism |
|---|---|
| Store and retrieve structured data | `PutDocument` / `QueryDocuments` effects |
| Store and list files | `WriteTextFile` / `ListFiles` effects |
| Admin settings panel with UI | Settings schema returned from your crate |
| AI-discoverable tools | Semantic manifest (tool descriptions + JSON schemas) |
| React to platform events | NATS JetStream subjects (event subscription in wasm-engine) |
| Frontend pages and actions | Schema-driven UI contributions via `AppUIService` |

---

## Guide Structure

| File | What you learn |
|---|---|
| **[01-concepts.md](01-concepts.md)** | The mental model, effect pattern, isolation boundaries, and security invariants. Read this first. |
| **[02-building-your-first-app.md](02-building-your-first-app.md)** | A complete, runnable step-by-step tutorial. You will have a working app at the end. |
| **[03-capabilities-reference.md](03-capabilities-reference.md)** | Every `HostEffect`, the settings schema, the semantic tool manifest, and the HTTP API surface — all in full detail with examples. |
| **[04-security-and-testing.md](04-security-and-testing.md)** | Mandatory security checklist and how to write and run tests for your app. |
| **[05-contacts-app-ux-spec.md](05-contacts-app-ux-spec.md)** | UX spec for the three-pane Contacts workspace, state model, journeys, accessibility, and platform dependency notes. |
| **[06-contacts-app-architecture.md](06-contacts-app-architecture.md)** | Architecture contract for `contacts-app`: action surface, collections, UI schema boundaries, testing strategy, and escalation dependencies. |

---

## Quick Reference

### App entry point
`apps/<YourApp>/backend/wasm-module/src/lib.rs` → `execute_action()`

### HTTP API for calling your app
```
POST /api/v1/apps/{app_id}/actions/{action_name}
Authorization: Bearer <jwt>
Content-Type: application/json

{ "arguments": { ... } }
```

### One platform code change needed
Add your app to `AppRegistry::new()` in:
`services/transformer-service/src/app_dispatch.rs`

### BlueprintApp is the canonical reference
`apps/BlueprintApp/backend/wasm-module/src/lib.rs` shows every concept in a working implementation.
