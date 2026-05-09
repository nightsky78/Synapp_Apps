# Calendar Management Contracts

Stage: 2 - Architect
App: `apps/first-party/calendar-management`
Date: 2026-05-08

## 1. ABI And Envelope Contract

All exports use the same Wasm ABI:

```rust
#[no_mangle]
pub extern "C" fn export_name(input_ptr: *const u8, input_len: usize) -> *mut u8
```

Input is a UTF-8 JSON object. Output is a null-terminated UTF-8 JSON string released by `free_string(ptr)`. Function names below are the exact Wasm export names and the exact `plugin.json` tool names.

Every input includes:

```json
{
  "context": {
    "user_id": "usr_123",
    "permission": "read|write|invite|respond|share|delegate|settings|audit|admin|none",
    "capabilities": ["calendar:read"],
    "available_effects": ["CalendarStoreRead"],
    "agent_id": "optional_agent_id",
    "request_id": "optional_idempotency_seed",
    "locale": "en-US",
    "time_zone": "Europe/Berlin"
  }
}
```

Every successful response uses:

```json
{
  "ok": {
    "operation": "export_name",
    "status": "accepted",
    "data": {},
    "host_effects": [],
    "warnings": []
  }
}
```

Every error response uses:

```json
{
  "err": {
    "code": "InvalidInput",
    "message": "Human-readable recovery guidance",
    "details": {}
  }
}
```

## 2. Shared Data Models

### HostEffect

```json
{
  "effect": "CalendarStoreWrite",
  "intent": "Create a calendar event",
  "payload": {},
  "idempotency_key": "create_event:usr_123:req_456",
  "depends_on": [],
  "on_failure": "abort_remaining",
  "audit": { "operation": "create_meeting", "risk": "critical" }
}
```

`effect` must be one of the host effects listed in `architecture.md`. `on_failure` is `abort_remaining`, `continue_with_warning`, or `requires_user_recovery`.

### CalendarContext

Fields: `user_id`, `permission`, `capabilities[]`, `available_effects[]`, optional `agent_id`, `request_id`, `locale`, `time_zone`, `policy`, `host_capabilities`, `audit_correlation_id`.

### CalendarSummary

Fields: `calendar_id`, `owner_id`, `display_name`, optional `description`, `kind`, `role`, `color`, `visible`, `display_order`, optional `overlay_group`, `default_reminders[]`, `time_zone`, `capabilities`, `sync_state`, optional `last_sync_at`, optional `revision`.

### TimeRange

Fields: `start`, `end`, `time_zone`, optional `all_day`. `start` and `end` are ISO 8601 timestamps for timed events or date strings for all-day ranges. End is exclusive.

### CalendarEvent

Fields: `event_id`, `calendar_id`, optional `series_id`, optional `occurrence_id`, optional `ical_uid`, `revision`, optional `etag`, `kind`, `status`, `title`, `start`, `end`, `all_day`, `time_zone`, optional `location`, optional `body_text`, optional `body_html`, `availability`, `privacy`, `importance`, `categories[]`, `reminders[]`, optional `recurrence`, optional `recurrence_exception`, optional `organizer`, `attendees[]`, optional `online_meeting`, optional `room`, optional `delegate_context`, optional `created_by_agent_id`, optional `last_modified_by_agent_id`, `created_at`, `updated_at`, optional `notification_state`.

### EventDraft

Same core fields as `CalendarEvent`, but `event_id`, `revision`, `etag`, `created_at`, and `updated_at` are optional. Required for save: `calendar_id`, `kind`, `start`, `end`, `time_zone`, `all_day`, `availability`, and `privacy`. Meeting drafts require at least one valid attendee before invite send.

### Attendee

Fields: `attendee_id` optional, `email` optional, `display_name` optional, `kind` (`required`, `optional`, `resource`, `room`), `response_status` (`none`, `accepted`, `tentative`, `declined`, `proposed_new_time`, `organizer`), optional `proposed_time`, optional `resolved_ref`, optional `permission_projection`.

### RecurrenceRule

Fields: `frequency`, `interval`, optional `days_of_week[]`, optional `day_of_month`, optional `week_of_month`, optional `month_of_year`, `end`, `time_zone`, optional `month_end_behavior`, optional `exceptions[]`. See `architecture.md` for provider-safe constraints.

### ReminderIntent

Fields: `reminder_id` optional, `method` (`host_notification`, `email`, `mobile_push`, `none`), `offset_minutes`, `state` (`planned`, `scheduled`, `snoozed`, `dismissed`, `failed`), optional `snooze_until`.

### Category

Fields: `category_id`, `name`, `color`, optional `description`, optional `calendar_id`, optional `revision`.

### FreeBusySnapshot

Fields: `subject_id`, `subject_kind` (`user`, `attendee`, `room`, `calendar`), `range`, `time_zone`, `blocks[]`. Each block has `start`, `end`, `availability` (`free`, `busy`, `tentative`, `out_of_office`, `working_elsewhere`, `unknown`), optional `source_projection`, optional `event_id` only when permitted.

### SuggestedSlot

Fields: `start`, `end`, `time_zone`, `score`, `required_status`, `optional_status`, optional `room_status`, `within_work_hours`, `conflicts[]`, `reasons[]`, `warnings[]`.

### SharePrincipal And Delegate

`SharePrincipal`: `principal_id`, optional `email`, optional `display_name`, `principal_kind`, `access_level` (`owner`, `delegate`, `edit`, `view`, `limited_details`, `availability_only`, `none`), `state`, optional `revision`.

`DelegateContext`: `owner_id`, `owner_display_name`, `delegate_user_id`, `access_level`, `send_on_behalf`, `audit_required`.

### UserCalendarPreferences

Fields: `default_view`, `density`, `default_calendar_id`, `time_zone`, optional `secondary_time_zone`, `work_days[]`, `work_hours`, `default_reminders[]`, `default_online_meeting`, `overlay_mode`, `category_sort`, optional `agent_activity_link`.

## 3. Common Validation And Errors

Common errors: `InvalidInput`, `PermissionDenied`, `EffectUnavailable`, `PolicyDenied`, `CalendarNotFound`, `EventNotFound`, `InvitationNotFound`, `CategoryNotFound`, `ReminderNotFound`, `ShareNotFound`, `ProjectionDenied`, `Conflict`, `RecurrenceUnsupported`, `AttendeeResolutionFailed`, `FreeBusyUnavailable`, `RoomUnavailable`, `ConferenceUnavailable`, `ReminderUnavailable`, `DestructiveConfirmationRequired`, `HostRejected`, `InternalError`.

Validation constants:

- Event title max 512 characters.
- Notes/body max 512,000 bytes combined.
- Location max 512 characters.
- Attendees max 500 total unless policy lowers it.
- Categories per event max 25.
- Reminders per event max 10; offset 0 to 40,320 minutes.
- Search query max 256 characters.
- Bulk event id selection max 500 unless query-scope confirmation is supplied.
- Recurrence preview max 50 occurrences and 5-year expansion horizon.
- Event end must be after start. All-day ranges use date-only exclusive end.
- IANA time zone id is required for timed events and recurring events.

## 4. Export Contracts

### `get_calendar_capabilities`

Purpose: Read permission, policy, host-effect, recurrence, offline, sharing, and feature availability for the app shell.

Input: `context`, optional `calendar_ids: string[]`.

Return data: `{ "capabilities": {}, "policy": {}, "host_effects": [], "default_view_state": {}, "warnings": [] }`.

Host effects: `CalendarStoreRead`, optional `OfflineCacheRead`, optional `AuditRead`.

Errors: `PermissionDenied`, `EffectUnavailable`, `InvalidInput`.

### `list_calendars`

Purpose: List visible and optionally hidden calendars, including shared/resource/subscription calendars and permissions.

Input: `context`, optional `include_hidden: boolean`, optional `refresh: boolean`, optional `snapshot: CalendarSummary[]`.

Return data: `{ "calendars": CalendarSummary[], "visibility_state": {}, "sync_state": {}, "needs_host_refresh": boolean }`.

Host effects: `CalendarStoreRead`, optional `OfflineCacheRead`.

Errors: `PermissionDenied`, `EffectUnavailable`.

### `save_calendar`

Purpose: Create or update calendar metadata such as name, color, description, default reminders, and visibility.

Input: `context`, `calendar: CalendarSummary`, optional `revision`, optional `create: boolean`.

Return data: `{ "calendar": CalendarSummary, "created": boolean, "updated": boolean }`.

Host effects: `CalendarStoreWrite`, optional `ReminderSchedule` for default reminder changes.

Errors: `InvalidInput`, `PermissionDenied`, `PolicyDenied`, `Conflict`, `EffectUnavailable`.

### `delete_calendar`

Purpose: Plan calendar deletion/disablement with clear event outcome.

Input: `context`, `calendar_id`, `mode` (`delete`, `hide`, `unsubscribe`), optional `revision`, optional `confirmation`.

Return data: `{ "calendar_id", "deletion_mode", "event_outcome", "requires_confirmation": boolean }`.

Host effects: `CalendarStoreDelete`, optional `CalendarStoreWrite`, optional `SubscriptionSync`.

Errors: `CalendarNotFound`, `PermissionDenied`, `PolicyDenied`, `DestructiveConfirmationRequired`, `Conflict`, `EffectUnavailable`.

### `set_calendar_view_state`

Purpose: Save calendar visibility, order, overlay/side-by-side mode, selected view, and selected date range.

Input: `context`, `view_state` with `selected_calendar_ids`, `hidden_calendar_ids`, `display_order`, `mode`, `view`, optional `date_anchor`.

Return data: `{ "view_state": {}, "normalized": boolean }`.

Host effects: `CalendarStoreWrite`.

Errors: `InvalidInput`, `PermissionDenied`, `EffectUnavailable`.

### `list_events`

Purpose: Read events for a visible range with private/limited projection.

Input: `context`, `range: TimeRange`, `calendar_ids: string[]`, optional `filters`, optional `projection` (`full`, `limited`, `availability_only`), optional `snapshot`, optional `refresh`.

Return data: `{ "events": CalendarEvent[], "range": TimeRange, "projection_applied": string, "sync_state": {}, "warnings": [] }`.

Host effects: `CalendarStoreRead`, optional `OfflineCacheRead`.

Errors: `InvalidInput`, `PermissionDenied`, `EffectUnavailable`, `ProjectionDenied`.

### `get_event`

Purpose: Read one event or recurring occurrence detail with allowed actions.

Input: `context`, `event_id`, optional `calendar_id`, optional `occurrence_id`, optional `projection`, optional `snapshot`, optional `refresh`.

Return data: `{ "event": CalendarEvent, "projection_applied": string, "allowed_actions": [], "warnings": [] }`.

Host effects: `CalendarStoreRead`.

Errors: `EventNotFound`, `PermissionDenied`, `ProjectionDenied`, `EffectUnavailable`.

### `search_events`

Purpose: Search events by text, attendee, organizer, location, category, calendar, date range, RSVP status, and meeting type.

Input: `context`, `query`, optional `filters`, optional `pagination`, optional `projection`, optional `snapshot`, optional `refresh`.

Return data: `{ "results": CalendarEvent[], "facets": {}, "pagination": {}, "projection_applied": string }`.

Host effects: `CalendarStoreRead`.

Errors: `InvalidInput`, `PermissionDenied`, `EffectUnavailable`.

### `validate_event_draft`

Purpose: Validate and normalize an appointment or meeting draft without mutation.

Input: `context`, `draft: EventDraft`, optional `existing_event: CalendarEvent`, optional `policy_snapshot`.

Return data: `{ "valid": boolean, "normalized_draft": EventDraft, "field_errors": [], "warnings": [], "required_actions": [] }`.

Host effects: none when policy snapshot is supplied; optional `CalendarStoreRead` otherwise.

Errors: `InvalidInput`, `PermissionDenied`, `PolicyDenied`, `RecurrenceUnsupported`, `EffectUnavailable`.

### `create_event`

Purpose: Plan appointment creation or meeting creation with optional invite, conference, and reminder effects.

Input: `context`, `draft: EventDraft`, `send_invites: boolean`, optional `confirmation`, optional `client_operation_id`.

Return data: `{ "event_preview": CalendarEvent, "persistence_status": "planned|draft_only", "notification_state": "none|planned|blocked|partial", "requires_host_execution": true }`.

Host effects: `CalendarStoreWrite`, optional `CalendarInviteSend`, `ConferenceLinkCreate`, `ReminderSchedule`, `AuditWrite`, `NotifyUser`.

Errors: `InvalidInput`, `PermissionDenied`, `PolicyDenied`, `EffectUnavailable`, `ConferenceUnavailable`, `ReminderUnavailable`.

### `update_event`

Purpose: Plan event update, meeting update, move/resize, attendee delta, recurrence scope update, or delegated mutation.

Input: `context`, `event_id`, optional `occurrence_id`, `patch`, optional `revision`, `notification_scope` (`none`, `changed_attendees`, `all_attendees`, `host_default`), `scope` (`single`, `occurrence`, `this_and_following`, `series`), optional `confirmation`.

Return data: `{ "event_preview": CalendarEvent, "attendee_delta": {}, "notification_state": {}, "conflict_state": {}, "requires_host_execution": true }`.

Host effects: `CalendarStoreWrite`, optional `CalendarInviteSend`, `ConferenceLinkCreate`, `ReminderSchedule`, `AuditWrite`, `NotifyUser`.

Errors: `EventNotFound`, `InvalidInput`, `PermissionDenied`, `PolicyDenied`, `Conflict`, `RecurrenceUnsupported`, `EffectUnavailable`.

### `delete_event`

Purpose: Plan appointment delete, meeting cancellation, occurrence deletion, this-and-following deletion, or series cancellation.

Input: `context`, `target` with `event_id` and optional `occurrence_id`, `scope` (`single`, `occurrence`, `this_and_following`, `series`), `mode` (`delete`, `cancel`), optional `message`, optional `revision`, optional `confirmation`.

Return data: `{ "target": {}, "scope", "deletion_state": {}, "notification_state": {}, "requires_host_execution": true }`.

Host effects: `CalendarStoreDelete`, optional `CalendarInviteSend`, `ReminderSchedule`, `AuditWrite`, `NotifyUser`.

Errors: `EventNotFound`, `PermissionDenied`, `PolicyDenied`, `Conflict`, `DestructiveConfirmationRequired`, `RecurrenceUnsupported`, `EffectUnavailable`.

### `copy_event`

Purpose: Copy an event or occurrence to another writable calendar.

Input: `context`, `event_id`, `destination_calendar_id`, optional `occurrence_id`, optional `copy_options` (`include_attendees`, `include_reminders`, `include_recurrence`, `as_draft`).

Return data: `{ "source_event_id", "new_event_preview": CalendarEvent, "requires_host_execution": true }`.

Host effects: `CalendarStoreRead`, `CalendarStoreWrite`.

Errors: `EventNotFound`, `CalendarNotFound`, `PermissionDenied`, `PolicyDenied`, `EffectUnavailable`.

### `move_event`

Purpose: Move an event to another calendar when both source and destination permissions allow it.

Input: `context`, `event_id`, `destination_calendar_id`, optional `occurrence_id`, optional `revision`, optional `notification_scope`.

Return data: `{ "event_preview": CalendarEvent, "source_calendar_id", "destination_calendar_id", "notification_state": {} }`.

Host effects: `CalendarStoreWrite`, optional `CalendarInviteSend`, `AuditWrite`.

Errors: `EventNotFound`, `CalendarNotFound`, `PermissionDenied`, `PolicyDenied`, `Conflict`, `EffectUnavailable`.

### `validate_recurrence`

Purpose: Validate provider-safe recurrence rules and normalize unsupported combinations before save.

Input: `context`, `event_time: TimeRange`, `recurrence: RecurrenceRule`, optional `policy_snapshot`.

Return data: `{ "valid": boolean, "normalized_recurrence": RecurrenceRule, "field_errors": [], "warnings": [] }`.

Host effects: none.

Errors: `InvalidInput`, `PolicyDenied`, `RecurrenceUnsupported`.

### `preview_recurrence`

Purpose: Return upcoming occurrence previews for create/edit UX.

Input: `context`, `event_time: TimeRange`, `recurrence: RecurrenceRule`, optional `limit` default 10 max 50.

Return data: `{ "occurrences": [], "truncated": boolean, "warnings": [] }`.

Host effects: none.

Errors: `InvalidInput`, `RecurrenceUnsupported`.

### `plan_free_busy_lookup`

Purpose: Validate and plan a host-owned free/busy lookup.

Input: `context`, `attendees: Attendee[]`, `range: TimeRange`, optional `granularity_minutes` default 30.

Return data: `{ "lookup_request": {}, "requires_host_execution": true }`.

Host effects: `FreeBusyLookup`.

Errors: `InvalidInput`, `PermissionDenied`, `EffectUnavailable`, `PolicyDenied`.

### `suggest_meeting_times`

Purpose: Score supplied free/busy and room/resource snapshots; no host I/O.

Input: `context`, `meeting_draft: EventDraft`, `free_busy_snapshot: FreeBusySnapshot[]`, optional `rooms_snapshot`, optional `preferences`.

Return data: `{ "suggestions": SuggestedSlot[], "conflicts": [], "warnings": [] }`.

Host effects: none.

Errors: `InvalidInput`, `FreeBusyUnavailable`.

### `search_rooms`

Purpose: Plan host-owned room/resource lookup and optional availability check.

Input: `context`, optional `query`, `range: TimeRange`, optional `capacity`, optional `features: string[]`, optional `location_hint`.

Return data: `{ "room_lookup_request": {}, "requires_host_execution": true }`.

Host effects: `RoomResourceLookup`.

Errors: `InvalidInput`, `PermissionDenied`, `EffectUnavailable`, `PolicyDenied`.

### `resolve_attendees`

Purpose: Normalize attendee entries and plan contact lookup for unresolved people/resources.

Input: `context`, `attendees: Attendee[]`, optional `allow_manual_email: boolean`.

Return data: `{ "normalized_attendees": Attendee[], "unresolved": [], "requires_host_execution": boolean }`.

Host effects: optional `ContactLookup`.

Errors: `InvalidInput`, `EffectUnavailable`, `AttendeeResolutionFailed`.

### `list_invitations`

Purpose: Read pending and recent meeting invitations.

Input: `context`, optional `filters`, optional `pagination`, optional `snapshot`, optional `refresh`.

Return data: `{ "invitations": [], "pagination": {}, "sync_state": {} }`.

Host effects: `CalendarStoreRead`.

Errors: `PermissionDenied`, `EffectUnavailable`.

### `get_invitation`

Purpose: Read one invitation detail, allowed responses, and optional conflict summary.

Input: `context`, `invitation_id`, optional `include_conflicts: boolean`.

Return data: `{ "invitation": {}, "allowed_responses": [], "conflicts": [], "warnings": [] }`.

Host effects: `CalendarStoreRead`, optional `FreeBusyLookup`.

Errors: `InvitationNotFound`, `PermissionDenied`, `EffectUnavailable`.

### `respond_to_invitation`

Purpose: Plan accept, tentative, decline, or no-response update.

Input: `context`, `invitation_id`, `response` (`accept`, `tentative`, `decline`), optional `scope` (`occurrence`, `series`), optional `message`, optional `send_response: boolean`.

Return data: `{ "invitation_id", "response_state": {}, "calendar_update_state": {}, "requires_host_execution": true }`.

Host effects: `CalendarInviteRespond`, `CalendarStoreWrite`, optional `NotifyUser`.

Errors: `InvitationNotFound`, `PermissionDenied`, `PolicyDenied`, `EffectUnavailable`, `HostRejected`.

### `propose_new_time`

Purpose: Plan a meeting response with proposed time.

Input: `context`, `invitation_id`, `proposed_time: TimeRange`, optional `scope`, optional `message`.

Return data: `{ "proposal_state": {}, "proposed_time": TimeRange, "requires_host_execution": true }`.

Host effects: `CalendarInviteRespond`, `CalendarStoreWrite`.

Errors: `InvitationNotFound`, `PermissionDenied`, `PolicyDenied`, `EffectUnavailable`, `InvalidInput`.

### `list_categories`

Purpose: Read user/calendar categories.

Input: `context`, optional `calendar_id`, optional `snapshot`, optional `refresh`.

Return data: `{ "categories": Category[] }`.

Host effects: `CalendarStoreRead`.

Errors: `PermissionDenied`, `EffectUnavailable`.

### `save_category`

Purpose: Create or update a category label/color.

Input: `context`, `category: Category`, optional `revision`.

Return data: `{ "category": Category, "created": boolean, "updated": boolean }`.

Host effects: `CalendarStoreWrite`.

Errors: `InvalidInput`, `PermissionDenied`, `Conflict`, `EffectUnavailable`.

### `delete_category`

Purpose: Delete category and optionally reassign or remove event category references.

Input: `context`, `category_id`, optional `replacement_category_id`, optional `confirmation`.

Return data: `{ "category_id", "event_update_plan": {}, "requires_host_execution": true }`.

Host effects: `CalendarStoreWrite`, optional `CalendarStoreDelete`.

Errors: `CategoryNotFound`, `PermissionDenied`, `DestructiveConfirmationRequired`, `EffectUnavailable`.

### `bulk_update_events`

Purpose: Plan bulk category, move, copy, delete, or privacy/availability update with explicit confirmation.

Input: `context`, `selection` (`event_ids[]` or `query_scope`), `operation`, `confirmation`.

Return data: `{ "affected_count", "operation_state": {}, "requires_host_execution": true }`.

Host effects: `CalendarStoreWrite`, optional `CalendarStoreDelete`, `CalendarInviteSend`, `AuditWrite`.

Errors: `InvalidInput`, `PermissionDenied`, `PolicyDenied`, `DestructiveConfirmationRequired`, `EffectUnavailable`.

### `get_sharing_state`

Purpose: Read calendar shares, delegate state, and allowed sharing actions.

Input: `context`, `calendar_id`, optional `snapshot`, optional `refresh`.

Return data: `{ "shares": [], "delegates": [], "allowed_actions": [] }`.

Host effects: `CalendarShareManage`, `CalendarStoreRead`.

Errors: `CalendarNotFound`, `PermissionDenied`, `EffectUnavailable`.

### `update_sharing`

Purpose: Grant, revoke, or change sharing/delegate access.

Input: `context`, `calendar_id`, `changes[]`, optional `message`, `confirmation`.

Return data: `{ "sharing_state": {}, "provisioning_state": {}, "requires_host_execution": true }`.

Host effects: `CalendarShareManage`, `ContactLookup`, `AuditWrite`, optional `NotifyUser`.

Errors: `CalendarNotFound`, `PermissionDenied`, `PolicyDenied`, `DestructiveConfirmationRequired`, `EffectUnavailable`.

### `accept_shared_calendar`

Purpose: Accept or provision a host-owned shared calendar invitation.

Input: `context`, `share_invitation_id`, optional `display_options`.

Return data: `{ "calendar": CalendarSummary, "provisioning_state": {} }`.

Host effects: `CalendarShareManage`, `CalendarStoreWrite`.

Errors: `ShareNotFound`, `PermissionDenied`, `PolicyDenied`, `EffectUnavailable`.

### `subscribe_calendar`

Purpose: Validate and plan Internet Calendar subscription through host sync.

Input: `context`, `subscription` with `url_ref` or host-safe `subscription_ref`, `display_name`, `color`, optional `refresh_interval_minutes`, `confirmation`.

Return data: `{ "subscription_state": {}, "calendar_preview": CalendarSummary }`.

Host effects: `SubscriptionSync`, `CalendarStoreWrite`.

Errors: `InvalidInput`, `PermissionDenied`, `PolicyDenied`, `EffectUnavailable`, `HostRejected`.

### `get_reminder_state`

Purpose: Read reminder delivery state for an event or calendar default.

Input: `context`, optional `event_id`, optional `calendar_id`.

Return data: `{ "reminders": ReminderIntent[], "delivery_state": {} }`.

Host effects: `CalendarStoreRead`, `ReminderSchedule`.

Errors: `ReminderNotFound`, `PermissionDenied`, `EffectUnavailable`.

### `snooze_or_dismiss_reminder`

Purpose: Plan host reminder snooze or dismiss action.

Input: `context`, `reminder_id`, `action` (`snooze`, `dismiss`), optional `snooze_until`.

Return data: `{ "reminder_state": ReminderIntent, "requires_host_execution": true }`.

Host effects: `ReminderSchedule`, `CalendarStoreWrite`.

Errors: `ReminderNotFound`, `PermissionDenied`, `InvalidInput`, `EffectUnavailable`.

### `get_user_preferences`

Purpose: Read calendar preferences and defaults.

Input: `context`, optional `include_defaults: boolean`, optional `snapshot`.

Return data: `{ "preferences": UserCalendarPreferences, "defaults": {}, "needs_host_refresh": boolean }`.

Host effects: `CalendarStoreRead`.

Errors: `PermissionDenied`, `EffectUnavailable`.

### `save_user_preferences`

Purpose: Validate and plan saving calendar settings.

Input: `context`, `preferences: UserCalendarPreferences`, optional `revision`.

Return data: `{ "preferences": UserCalendarPreferences, "updated": boolean }`.

Host effects: `CalendarStoreWrite`.

Errors: `InvalidInput`, `PermissionDenied`, `PolicyDenied`, `Conflict`, `EffectUnavailable`.

### `get_offline_state`

Purpose: Read host cache state, last sync timestamp, and offline mutation policy.

Input: `context`, optional `calendar_ids: string[]`.

Return data: `{ "offline_state": {}, "last_sync_at": null, "mutation_policy": "blocked|queued|draft_only|host_defined" }`.

Host effects: `OfflineCacheRead`.

Errors: `PermissionDenied`, `EffectUnavailable`.

### `get_activity_log`

Purpose: Read sanitized audit/activity projections for user and agent calendar actions.

Input: `context`, optional `filters`, optional `pagination`.

Return data: `{ "activities": [], "pagination": {} }`.

Host effects: `AuditRead`.

Errors: `PermissionDenied`, `EffectUnavailable`, `PolicyDenied`.

## 5. Contract Gates

| Gate | Status | Notes |
| --- | --- | --- |
| Exact export names supplied | Pass | All names above must match Rust exports and `plugin.json` tools. |
| UX operations covered | Pass | Views, event CRUD, meetings, scheduling assistant, invitations, recurrence, calendars, sharing/delegates, subscriptions, categories, reminders, offline, settings, and audit are covered. |
| Host-owned effects explicit | Pass | Each export lists required host effects and avoids direct I/O. |
| Error semantics explicit | Pass | Shared errors plus export-specific errors are defined. |
| Schema synchronization possible | Pass | `plugin-schema-plan.md` maps every export to input/output definitions. |
