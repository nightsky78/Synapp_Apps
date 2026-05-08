# Mail Client Platform Integration Investigation

Date: 2026-05-08  
Issue: Mail Client main view renders as a settings form instead of a real inbox UI

## Summary

Root cause is mixed, with an immediate app-side defect.

The Mail Client package publishes `ui_schemas.main` as a JSON Schema object containing configuration fields (`mailbox_id`, `search_query`, `thread_view`, `focused_inbox`, `preview_pane`, `emails_per_page`). The platform contract says `ui_schemas.main` is the canonical primary app view and wins over legacy `ui.entrypoint` metadata. The host frontend therefore receives `schema_type: "json_schema"` and intentionally renders `SchemaFormRenderer`, which is a configuration form that saves user app settings.

The host also has a platform gap for a rich native mailbox layout: its `page_layout_schema` branch currently renders generic component preview cards, not real `MessageList`, `MessageThread`, `MailComposer`, `SearchBar`, or `Toolbar` primitives. Therefore a best-in-class mailbox UI cannot be delivered by page-layout metadata alone until the host renderer is extended. The fastest app-owned fix is to make the Mail Client route use the packaged React UI entrypoint/workspace and remove/deprecate `ui_schemas.main` as a form-rendered main contract.

## Reproduction Evidence

### Live Endpoint Attempt

Command attempted against the running local Transformer:

```bash
curl http://127.0.0.1:8080/api/v1/apps/mail-client/ui-schema/main
```

Observed response without a bearer token:

```json
{
  "app_id": "mail-client",
  "view": "main",
  "status": "permission_denied",
  "schema_type": "none",
  "source": null,
  "json_schema": null,
  "page_layout_schema": null,
  "components": null,
  "error": {
    "code": "missing_bearer_token",
    "message": "App UI schema is not available for this principal."
  }
}
```

The shell did not contain a valid Synapp bearer token, so a credentialed API payload could not be captured directly. The backend resolver and installed catalog manifest determine the credentialed result unambiguously for an installed/enabled app.

### Manifest Payload Published By Mail Client

Current app manifest and catalog entry publish:

```json
{
  "ui_schemas": {
    "main": {
      "type": "object",
      "title": "Mail Main View",
      "description": "Schema-driven configuration for the host-rendered Mail main page.",
      "properties": {
        "mailbox_id": { "type": "string", "title": "Mailbox", "default": "inbox" },
        "search_query": { "type": "string", "title": "Search", "default": "" },
        "thread_view": { "type": "boolean", "title": "Group by Conversation", "default": true },
        "focused_inbox": { "type": "boolean", "title": "Focused Inbox", "default": true },
        "preview_pane": { "type": "string", "title": "Preview Pane", "enum": ["right", "bottom", "off"] },
        "emails_per_page": { "type": "integer", "title": "Emails Per Page", "enum": [25, 50, 100] }
      }
    }
  }
}
```

For a valid installed/enabled user token, the backend resolver will return approximately:

```json
{
  "app_id": "mail-client",
  "view": "main",
  "status": "available",
  "schema_type": "json_schema",
  "source": "ui_schemas.main",
  "json_schema": { "type": "object", "title": "Mail Main View", "properties": {} },
  "page_layout_schema": null,
  "components": null,
  "error": null
}
```

The visible UI evidence matches this path: the page shows generated form fields and a `Save settings` button instead of mailbox navigation, message list, reading pane, and compose.

### Enabled App Metadata

The local unauthenticated enabled-apps probe returned `401 Missing Bearer token`:

```bash
curl http://127.0.0.1:8080/api/v1/apps/enabled
```

Catalog metadata confirms the app identity and installed package metadata currently available to the lifecycle system:

- `app_id`: `mail-client`
- `version`: `2.1.0`
- `category`: `productivity`
- `verification_status`: `metadata_verified`
- capabilities: `mail:read`, `mail:draft`, `mail:send`, `mail:organize`
- manifest source commit: `420ad2b67f35e3b026c2e46a46d0c9bb1d165570`
- package URL: `mail-client-2.1.0.tar.gz`

## Host Contract Findings

Backend contract:

- `GET /api/v1/apps/{app_id}/ui-schema/{view}` requires bearer auth.
- `ui_schemas.{view}` is the canonical source.
- For `view=main`, legacy `ui.entrypoint` and `ui.contributions` are considered only when `ui_schemas.main` is absent.
- Response `schema_type` is `json_schema`, `page_layout_schema`, or `none`.
- `ui_schemas.main` is intentionally rendered before any legacy entrypoint.

Frontend route behavior:

- `AppPage.tsx` calls `getAppUiSchema(appId, "main")`.
- If `status=available` and `schema_type=json_schema`, it renders `SchemaFormRenderer`.
- If `status=available` and `schema_type=page_layout_schema`, it renders `GenericPageLayoutRenderer`.
- Recovery states are shown only for absent/invalid/unsupported/conflicting/disabled/permission failures.

Schema renderer capability:

- `SchemaFormRenderer.tsx` renders labels and fields from `schema.properties`.
- Supported field behavior is limited to:
  - `boolean` -> checkbox with `Enabled` text.
  - `integer` / `number` -> numeric input.
  - `array` of strings -> textarea.
  - all other fields -> text input.
- Form submission calls `putUserAppSettings(appId, payload)` and shows `App settings saved`.
- It has no mailbox UI primitives, no message list, no folder tree, no reading pane, no compose surface, no action toolbar, no IMAP/SMTP account wizard, and no data binding to mail actions.

Page-layout renderer capability:

- The host recognizes `page_layout_schema` metadata.
- Current frontend implementation only displays generic component preview cards with component type names.
- It does not implement the developer-guide primitives such as `MessageList`, `MessageThread`, `MailComposer`, `SearchBar`, `AttachmentList`, or `Toolbar`.

## Developer Contract Comparison

The platform developer guide states that the primary app view contract is `ui_schemas.main`, and that if both canonical and legacy declarations exist, `ui_schemas.main` wins. It also documents a future/intended schema-driven page layout contract with supported mail components.

The lifecycle registry confirms the same source-of-truth order:

1. `ui_schemas.main`
2. safe `ui.entrypoint` plus `ui.contributions`
3. structured failure state

The Transformer service reference confirms the API response shape and resolution behavior. This means the host is currently behaving according to its documented contract when it renders Mail Client as a form.

## Responsibility Split

Decision: mixed, with app team owning the immediate breakage and platform team owning rich native page-layout support.

App-side defect:

- Mail Client ships a settings-style JSON Schema as the canonical main app view.
- The manifest description still says host user settings own account credentials through a `settings_panel` schema.
- Per-user account setup is exposed through admin/settings-style metadata instead of an in-app/user-scoped account setup flow.
- The current `ui_schemas.main` prevents legacy `ui.entrypoint` resolution from ever being used.

Platform gap:

- Host can resolve `page_layout_schema`, but the frontend page-layout renderer is a generic preview, not a functional schema-driven app page renderer.
- Host app action endpoint currently returns `501 NOT_IMPLEMENTED` for `POST /api/v1/apps/{app_id}/actions/{action_name}`, so a fully host-rendered native mailbox page cannot execute mail workflows through the generic action bridge yet.

## Fix Options

### Option A: App Package Change Only

Owner: App team  
Effort: medium  
Recommendation: fastest closure for this visible bug.

Implementation:

- Remove or deprecate `ui_schemas.main` from `apps/first-party/mail-client/synapp.app.json` and catalog manifest so the bad JSON Schema no longer wins.
- Make the packaged React UI the actual main mail workspace and route it to inbox-first state.
- Keep `ui.entrypoint: "ui/dist/index.html"` and navigation target `app://mail-client/inbox`.
- Move account configuration into the React app under user-scoped routes such as `/settings/accounts`, `/settings/identities`, and `/settings/preferences`.
- Replace vague `general` settings with explicit reading, compose, notification, and sync preference groups.
- Add or update Wasm/plugin exports for account setup planning, connection test planning, preferences, identities, signatures, and legacy migration.

Impacted app files:

- `apps/first-party/mail-client/synapp.app.json`
- `apps/first-party/mail-client/plugin.json`
- `apps/first-party/mail-client/src/lib.rs`
- `apps/first-party/mail-client/ui/src/App.jsx`
- `apps/first-party/mail-client/ui/src/bridge.js`
- `apps/first-party/mail-client/ui/src/context/AppContext.jsx`
- `apps/first-party/mail-client/ui/src/components/*`
- `apps/first-party/mail-client/ui/src/styles/global.css`
- `catalog/apps/mail-client.v1.json`
- `catalog/index.v1.json` if package/hash/version metadata changes
- `docs/mail-client/*`

Risk:

- If the current host strictly requires `ui_schemas.main`, removing it may produce `absent` unless legacy `ui.entrypoint` is supported for raw packaged UI. Current documented host policy rejects raw DOM/JS injection for native rendering, so this option depends on how Synapp loads packaged `ui/dist/index.html` outside the schema endpoint.

### Option B: Host Renderer Change Only

Owner: Platform team  
Effort: high  
Recommendation: insufficient alone because the app still publishes the wrong main schema.

Implementation:

- Extend `AppPage.tsx` and the app platform renderer to load safe app-owned UI entrypoints or implement real schema primitives.
- Implement `MessageList`, `MessageThread`, `MailComposer`, `SearchBar`, `Toolbar`, `AttachmentList`, and settings primitives for `page_layout_schema`.
- Wire generic action dispatch so host-rendered components can call Wasm app actions.
- Add user-scoped app account/preference storage endpoints distinct from admin settings.

Risk:

- Existing Mail Client would still choose `ui_schemas.main` and render as form until the app manifest changes.

### Option C: Coordinated App And Host Contract Update

Owner: Both app and platform teams  
Effort: high  
Recommendation: best long-term fix.

Implementation:

- App team changes Mail Client to publish either a real page-layout contract or a sanctioned packaged React entrypoint contract for the main route.
- Platform team implements the chosen primary view contract:
  - If page layout: implement real mail primitives and action dispatch.
  - If packaged React entrypoint: define safe loading, routing, data bridge, CSP, sandboxing, and permission model.
- Both teams split user-scoped account setup from admin policy settings.
- Both teams add compatibility migration for old `ui_schemas.main` and old `settings.general` data.

Risk:

- Requires versioned contract migration. Existing apps using JSON Schema as a configuration form should continue working, while Mail Client should opt into the richer page/app workspace contract.

## Recommended Path

Use Option C as the product direction, with an app-team first implementation slice:

1. App team removes the settings-style `ui_schemas.main` from Mail Client and replaces the main route with a real inbox-first React workspace in the package.
2. App team moves account setup and preferences into user-scoped in-app routes and adds migration tooling for old settings data.
3. Platform team either sanctions packaged app entrypoint loading for `/apps/{app_id}` or implements real page-layout primitives plus action dispatch.
4. Platform team keeps admin app settings limited to policy/capability configuration, not user credentials.

## Implementation Tickets

### Ticket 1: Mail Client Main View Contract

Owner: App team

- Remove/deprecate `ui_schemas.main` settings form from Mail Client manifest.
- Publish a main workspace contract that cannot be interpreted as app settings.
- Update catalog package metadata and hashes.
- Add a regression test that `mail-client` main view no longer exposes `mailbox_id`, `thread_view`, or `emails_per_page` as saveable settings fields.

### Ticket 2: User-Scoped Account Setup

Owner: App team

- Build account setup UI for IMAP/SMTP and OAuth placeholders.
- Add Wasm/plugin exports for validation, connection test planning, completion, identities, signatures, preferences, and migration.
- Remove per-user accounts from admin settings metadata.
- Rename `General Settings` into explicit Reading, Compose, Notifications, and Sync preference sections.

### Ticket 3: Host Primary App Renderer Contract

Owner: Platform team

- Decide whether primary app pages are packaged React entrypoints, native page layouts, or both.
- If native page layouts are supported, implement real mail primitives rather than preview cards.
- If packaged UI entrypoints are supported, implement safe loading and app bridge semantics.
- Add tests for precedence rules and render behavior.

### Ticket 4: App Action Bridge

Owner: Platform team

- Replace the current `501 NOT_IMPLEMENTED` app action stub with the Wasm runtime bridge.
- Ensure host-rendered or packaged UI can invoke app actions with user context and permission enforcement.
- Add audit metadata for user and agent mail actions.

### Ticket 5: Legacy Migration

Owner: Both

- Inspect old `ui_schemas.main` and settings values.
- Migrate reading/compose/notification/sync preferences into user-scoped settings.
- Import account metadata only; require reconnect/retest for secrets.
- Never migrate credentials into admin policy.

## Acceptance Criteria For Closure

- Opening Mail Client as an installed/enabled user shows an inbox workspace with folder/account navigation, message list, reading pane or empty account setup, search, refresh, and compose.
- The main route no longer renders `SchemaFormRenderer` for Mail Client.
- Saving preferences changes visible mail behavior; it does not merely save inert settings.
- Per-user accounts are configured in user-scoped app/account settings, not admin app settings.
- Admin settings expose only organization policy/capability flags.
- Manual IMAP/SMTP account setup includes incoming/outgoing settings, secure credential references, connection test planning, receive-only handling, and default sender behavior.
- The UI schema endpoint for `main` returns either a non-form primary app contract or no longer blocks the packaged mail workspace.
- Agent/Wasm tools and human UI cover the same mail operations.

## Regression Tests

Unit tests:

- App manifest/catalog test: `mail-client` must not publish a settings-style `ui_schemas.main` JSON Schema with fields like `mailbox_id`, `thread_view`, or `emails_per_page`.
- AppPage test: when Mail Client uses the approved workspace contract, the page does not render `SchemaFormRenderer` and does render the mailbox workspace or supported page-layout renderer.
- SchemaFormRenderer test: remains valid for settings panels but is not used for Mail Client main route.
- Transformer resolver test: precedence behavior is explicit for canonical schema, page-layout schema, and legacy entrypoint metadata.
- Account setup schema tests: raw secrets are not present; only `secret_input_ref`, `secret_ref`, `oauth_connection_ref`, or `authorization_ref` appear.

E2E tests:

- Installed/enabled Mail Client opens to inbox-first workspace for a user with an account.
- No-account user lands in user-scoped Add Account flow.
- Admin app settings page does not show per-user mail accounts or `General Settings`.
- Manual IMAP/SMTP setup validates required fields and displays independent incoming/outgoing test results.
- Main route API response and UI route rendering agree on the selected app contract.