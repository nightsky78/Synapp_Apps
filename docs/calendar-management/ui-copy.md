# Calendar Management UI Copy

Stage: 3 - UI-Designer  
App: `apps/first-party/calendar-management`  
Date: 2026-05-08

## 1. App And Navigation Labels

| Surface | Copy |
| --- | --- |
| App name | Calendar Management |
| Default route title | Calendar |
| New event | New event |
| Today | Today |
| Previous range | Previous |
| Next range | Next |
| Date picker label | Go to date |
| Search placeholder | Search calendar |
| Refresh | Refresh |
| Scheduling Assistant | Scheduling |
| Invitations | Invitations |
| Calendars | Calendars |
| Settings | Settings |
| Activity | Activity |
| Offline status | Offline state |

## 2. View And Layout Labels

| Control | Copy |
| --- | --- |
| Day view | Day |
| Work week view | Work week |
| Week view | Week |
| Month view | Month |
| Agenda view | Agenda |
| Overlay calendars | Overlay |
| Side-by-side calendars | Side by side |
| Show calendar | Show calendar |
| Hide calendar | Hide calendar |
| Calendar role label | Role |
| Sync current | Current |
| Sync stale | Stale |
| Sync offline cached | Offline cached |
| Sync failed | Sync failed |

## 3. Event Detail And Editor Copy

Labels:

- Title
- Calendar
- Starts
- Ends
- All day
- Time zone
- Location
- Availability
- Privacy
- Importance
- Category
- Reminder
- Repeat
- Attendees
- Required attendees
- Optional attendees
- Room
- Online meeting
- Notes
- Notification scope
- Recurrence scope

Actions:

- Save
- Send invite
- Send update
- Validate
- Edit
- Copy
- Move
- Delete
- Cancel meeting
- Delete occurrence
- Cancel series
- Preview recurrence
- Validate recurrence
- Open Scheduling Assistant
- Get reminder state
- Snooze reminder
- Dismiss reminder

Status copy:

- Draft saved for review.
- Event plan created. Host execution is required to persist it.
- Meeting update planned. Attendee notifications depend on host execution.
- Reminder change planned.
- The event changed elsewhere. Review the latest version before saving.

Validation copy:

- Add an end time after the start time.
- Choose a calendar that allows edits.
- Add at least one attendee before sending invitations.
- This recurrence pattern is not supported by the current host policy.
- Invite delivery is unavailable. You can save a draft only if policy allows it.
- Online meeting creation is unavailable from the host.

Empty copy:

- No event selected.
- Select an event or create a new one.
- No events in this range.
- Create an event for this time.

Destructive confirmation copy:

- Delete this appointment?
- Cancel this meeting and notify attendees?
- Delete only this occurrence?
- Cancel the entire series?
- This action affects multiple events. Confirm before continuing.

## 4. Scheduling Assistant Copy

Labels:

- Attendee availability
- Suggested times
- Required
- Optional
- Unknown availability
- Busy
- Tentative
- Free
- Out of office
- Working elsewhere
- Room search
- Capacity
- Features
- Location hint
- Time zones

Actions:

- Resolve attendees
- Check availability
- Find times
- Search rooms
- Use this time
- Back to editor

Empty copy:

- Add attendees to compare availability.
- No suggestions yet.
- Search for a room or enter a location manually.

Error and recovery copy:

- Free/busy is unavailable. You can still save the draft with conflicts visible.
- Room lookup is unavailable. Enter a manual location.
- Some attendees could not be resolved. Check the address or keep it as manual entry.
- This time has conflicts. Send anyway only after confirming.

Suggested slot reasons:

- Required attendees are free.
- Only optional attendees have conflicts.
- Room is available.
- Within work hours.
- Outside work hours.
- Fewest conflicts in this range.

## 5. Invitations Copy

Labels:

- Pending invitations
- Recent responses
- Organizer
- Proposed time
- Response message
- Send response
- Keep declined meetings visible

Actions:

- Accept
- Tentative
- Decline
- Propose new time
- Load invitation
- Retry response

Empty copy:

- No pending invitations.
- Recent invitations will appear here.

Error and recovery copy:

- Response could not be planned. Your previous RSVP is unchanged.
- Proposing a new time is not available for this invitation.
- Invitation details are unavailable. Refresh or check permissions.

## 6. Calendars, Sharing, And Subscriptions Copy

Labels:

- Calendar name
- Description
- Color
- Default reminders
- Sharing
- Delegates
- Shared calendars
- Subscriptions
- Internet calendar
- Access level
- Send on behalf
- Event outcome

Actions:

- Create calendar
- Save calendar
- Hide calendar
- Delete calendar
- Unsubscribe
- Get sharing state
- Share calendar
- Revoke access
- Add delegate
- Accept shared calendar
- Subscribe

Empty copy:

- No calendars are available.
- Create a calendar or accept a shared calendar to get started.
- No shares or delegates for this calendar.

Error and recovery copy:

- Calendar changes are unavailable because write access is missing.
- Sharing is disabled by policy for this calendar.
- This calendar cannot be deleted. You may be able to hide it instead.
- Subscription sync is unavailable. Try again when host sync is restored.

## 7. Settings, Categories, Reminders, Offline, Activity Copy

Settings labels:

- Default view
- Density
- Default calendar
- Primary time zone
- Secondary time zone
- Work days
- Work hours
- Default online meeting
- Agent activity link

Settings actions:

- Load preferences
- Save preferences
- Reset draft settings

Category actions:

- Load categories
- Save category
- Delete category
- Reassign events

Offline actions:

- Check offline state
- Retry sync
- Keep as draft

Reminder actions:

- Load reminders
- Snooze
- Dismiss

Activity actions:

- Load activity
- Filter activity

Empty copy:

- No categories yet.
- No reminders are scheduled for this selection.
- No recent activity is available.
- Offline information is not available from the host.

Error and recovery copy:

- Settings could not be saved. Review permissions and try again.
- Category deletion needs confirmation because events may be updated.
- Offline mutation policy blocks this action.
- Activity is unavailable with the current permission.

## 8. ARIA Labels And Tooltips

| Control | `aria-label` or tooltip |
| --- | --- |
| New event | Create a new calendar event |
| Today | Go to today |
| Previous | Show previous calendar range |
| Next | Show next calendar range |
| View switcher | Change calendar view |
| Search | Search calendar events |
| Refresh | Refresh calendars and events |
| Calendar checkbox | Toggle calendar visibility |
| Category checkbox | Toggle category filter |
| Event tile | Open event details |
| Empty slot | Create event at this time |
| Close panel | Close details panel |
| Save | Save event changes |
| Send invite | Send meeting invitation through host |
| Delete | Delete or cancel selected event |
| Resolve attendees | Resolve attendees through host contacts |
| Check availability | Check free and busy availability |
| Find times | Suggest meeting times |
| Search rooms | Search rooms through host resources |
| Accept | Accept invitation |
| Tentative | Tentatively accept invitation |
| Decline | Decline invitation |
| Propose new time | Propose a different meeting time |
| Save preferences | Save calendar preferences |
| Load activity | Load sanitized activity log |

## 9. Toast And Banner Copy

Success:

- Calendar refreshed.
- Event loaded.
- Draft validated.
- Event save planned.
- Meeting invite planned.
- Invitation response planned.
- Preferences saved.
- Sharing update planned.
- Offline state loaded.

Warning:

- Host execution is required to finish this action.
- Some details are hidden by permissions.
- You are viewing cached calendar data.
- Agent attribution is shown only when provided by the host.

Failure:

- Could not load calendar data.
- Could not plan this action.
- Permission denied for this operation.
- Required host effect is unavailable.
- The selected item no longer exists.
