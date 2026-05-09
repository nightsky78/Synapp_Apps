# Calendar Management plugin.json Schema Plan

Stage: 2 - Architect
App: `apps/first-party/calendar-management`
Date: 2026-05-08

## 1. Synchronization Rule

`plugin.json` is the semantic contract for the Wasm execution interface. Every export in `contracts.md` must have a matching entry in both:

- `executionInterface.exports[]` with exact `name`, `inputSchema`, and `outputSchema`.
- `semanticInterface.tools[]` with exact `name`, user-facing description, category, capability, and risk notes where supported.

Whenever a Rust request struct, response data shape, host-effect payload, permission requirement, recurrence rule, projection rule, or error enum changes, `plugin.json` must be updated in the same commit.

Required execution metadata:

```json
{
  "executionInterface": {
    "wasmTarget": "wasm32-wasip1",
    "wasmPath": "target/wasm32-wasip1/release/calendar_management.wasm",
    "memoryManagement": {
      "returnedStringsAreNullTerminated": true,
      "deallocatorExport": "free_string",
      "inputEncoding": "utf-8-json"
    }
  }
}
```

## 2. Shared Definitions

Add or tighten these definitions under `definitions` and reuse them with `$ref`. Use `additionalProperties: false` except for explicitly host-extensible metadata maps.

### Envelope Definitions

- `RequestEnvelope`: requires `context`.
- `CalendarContext`: requires `user_id`, `permission`, `available_effects`; optional `capabilities`, `agent_id`, `request_id`, `locale`, `time_zone`, `policy`, `host_capabilities`, `audit_correlation_id`.
- `ResultEnvelope`: `oneOf` success or error.
- `SuccessEnvelope`: requires `ok.operation`, `ok.status`, `ok.data`, `ok.host_effects`; optional `ok.warnings`.
- `ErrorEnvelope`: requires `err`.
- `Error`: requires `code`, `message`; optional `details`. Enum codes must include all codes in `contracts.md`.
- `Warning`: requires `code`, `message`; optional `details`.
- `HostEffect`: requires `effect`, `intent`, `payload`, `idempotency_key`; optional `depends_on`, `on_failure`, `audit`.
- `HostEffectName`: enum `CalendarStoreRead`, `CalendarStoreWrite`, `CalendarStoreDelete`, `CalendarInviteSend`, `CalendarInviteRespond`, `FreeBusyLookup`, `RoomResourceLookup`, `ContactLookup`, `ConferenceLinkCreate`, `ReminderSchedule`, `CalendarShareManage`, `SubscriptionSync`, `OfflineCacheRead`, `AuditRead`, `AuditWrite`, `NotifyUser`.

### Primitive And Common Definitions

- `HostId`: string, 1 to 128 chars, opaque id pattern compatible with host ids.
- `IanaTimeZone`: non-empty string, max 128 chars.
- `IsoDateTime`: string with `format: date-time`.
- `IsoDate`: string with `format: date`.
- `BoundedText`: string max 512 chars.
- `LongBody`: string max 512000 chars.
- `Color`: string pattern for hex color or host token.
- `Pagination`: `limit` 1 to 100, optional `cursor`.
- `TimeRange`: requires `start`, `end`, `time_zone`; optional `all_day`; `start` and `end` accept date-time or date strings depending on `all_day`.
- `FieldError`: requires `field_path`, `code`, `message`.
- `Confirmation`: requires `confirmed`, optional `confirmation_token`, `acknowledged_risks[]`.
- `RevisionRef`: string max 256.

### Calendar Definitions

- `CalendarSummary`: fields from `contracts.md`; require `calendar_id`, `display_name`, `kind`, `role`, `color`, `visible`, `capabilities` for stored calendars.
- `CalendarKind`: enum `primary`, `secondary`, `shared`, `resource`, `subscription`, `birthday`, `holiday`, `host_defined`.
- `CalendarRole`: enum `owner`, `editor`, `delegate`, `viewer`, `limited_viewer`, `none`.
- `CalendarCapabilities`: object booleans for `can_create`, `can_update`, `can_delete`, `can_share`, `can_delegate`, `can_copy_from`, `can_move_from`, `can_move_to`, `can_subscribe`, `can_writeback_subscription`, `can_view_private_full`.
- `CalendarViewState`: selected calendars, hidden calendars, display order, mode `overlay|side_by_side`, view `day|work_week|week|month|agenda`, date anchor.
- `SyncState`: `state` enum `current`, `syncing`, `stale`, `offline_cached`, `failed`, `unknown`; optional `last_sync_at`, `message`.

### Event Definitions

- `CalendarEvent`: full event model from `contracts.md`.
- `EventDraft`: same editable fields as `CalendarEvent`, with optional ids/revisions.
- `EventPatch`: partial editable event fields plus explicit `unset_fields[]`.
- `EventKind`: enum `appointment`, `meeting`, `out_of_office`, `reminder_only`, `subscription_event`.
- `EventStatus`: enum `confirmed`, `tentative`, `cancelled`, `draft`.
- `Availability`: enum `free`, `busy`, `tentative`, `out_of_office`, `working_elsewhere`, `unknown`.
- `Privacy`: enum `normal`, `private`, `confidential`, `public`.
- `Importance`: enum `low`, `normal`, `high`.
- `ProjectionLevel`: enum `full`, `limited`, `availability_only`, `none`.
- `NotificationScope`: enum `none`, `changed_attendees`, `all_attendees`, `host_default`.
- `RecurrenceScope`: enum `single`, `occurrence`, `this_and_following`, `series`.
- `DeleteMode`: enum `delete`, `cancel`.
- `Attendee`: model from `contracts.md`; email uses `format: email` when present.
- `Organizer`: `organizer_id`, optional `email`, `display_name`.
- `OnlineMeeting`: `requested`, `provider`, `join_url_ref`, `state` enum `not_requested`, `requested`, `created`, `failed`, `unavailable`; no provider credentials.
- `RoomResource`: `resource_id`, `display_name`, optional `email`, `capacity`, `features[]`, `availability`.
- `DelegateContext`: model from `contracts.md`.
- `NotificationState`: `state` enum `none`, `planned`, `sent`, `partially_sent`, `blocked`, `failed`, `queued`, `draft_only`; optional `recipients[]`, `message`.

### Recurrence Definitions

- `RecurrenceRule`: requires `frequency`, `interval`, `end`, `time_zone`; optional fields from `contracts.md`.
- `RecurrenceFrequency`: enum `daily`, `weekly`, `monthly`, `yearly`.
- `DayOfWeek`: enum `MO`, `TU`, `WE`, `TH`, `FR`, `SA`, `SU`.
- `WeekOfMonth`: enum `first`, `second`, `third`, `fourth`, `last`.
- `RecurrenceEnd`: requires `mode`; mode enum `never`, `after_count`, `on_date`; `count` 1 to 999; `until` date.
- `MonthEndBehavior`: enum `skip_missing_day`, `last_day_of_month`.
- `RecurrenceException`: `occurrence_id`, `original_start`, `state` enum `detached`, `deleted`, `moved`; optional `event_patch`.
- `OccurrencePreview`: `occurrence_id`, `start`, `end`, `time_zone`, optional `dst_warning`, `is_exception`.

### Scheduling Definitions

- `FreeBusySnapshot`, `FreeBusyBlock`, `SuggestedSlot`, `ConflictDetail`, `WorkingHours`, `RoomSearchCriteria`, `RoomLookupRequest`, `FreeBusyLookupRequest`.
- `FreeBusyAvailability`: enum `free`, `busy`, `tentative`, `out_of_office`, `working_elsewhere`, `unknown`.
- `SuggestedSlotReason`: enum `required_attendees_free`, `optional_conflicts_only`, `room_available`, `within_work_hours`, `outside_work_hours`, `timezone_reasonable`, `least_conflicts`.

### Invitations, Sharing, Reminders, And Settings

- `InvitationSummary`, `InvitationDetail`, `InvitationResponse`, `ProposedTime`.
- `Category`: model from `contracts.md`.
- `ReminderIntent`: model from `contracts.md`.
- `SharePrincipal`, `SharingChange`, `SharingState`, `DelegateState`, `SubscriptionRequest`, `SubscriptionState`.
- `UserCalendarPreferences`, `WorkHours`, `OfflineState`, `ActivityLogEntry`.
- `BulkSelection`: one of `event_ids[]` or `query_scope` with `estimated_count` and confirmation.
- `BulkOperation`: category add/remove, move, copy, delete, privacy update, availability update.

Security-sensitive schema rule: no definition may contain a property named `password`, `app_password`, `access_token`, `refresh_token`, `client_secret`, `graph_token`, `caldav_password`, `smtp_password`, or `conference_secret`. Use host-owned refs such as `connection_ref`, `join_url_ref`, `subscription_ref`, or `secret_ref` only when needed.

## 3. synapp.app.json Schema Plan

Implementation must add `apps/first-party/calendar-management/synapp.app.json` with:

- `app_id`: `calendar-management`.
- `runtime`: `wasm32-wasip1`.
- `entrypoint`: `target/wasm32-wasip1/release/calendar_management.wasm`.
- `ui.entrypoint`: `ui/dist/index.html`.
- Navigation contribution target: `app://calendar-management/calendar`.
- Capabilities: `calendar:read`, `calendar:write`, `calendar:invite`, `calendar:respond`, `calendar:share`, `calendar:delegate`, `calendar:settings`, `calendar:audit`.

The manifest must not expose a schema-rendered main app form. Settings contributions, if added, must be user preferences or admin policy only and must not contain event bodies, invite content, free/busy details, provider tokens, or transport credentials.

## 4. Export Schema Map

Every row below must be added to `executionInterface.exports[]` and `semanticInterface.tools[]`.

| Export | Input Schema Definition | Output Data Definition | Semantic Category | Capability | Required Host Effects |
| --- | --- | --- | --- | --- | --- |
| `get_calendar_capabilities` | `GetCalendarCapabilitiesInput` | `CalendarCapabilitiesData` | query | `calendar:read` | `CalendarStoreRead`, optional `OfflineCacheRead`, `AuditRead` |
| `list_calendars` | `ListCalendarsInput` | `ListCalendarsData` | query | `calendar:read` | `CalendarStoreRead` |
| `save_calendar` | `SaveCalendarInput` | `SaveCalendarData` | mutation | `calendar:write` | `CalendarStoreWrite`, optional `ReminderSchedule` |
| `delete_calendar` | `DeleteCalendarInput` | `DeleteCalendarData` | mutation | `calendar:write` | `CalendarStoreDelete`, optional `CalendarStoreWrite`, `SubscriptionSync` |
| `set_calendar_view_state` | `SetCalendarViewStateInput` | `SetCalendarViewStateData` | mutation | `calendar:settings` | `CalendarStoreWrite` |
| `list_events` | `ListEventsInput` | `ListEventsData` | query | `calendar:read` | `CalendarStoreRead`, optional `OfflineCacheRead` |
| `get_event` | `GetEventInput` | `GetEventData` | query | `calendar:read` | `CalendarStoreRead` |
| `search_events` | `SearchEventsInput` | `SearchEventsData` | query | `calendar:read` | `CalendarStoreRead` |
| `validate_event_draft` | `ValidateEventDraftInput` | `EventValidationData` | validation | `calendar:write` | optional `CalendarStoreRead` |
| `create_event` | `CreateEventInput` | `CreateEventData` | mutation | `calendar:write`, optional `calendar:invite` | `CalendarStoreWrite`, optional `CalendarInviteSend`, `ConferenceLinkCreate`, `ReminderSchedule`, `AuditWrite`, `NotifyUser` |
| `update_event` | `UpdateEventInput` | `UpdateEventData` | mutation | `calendar:write`, optional `calendar:invite` | `CalendarStoreWrite`, optional `CalendarInviteSend`, `ConferenceLinkCreate`, `ReminderSchedule`, `AuditWrite`, `NotifyUser` |
| `delete_event` | `DeleteEventInput` | `DeleteEventData` | mutation | `calendar:write`, optional `calendar:invite` | `CalendarStoreDelete`, optional `CalendarInviteSend`, `ReminderSchedule`, `AuditWrite`, `NotifyUser` |
| `copy_event` | `CopyEventInput` | `CopyEventData` | mutation | `calendar:write` | `CalendarStoreRead`, `CalendarStoreWrite` |
| `move_event` | `MoveEventInput` | `MoveEventData` | mutation | `calendar:write` | `CalendarStoreWrite`, optional `CalendarInviteSend`, `AuditWrite` |
| `validate_recurrence` | `ValidateRecurrenceInput` | `RecurrenceValidationData` | validation | `calendar:write` | none |
| `preview_recurrence` | `PreviewRecurrenceInput` | `RecurrencePreviewData` | query | `calendar:read` | none |
| `plan_free_busy_lookup` | `PlanFreeBusyLookupInput` | `FreeBusyLookupPlanData` | mutation-plan | `calendar:read` | `FreeBusyLookup` |
| `suggest_meeting_times` | `SuggestMeetingTimesInput` | `SuggestedTimesData` | validation | `calendar:read` | none |
| `search_rooms` | `SearchRoomsInput` | `RoomSearchPlanData` | mutation-plan | `calendar:read` | `RoomResourceLookup` |
| `resolve_attendees` | `ResolveAttendeesInput` | `ResolveAttendeesData` | mutation-plan | `calendar:read` | optional `ContactLookup` |
| `list_invitations` | `ListInvitationsInput` | `ListInvitationsData` | query | `calendar:read` | `CalendarStoreRead` |
| `get_invitation` | `GetInvitationInput` | `GetInvitationData` | query | `calendar:read` | `CalendarStoreRead`, optional `FreeBusyLookup` |
| `respond_to_invitation` | `RespondToInvitationInput` | `InvitationResponseData` | mutation | `calendar:respond` | `CalendarInviteRespond`, `CalendarStoreWrite`, optional `NotifyUser` |
| `propose_new_time` | `ProposeNewTimeInput` | `ProposeNewTimeData` | mutation | `calendar:respond` | `CalendarInviteRespond`, `CalendarStoreWrite` |
| `list_categories` | `ListCategoriesInput` | `ListCategoriesData` | query | `calendar:read` | `CalendarStoreRead` |
| `save_category` | `SaveCategoryInput` | `SaveCategoryData` | mutation | `calendar:settings` | `CalendarStoreWrite` |
| `delete_category` | `DeleteCategoryInput` | `DeleteCategoryData` | mutation | `calendar:settings` | `CalendarStoreWrite`, optional `CalendarStoreDelete` |
| `bulk_update_events` | `BulkUpdateEventsInput` | `BulkUpdateEventsData` | mutation | `calendar:write` | `CalendarStoreWrite`, optional `CalendarStoreDelete`, `CalendarInviteSend`, `AuditWrite` |
| `get_sharing_state` | `GetSharingStateInput` | `SharingStateData` | query | `calendar:share` | `CalendarShareManage`, `CalendarStoreRead` |
| `update_sharing` | `UpdateSharingInput` | `UpdateSharingData` | mutation | `calendar:share` | `CalendarShareManage`, `ContactLookup`, `AuditWrite`, optional `NotifyUser` |
| `accept_shared_calendar` | `AcceptSharedCalendarInput` | `AcceptSharedCalendarData` | mutation | `calendar:share` | `CalendarShareManage`, `CalendarStoreWrite` |
| `subscribe_calendar` | `SubscribeCalendarInput` | `SubscribeCalendarData` | mutation | `calendar:share` | `SubscriptionSync`, `CalendarStoreWrite` |
| `get_reminder_state` | `GetReminderStateInput` | `ReminderStateData` | query | `calendar:read` | `CalendarStoreRead`, `ReminderSchedule` |
| `snooze_or_dismiss_reminder` | `SnoozeOrDismissReminderInput` | `SnoozeOrDismissReminderData` | mutation | `calendar:write` | `ReminderSchedule`, `CalendarStoreWrite` |
| `get_user_preferences` | `GetUserPreferencesInput` | `UserPreferencesData` | query | `calendar:settings` | `CalendarStoreRead` |
| `save_user_preferences` | `SaveUserPreferencesInput` | `UserPreferencesMutationData` | mutation | `calendar:settings` | `CalendarStoreWrite` |
| `get_offline_state` | `GetOfflineStateInput` | `OfflineStateData` | query | `calendar:read` | `OfflineCacheRead` |
| `get_activity_log` | `GetActivityLogInput` | `ActivityLogData` | query | `calendar:audit` | `AuditRead` |

## 5. Input Definition Requirements

- `GetCalendarCapabilitiesInput`: `context`, optional `calendar_ids` max 100.
- `ListCalendarsInput`: `context`, optional `include_hidden`, `refresh`, `snapshot`.
- `SaveCalendarInput`: `context`, `calendar`, optional `revision`, optional `create`.
- `DeleteCalendarInput`: `context`, `calendar_id`, `mode`, optional `revision`, optional `confirmation`.
- `SetCalendarViewStateInput`: `context`, `view_state`.
- `ListEventsInput`: `context`, `range`, `calendar_ids` 1 to 100, optional `filters`, `projection`, `snapshot`, `refresh`.
- `GetEventInput`: `context`, `event_id`, optional `calendar_id`, `occurrence_id`, `projection`, `snapshot`, `refresh`.
- `SearchEventsInput`: `context`, `query` max 256, optional `filters`, `pagination`, `projection`, `snapshot`, `refresh`.
- `ValidateEventDraftInput`: `context`, `draft`, optional `existing_event`, optional `policy_snapshot`.
- `CreateEventInput`: `context`, `draft`, `send_invites`, optional `confirmation`, optional `client_operation_id`.
- `UpdateEventInput`: `context`, `event_id`, `patch`, `scope`, `notification_scope`, optional `occurrence_id`, `revision`, `confirmation`.
- `DeleteEventInput`: `context`, `target`, `scope`, `mode`, optional `message`, `revision`, `confirmation`.
- `CopyEventInput`: `context`, `event_id`, `destination_calendar_id`, optional `occurrence_id`, `copy_options`.
- `MoveEventInput`: `context`, `event_id`, `destination_calendar_id`, optional `occurrence_id`, `revision`, `notification_scope`.
- `ValidateRecurrenceInput`: `context`, `event_time`, `recurrence`, optional `policy_snapshot`.
- `PreviewRecurrenceInput`: `context`, `event_time`, `recurrence`, optional `limit` max 50.
- `PlanFreeBusyLookupInput`: `context`, `attendees` max 500, `range`, optional `granularity_minutes` enum 5, 10, 15, 30, 60.
- `SuggestMeetingTimesInput`: `context`, `meeting_draft`, `free_busy_snapshot`, optional `rooms_snapshot`, optional `preferences`.
- `SearchRoomsInput`: `context`, `range`, optional `query`, `capacity`, `features`, `location_hint`.
- `ResolveAttendeesInput`: `context`, `attendees` max 500, optional `allow_manual_email`.
- `ListInvitationsInput`: `context`, optional `filters`, `pagination`, `snapshot`, `refresh`.
- `GetInvitationInput`: `context`, `invitation_id`, optional `include_conflicts`.
- `RespondToInvitationInput`: `context`, `invitation_id`, `response`, optional `scope`, `message`, `send_response`.
- `ProposeNewTimeInput`: `context`, `invitation_id`, `proposed_time`, optional `scope`, `message`.
- `ListCategoriesInput`: `context`, optional `calendar_id`, `snapshot`, `refresh`.
- `SaveCategoryInput`: `context`, `category`, optional `revision`.
- `DeleteCategoryInput`: `context`, `category_id`, optional `replacement_category_id`, `confirmation`.
- `BulkUpdateEventsInput`: `context`, `selection`, `operation`, `confirmation`.
- `GetSharingStateInput`: `context`, `calendar_id`, optional `snapshot`, `refresh`.
- `UpdateSharingInput`: `context`, `calendar_id`, `changes`, `confirmation`, optional `message`.
- `AcceptSharedCalendarInput`: `context`, `share_invitation_id`, optional `display_options`.
- `SubscribeCalendarInput`: `context`, `subscription`, `confirmation`.
- `GetReminderStateInput`: `context`, one of `event_id` or `calendar_id`.
- `SnoozeOrDismissReminderInput`: `context`, `reminder_id`, `action`, optional `snooze_until`.
- `GetUserPreferencesInput`: `context`, optional `include_defaults`, `snapshot`.
- `SaveUserPreferencesInput`: `context`, `preferences`, optional `revision`.
- `GetOfflineStateInput`: `context`, optional `calendar_ids`.
- `GetActivityLogInput`: `context`, optional `filters`, `pagination`.

## 6. Output Data Definition Requirements

Each export output schema must be `ResultEnvelope` with `ok.data` constrained to the mapped data definition:

- `CalendarCapabilitiesData`: `capabilities`, `policy`, `host_effects`, `default_view_state`, `warnings`.
- `ListCalendarsData`: `calendars`, `visibility_state`, `sync_state`, `needs_host_refresh`.
- `SaveCalendarData`: `calendar`, `created`, `updated`.
- `DeleteCalendarData`: `calendar_id`, `deletion_mode`, `event_outcome`, `requires_confirmation`.
- `SetCalendarViewStateData`: `view_state`, `normalized`.
- `ListEventsData`: `events`, `range`, `projection_applied`, `sync_state`, `warnings`.
- `GetEventData`: `event`, `projection_applied`, `allowed_actions`, `warnings`.
- `SearchEventsData`: `results`, `facets`, `pagination`, `projection_applied`.
- `EventValidationData`: `valid`, `normalized_draft`, `field_errors`, `warnings`, `required_actions`.
- `CreateEventData`: `event_preview`, `persistence_status`, `notification_state`, `requires_host_execution`.
- `UpdateEventData`: `event_preview`, `attendee_delta`, `notification_state`, `conflict_state`, `requires_host_execution`.
- `DeleteEventData`: `target`, `scope`, `deletion_state`, `notification_state`, `requires_host_execution`.
- `CopyEventData`: `source_event_id`, `new_event_preview`, `requires_host_execution`.
- `MoveEventData`: `event_preview`, `source_calendar_id`, `destination_calendar_id`, `notification_state`.
- `RecurrenceValidationData`: `valid`, `normalized_recurrence`, `field_errors`, `warnings`.
- `RecurrencePreviewData`: `occurrences`, `truncated`, `warnings`.
- `FreeBusyLookupPlanData`: `lookup_request`, `requires_host_execution`.
- `SuggestedTimesData`: `suggestions`, `conflicts`, `warnings`.
- `RoomSearchPlanData`: `room_lookup_request`, `requires_host_execution`.
- `ResolveAttendeesData`: `normalized_attendees`, `unresolved`, `requires_host_execution`.
- `ListInvitationsData`: `invitations`, `pagination`, `sync_state`.
- `GetInvitationData`: `invitation`, `allowed_responses`, `conflicts`, `warnings`.
- `InvitationResponseData`: `invitation_id`, `response_state`, `calendar_update_state`, `requires_host_execution`.
- `ProposeNewTimeData`: `proposal_state`, `proposed_time`, `requires_host_execution`.
- `ListCategoriesData`: `categories`.
- `SaveCategoryData`: `category`, `created`, `updated`.
- `DeleteCategoryData`: `category_id`, `event_update_plan`, `requires_host_execution`.
- `BulkUpdateEventsData`: `affected_count`, `operation_state`, `requires_host_execution`.
- `SharingStateData`: `shares`, `delegates`, `allowed_actions`.
- `UpdateSharingData`: `sharing_state`, `provisioning_state`, `requires_host_execution`.
- `AcceptSharedCalendarData`: `calendar`, `provisioning_state`.
- `SubscribeCalendarData`: `subscription_state`, `calendar_preview`.
- `ReminderStateData`: `reminders`, `delivery_state`.
- `SnoozeOrDismissReminderData`: `reminder_state`, `requires_host_execution`.
- `UserPreferencesData`: `preferences`, `defaults`, `needs_host_refresh`.
- `UserPreferencesMutationData`: `preferences`, `updated`.
- `OfflineStateData`: `offline_state`, `last_sync_at`, `mutation_policy`.
- `ActivityLogData`: `activities`, `pagination`.

## 7. Semantic Tool Descriptions

Descriptions must make the host boundary clear.

Required wording patterns:

- Query tools: "Read host-managed calendar data for the current user with permission-limited projections."
- Validation tools: "Validate and normalize calendar input in Wasm without performing persistence or network I/O."
- Mutation-plan tools: "Validate and plan host-owned effects; Wasm does not execute provider, network, invite, reminder, or persistence operations."
- Sharing/delegate tools: "Plan host-owned sharing or delegated access changes with audit-aware payloads."
- Subscription tools: "Plan host-owned Internet Calendar subscription sync; Wasm never fetches remote URLs."
- Audit tools: "Read sanitized host audit projections without exposing audit internals or private event details beyond permission."

Risk notes:

- `calendar:read`: high, because event content and availability are sensitive.
- `calendar:write`: high, because events, calendars, categories, and reminders can be changed or deleted.
- `calendar:invite`: critical, because invitations, updates, and cancellations reach other people.
- `calendar:respond`: high, because RSVP state affects attendance and organizer tracking.
- `calendar:share`: high, because access to calendar data can be granted or revoked.
- `calendar:delegate`: critical, because actions may be performed on behalf of another owner.
- `calendar:settings`: medium, because preferences and defaults affect future scheduling.
- `calendar:audit`: high, because activity projections may expose sensitive behavioral metadata.

## 8. Schema Tightening Checklist

For every input schema:

- Require `context` and all operation-specific fields listed above.
- Set `additionalProperties: false` unless host metadata explicitly needs extension.
- Add `minLength`, `maxLength`, `minItems`, `maxItems`, `minimum`, and `maximum` constraints from `contracts.md`.
- Use enums for permission, capabilities, effect names, view mode, event kind/status, availability, privacy, recurrence, RSVP, projection, notification scope, sharing access, reminder action, offline mutation policy, and error codes.
- Use `format: email` for attendee/share principal email fields while preserving manual-entry fallback.
- Use opaque refs for join URLs, subscriptions, connections, and any host-sensitive data.

For every output schema:

- Use `ResultEnvelope` consistently.
- Constrain `ok.data` to the export-specific data definition.
- Constrain `ok.host_effects[]` to `HostEffect`.
- Include warnings for private projection, unknown free/busy, unsupported recurrence operations, partial invite delivery, conference/reminder unavailability, stale snapshots, offline cache, delegate action, and destructive confirmation.

## 9. Compatibility And Implementation Order

1. Add `apps/first-party/calendar-management` scaffold with Rust Wasm, React UI, `plugin.json`, and `synapp.app.json` together.
2. Implement shared envelope and model definitions in Rust and `plugin.json` first.
3. Implement read/query exports: capabilities, calendars, events, search, categories, preferences, offline state.
4. Implement validation/planning exports: event draft, recurrence, free/busy plan, suggestions, rooms, attendees.
5. Implement mutation exports: create/update/delete/copy/move events, invitations, categories, sharing, subscription, reminders, preferences.
6. Wire every export to a UI surface matching `ux-spec.md`.
7. Add catalog entry and package metadata after manifest/plugin paths are stable.

Compatibility requirements:

- The app must compile to `wasm32-wasip1` without native network, database, TLS, C-binding, thread, or AI dependencies.
- Host effects and JSON payloads are plain data contracts, mirrored between Rust request/response structs and `plugin.json`.
- UI routes must remain the primary human experience; `plugin.json` tools are not a substitute for the human UI.
- No direct Graph, Exchange, CalDAV, SMTP, room-provider, conference-provider, reminder, subscription, or audit calls occur inside app code.

## 10. Schema Gates

| Gate | Status | Required Action |
| --- | --- | --- |
| Every Wasm export has input/output schema mapping | Pass by plan | Export map covers all contracts in `contracts.md`. |
| Shared definitions cover required models | Pass by plan | Context, ids, calendar/event/attendee/recurrence/reminder/category/free-busy/share/host-effect/warning/error models are listed. |
| UX operations are tool-addressable | Pass by plan | Calendar views, events, meetings, scheduling assistant, invitations, recurrence, calendars, sharing/delegates, subscriptions, categories, reminders, offline, settings, and audit are mapped. |
| Host-effect boundary is explicit | Pass by plan | All network/persistence/provider operations are host effects. |
| `plugin.json` sync requirements explicit | Pass by plan | Struct/schema changes must ship together. |
| Synapp violations avoided | Pass by plan | No AI logic, no direct network/database, full Dual-Interface, wasm32-wasip1 target. |
