# 06 - Contacts App Architecture

> **Goal:** Turn the UX spec into implementation-ready contracts for `contacts-app` while staying inside current platform constraints.

## Stage Status

- Stage: Synapp Apps Architect
- Status: PASSED
- Liaison consulted: yes

## Scope and Boundaries

- Repository scope: `/home/johannes/projects/Synapp_apps`
- App id: `contacts-app`
- Runtime contract: manifest-first action/tool declaration, schema-driven UI, principal-scoped storage
- App logic: pure action handlers that emit host effects only

## Action Contract

All app actions are declared in `synapp.app.json` tools and routed through the app action endpoint.

| Action | Purpose | Required scope |
|---|---|---|
| `contacts.list` | Return list view model and filters for current principal | `contacts:read` |
| `contacts.create` | Create a manual contact | `contacts:write` |
| `contacts.update` | Update manual fields and user overlays | `contacts:write` |
| `contacts.delete` | Soft-delete contact via archive flag | `contacts:write` |
| `contacts.sync-directory` | Refresh platform-user projection status | `contacts:write` |

### Agent-facing parity note

- Target parity: agents and humans must invoke the same contacts action surface for the same principal.
- Current limitation: app action execution parity for agents is platform-dependent and must be tracked as an escalation.

## Data Model

Data is stored under `CallerCtx.principal_id` only. No action accepts principal or actor identifiers from input.

### Collections

- `contacts`: canonical contact records (manual and projected)
- `contact_groups`: user-defined groups and membership indexes
- `contact_sync_state`: directory sync status and watermark
- `contact_bootstrap_requests`: explicit requests used when directory enumeration is unavailable

### Contact document shape

```json
{
  "id": "host-generated-or-stable-key",
  "display_name": "Jane Doe",
  "first_name": "Jane",
  "last_name": "Doe",
  "organization": "Synapp",
  "job_title": "PM",
  "emails": ["jane@example.com"],
  "phones": ["+49-..."],
  "groups": ["all", "platform-users"],
  "notes": "prefers async updates",
  "favorite": false,
  "source": "manual|platform_user",
  "source_user_id": "optional-platform-user-id",
  "archived": false,
  "updated_at": "iso-8601"
}
```

### Determinism rules

- Manual contacts use host-generated IDs.
- Platform-projected contacts use stable external keys to guarantee idempotent upsert.
- Delete is represented as `archived=true` to preserve history and avoid accidental resurrection conflicts.

## UI Schema Boundary

- `ui_schemas.main`: 3-pane workspace envelope with `contacts.workspace.v1` component.
- `ui_schemas.settings`: safe settings metadata only.
- Raw HTML, inline script, iframe, and DOM handler injection are forbidden.

## Host Effect Plan

Current implementation plan uses principal-scoped document effects:

- `PutDocument` for create/update/delete/sync state writes
- `QueryDocuments` for list/detail/group queries

`WriteTextFile` and `ListFiles` are not required for initial contacts scope.

## Security Constraints

- `#![forbid(unsafe_code)]` in Rust app crate.
- No direct network or filesystem APIs.
- No actor/principal overrides from caller arguments.
- Validate all user-provided fields (required name, email format, bounded text lengths).
- Negative-path behavior is explicit for insufficient scopes and cross-principal attempts.

## Testing Strategy

### Unit tests

- action dispatch and unknown action fallback
- valid create/update/delete emits expected host effects
- validation failures return `ok=false` and no effects
- sync action returns explicit blocked/sync-required payload when directory source unavailable

### Integration/E2E expectations

- three-pane UI renders with stable selectors
- create/edit/delete roundtrip updates list and detail panes
- platform-users group displays sync state clearly
- principal isolation verified across two identities
- agent parity tests run when platform endpoint support is available

## Platform Escalations

### Escalation A - Agent action parity

- Problem: app action runtime parity for agent actors is not guaranteed on current route.
- Impact: "managed by human and agent" cannot be fully met through app-only changes.
- Requested platform outcome: enable scoped agent invocation of app actions with the same action contracts.

### Escalation B - Auto-bootstrap all registered users

- Problem: app runtime lacks a documented directory enumeration contract for all registered users.
- Impact: automatic contacts seeding cannot be completed natively by the app today.
- Requested platform outcome: provide safe, principal-scoped directory projection contract or host-managed bootstrap feed.

## Handoff to UI-Designer

- Preserve 3-pane screenshot-inspired composition.
- Include explicit sync-status affordance for platform-users group.
- Keep selection stable while editing.
- Ensure keyboard-first navigation and field-level validation states.

## Handoff to Implementer

- Scaffold `apps/first-party/contacts-app` with manifest, plugin schema, Rust action planner, and UI shell.
- Implement contacts CRUD with principal-scoped document effects.
- Implement sync-state fallback for directory unavailability.
- Add deterministic tests for success and failure paths.
