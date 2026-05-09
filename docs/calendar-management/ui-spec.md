# Calendar Management UI Specification

Stage: 3 - UI-Designer  
App: `apps/first-party/calendar-management`  
Date: 2026-05-08

## 1. UI Intent

Calendar Management opens directly into a dense calendar workspace inspired by Outlook Calendar, without becoming a clone. The first viewport is the working calendar: command bar, calendar navigation, event canvas, and event detail/editor. Settings, sharing, invitations, reminders, offline recovery, and activity are reachable from the same shell but do not replace the primary calendar surface.

The UI is contract-first. Each human action maps to a planned Wasm export from `contracts.md`, and the React bridge invokes that exact export name when the Synapp host is available. In local development, deterministic sample data is used so the Developer stage can wire validation and host execution later.

## 2. Primary User Journeys

### Open And Scan The Calendar

1. Load capabilities, offline state, user preferences, calendars, categories, invitations, activity, and the current visible event range.
2. Show the selected date range in day, work-week, week, month, or agenda mode.
3. Allow immediate filtering by calendar visibility, category, search query, and overlay or side-by-side display mode.
4. Selecting an event opens the right detail panel with permitted fields and allowed actions.

States:

- Idle: calendar grid/list is visible with sync and permission indicators.
- Loading: command bar and layout remain visible; grid uses row placeholders and disables mutating controls.
- Empty: the grid remains visible with an inline empty-range message and create action.
- Failure: a recoverable error banner appears above the grid with retry and offline-state access.
- Permission-limited: event tiles show allowed projection only, and edit/delete controls are disabled with a reason.

### Create Or Edit An Event

1. User clicks New event or selects a time slot.
2. Right panel opens the editor with calendar, title, date/time, all-day, time zone, availability, privacy, category, reminders, recurrence, location, notes, attendees, online meeting, and room controls.
3. Draft changes call validation controls before save.
4. Save appointment maps to `create_event` or `update_event`; send meeting maps to the same exports with invite notification options.
5. Failed saves keep the draft open.

States:

- Idle: editable form with Save and Validate actions.
- Loading: Save/Send shows planning state and inputs remain readable.
- Success: status toast and preview event appear in the calendar.
- Failure: field-level errors show near affected controls and a panel error keeps draft values.
- Empty draft: title can be blank only if policy allows; display fallback is `Untitled event`.

### Use Scheduling Assistant

1. User opens Scheduling from an event draft or command bar.
2. Assistant shows attendees, free/busy rows, suggested slots, room search, time-zone context, and conflict warnings.
3. Resolve attendees maps to `resolve_attendees`.
4. Check availability maps to `plan_free_busy_lookup`.
5. Suggestions map to `suggest_meeting_times`.
6. Room search maps to `search_rooms`.
7. Choosing a slot updates the draft and returns to the editor.

States:

- Empty: no attendees yet; manual entry is still allowed.
- Loading: free/busy rows show pending lookup.
- Success: suggested slots sorted by score with reasons.
- Failure: availability unknown is visually distinct from busy.
- Policy-limited: unavailable room/conference/contact effects disable only affected controls.

### Respond To Invitations

1. Invitations panel lists pending and recent requests.
2. Selecting one loads detail, conflicts, allowed responses, recurrence scope, and response message.
3. Accept, Tentative, Decline map to `respond_to_invitation`.
4. Propose time maps to `propose_new_time`.

States:

- Empty: no pending invitations.
- Loading: list and detail skeletons.
- Success: response state updates in place.
- Failure: previous RSVP remains visible with retry.
- Limited: unavailable proposal support is disabled with an inline reason.

### Manage Calendars, Sharing, And Subscriptions

1. Calendars panel shows owned, shared, resource, and subscription calendars with visibility, color, role, sync, and permissions.
2. Create/update calendar maps to `save_calendar`.
3. Hide/delete/unsubscribe maps to `delete_calendar`.
4. Sharing detail maps to `get_sharing_state`.
5. Grant/revoke/delegate maps to `update_sharing`.
6. Accept shared invitation maps to `accept_shared_calendar`.
7. Subscribe to Internet Calendar maps to `subscribe_calendar`.

States:

- Empty: offer Create calendar, Accept shared calendar, and Subscribe where effects allow.
- Loading: calendar rows remain stable.
- Success: changed calendar row is highlighted briefly.
- Failure: destructive or policy-denied messages explain whether events are affected.
- Confirmation required: delete/unsubscribe/share changes require explicit confirmation.

### Settings, Reminders, Offline, Activity, And Bulk Actions

1. Settings exposes preferences, work hours, default view, density, time zones, overlay mode, categories, default reminders, and agent activity link.
2. Category list maps to `list_categories`; create/update/delete category maps to `save_category` and `delete_category`.
3. Reminder panel maps to `get_reminder_state` and `snooze_or_dismiss_reminder`.
4. Offline banner and panel map to `get_offline_state`.
5. Activity panel maps to `get_activity_log` and labels agent attribution only when host metadata exists.
6. Bulk selection bar maps to `bulk_update_events` after confirmation.

States:

- Empty categories/activity/reminders: show concise empty copy and reachable creation or retry actions.
- Loading: controls are disabled only for the operation in flight.
- Success: non-blocking confirmation toast.
- Failure: inline error with Retry or Review permissions.
- Offline: mutating actions show queued, draft-only, or blocked policy before execution.

## 3. Desktop Layout

### App Shell

- Top command bar, fixed height.
- Left navigation, fixed width with scroll.
- Main calendar canvas, flexible width.
- Right panel, fixed width on desktop; collapsible by choosing another panel or closing selection.

### Top Command Bar

Required controls:

- App name and permission badge.
- New event button.
- Today, previous, next, and date picker.
- View segmented control: Day, Work week, Week, Month, Agenda.
- Search field.
- Refresh/sync status.
- Invitations, Calendars, Settings, Activity, and Offline controls.

Behavior:

- Previous/next changes the selected range and triggers `list_events`.
- View switch triggers `set_calendar_view_state` and then `list_events`.
- Search triggers `search_events` and displays results in agenda-compatible list form.
- Refresh triggers `list_calendars`, `list_events`, and visible panel reads.

### Left Navigation

Required sections:

- Mini month picker with Today indicator and selected date.
- Calendar list with color swatches, visibility toggles, role labels, and sync states.
- Shared/resource/subscription grouping.
- Category filters.
- Shortcut buttons for Scheduling Assistant and Invitations.

Behavior:

- Calendar visibility toggles call `set_calendar_view_state`.
- Calendar row menu opens calendar management for `save_calendar`, `delete_calendar`, sharing, and subscriptions.
- Category filter calls `list_events` with filters.

### Main Calendar Canvas

Views:

- Day: one day time grid with all-day row and hourly slots.
- Work week: Monday through Friday lanes.
- Week: seven day lanes.
- Month: month cells with compact event chips and overflow indicator.
- Agenda: grouped list by day with status, location, attendee, category, and projection markers.

Event tile data:

- Title or projection-safe label.
- Time range.
- Calendar color and category color.
- Meeting, recurrence, reminder, private, online, room, conflict, delegate, and agent attribution indicators where host data permits.

Behavior:

- Click event: `get_event`, open detail panel.
- Double click slot: open create draft with selected time.
- Drag/resize placeholder: maps to `update_event` or `move_event` after validation in Developer stage.
- Bulk select in agenda/search: maps to `bulk_update_events` after confirmation.

### Right Panel

Panel modes:

- Event detail: inspect event, RSVP, edit, copy, move, delete/cancel, reminder state.
- Event editor: create/update form with validation and recurrence preview.
- Scheduling Assistant: free/busy, suggestions, rooms, attendee resolution.
- Invitations: list/detail and response controls.
- Calendars: calendars, sharing, delegates, subscriptions.
- Settings: preferences, categories, reminders, offline, activity link.
- Activity: audit projection and agent-attributed actions.

The panel uses tabs or panel headers, not nested cards. Repeated rows may use simple bordered list items.

## 4. Responsive Behavior

Desktop >= 1180px:

- Four-zone layout: command bar, left nav, main canvas, right panel.
- Week and work-week views show columns simultaneously.
- Right panel remains visible when an event is selected.

Tablet 760px to 1179px:

- Left nav collapses to a filter drawer.
- Right panel overlays the canvas at 420px max width.
- Work-week/week retain columns but hide secondary metadata in tiles.

Mobile < 760px:

- Default view becomes day or agenda based on preference.
- Left nav and right panel become full-screen drawers.
- Month is navigational overview; event details open full screen.
- Command bar compacts to New, Today, View, Search, and More.
- Touch targets are at least 44px high.

## 5. Component Behavior Notes

- `CalendarShell`: owns selected view, date anchor, active panel, selected event, active filters, and loading/error state.
- `CommandBar`: emits navigation, search, view, create, refresh, and panel-open intents.
- `CalendarSidebar`: emits calendar/category visibility and opens management surfaces.
- `CalendarCanvas`: renders day/work-week/week/month/agenda from the same normalized event data.
- `EventPanel`: switches between detail and editor mode, keeps unsaved drafts local, and calls validate/save/delete/copy/move exports.
- `SchedulingAssistantPanel`: works from the active draft and never fetches provider data directly.
- `InvitationsPanel`: supports list, detail, RSVP, and proposed-time flows.
- `CalendarsPanel`: supports calendars, sharing, delegates, and subscriptions.
- `SettingsPanel`: supports preferences, categories, reminder defaults, offline state, and activity access.
- `Bridge`: exposes `invoke(operation, payload)` and `getContext()`; no AI routing, network provider access, or persistence is implemented in the UI.

## 6. UI To Export Mapping

| Export | Human UI Surface |
| --- | --- |
| `get_calendar_capabilities` | App load, permission badge, unavailable feature notices. |
| `list_calendars` | Left calendar list and Calendars panel refresh. |
| `save_calendar` | Calendar create/edit form, color/name/visibility/default reminders. |
| `delete_calendar` | Calendar row Hide/Delete/Unsubscribe confirmation. |
| `set_calendar_view_state` | View switcher, visible calendars, overlay/side-by-side mode, selected date. |
| `list_events` | Calendar grid/list range refresh. |
| `get_event` | Event selection and event detail panel. |
| `search_events` | Command bar search and agenda-style results. |
| `validate_event_draft` | Event editor Validate action and save preflight. |
| `create_event` | New appointment/meeting Save or Send. |
| `update_event` | Edit event, drag/resize placeholder, attendee/category/reminder changes. |
| `delete_event` | Delete appointment, cancel meeting, delete occurrence/series confirmation. |
| `copy_event` | Event detail Copy to calendar action. |
| `move_event` | Event detail Move to calendar action and destination select. |
| `validate_recurrence` | Recurrence editor Validate recurrence action. |
| `preview_recurrence` | Recurrence preview list in editor. |
| `plan_free_busy_lookup` | Scheduling Assistant Check availability. |
| `suggest_meeting_times` | Scheduling Assistant Find times. |
| `search_rooms` | Scheduling Assistant room search. |
| `resolve_attendees` | Attendee entry Resolve action. |
| `list_invitations` | Invitations panel list refresh. |
| `get_invitation` | Invitation detail selection. |
| `respond_to_invitation` | Accept, Tentative, Decline controls. |
| `propose_new_time` | Propose time control in Invitations panel. |
| `list_categories` | Category filters and Settings categories list. |
| `save_category` | Category create/edit form. |
| `delete_category` | Category delete/reassign confirmation. |
| `bulk_update_events` | Agenda/search multi-select bulk action bar. |
| `get_sharing_state` | Sharing tab for selected calendar. |
| `update_sharing` | Grant/revoke/delegate controls. |
| `accept_shared_calendar` | Accept shared calendar invitation control. |
| `subscribe_calendar` | Subscribe calendar form. |
| `get_reminder_state` | Event reminders panel and reminder defaults state. |
| `snooze_or_dismiss_reminder` | Reminder banner Snooze/Dismiss actions. |
| `get_user_preferences` | Settings panel load. |
| `save_user_preferences` | Settings Save preferences. |
| `get_offline_state` | Offline status button and recovery panel. |
| `get_activity_log` | Activity panel and agent activity link. |

## 7. Accessibility Baseline

- All icon-like controls have text labels, `aria-label`, or tooltips.
- Segmented controls use `aria-pressed` on buttons.
- Calendar grid uses semantic buttons for selectable events and slots; agenda uses list semantics.
- Form controls have visible labels and error text connected by `aria-describedby` in implementation.
- Focus order follows command bar, left navigation, calendar canvas, then right panel.
- Right-panel overlays trap focus only when modal on mobile.
- Color is never the only indicator: calendar color is paired with labels, role text, or status text.
- Contrast targets WCAG AA for normal text and controls.
- Loading and toast messages use polite live regions; destructive confirmations use assertive dialog text.

## 8. Visual Direction

- Operational, dense, restrained, and predictable.
- White and light gray surfaces with clear borders, modest accent color, and category swatches.
- No hero page, no marketing section, no decorative gradients, and no nested cards.
- Cards are limited to repeated items such as event rows, invitation rows, and suggested slots, with radius 6px or less.
- Primary actions are compact buttons; binary settings use checkboxes or segmented controls; dates and times use native inputs/selects for the scaffold.
