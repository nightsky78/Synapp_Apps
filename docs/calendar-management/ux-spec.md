# Calendar Management UX Specification

Stage: 1 - UX-Spec  
App: `apps/first-party/calendar-management`  
App ID: `calendar-management`  
Display name: `Calendar Management`  
Date: 2026-05-08

## 1. Product Scope

Calendar Management must be a full human-usable calendar application for Synapp, inspired by Outlook Calendar but adapted to Synapp's dual-interface model. The first screen must open to a calendar work surface where a user can view time, create and manage events, organize meetings, inspect conflicts, manage multiple calendars, and understand sharing or delegate access. It must not be a schema form, admin-only settings page, or AI-only tool demo.

The app serves two consumers equally:

- Human users use a browser UI to view schedules, create appointments, organize meetings, respond to invitations, manage recurrence, compare calendars, and recover from scheduling failures.
- AI agents use the Wasm/plugin semantic interface through the Synapp Broker to perform the same calendar operations within explicit permissions and host-managed effects.

The app remains a dumb executor. It validates calendar intent, normalizes event data, plans host effects, and returns structured outcomes. It must not include LLM prompts, AI routing, OpenAI SDKs, LangChain, direct database access, raw network sockets, mail sending logic, conference-provider logic, reminder dispatch, contact lookup, or persistence. The Synapp host owns those effects.

### Target Users

- Individual Synapp users managing their own work and personal schedule.
- Team members scheduling meetings with colleagues, rooms, time zones, reminders, categories, and online meeting links.
- Delegates or assistants who manage calendars they have been granted permission to edit.
- Calendar owners who share calendars with view or edit access.
- AI agents acting on behalf of a user to draft events, find times, update meetings, prepare calendar summaries, or manage events within granted permissions.

### MVP Goals

- Provide an Outlook-like day, work week, week, month, agenda, and side-by-side or overlay multi-calendar experience.
- Support creating, editing, moving, canceling, and deleting appointments and meetings, including all-day events, time zones, categories, reminders, locations, attendees, and recurrence.
- Support scheduling-assistant behavior using host-owned free/busy, room/location search, contact lookup, invite delivery, attendee response tracking, and conference link creation.
- Support multiple calendars, shared calendars, and delegate restrictions with clear user-visible permissions.
- Make every operation exposed to agents reachable from the human UI.
- Make host-effect limitations visible and recoverable without losing event drafts or user intent.

### Success Outcomes

- A user can open the app and immediately understand today's schedule, upcoming events, current calendar scope, and sync/permission state.
- A user can create a simple appointment in under one flow from a selected time slot.
- A user can schedule a meeting with required and optional attendees, inspect conflicts, choose an available time, add a room or online meeting, and send invitations through host effects.
- A user can manage recurring meetings without accidentally changing the wrong occurrence or series.
- A delegate can see what they can and cannot do on another user's calendar before attempting an action.
- An agent can perform calendar work only within explicit permissions, and humans can inspect or recover from agent-created or agent-updated events.

## 2. Product Principles

- Calendar-first workspace: the default route is a usable calendar, not settings.
- Time is the primary object: visual density, navigation, and commands should help users scan availability and act quickly.
- Human control for sensitive actions: sending invitations, canceling meetings, changing recurrence, changing someone else's calendar, and destructive bulk updates must be explicit and recoverable.
- Host-owned effects are visible: when invite sending, reminders, free/busy, contacts, rooms, conference links, persistence, or audit are unavailable, the UI explains the impacted feature and keeps local draft state where possible.
- Agent actions are transparent: agent-created or agent-modified calendar items are visible where host metadata is available, especially for sends, cancellations, recurrence changes, and delegate actions.

## 3. Information Architecture

### Primary App Routes

- `/calendar`: Default calendar work surface, opening to the user's last selected view or today.
- `/calendar/day/:date`: Day view.
- `/calendar/work-week/:date`: Work week view.
- `/calendar/week/:date`: Full week view.
- `/calendar/month/:date`: Month view.
- `/calendar/agenda`: Agenda/list view.
- `/calendar/event/new`: Create appointment or meeting.
- `/calendar/event/:eventId`: Event detail and edit surface.
- `/calendar/event/:eventId/occurrence/:occurrenceId`: Recurring occurrence detail.
- `/calendar/scheduling-assistant`: Find time, inspect free/busy, rooms, time zones, and send meeting invite.
- `/calendar/invitations`: Pending meeting invitations and responses.
- `/calendar/calendars`: Calendar list, creation, sharing, subscriptions, overlays, and permissions.
- `/calendar/settings`: User preferences for time zone, work hours, default reminders, view density, categories, and agent activity links.

### Main Workspace Layout

Desktop layout:

- Top command bar: display name, today, previous/next, date picker, view switcher, search, create event, refresh/sync status, permission indicator, settings.
- Left navigation: mini month picker, selected calendar list, shared calendars, room/resource calendars where available, category filters, invitations shortcut.
- Main calendar canvas: day/work-week/week/month/agenda grid with events, free/busy overlays, drag handles where permitted, current time indicator, and all-day row.
- Right detail panel or modal: selected event summary, RSVP controls, meeting details, attendees, recurrence, organizer controls, and recovery notices.
- Scheduling Assistant panel: attendee list, free/busy grid, suggested slots, room/location search, time-zone selector, and send/update action.

Mobile/tablet layout:

- Compact top bar with today, view switcher, create event, and calendar filter.
- Day and agenda are primary mobile views; month can be a navigational overview.
- Event detail opens as a full screen with a reliable back path.
- Scheduling Assistant uses stacked sections for attendees, date/time suggestions, and meeting details.

## 4. Human User Journeys

### 4.1 First Run And Empty Calendar

1. User opens Calendar Management.
2. App loads user permissions, host-effect availability, calendar list, categories, and current date.
3. If no calendars are available, the app shows an empty calendar setup state with actions to create a calendar, open shared calendar, add Internet Calendar subscription if host supports it, or retry host sync.
4. If calendars exist but contain no events in the selected range, the app shows the calendar grid and a calm empty state for that range.

Required behavior:

- The app must not require admin setup before showing the calendar work surface.
- If host calendar persistence is unavailable, creation and mutation actions are disabled with a clear reason.
- If read permission is missing, the app shows a permission-limited state and does not display event data.
- Empty range states must still allow event creation when the user has write permission.

Future Wasm/tool surface mapping:

- Read calendar summaries, read visible event range, read permissions and host capabilities, create calendar, subscribe/open shared calendar.

### 4.2 Navigate And Inspect Calendar

1. User opens the default calendar view.
2. User switches between day, work week, week, month, and agenda.
3. User navigates to today, previous/next range, or a selected date.
4. User filters visible calendars and categories.
5. User selects an event to inspect details.

Required behavior:

- Visible time range, time zone, selected calendars, and loading/sync state are always clear.
- Events show title, time, calendar color, category color, meeting/appointment status, recurrence indicator, reminder indicator, private/limited-detail indicator, online/in-person indicator where known, and conflict marker when applicable.
- Private or restricted events must display only the details allowed by the user's permission.
- Overlay mode combines selected calendars into one grid; side-by-side mode preserves separate columns or lanes.

Future Wasm/tool surface mapping:

- List events, get event detail, search/filter events, get calendar visibility state, normalize display ranges, validate permission-limited event projections.

### 4.3 Create Appointment

1. User selects a time slot or clicks create.
2. App opens a create surface defaulted to selected date/time, current calendar, current time zone, and user reminder preference.
3. User enters title, start/end date and time, all-day state, location, category/color, reminder, busy/free status, privacy, notes, and optional recurrence.
4. User saves the appointment.
5. App writes through host persistence and shows the event on the calendar.

Required behavior:

- End time must be after start time unless all-day handling normalizes the range.
- All-day events occupy the all-day row and use date-only semantics.
- A user can create an out-of-office event by setting availability to out of office.
- Reminder choices are stored as host reminder intents; the app does not dispatch reminders.
- If saving fails, the event draft remains open and editable.

Future Wasm/tool surface mapping:

- Validate event draft, create event, update event draft, read category defaults, plan reminder host effect, plan persistence host effect.

### 4.4 Create Meeting With Attendees

1. User creates a new event and adds required and optional attendees.
2. Contact autocomplete is host-assisted when available.
3. User sets title, date/time, time zone, location, online meeting option, reminders, category, agenda/notes, and recurrence if needed.
4. App checks conflicts for the organizer and attendees when free/busy is available.
5. User sends invitations.
6. App shows organizer tracking state and the event appears as a meeting.

Required behavior:

- Invite delivery is a host mail/calendar effect, never direct SMTP or network from Wasm.
- Required and optional attendees are visually distinct.
- Sending is disabled until organizer calendar, valid title or equivalent policy, valid time, and at least one valid attendee exist.
- If contact lookup is unavailable, the user can enter email addresses manually.
- If invite sending is unavailable, user may save a local/calendar-only draft when persistence is available, but the UI must clearly state that attendees were not invited.
- Online meeting creation is a host conference effect. If unavailable, the toggle is disabled or saves as requested-but-not-created depending on host policy.

Future Wasm/tool surface mapping:

- Validate meeting draft, resolve attendee identifiers through host effect, create meeting, plan invite-send host effect, plan conference-link host effect, read organizer tracking state.

### 4.5 Scheduling Assistant And Suggested Times

1. User opens Scheduling Assistant from a meeting draft.
2. User adds required and optional attendees.
3. App requests host free/busy for selected attendees and date range.
4. App displays attendee availability, organizer availability, time zones, conflicts, working-hours boundaries, and room/resource availability where available.
5. App suggests available slots and lets the user choose one.
6. User sends or returns to event editor with the selected time.

Required behavior:

- Free/busy data is host-owned and permission-limited.
- The UI must distinguish unknown availability from busy availability.
- Suggested slots must explain why a slot is recommended: all required attendees free, optional conflicts only, room available, within work hours, or outside work hours.
- Room/location search is host-owned. If unavailable, the user can enter a manual location.
- Time-zone differences are visible when attendees or event time zone differ from the user's default.
- The assistant must not hide conflicts; it may allow sending with conflicts after explicit user confirmation.

Future Wasm/tool surface mapping:

- Plan free/busy lookup, normalize availability windows, score suggested slots, validate selected slot, plan room/resource search, update meeting draft time.

### 4.6 Respond To Meeting Invitation

1. User opens an invitation from calendar, invitations view, or host notification deep link.
2. User reviews organizer, attendees, location, time, recurrence, conflicts, and notes.
3. User accepts, tentatively accepts, declines, or proposes a new time when host supports proposals.
4. User optionally sends a response message.
5. App updates calendar visibility and response status.

Required behavior:

- Response actions require calendar write/respond permission and host invite-response effect.
- If proposing a new time is unavailable, the action is hidden or disabled with explanation.
- Declining a meeting must explain whether the event remains visible as declined or is removed from the calendar according to host behavior.
- Response failure keeps the previous RSVP state and offers retry.

Future Wasm/tool surface mapping:

- Get invitation detail, validate response, submit RSVP, propose new time, update local response projection.

### 4.7 Update Existing Event Or Meeting

1. User selects an event and chooses edit.
2. App opens an edit surface with revision/conflict metadata where available.
3. User changes fields such as title, time, location, attendees, reminder, category, privacy, notes, or recurrence.
4. For meetings, app explains who will receive updates.
5. User saves or sends update.

Required behavior:

- Appointments save without invite-send behavior unless converted to a meeting.
- Meeting updates must distinguish updates to all attendees from updates to changed attendees where host supports that choice.
- Adding or removing attendees must show delivery impact before save.
- Changing time must surface organizer and attendee conflicts when free/busy is available.
- If the event changed elsewhere, the user sees conflict recovery before overwriting.

Future Wasm/tool surface mapping:

- Validate update, compare event revisions, update event, update meeting, plan attendee delta, plan update-send host effect, resolve conflict.

### 4.8 Cancel Or Delete Event

1. User selects an event.
2. User chooses delete or cancel.
3. App distinguishes appointment delete, meeting cancellation, occurrence deletion, and series cancellation.
4. User confirms destructive action and, for meetings, may include a cancellation note.
5. App applies host persistence and invite/cancel effects.

Required behavior:

- Deleting a personal appointment requires confirmation only when policy or recurrence risk warrants it.
- Canceling a meeting with attendees requires explicit confirmation and explains attendee notification.
- Permanent delete, broad date-range delete, and series delete require stronger confirmation.
- If host invite cancellation is unavailable, meeting cancellation cannot be presented as fully sent; the UI must allow saving a local cancellation only when policy allows and clearly warn attendees were not notified.

Future Wasm/tool surface mapping:

- Delete event, cancel meeting, cancel occurrence, cancel series, plan attendee notification, validate destructive scope.

### 4.9 Recurring Events And Meetings

1. User creates or edits an event and opens recurrence options.
2. User chooses repeat pattern such as daily, weekly, monthly, yearly, weekdays, custom interval, end after count, end by date, or no end where policy allows.
3. User saves the series.
4. When editing/deleting an occurrence, app asks whether to modify only this occurrence, this and following occurrences, or entire series where supported.

Required behavior:

- Recurrence preview must show the next several occurrences before save.
- Invalid recurrence rules must be caught before host persistence.
- Time-zone and daylight-saving changes must be represented in user-facing terms, especially for cross-time-zone meetings.
- A detached occurrence must be visibly tied to the series but editable as an exception.
- If a recurrence operation is unsupported by the host, the UI must explain and offer the closest safe action.

Future Wasm/tool surface mapping:

- Validate recurrence rule, preview recurrence instances, create series, update occurrence, update following occurrences, update series, delete occurrence, delete series.

### 4.10 Multiple Calendars

1. User opens the calendars panel.
2. User creates a new calendar with name, color, default reminders, visibility, and optional description.
3. User toggles calendars on/off, changes color, renames, deletes, overlays, or views side-by-side.
4. User copies or moves events between calendars when permitted.

Required behavior:

- Default and system calendars cannot be deleted if host policy forbids it.
- Deleting a calendar with events requires confirmation and explains whether events are deleted, moved, or preserved by host behavior.
- Moving an event to a calendar where the user lacks write permission is blocked.
- Calendar color changes affect UI display but must not imply category changes unless explicitly chosen.

Future Wasm/tool surface mapping:

- List calendars, create calendar, update calendar, delete calendar, set visibility/order, copy event, move event, validate calendar permissions.

### 4.11 Sharing, Delegates, And Subscriptions

1. Calendar owner opens sharing settings for a calendar.
2. Owner grants view-only, limited details, edit, or delegate access where host supports it.
3. Recipient sees the shared calendar in their calendar list after host acceptance or provisioning.
4. Delegate edits events within granted permissions.
5. User may add Internet Calendar subscription where host supports subscriptions.

Required behavior:

- Sharing and delegate permissions are host-managed policy and audit effects.
- UI must distinguish owner, editor, delegate, view-only, limited-details, and unavailable states.
- View-only users can inspect permitted details but cannot drag, edit, delete, respond for owner, or send updates.
- Delegates must see when an action will be performed on behalf of the calendar owner.
- Internet Calendar subscriptions are read-only unless host explicitly supports writeback.
- Permission-denied errors must identify the missing permission without exposing private event details.

Future Wasm/tool surface mapping:

- Read sharing state, validate share request, grant/revoke calendar access, accept shared calendar, list delegates, perform delegated event mutation, subscribe to external calendar.

### 4.12 Search, Categories, And Calendar Organization

1. User searches by title, attendee, location, notes, category, calendar, organizer, date range, or meeting status.
2. Results appear in agenda/list style with filter chips.
3. User opens, edits, categorizes, moves, or copies events from results.
4. User manages categories/colors from settings.

Required behavior:

- Active search scope and filters are visible and removable.
- Search failure preserves the previous visible calendar range.
- Categories are user-visible labels/colors; they do not grant permissions.
- Bulk category/move/delete actions require clear scope confirmation.

Future Wasm/tool surface mapping:

- Search events, apply category, remove category, list categories, create/update/delete category, bulk update events.

### 4.13 Reminders, Notifications, And Offline Behavior

1. User sets reminders on event drafts or defaults in settings.
2. Host schedules or dispatches reminders according to platform capability.
3. User sees reminder state on events and can snooze/dismiss only if host exposes reminder actions.
4. If offline or host reminders are unavailable, app distinguishes visible event data from unavailable reminder delivery.

Required behavior:

- The app must not dispatch notifications directly from Wasm.
- Reminder settings must show whether host reminder effects are available.
- Offline/cached calendar data may be shown only if host provides it, with last sync time.
- Mutations while offline are blocked, queued, or saved as drafts depending on host capability; the UI must state which behavior applies.

Future Wasm/tool surface mapping:

- Validate reminder intent, plan reminder host effect, read reminder status, snooze/dismiss reminder where host supports it, read offline cache state.

## 5. Agent-Assisted And Headless Workflows

Agents use the same capability surface through the Synapp Broker. The app validates inputs and plans host effects; the Broker owns permission injection, agent identity, audit, and AI routing.

### Agent Working Models

- Read-only scheduling assistant: lists calendars, reads permitted event ranges, searches events, summarizes busy/free blocks through structured data, and inspects invitations.
- Drafting assistant: creates unsent event or meeting drafts, proposes time changes, prepares attendee lists, and requests human review before invite delivery.
- Scheduling assistant with send permission: creates meetings, sends invitations, updates meetings, cancels meetings, and responds to invitations when explicitly granted.
- Delegate assistant: acts on shared or delegated calendars only within the owner's granted permission and host policy.
- Organizer assistant: manages calendars, categories, sharing requests, and bulk event organization when granted broader calendar-management permissions.

### Human Control Points

- Agent-created events or meeting drafts are labeled in the UI where host metadata is available.
- Sending invitations, canceling meetings, changing recurrence, modifying delegated calendars, sharing calendars, and broad bulk changes must be auditable and may require host confirmation depending on policy.
- Users can review recent agent calendar activity from settings or host-provided audit links.
- Users can revoke or reduce agent calendar permissions from a visible permission surface or deep link.

### Headless Outcomes

- Agent event creation produces the same calendar items a human action would produce, subject to permissions and host effects.
- Agent scheduling must handle unavailable free/busy, invite sending, contact lookup, room search, and conference-link effects as structured recoverable errors.
- Agent updates to meetings must include the intended notification scope: no send, changed attendees, all attendees, or host-default policy.
- Agent actions must never expose credentials, private event details beyond permission, or raw host secrets.

## 6. Permissions And Trust Boundaries

### User-Visible Permissions

The UI must show relevant capability state at the point of action:

- Calendar read unavailable because permission, host storage, or calendar connection is missing.
- Event creation unavailable because write permission or persistence effect is missing.
- Invite sending unavailable because send-invite host effect or permission is missing.
- Free/busy unavailable because lookup effect, attendee policy, or permission is missing.
- Conference link unavailable because host conference effect or provider connection is missing.
- Reminder unavailable because host reminder effect is missing.
- Sharing/delegate management unavailable because owner/admin permission is missing.

### Trust Boundaries

- Wasm owns validation, recurrence normalization, conflict-shape reasoning, draft normalization, and host-effect planning.
- Host owns calendar persistence, contacts, attendee identity resolution, mail/invite sending, reminders, free/busy lookup, room/resource lookup, conference link creation, Internet Calendar subscriptions, offline cache, and audit.
- UI owns human interaction state, visible calendar selection, unsaved draft buffers, optimistic feedback, and recovery flows.
- Synapp Broker owns AI routing, permission context, agent identity, policy injection, and audit correlation.
- Admin surfaces own organization policy and connectors, not user event content editing.

### Sensitive Data Requirements

- Event details require calendar read permission and must respect private or limited-details projections.
- Creating or editing events requires calendar write permission for the target calendar.
- Meeting invite delivery requires invite/send permission and host effect availability.
- Responding to invitations requires response permission for the user's calendar and host response effect.
- Sharing and delegate changes require owner or admin-granted share permission.
- Raw host credentials, provider tokens, mail transport details, conference provider secrets, and audit internals are never returned to Wasm or exposed to agents.

## 7. States And Recovery

### Global App States

- Initializing: app shell loads permissions, host capabilities, calendars, categories, current date, and selected view.
- Ready: visible calendar grid/list is interactive.
- Empty calendar list: no calendars available; create/open shared calendar actions appear when permitted.
- Empty visible range: selected calendar range has no events; creation remains available if permitted.
- Permission limited: app shell is visible, restricted actions are disabled, and explanations are local to each action.
- Offline/degraded: cached events shown only if host supports cache, with last sync time and disabled or queued mutations according to host capability.
- Host effect unavailable: impacted features are named and disabled without implying the user caused the issue.

### Loading States

- Calendar list loading.
- Event range loading with skeleton blocks or agenda rows.
- Event detail loading.
- Search loading with previous visible range preserved.
- Free/busy loading by attendee and room/resource.
- Suggested times loading.
- Invite send/update/cancel in progress with duplicate-submit protection.
- Recurrence preview loading or calculating.
- Sharing/delegate state loading.

### Empty States

- No calendars: offer create calendar, open shared calendar, subscribe if supported, or retry.
- No events in range: show date range and create action.
- No search results: show query/filter chips and broaden actions.
- No invitations: show clear pending-invitations state.
- No attendees: meeting editor prompts add required or optional attendees.
- No free/busy data: show unknown availability, not free.
- No categories: offer create category where permitted.

### Error States

- Calendar read failed: preserve shell, show retry, and do not show stale data as current without timestamp.
- Event save failed: keep draft open with field-level or host-level reason.
- Invite send failed: preserve meeting draft and show whether event was saved without invitations.
- Free/busy failed: keep attendee list and allow manual scheduling or retry.
- Conference link creation failed: allow save/send without link or retry, depending on user confirmation.
- Reminder scheduling failed: save event only after user acknowledges reminder will not be active, if policy allows.
- Sharing update failed: preserve previous sharing state and offer retry.
- Subscription failed: show host/provider reason and keep entered URL hidden if policy treats it as sensitive.
- Permission denied: explain missing capability and provide a request-access or switch-calendar path where available.

### Conflict States

- Event changed elsewhere: show local and remote updated timestamps and allow reload remote, keep local as copy, overwrite if permitted, or discard local.
- Event deleted elsewhere: remove from grid, offer restore only if host supports recovery, and keep local unsaved edits as a copy.
- Meeting attendees changed elsewhere: show attendee delta before saving or sending updates.
- Recurrence series changed elsewhere: require reload before editing occurrence or series.
- Calendar permission revoked during session: stop mutation actions, refresh visible detail projections, and explain revocation.
- Delegate owner changed permissions: block delegated actions and identify the affected calendar.

### Recurrence Edge Cases

- Daylight-saving transitions: preview occurrences using event time zone and show affected local times where relevant.
- Month-end patterns: explain how events on the 29th, 30th, or 31st behave in shorter months.
- No-end recurrence: allowed only if host policy permits; otherwise require end date or count.
- Detached occurrence edits: show that the occurrence differs from series defaults.
- Editing this-and-following: available only when host supports splitting a series; otherwise disabled with explanation.
- Attendee response on recurring meetings: response must identify whether it applies to one occurrence or the series.

### Recovery Behavior

- Never lose unsaved event or meeting draft content without explicit discard confirmation.
- Failed optimistic moves, resizes, category changes, or deletes must roll back or remain visibly pending until reconciled.
- User-correctable errors link to the relevant editor field or settings surface.
- Host/admin policy errors explain that the action is unavailable under current policy.
- Invite and cancellation failures must clearly distinguish saved calendar data from attendee notification state.

## 8. Feature Requirements

### Calendar Views

- Day, work week, week, month, and agenda views.
- Today navigation, date picker, previous/next range, and current time indicator.
- Time-zone display and user default time-zone preference.
- All-day row and timed grid.
- Event drag/move/resize where write permission exists, with keyboard and form alternatives.
- Side-by-side and overlay display for multiple calendars.

### Events And Meetings

- Appointment and meeting creation.
- Title, calendar, start/end, all-day, time zone, location, online meeting option, availability status, privacy, category/color, reminder, notes/description, and recurrence.
- Required and optional attendees.
- Organizer tracking for accepted, tentative, declined, no response, and proposed time states where host exposes them.
- In-person attendance indication where host supports it.
- Out-of-office availability.
- Forwarding or copy/share actions only when host policy permits.

### Scheduling Assistant

- Required and optional attendee free/busy grid.
- Suggested times.
- Unknown availability state.
- Room/resource search and availability where host supports it.
- Working-hours and time-zone awareness.
- Send/update from assistant after validation.

### Recurrence

- Daily, weekly, monthly, yearly, weekdays, and custom interval patterns.
- End by date, end after count, and host-policy-controlled no-end recurrence.
- Occurrence, this-and-following, and series operations where supported.
- Recurrence preview before save.

### Calendars, Sharing, And Delegates

- Create, rename, recolor, reorder, show/hide, delete calendars where permitted.
- Copy/move events between calendars where permitted.
- Shared calendars with view-only, limited details, edit, and delegate states.
- Internet Calendar subscriptions as read-only unless host supports more.
- Permission indicators in calendar list and action surfaces.

### Search And Organization

- Search by text, attendee, organizer, location, category, calendar, date range, RSVP status, and meeting/appointment type where host supports indexes.
- Category create/edit/delete and event category assignment.
- Bulk category, move, copy, or delete with confirmation for broad scopes.

### Settings

- Default view and density.
- Default calendar.
- Time zone and secondary time zone where host supports it.
- Work days and work hours.
- Default reminders.
- Default online meeting preference.
- Category management.
- Agent activity and permission deep links.

## 9. Human Workflow To Future Tool Surface Map

This is a high-level UX-to-capability map for Architect. It intentionally avoids final ABI names and JSON schemas.

| Human workflow | Future Wasm/tool surface |
| --- | --- |
| Open calendar workspace | Read app capabilities, permissions, calendars, categories, selected visible range, and sync state |
| Navigate views and ranges | Normalize date range, list events for range, project permission-limited details |
| Create appointment | Validate event draft, create event, plan persistence/reminder effects |
| Create meeting | Validate meeting draft, resolve attendees, create meeting, plan invite-send/conference/reminder effects |
| Use Scheduling Assistant | Plan free/busy lookup, score suggested slots, search rooms/resources, update draft time/location |
| Respond to invitation | Read invitation, validate RSVP, submit response, propose new time |
| Edit event or meeting | Validate update, compare revision, update event, plan update notification scope |
| Cancel/delete event | Validate destructive scope, delete appointment, cancel meeting/occurrence/series, plan cancellation notification |
| Manage recurrence | Validate recurrence rule, preview occurrences, update occurrence/following/series |
| Manage calendars | List/create/update/delete calendars, set display order, move/copy events |
| Manage sharing/delegates | Read share state, grant/revoke access, accept shared calendar, validate delegated mutation |
| Search and categories | Search events, list/manage categories, assign/remove categories, bulk update |
| Reminders/offline | Validate reminder intent, read reminder state, plan snooze/dismiss where supported, read offline cache state |
| Agent activity | Read audit/activity projections supplied by host, link actions to agent metadata |

## 10. Observable Acceptance Criteria

### Main App

- Given a user with at least one readable calendar, when they open Calendar Management, then the first visible screen is a calendar workspace with date navigation, view switcher, selected calendar list, visible time range, create action, search, and sync or permission status.
- Given no calendars are available, when the app opens, then the user sees an empty calendar-list state with permitted actions to create, open shared, subscribe, or retry.
- Given events are loading, when the user changes date range, then the app shows loading state for the new range while preserving navigational context.
- Given a user switches between day, work week, week, month, and agenda, then the selected date/range remains coherent and selected calendars remain unchanged.

### Event And Meeting Creation

- Given a user selects an empty time slot, when the create surface opens, then it is prefilled with the selected calendar, start/end time, date, time zone, and default reminder.
- Given a user enters an end time before the start time, when they save, then the app blocks save and identifies the time error.
- Given a user creates an all-day event, when it is saved, then it appears in the all-day row or all-day agenda grouping.
- Given a user creates a meeting with attendees, when invite sending succeeds, then the event appears as a meeting and attendee tracking state is visible where host exposes it.
- Given invite sending fails after event persistence succeeds, then the UI clearly states the event was saved but attendees were not invited and offers retry or recovery.

### Scheduling Assistant

- Given required and optional attendees are added, when free/busy lookup completes, then required and optional attendee availability are visually distinct.
- Given free/busy is unavailable for an attendee, then that attendee is shown as unknown, not free.
- Given suggested slots are returned, then each suggestion identifies whether required attendees, optional attendees, rooms, work hours, and time zones are satisfied.
- Given the user chooses a conflicted time, when they send, then the app requires confirmation that conflicts remain.

### Recurrence

- Given a user configures recurrence, when the rule is valid, then the app shows a preview of upcoming occurrences before save.
- Given a recurrence rule is invalid or unsupported, when the user saves, then the app blocks save and explains the unsupported part.
- Given a user edits a recurring event occurrence, then the app asks whether the change applies to the occurrence, this and following occurrences where supported, or the whole series.
- Given an occurrence has been detached from a series, then event detail identifies it as an exception.

### Updates, Deletes, And Conflicts

- Given a user edits a meeting time, when free/busy is available, then organizer and attendee conflicts are surfaced before update send.
- Given a user removes an attendee, when saving a meeting update, then the app explains whether all attendees or changed attendees will receive updates.
- Given a user cancels a meeting with attendees, when confirming, then the app explains attendee notification and optional cancellation note behavior.
- Given an event was changed elsewhere, when the user attempts to save, then the app offers reload remote, keep local as copy, overwrite if permitted, or discard local.

### Calendars, Sharing, And Delegates

- Given multiple calendars are selected, when the user switches between overlay and side-by-side, then the same calendars remain visible with distinct colors or lanes.
- Given a view-only shared calendar is selected, when the user opens an event, then edit, drag, delete, and send-update actions are disabled with permission explanation.
- Given a delegate edits another user's calendar, when saving, then the UI identifies that the action is on behalf of the calendar owner.
- Given a calendar delete contains events, when the user requests deletion, then the app explains the event outcome before applying the deletion.

### Permissions, Host Effects, And Offline

- Given calendar read permission is missing, when the app opens, then event details are not displayed and the user sees the missing capability.
- Given host persistence is unavailable, when the user opens create/edit, then save actions are disabled or draft-only according to host capability and clearly labeled.
- Given conference-link creation is unavailable, when the user toggles online meeting, then the app disables the option or requires confirmation to send without a link.
- Given the app is offline with cached events, then it shows last sync time and clearly identifies whether mutations are blocked, queued, or saved as drafts.

### Agents

- Given an agent has read-only calendar permission, when it attempts to create, update, delete, or send invites, then the tool response denies the operation and the UI can show the missing permission.
- Given an agent creates a meeting draft, when the human opens the event or draft, then agent attribution is visible where host metadata exists.
- Given an agent sends, updates, or cancels a meeting with permission, when the user reviews recent activity, then the action is auditable by calendar, event, attendee scope, timestamp, and originating agent where host metadata exists.
- Given agent permission is revoked during a session, when the agent attempts another mutation, then the operation fails with a permission-denied result and no event data is mutated.

## 11. Non-Goals For MVP

- Pixel-perfect visual design, final component styling, and final copy strings.
- Final Wasm ABI names, JSON schemas, or `plugin.json` contracts; Architect owns those.
- Native Exchange, Graph, CalDAV, SMTP, Teams, Zoom, or Google API calls from the app module.
- AI reasoning, prompts, autonomous scheduling decisions, or agent routing inside the app.
- Full To Do/task integration. The product should remain future-aware by preserving extension points for task-linked reminders or follow-ups, but tasks are not required in MVP.
- Advanced resource booking policy management beyond host-provided room/resource search and availability.
- Offline-first calendar editing unless the host provides queueing or draft persistence effects.

## 12. Open Questions For Later Pipeline Stages

- Which host effects exist at launch for calendar persistence, free/busy, invites, contacts, reminders, rooms, conference links, sharing, subscriptions, and audit?
- What permission strings should the Broker inject for read, write, invite-send, respond, share, delegate, and administer-calendar actions?
- Does the host support proposed times, forwarding meetings, in-person attendance tracking, and changed-attendees-only update delivery?
- What recurrence rule subset should be guaranteed across host providers?
- How should private events be projected for delegates, view-only users, and agents?
- Does the host support offline cached reads, offline mutation queueing, or only online operation?
- What audit metadata is available for agent-created events, sends, cancellations, sharing changes, and delegate actions?
- Should Internet Calendar subscriptions be part of MVP if host subscription effects are not ready, or deferred behind capability detection?

## 13. Pipeline Handoff Notes

Architect should translate this UX spec into contracts for calendar data models, recurrence rules, permission context, host effect intents, and exact Wasm exports/plugin tools. The contracts must preserve the dual-interface rule: every export must have a corresponding UI path, and every UI operation that mutates or reads host-owned calendar data must map to sandbox-safe Wasm validation plus host effects.

UI-Designer should produce a calendar-first workspace with Outlook-inspired density and predictable scheduling flows: calendar navigation, event editor, Scheduling Assistant, invitations, calendar sharing, recurrence editing, and all states above.

Developer should implement the app under `apps/first-party/calendar-management`, register the UI entrypoint in `synapp.app.json`, keep Rust/Go compatible with `wasm32-wasip1` or `wasm32-unknown-unknown`, update `plugin.json` alongside all tool signatures, and avoid direct network/database/AI logic in the app.