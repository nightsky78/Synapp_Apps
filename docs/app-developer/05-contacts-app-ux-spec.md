# 05 - Contacts App UX Spec

> **Goal:** Define the UX contract for a Contacts app that renders as a schema-driven, three-pane workspace and stays within Synapp's app-platform rules: pure app logic, caller-scoped data, and safe metadata-only UI declarations.

---

## Product Summary

The Contacts app is a three-pane workspace:

- left pane for contact groups and navigation
- center pane for searchable contact lists
- right pane for contact details, editing, and creation

The app serves both human users and agents, but all records are scoped to `CallerCtx.principal_id`. An agent acts on behalf of its owning human principal and must never open a separate data silo.

All registered platform users are expected to appear as contacts in the current principal's address book. Because the platform does not yet expose an in-app HostEffect to enumerate every registered platform user, the UX spec includes an explicit directory sync state and an escalation note for the missing platform capability.

---

## Platform Contract Assumptions

This UX spec assumes the app follows the canonical Synapp app model:

- app logic is pure and returns `HostEffect` values instead of doing I/O directly
- identity comes from `CallerCtx` and not from request arguments
- `principal_id` is the storage namespace boundary
- schema-driven UI is the only approved way to render host-native UI
- no raw HTML, CSS, JavaScript, iframe, or DOM injection is allowed in the app schema
- manifest-declared actions are the only app actions the UI may invoke

For action naming and scope planning, this spec assumes a Contacts app manifest will expose caller-scoped read/write actions such as `contacts.list`, `contacts.create`, `contacts.update`, `contacts.delete`, and `contacts.sync-directory`.

---

## Information Architecture

### Desktop layout

The workspace should read like a focused contacts manager rather than a generic CRUD form:

- left rail: group navigation and saved filters
- center column: search box, contact list, bulk-select affordances, and sort controls
- right column: selected contact profile, read-only details, and inline edit form

The left rail should stay narrow and persistent. The center column should be the primary browsing surface. The right column should behave like an inspector/editor that can switch between view and edit modes without leaving the workspace.

### Recommended navigation groups

- All contacts
- Platform users
- Favorites
- Recently updated
- User-defined groups
- Archived

The `Platform users` group is a virtual group backed by directory sync, not a manually curated list.

---

## Screen and State Model

### Core states

| State | When it appears | Left pane | Center pane | Right pane |
|---|---|---|---|---|
| `loading` | Initial route open or navigation refresh | Skeleton groups | Skeleton list | Empty inspector shell |
| `syncing_directory` | First launch or manual directory refresh | Sync badge and disabled actions | Partial or placeholder list | Sync message and no editor actions |
| `empty_no_contacts` | No manual contacts and no directory snapshot yet | Groups visible | Empty state with create CTA | Empty inspector with guidance |
| `list_ready` | Contacts loaded and no row selected | Active filter highlighted | Searchable list with sorting | Prompt to select or create |
| `detail_viewing` | A contact is selected | Active group and filters preserved | Selected row highlighted | Read-only contact profile and actions |
| `editing_existing` | Edit invoked from a selected contact | Stable filter state | Selected row stays highlighted | Editable form with Save and Cancel |
| `creating_new` | New contact action invoked | Stable filter state | Optional empty selection | Blank form with Save and Cancel |
| `saving` | Persist action in progress | Controls disabled | List remains visible | Save in progress indicator |
| `delete_confirm` | Delete requested | Controls dimmed | Selected row remains highlighted | Confirmation dialog or inline confirm state |
| `sync_error` | Directory refresh failed | Retry affordance visible | Last known list remains usable | Error copy with retry |
| `permission_denied` | Scope or principal check fails | Filtered or empty navigation | No results shown | Structured denial state |
| `error_unavailable` | Backend or host failure | Navigation visible | Cached list if available | Retryable error state |

### Detail/editor submodes

The right pane should support three submodes:

1. `view` for read-only inspection
2. `edit` for updating an existing contact
3. `new` for creating a contact

The transition between `view` and `edit` should be in-place so users do not lose list context.

---

## Contact Data Model

The UX should present the following conceptual fields, with source-aware behavior:

- display name
- first name
- last name
- organization
- job title
- email addresses
- phone numbers
- group memberships
- notes
- favorite flag
- avatar or initials fallback
- source label: manual, imported, or platform user
- source principal link when the contact corresponds to a registered platform user
- last synced timestamp for directory-backed fields

### Sync-aware field behavior

- directory-backed identity fields are read-only when the contact is synced from the platform directory
- user-owned overlay fields such as notes, favorites, labels, and custom grouping remain editable
- if the user edits a directory-backed contact, the UI must make it clear whether the change is a personal overlay or a source field override
- merged contacts should preserve the platform principal reference and show merge history in the detail pane

### Storage and identity rules

- every contact record is stored under the current `principal_id` namespace
- `principal_id` is never accepted from the UI as an override value
- the UI must never imply that one principal can browse another principal's address book by searching for a different identifier

---

## User Journeys

### 1. Browse contacts

User opens the Contacts app and lands on the last selected group or the `All contacts` group.

Acceptance criteria:

- the left rail shows groups immediately, even during list loading
- the center column supports type-ahead search across display name, email, organization, and notes
- selecting a row updates the right pane without a full-page navigation
- the selected contact remains stable while the user changes search terms or sort order

### 2. Create a contact

User chooses New contact from the empty state, toolbar, or keyboard shortcut.

Acceptance criteria:

- the right pane opens in create mode with a blank form
- required fields are validated before save
- invalid email or phone values are rejected with field-level feedback
- a successful create returns the user to the new contact's detail view
- the new record remains scoped to the current `principal_id`

### 3. Edit an existing contact

User opens a contact, enters edit mode, changes data, and saves.

Acceptance criteria:

- the edit form is prefilled from the selected contact
- unsaved changes are preserved until the user cancels or saves
- save updates the selected contact in place and keeps the list selection stable
- if the contact is directory-backed, the UI distinguishes synced fields from personal overlay fields

### 4. Manage groups and filters

User switches between system groups and user-defined groups.

Acceptance criteria:

- group changes update only the center list and right pane selection, not the global app state
- the `Platform users` group is always visible, even if the sync snapshot is empty
- empty groups render a useful state with create or sync guidance

### 5. Auto-bootstrap platform users

The app should surface every registered platform user as a contact for the current principal.

Acceptance criteria:

- the workspace exposes a `syncing_directory` state while the bootstrap data is being refreshed
- platform-sourced contacts appear in the `Platform users` group after a successful sync
- repeated syncs are idempotent and preserve user overlays
- if platform user enumeration is unavailable, the UI shows a structured empty or sync-required state instead of pretending the contact list is complete

---

## Agent Journeys

The app must support agent use with the same logical contract as the human UI.

### Agent browse and mutate

An authorized agent should be able to list, create, update, and delete contacts for the owning human principal once platform parity for app actions is available.

Acceptance criteria:

- agent calls use the same action names and validation rules as human calls
- the same principal-scoped data appears for the agent and the owning human
- audit-friendly actor metadata is preserved through the host and does not leak into the UI as editable state
- missing agent parity returns a structured blocked state in product planning, not a silent fallback to human-only assumptions

### Agent sync operation

If the platform exposes a directory refresh action or equivalent parity endpoint, the agent should be able to trigger contact bootstrap refresh on behalf of the principal.

Acceptance criteria:

- refresh is explicit and explainable in the UI copy
- the action is idempotent
- partial failures do not corrupt existing contacts

---

## Permissions and Validation Rules

The UX must enforce the following visible and backend-aligned rules:

- the current caller may only act on contacts in their own principal namespace
- missing or insufficient scope produces a structured denial state
- create and update actions reject malformed contact data before persistence
- delete requires explicit confirmation and must not be a silent destructive action
- synced platform-user contacts may not have their source identity fields changed without a deliberate merge or override flow

Negative-path requirements:

- invalid email addresses are rejected with field-specific copy
- empty display names are rejected for manual contacts
- cross-principal requests return a permission-denied state, not an empty list that looks successful
- attempting to edit read-only synced fields shows why the field is locked
- if directory sync fails, the last known local data remains visible and editable where appropriate

---

## Accessibility Notes

- each pane must be announced as a landmark region or labeled group so screen reader users can orient quickly
- the search box must receive a visible focus state and a clear accessible label
- row selection must be keyboard accessible with arrow keys and Enter
- the detail pane must preserve focus after save or cancel so keyboard users do not lose context
- form errors should be associated with the field that failed validation and summarized at the top of the editor
- color alone must not indicate favorite, synced, or selected state
- reduced-motion users should not be forced through decorative transitions between list and detail states

---

## Mobile Behavior

The app should collapse gracefully on narrow screens.

Required behavior:

- the left rail becomes a drawer or collapsible filter panel
- the center list becomes the primary screen surface
- the right detail/editor opens as a full-screen sheet or stacked route
- search remains immediately accessible at the top of the list view
- save and cancel actions remain pinned within thumb reach in edit mode

Mobile acceptance criteria:

- a contact can be created, edited, and saved without horizontal scrolling
- the selected contact is easy to return to after closing the editor
- group navigation remains available without obscuring the list indefinitely

---

## Suggested Manifest and Action Surface

The UX spec assumes a manifest-declared action surface similar to the following logical grouping:

| Action | Purpose | Suggested scope |
|---|---|---|
| `contacts.list` | Fetch groups and contacts for the current principal | `contacts:read` |
| `contacts.create` | Create a manual contact | `contacts:write` |
| `contacts.update` | Update a contact or personal overlay fields | `contacts:write` |
| `contacts.delete` | Delete or archive a contact | `contacts:write` |
| `contacts.sync-directory` | Refresh platform-user contacts for the current principal | `contacts:write` |

The app UI should not depend on any action that is not declared in the manifest, and it must not assume any action can reach outside the principal-scoped namespace.

---

## Testing and Acceptance Criteria

This UX spec is only acceptable if the app's tests cover both happy paths and failure paths.

Required coverage:

- desktop workspace renders the three-pane layout with stable group, list, and detail regions
- search narrows the center list without collapsing the right pane unexpectedly
- create and edit flows validate required fields and invalid emails
- delete requires confirmation and removes the contact from the list after success
- synced platform-user contacts load into the `Platform users` group after bootstrap
- empty and sync-required states are rendered explicitly when the directory cannot be enumerated
- agent parity cases either pass through the same action contract or fail with a documented platform blocker until the parity endpoint exists
- cross-principal and missing-scope paths are tested as explicit negative cases

---

## Assumptions, Out-of-Scope, and Escalations

### Assumptions

- schema-driven UI is the only render path
- contact data is isolated by `principal_id`
- the Contacts app uses the host-managed effect model and does not call external services directly
- UI state is driven by app data and manifest-declared actions, not by injected scripts

### Out of scope

- external address book import from Gmail, Outlook, or other third-party providers
- social-network graph features
- real-time presence unless the platform later provides it as a safe native field
- raw HTML or custom client-side scripting
- cross-principal contact sharing

### Dependency and escalation notes

1. **Agent parity for app actions is currently a platform gap.**
   The current `/api/v1` app action surface is human-only. This spec therefore treats agent invocation of Contacts actions as a required follow-up platform update, not as an app-only implementation detail.

2. **Automatic platform-user bootstrap is currently blocked by missing enumeration support.**
   The app cannot enumerate all registered platform users from inside the current app-effect model. The UX therefore needs an explicit sync state and a platform escalation note requesting a caller-scoped directory enumeration capability or an equivalent host-side bootstrap feed.

3. **Mitigation until platform work lands.**
   Show a clear `sync required` or `directory unavailable` state, preserve manual contacts, and never present an incomplete directory as if it were complete.

---

## Delivery Checklist

- [ ] Contacts workspace is defined as a schema-driven three-pane layout
- [ ] List, detail, edit, and new-contact states are explicit
- [ ] Principal-scoped permissions and negative paths are documented
- [ ] Accessibility and mobile behavior are defined
- [ ] Agent parity gap is called out explicitly
- [ ] Directory bootstrap gap is called out explicitly
- [ ] The spec is linked from the app-developer guide
