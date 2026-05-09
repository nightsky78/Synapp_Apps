# Calendar Management Architecture

Stage: 2 - Architect
App: `apps/first-party/calendar-management`
Source UX contract: `docs/calendar-management/ux-spec.md`
Date: 2026-05-08

## 1. Purpose, Boundaries, And Assumptions

Calendar Management is a first-party Synapp calendar workspace inspired by Outlook Calendar. The default route opens a real calendar work surface for day, work-week, week, month, agenda, scheduling assistant, invitations, calendars, sharing, delegates, subscriptions, and settings. Human users and AI agents use the same capability surface: humans through the React/Web UI, agents through Wasm exports and `plugin.json` tools routed by the Synapp Broker.

The Wasm module is a pure planner and validator. It validates calendar drafts, normalizes times and recurrence, computes permission-limited projections, scores suggested slots from supplied free/busy data, compares revisions, and returns host-effect plans. It never persists events directly, sends invitations, performs free/busy lookup, contacts providers, creates conference links, schedules reminders, syncs Internet calendars, opens network sockets, reads databases, or contains AI logic.

Core assumptions:

- Runtime target is `wasm32-wasip1`.
- Wasm ABI is UTF-8 JSON input and null-terminated JSON output released through `free_string`.
- Host injects `context` with user identity, permissions, available effects, request idempotency seed, optional agent identity, policy snapshots, and audit correlation metadata.
- Host owns calendar persistence, contacts, invite delivery, RSVP transport, free/busy, rooms/resources, conference links, reminders, sharing/delegate provisioning, Internet Calendar subscription sync, offline cache, and audit.
- The UI must expose every operation represented by the Wasm/plugin surface.
- Calendar event content is permission-limited. Private or limited-detail records are projected before display or tool output.

## 2. UI And Manifest Architecture

`synapp.app.json` must register a real UI entrypoint and calendar-first route during implementation:

- `ui.entrypoint`: `ui/dist/index.html`.
- Navigation target: `app://calendar-management/calendar`.
- Deep links required by UX: `/calendar`, `/calendar/day/:date`, `/calendar/work-week/:date`, `/calendar/week/:date`, `/calendar/month/:date`, `/calendar/agenda`, `/calendar/event/new`, `/calendar/event/:eventId`, `/calendar/event/:eventId/occurrence/:occurrenceId`, `/calendar/scheduling-assistant`, `/calendar/invitations`, `/calendar/calendars`, `/calendar/settings`.
- The main route must not be a host-rendered schema form. Any settings schema must be secondary to the app shell and must not contain event data or provider secrets.

The UI owns interaction state: selected view, selected calendars, overlay versus side-by-side layout, unsaved drafts, optimistic drag/resize feedback, field focus, confirmation dialogs, and recovery choices. The Wasm module owns deterministic validation and planning only.

## 3. Runtime Components

| Component | Responsibility |
| --- | --- |
| React/Web UI | Calendar workspace, event editor, scheduling assistant, invitations, calendars/sharing/settings surfaces, optimistic states, recovery flows, accessibility, and all human workflows. |
| Wasm planner | Pure validation, normalization, recurrence preview, slot scoring, permission checks, private projections, host-effect planning, idempotency key generation, and structured errors. |
| Synapp Host Broker | Tool routing, permission injection, user/agent identity, host effect execution, audit correlation, policy enforcement, and AI routing outside the app. |
| Calendar host services | Event/calendar store, invite delivery, RSVP transport, free/busy, contacts, rooms/resources, conference links, reminders, subscriptions, offline cache, and audit records. |
| Admin/policy service | Organization policy for sharing, delegates, no-end recurrence, external attendees, room providers, online meetings, subscription URLs, retention, and agent controls. |

The export ABI is:

```rust
#[no_mangle]
pub extern "C" fn export_name(input_ptr: *const u8, input_len: usize) -> *mut u8
```

Every request is a JSON object containing `context`. Every successful response has `ok.operation`, `ok.status: "accepted"`, `ok.data`, `ok.host_effects`, and optional `ok.warnings`. Every failure has `err.code`, `err.message`, and optional `err.details`.

## 4. Permissions And Capabilities

Recommended app capabilities:

| Capability | Risk | Gates |
| --- | --- | --- |
| `calendar:read` | high | List calendars, read event ranges, get event detail, search, invitations, categories, reminders, offline state, activity projections. |
| `calendar:write` | high | Create/update/delete appointments, move/resize/copy events, categories, calendars, settings, reminders. |
| `calendar:invite` | critical | Create meetings with invitations, send updates, cancel meetings, add/remove attendees, online meeting creation requests. |
| `calendar:respond` | high | Accept/tentative/decline invitations and propose new time. |
| `calendar:share` | high | Grant/revoke sharing, accept shared calendars, manage subscriptions where user-owned. |
| `calendar:delegate` | critical | Mutate another owner's calendar on their behalf, send delegated updates, inspect delegate state. |
| `calendar:settings` | medium | User preferences, work hours, default calendar, default reminders, categories, display state. |
| `calendar:audit` | high | Read host-supplied activity/audit projections and agent attribution links. |

The broker may map these to shorter internal enum values in `context.permission`, but `plugin.json` must expose the semantic capability strings above. Mutating exports must reject requests when permission, calendar role, host effect, or policy is missing.

## 5. Host Effects

Required host effects for the app family:

- `CalendarStoreRead`: read calendar summaries, event ranges, invitations, categories, preferences, revisions, and sanitized snapshots.
- `CalendarStoreWrite`: create/update events, calendars, categories, preferences, response projections, display state, and queued/draft mutations.
- `CalendarStoreDelete`: delete events, occurrences, series, calendars, categories, drafts, and obsolete local state.
- `CalendarInviteSend`: send meeting invitations, updates, cancellations, and changed-attendee notifications.
- `CalendarInviteRespond`: accept, tentative, decline, and propose new time through host calendar/mail services.
- `FreeBusyLookup`: read permission-limited attendee and organizer availability.
- `RoomResourceLookup`: search rooms/resources and read availability/capabilities.
- `ContactLookup`: resolve attendees, owners, delegates, and share recipients.
- `ConferenceLinkCreate`: create or update host-owned online meeting links.
- `ReminderSchedule`: schedule, update, cancel, snooze, or dismiss reminder intents.
- `CalendarShareManage`: grant, revoke, accept, and list sharing/delegate access.
- `SubscriptionSync`: validate and subscribe to Internet Calendar feeds through host services.
- `OfflineCacheRead`: read cached state, last sync time, and mutation policy.
- `AuditRead`: read sanitized activity projections and agent attribution.
- `AuditWrite`: record sensitive mutation plans when host policy requires explicit audit events.
- `NotifyUser`: host notifications for send/update/cancel/reminder/recovery outcomes.

No host effect payload may contain provider credentials, OAuth tokens, raw network endpoints with secrets, or audit internals. Invite delivery, RSVP, conference links, reminders, subscriptions, and sync are never executed inside Wasm.

## 6. Data Ownership Model

### Calendar

`CalendarSummary` is host-persisted and permission-limited:

- `calendar_id`, `owner_id`, `display_name`, optional `description`.
- `kind`: `primary`, `secondary`, `shared`, `resource`, `subscription`, `birthday`, `holiday`, or `host_defined`.
- `role`: `owner`, `editor`, `delegate`, `viewer`, `limited_viewer`, or `none`.
- `color`, `visible`, `display_order`, `overlay_group`, `default_reminders`, `time_zone`.
- `capabilities`: create/update/delete/share/delegate/copy/move/subscription/writeback flags.
- `sync_state`, `last_sync_at`, `revision`.

### Event And Meeting

`CalendarEvent` represents appointments, meetings, recurring series masters, and occurrences:

- Identity: `event_id`, `calendar_id`, optional `series_id`, optional `occurrence_id`, `ical_uid`, `revision`, `etag`.
- Classification: `kind` (`appointment`, `meeting`, `out_of_office`, `reminder_only`, `subscription_event`), `status` (`confirmed`, `tentative`, `cancelled`, `draft`), `meeting_status`.
- Time: `start`, `end`, `all_day`, `time_zone`, optional `original_time_zone`, `floating_time` false by default.
- Content: `title`, optional `location`, `body_text`, `body_html`, `categories`, `privacy`, `availability`, `importance`.
- Meeting data: organizer, attendees, response tracking, online meeting request/result, room/resource refs.
- Recurrence: optional `recurrence`, optional `recurrence_exception`.
- Reminders: `reminders[]` as host reminder intents.
- Metadata: `created_at`, `updated_at`, optional `created_by_agent_id`, `last_modified_by_agent_id`, `delegate_context`, `notification_state`.

### Private And Limited-Detail Projection

Before event details reach UI or tools, Wasm validates the projection level supplied by host or computes a conservative projection from role/privacy:

| Projection | Fields Allowed |
| --- | --- |
| `full` | All permitted event fields except host secrets and audit internals. |
| `limited` | Time range, calendar id, busy/free status, privacy marker, recurrence marker, and generic label such as `Busy`; no notes, attendees, location, categories, or organizer details unless host policy includes them. |
| `availability_only` | Time range and availability status only. |
| `none` | Event existence is hidden; only aggregate availability may be returned if free/busy policy allows it. |

Private events default to `limited` for viewers and delegates unless host explicitly grants full private access. Permission-denied errors must not include private event titles, locations, attendees, or notes.

## 7. Recurrence Representation And Constraints

The app uses a provider-safe recurrence model that can be translated to common calendar providers and iCalendar RRULEs without provider-specific APIs in Wasm.

`RecurrenceRule` fields:

- `frequency`: `daily`, `weekly`, `monthly`, `yearly`.
- `interval`: integer 1 to 99.
- `days_of_week`: optional array of `MO`, `TU`, `WE`, `TH`, `FR`, `SA`, `SU`.
- `day_of_month`: optional integer 1 to 31.
- `week_of_month`: optional `first`, `second`, `third`, `fourth`, `last`.
- `month_of_year`: optional integer 1 to 12.
- `end`: `{ "mode": "never|after_count|on_date", "count"?, "until"? }`.
- `time_zone`: IANA time zone id required for timed recurring events.
- `exceptions`: occurrence ids or original start timestamps with detached/deleted metadata.

Provider-safe constraints:

- No second/minute/hourly recurrence in MVP.
- No multiple RRULEs per event in MVP.
- No arbitrary `BYSETPOS` except `week_of_month` values listed above.
- No no-end recurrence unless host policy explicitly allows it.
- Count must be 1 to 999.
- Preview expansion is capped at 50 returned occurrences and 5 years of search horizon.
- Month-end rules must declare host behavior: `skip_missing_day` or `last_day_of_month`.
- Daylight-saving previews use the event time zone and return warnings when local attendee times shift.
- `this_and_following` edits are allowed only when `context.host_capabilities.recurrence_split` or the supplied calendar capability allows series splitting.

## 8. Host-Effect Plan And Idempotency Strategy

Every mutating export returns host-effect plans instead of performing side effects. Each `HostEffect` includes:

- `effect`: host effect name.
- `intent`: concise human-readable purpose.
- `payload`: sanitized structured data.
- `idempotency_key`: deterministic key.
- `depends_on`: optional prior effect keys.
- `on_failure`: `abort_remaining`, `continue_with_warning`, or `requires_user_recovery`.
- `audit`: optional sanitized operation label and risk level.

Idempotency keys are derived from `context.request_id` when present, otherwise from `context.user_id`, export name, target ids, revision, and a stable hash of the normalized operation payload. Invite, cancellation, sharing, delegate, subscription, and reminder effects must use stable keys so retries do not duplicate sends or provisioning. For multi-effect mutations, persistence effects run before notification effects unless the host requires transactional execution; response data must distinguish `saved`, `sent`, `partially_sent`, `queued`, and `draft_only` states.

## 9. Exact Wasm Export Surface

All exports share the ABI and envelope described above. Input and return data types are expanded in `contracts.md`; `plugin-schema-plan.md` maps each to JSON Schema.

| Export | Purpose | Key Params | Return Data | Capability | Required Effects |
| --- | --- | --- | --- | --- | --- |
| `get_calendar_capabilities` | Read permission, policy, host-effect, and feature availability. | `context`, optional `calendar_ids` | `capabilities`, `policy`, `host_effects`, `default_view_state` | `calendar:read` | `CalendarStoreRead`, optional `OfflineCacheRead` |
| `list_calendars` | List calendars visible to user/agent. | `context`, optional `include_hidden`, `refresh` | `calendars[]`, `visibility_state`, `sync_state` | `calendar:read` | `CalendarStoreRead` |
| `save_calendar` | Create or update calendar metadata. | `context`, `calendar`, optional `revision` | `calendar`, `created`, `updated` | `calendar:write` | `CalendarStoreWrite` |
| `delete_calendar` | Delete or disable a calendar with event outcome plan. | `context`, `calendar_id`, `mode`, optional `revision`, `confirmation` | `calendar_id`, `deletion_mode`, `event_outcome` | `calendar:write` | `CalendarStoreDelete`, optional `CalendarStoreWrite` |
| `set_calendar_view_state` | Save visibility/order/overlay/view preferences. | `context`, `view_state` | `view_state` | `calendar:settings` | `CalendarStoreWrite` |
| `list_events` | Read range events with projection. | `context`, `range`, `calendar_ids`, filters, `projection` | `events[]`, `range`, `projection_applied`, `sync_state` | `calendar:read` | `CalendarStoreRead`, optional `OfflineCacheRead` |
| `get_event` | Read one event/occurrence detail. | `context`, `event_id`, optional `occurrence_id`, `projection` | `event`, `projection_applied`, `allowed_actions` | `calendar:read` | `CalendarStoreRead` |
| `search_events` | Search events with structured filters. | `context`, `query`, `filters`, `pagination` | `results[]`, `facets`, `pagination` | `calendar:read` | `CalendarStoreRead` |
| `validate_event_draft` | Validate appointment/meeting draft without mutation. | `context`, `draft`, optional `existing_event` | `valid`, `normalized_draft`, `field_errors`, `warnings`, `required_actions` | `calendar:write` | none or `CalendarStoreRead` for policy snapshot fallback |
| `create_event` | Plan appointment or meeting creation. | `context`, `draft`, `send_invites`, optional `confirmation` | `event_preview`, `persistence_status`, `notification_state` | `calendar:write`, optional `calendar:invite` | `CalendarStoreWrite`, optional `CalendarInviteSend`, `ConferenceLinkCreate`, `ReminderSchedule`, `AuditWrite` |
| `update_event` | Plan appointment/meeting update, move, resize, attendee/category changes. | `context`, `event_id`, optional `occurrence_id`, `patch`, `revision`, `notification_scope` | `event_preview`, `attendee_delta`, `notification_state` | `calendar:write`, optional `calendar:invite` | `CalendarStoreWrite`, optional `CalendarInviteSend`, `ConferenceLinkCreate`, `ReminderSchedule`, `AuditWrite` |
| `delete_event` | Plan appointment delete, meeting cancel, occurrence/series delete. | `context`, `target`, `scope`, `mode`, optional `message`, `confirmation` | `target`, `scope`, `deletion_state`, `notification_state` | `calendar:write`, optional `calendar:invite` | `CalendarStoreDelete`, optional `CalendarInviteSend`, `ReminderSchedule`, `AuditWrite` |
| `copy_event` | Copy event to another calendar. | `context`, `event_id`, `destination_calendar_id`, optional `occurrence_id`, `copy_options` | `source_event_id`, `new_event_preview` | `calendar:write` | `CalendarStoreRead`, `CalendarStoreWrite` |
| `move_event` | Move event to another calendar where permitted. | `context`, `event_id`, `destination_calendar_id`, optional `revision` | `event_preview`, `source_calendar_id`, `destination_calendar_id` | `calendar:write` | `CalendarStoreWrite`, optional `CalendarInviteSend` |
| `validate_recurrence` | Validate recurrence rule and provider-safe constraints. | `context`, `event_time`, `recurrence` | `valid`, `normalized_recurrence`, `field_errors`, `warnings` | `calendar:write` | none |
| `preview_recurrence` | Expand upcoming occurrences for UI preview. | `context`, `event_time`, `recurrence`, `limit` | `occurrences[]`, `warnings` | `calendar:read` | none |
| `plan_free_busy_lookup` | Plan host free/busy lookup. | `context`, `attendees`, `range`, `granularity_minutes` | `lookup_request`, `requires_host_execution` | `calendar:read` | `FreeBusyLookup` |
| `suggest_meeting_times` | Score supplied free/busy and room data. | `context`, `meeting_draft`, `free_busy_snapshot`, optional `rooms_snapshot` | `suggestions[]`, `conflicts[]`, `warnings` | `calendar:read` | none |
| `search_rooms` | Plan room/resource search and availability lookup. | `context`, `query`, `range`, `capacity`, `features` | `room_lookup_request`, `requires_host_execution` | `calendar:read` | `RoomResourceLookup` |
| `resolve_attendees` | Plan host contact/attendee resolution. | `context`, `attendees` | `normalized_attendees`, `unresolved[]`, `host_effects` | `calendar:read` | `ContactLookup` |
| `list_invitations` | Read pending/recent invitations. | `context`, filters, `pagination` | `invitations[]`, `pagination` | `calendar:read` | `CalendarStoreRead` |
| `get_invitation` | Read one invitation detail with conflicts. | `context`, `invitation_id` | `invitation`, `allowed_responses`, `conflicts[]` | `calendar:read` | `CalendarStoreRead`, optional `FreeBusyLookup` |
| `respond_to_invitation` | Plan RSVP or no-response state update. | `context`, `invitation_id`, `response`, optional `message` | `invitation_id`, `response_state`, `calendar_update_state` | `calendar:respond` | `CalendarInviteRespond`, `CalendarStoreWrite` |
| `propose_new_time` | Plan meeting response with proposed time. | `context`, `invitation_id`, `proposed_time`, optional `message` | `proposal_state`, `proposed_time` | `calendar:respond` | `CalendarInviteRespond`, `CalendarStoreWrite` |
| `list_categories` | Read category labels/colors. | `context`, optional `calendar_id` | `categories[]` | `calendar:read` | `CalendarStoreRead` |
| `save_category` | Create/update category. | `context`, `category`, optional `revision` | `category` | `calendar:settings` | `CalendarStoreWrite` |
| `delete_category` | Delete category and plan event unassignment behavior. | `context`, `category_id`, `replacement_category_id` | `category_id`, `event_update_plan` | `calendar:settings` | `CalendarStoreWrite`, optional `CalendarStoreDelete` |
| `bulk_update_events` | Plan confirmed bulk category/move/delete operations. | `context`, `selection`, `operation`, `confirmation` | `affected_count`, `operation_state` | `calendar:write` | `CalendarStoreWrite`, optional `CalendarStoreDelete`, `CalendarInviteSend`, `AuditWrite` |
| `get_sharing_state` | Read sharing and delegates for a calendar. | `context`, `calendar_id` | `shares[]`, `delegates[]`, `allowed_actions` | `calendar:share` | `CalendarShareManage`, `CalendarStoreRead` |
| `update_sharing` | Grant/revoke view/edit/delegate access. | `context`, `calendar_id`, `changes`, `confirmation` | `sharing_state`, `provisioning_state` | `calendar:share` | `CalendarShareManage`, `ContactLookup`, `AuditWrite` |
| `accept_shared_calendar` | Accept/provision a shared calendar invitation. | `context`, `share_invitation_id`, `display_options` | `calendar` | `calendar:share` | `CalendarShareManage`, `CalendarStoreWrite` |
| `subscribe_calendar` | Plan Internet Calendar subscription. | `context`, `subscription`, `confirmation` | `subscription_state`, `calendar_preview` | `calendar:share` | `SubscriptionSync`, `CalendarStoreWrite` |
| `get_reminder_state` | Read reminder status for event or calendar. | `context`, `event_id` or `calendar_id` | `reminders[]`, `delivery_state` | `calendar:read` | `CalendarStoreRead`, `ReminderSchedule` |
| `snooze_or_dismiss_reminder` | Plan reminder snooze/dismiss. | `context`, `reminder_id`, `action`, optional `snooze_until` | `reminder_state` | `calendar:write` | `ReminderSchedule`, `CalendarStoreWrite` |
| `get_user_preferences` | Read calendar settings. | `context`, optional `include_defaults` | `preferences`, `defaults` | `calendar:settings` | `CalendarStoreRead` |
| `save_user_preferences` | Save calendar settings. | `context`, `preferences`, optional `revision` | `preferences` | `calendar:settings` | `CalendarStoreWrite` |
| `get_offline_state` | Read host cache and offline mutation policy. | `context` | `offline_state`, `last_sync_at`, `mutation_policy` | `calendar:read` | `OfflineCacheRead` |
| `get_activity_log` | Read sanitized audit/activity projection. | `context`, filters, `pagination` | `activities[]`, `pagination` | `calendar:audit` | `AuditRead` |

## 10. Validation And Error Semantics

Common error codes:

- `InvalidInput`: malformed JSON, missing field, invalid enum, invalid range, unsupported recurrence combination, invalid date/time, invalid attendee, invalid confirmation.
- `PermissionDenied`: context permission or calendar role does not satisfy the operation.
- `EffectUnavailable`: required host effect is absent from `context.available_effects`.
- `PolicyDenied`: host/admin policy disallows no-end recurrence, external attendee, sharing, delegate, online meeting, subscription, bulk delete, or offline mutation.
- `CalendarNotFound`, `EventNotFound`, `InvitationNotFound`, `CategoryNotFound`, `ReminderNotFound`, `ShareNotFound`.
- `ProjectionDenied`: requested detail level is not allowed; response may include only allowed projection.
- `Conflict`: supplied revision/etag is stale or event changed/deleted elsewhere.
- `RecurrenceUnsupported`: operation requires provider support not present, such as this-and-following split.
- `AttendeeResolutionFailed`, `FreeBusyUnavailable`, `RoomUnavailable`, `ConferenceUnavailable`, `ReminderUnavailable`.
- `DestructiveConfirmationRequired`: broad delete/cancel/series/share/bulk operation lacks confirmation.
- `HostRejected`: host rejected an otherwise valid plan.
- `InternalError`: unexpected serialization or planner failure.

Important validation constants:

- Event title: max 512 characters; title may be empty only if host policy allows untitled events, then normalized to `Untitled event` for display.
- Notes/body text or HTML: max 512,000 bytes combined.
- Location: max 512 characters.
- Attendees: max 500 total, unless policy lowers the cap.
- Categories per event: max 25.
- Reminders per event: max 10; offset range 0 to 40,320 minutes before start.
- Search query: max 256 characters.
- Bulk mutation selection: max 500 event ids or a confirmed query scope with `estimated_count`.
- Event duration: end must be after start; all-day events use date-only inclusive start/exclusive end semantics.
- Time zones: IANA ids required for timed events; UTC timestamps may be accepted only with explicit `time_zone` for rendering.

## 11. Wasm Compatibility And Security Gates

| Gate | Status | Notes |
| --- | --- | --- |
| Full Dual-Interface design | Pass by design | UI, Wasm exports, and `plugin.json` schema plan are all required deliverables. |
| Wasm exports are planner/validator only | Pass by design | Host effects own persistence, invites, responses, free/busy, contacts, rooms, conference links, reminders, sharing, subscriptions, offline cache, and audit. |
| No AI logic in app | Pass by design | Broker owns AI routing; app contains no prompts, LLM SDKs, LangChain, or autonomous reasoning. |
| No direct network/database access | Pass by design | No Graph, Exchange, CalDAV, SMTP, TCP/UDP, DB, OAuth, or conference provider calls from Wasm. |
| `wasm32-wasip1` compatibility | Pass by design | Pure Rust with `serde`/`serde_json`-class dependencies only; no native TLS, C bindings, threads, async network runtime, filesystem persistence, or OS-specific APIs. |
| Private/limited projections | Pass by design | Projection model prevents event detail leakage through UI or tools. |
| `plugin.json` synchronization explicit | Pass by design | See `plugin-schema-plan.md`; every export has input/output schema mapping. |
