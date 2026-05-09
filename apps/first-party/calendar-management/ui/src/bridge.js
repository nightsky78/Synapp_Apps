const hasHost = Boolean(window.__synapp);

const now = new Date('2026-05-08T09:00:00+02:00');

export const SAMPLE_CALENDARS = [
  {
    calendar_id: 'cal-primary',
    owner_id: 'usr-johannes',
    display_name: 'Johannes',
    description: 'Primary work calendar',
    kind: 'primary',
    role: 'owner',
    color: '#0078d4',
    visible: true,
    display_order: 1,
    time_zone: 'Europe/Berlin',
    capabilities: { can_create: true, can_update: true, can_delete: true, can_share: true, can_delegate: true, can_move_to: true },
    sync_state: 'current',
    last_sync_at: '2026-05-08T08:55:00+02:00',
  },
  {
    calendar_id: 'cal-team',
    owner_id: 'team-product',
    display_name: 'Product Team',
    description: 'Shared planning calendar',
    kind: 'shared',
    role: 'editor',
    color: '#498205',
    visible: true,
    display_order: 2,
    time_zone: 'Europe/Berlin',
    capabilities: { can_create: true, can_update: true, can_delete: false, can_share: false, can_delegate: false, can_move_to: true },
    sync_state: 'current',
    last_sync_at: '2026-05-08T08:50:00+02:00',
  },
  {
    calendar_id: 'cal-room',
    owner_id: 'room-berlin-4a',
    display_name: 'Berlin 4A Room',
    description: 'Resource calendar',
    kind: 'resource',
    role: 'viewer',
    color: '#b146c2',
    visible: false,
    display_order: 3,
    time_zone: 'Europe/Berlin',
    capabilities: { can_create: false, can_update: false, can_delete: false, can_share: false, can_delegate: false, can_move_to: false },
    sync_state: 'stale',
    last_sync_at: '2026-05-08T07:15:00+02:00',
  },
];

export const SAMPLE_CATEGORIES = [
  { category_id: 'cat-focus', name: 'Focus', color: '#8764b8', description: 'Deep work' },
  { category_id: 'cat-customer', name: 'Customer', color: '#c239b3', description: 'Customer meetings' },
  { category_id: 'cat-travel', name: 'Travel', color: '#ca5010', description: 'Travel and logistics' },
];

export const SAMPLE_EVENTS = [
  {
    event_id: 'evt-standup',
    calendar_id: 'cal-team',
    revision: 'r17',
    kind: 'meeting',
    status: 'confirmed',
    title: 'Product standup',
    start: '2026-05-08T09:30:00+02:00',
    end: '2026-05-08T10:00:00+02:00',
    all_day: false,
    time_zone: 'Europe/Berlin',
    location: 'Berlin 4A',
    availability: 'busy',
    privacy: 'normal',
    importance: 'normal',
    categories: ['cat-focus'],
    reminders: [{ reminder_id: 'rem-standup', method: 'host_notification', offset_minutes: 10, state: 'scheduled' }],
    recurrence: { frequency: 'weekly', interval: 1, days_of_week: ['MO', 'TU', 'WE', 'TH', 'FR'], end: { mode: 'never' }, time_zone: 'Europe/Berlin' },
    organizer: { display_name: 'Mara Lee', email: 'mara@example.com' },
    attendees: [
      { display_name: 'Johannes', email: 'johannes@example.com', kind: 'required', response_status: 'accepted' },
      { display_name: 'Anika Rao', email: 'anika@example.com', kind: 'required', response_status: 'tentative' },
    ],
    online_meeting: { requested: true, state: 'created', provider: 'host' },
    created_at: '2026-04-20T10:00:00+02:00',
    updated_at: '2026-05-07T15:20:00+02:00',
  },
  {
    event_id: 'evt-focus',
    calendar_id: 'cal-primary',
    revision: 'r2',
    kind: 'appointment',
    status: 'confirmed',
    title: 'Focus block',
    start: '2026-05-08T10:30:00+02:00',
    end: '2026-05-08T12:00:00+02:00',
    all_day: false,
    time_zone: 'Europe/Berlin',
    availability: 'busy',
    privacy: 'private',
    importance: 'normal',
    categories: ['cat-focus'],
    reminders: [],
    attendees: [],
    created_by_agent_id: 'agent-planner-7',
    created_at: '2026-05-06T13:00:00+02:00',
    updated_at: '2026-05-06T13:00:00+02:00',
  },
  {
    event_id: 'evt-customer',
    calendar_id: 'cal-primary',
    revision: 'r4',
    kind: 'meeting',
    status: 'tentative',
    title: 'Customer onboarding review',
    start: '2026-05-08T14:00:00+02:00',
    end: '2026-05-08T15:00:00+02:00',
    all_day: false,
    time_zone: 'Europe/Berlin',
    location: 'Teams',
    availability: 'tentative',
    privacy: 'normal',
    importance: 'high',
    categories: ['cat-customer'],
    reminders: [{ reminder_id: 'rem-customer', method: 'host_notification', offset_minutes: 15, state: 'planned' }],
    organizer: { display_name: 'Nina Patel', email: 'nina@example.com' },
    attendees: [
      { display_name: 'Johannes', email: 'johannes@example.com', kind: 'required', response_status: 'tentative' },
      { display_name: 'Sales Ops', email: 'sales@example.com', kind: 'optional', response_status: 'none' },
    ],
    online_meeting: { requested: true, state: 'requested', provider: 'host' },
    created_at: '2026-05-04T09:00:00+02:00',
    updated_at: '2026-05-07T16:10:00+02:00',
  },
  {
    event_id: 'evt-travel',
    calendar_id: 'cal-primary',
    revision: 'r1',
    kind: 'out_of_office',
    status: 'confirmed',
    title: 'Travel to Munich',
    start: '2026-05-09',
    end: '2026-05-10',
    all_day: true,
    time_zone: 'Europe/Berlin',
    availability: 'out_of_office',
    privacy: 'normal',
    importance: 'normal',
    categories: ['cat-travel'],
    reminders: [],
    attendees: [],
    created_at: '2026-05-01T09:00:00+02:00',
    updated_at: '2026-05-01T09:00:00+02:00',
  },
];

export const SAMPLE_INVITATIONS = [
  {
    invitation_id: 'inv-design-review',
    title: 'Design review',
    organizer: 'Mara Lee',
    start: '2026-05-08T16:00:00+02:00',
    end: '2026-05-08T16:45:00+02:00',
    location: 'Berlin 2B',
    response_status: 'none',
    conflicts: ['Focus block moved earlier; no hard conflict.'],
  },
  {
    invitation_id: 'inv-platform-sync',
    title: 'Platform sync',
    organizer: 'Arun Mehta',
    start: '2026-05-09T11:00:00+02:00',
    end: '2026-05-09T12:00:00+02:00',
    location: 'Online',
    response_status: 'tentative',
    conflicts: ['Outside configured work days.'],
  },
];

export const SAMPLE_ACTIVITIES = [
  { activity_id: 'act-1', label: 'Created Focus block', actor: 'agent-planner-7', target: 'Focus block', at: '2026-05-06T13:00:00+02:00', risk: 'low' },
  { activity_id: 'act-2', label: 'Updated Product standup attendees', actor: 'Mara Lee', target: 'Product standup', at: '2026-05-07T15:20:00+02:00', risk: 'medium' },
];

const STUB_CONTEXT = {
  user_id: 'usr-johannes',
  permission: 'write',
  capabilities: ['calendar:read', 'calendar:write', 'calendar:invite', 'calendar:respond', 'calendar:share', 'calendar:settings', 'calendar:audit'],
  available_effects: [
    'CalendarStoreRead',
    'CalendarStoreWrite',
    'CalendarStoreDelete',
    'CalendarInviteSend',
    'CalendarInviteRespond',
    'FreeBusyLookup',
    'RoomResourceLookup',
    'ContactLookup',
    'ConferenceLinkCreate',
    'ReminderSchedule',
    'CalendarShareManage',
    'SubscriptionSync',
    'OfflineCacheRead',
    'AuditRead',
    'AuditWrite',
    'NotifyUser',
  ],
  agent_id: null,
  request_id: 'dev-calendar-request',
  locale: 'en-US',
  time_zone: 'Europe/Berlin',
};

function ok(operation, data = {}, hostEffects = []) {
  return { ok: { operation, status: 'accepted', data, host_effects: hostEffects, warnings: [] } };
}

const STUB_RESPONSES = {
  get_calendar_capabilities: () => ok('get_calendar_capabilities', {
    capabilities: STUB_CONTEXT.capabilities,
    policy: { allow_external_attendees: true, allow_no_end_recurrence: false, offline_mutation_policy: 'queued' },
    host_effects: STUB_CONTEXT.available_effects,
    default_view_state: { view: 'work_week', mode: 'overlay', date_anchor: '2026-05-08' },
    warnings: [],
  }),
  list_calendars: () => ok('list_calendars', { calendars: SAMPLE_CALENDARS, visibility_state: {}, sync_state: { state: 'current' }, needs_host_refresh: false }),
  save_calendar: (payload) => ok('save_calendar', { calendar: payload.calendar || SAMPLE_CALENDARS[0], created: Boolean(payload.create), updated: !payload.create }, [{ effect: 'CalendarStoreWrite' }]),
  delete_calendar: (payload) => ok('delete_calendar', { calendar_id: payload.calendar_id, deletion_mode: payload.mode || 'hide', event_outcome: 'events_preserved', requires_confirmation: false }, [{ effect: 'CalendarStoreDelete' }]),
  set_calendar_view_state: (payload) => ok('set_calendar_view_state', { view_state: payload.view_state, normalized: true }, [{ effect: 'CalendarStoreWrite' }]),
  list_events: (payload) => ok('list_events', { events: SAMPLE_EVENTS, range: payload.range || {}, projection_applied: 'full', sync_state: { state: 'current' }, warnings: [] }),
  get_event: (payload) => ok('get_event', { event: SAMPLE_EVENTS.find((event) => event.event_id === payload.event_id) || SAMPLE_EVENTS[0], projection_applied: 'full', allowed_actions: ['edit', 'copy', 'move', 'delete'], warnings: [] }),
  search_events: (payload) => ok('search_events', { results: SAMPLE_EVENTS.filter((event) => event.title.toLowerCase().includes((payload.query || '').toLowerCase())), facets: {}, pagination: {}, projection_applied: 'full' }),
  validate_event_draft: (payload) => ok('validate_event_draft', { valid: Boolean(payload.draft?.start && payload.draft?.end), normalized_draft: payload.draft, field_errors: [], warnings: [], required_actions: [] }),
  create_event: (payload) => ok('create_event', { event_preview: { ...payload.draft, event_id: `evt-${Date.now()}` }, persistence_status: 'planned', notification_state: payload.send_invites ? 'planned' : 'none', requires_host_execution: true }, [{ effect: 'CalendarStoreWrite' }]),
  update_event: (payload) => ok('update_event', { event_preview: { ...SAMPLE_EVENTS[0], ...payload.patch }, attendee_delta: {}, notification_state: { state: payload.notification_scope || 'host_default' }, conflict_state: { state: 'none' }, requires_host_execution: true }, [{ effect: 'CalendarStoreWrite' }]),
  delete_event: (payload) => ok('delete_event', { target: payload.target, scope: payload.scope, deletion_state: { state: 'planned' }, notification_state: { state: payload.mode === 'cancel' ? 'planned' : 'none' }, requires_host_execution: true }, [{ effect: 'CalendarStoreDelete' }]),
  copy_event: (payload) => ok('copy_event', { source_event_id: payload.event_id, new_event_preview: { ...SAMPLE_EVENTS[0], calendar_id: payload.destination_calendar_id }, requires_host_execution: true }, [{ effect: 'CalendarStoreWrite' }]),
  move_event: (payload) => ok('move_event', { event_preview: { ...SAMPLE_EVENTS[0], calendar_id: payload.destination_calendar_id }, source_calendar_id: 'cal-primary', destination_calendar_id: payload.destination_calendar_id, notification_state: {} }, [{ effect: 'CalendarStoreWrite' }]),
  validate_recurrence: (payload) => ok('validate_recurrence', { valid: true, normalized_recurrence: payload.recurrence, field_errors: [], warnings: [] }),
  preview_recurrence: () => ok('preview_recurrence', { occurrences: ['2026-05-15T09:30:00+02:00', '2026-05-22T09:30:00+02:00', '2026-05-29T09:30:00+02:00'], truncated: false, warnings: [] }),
  plan_free_busy_lookup: () => ok('plan_free_busy_lookup', { lookup_request: { range: { start: '2026-05-08T09:00:00+02:00', end: '2026-05-08T17:00:00+02:00' } }, requires_host_execution: true }, [{ effect: 'FreeBusyLookup' }]),
  suggest_meeting_times: () => ok('suggest_meeting_times', { suggestions: SAMPLE_SUGGESTIONS, conflicts: [], warnings: [] }),
  search_rooms: () => ok('search_rooms', { room_lookup_request: { query: 'Berlin', capacity: 8 }, requires_host_execution: true }, [{ effect: 'RoomResourceLookup' }]),
  resolve_attendees: (payload) => ok('resolve_attendees', { normalized_attendees: payload.attendees || [], unresolved: [], requires_host_execution: true }, [{ effect: 'ContactLookup' }]),
  list_invitations: () => ok('list_invitations', { invitations: SAMPLE_INVITATIONS, pagination: {}, sync_state: { state: 'current' } }),
  get_invitation: (payload) => ok('get_invitation', { invitation: SAMPLE_INVITATIONS.find((item) => item.invitation_id === payload.invitation_id) || SAMPLE_INVITATIONS[0], allowed_responses: ['accept', 'tentative', 'decline', 'propose_new_time'], conflicts: ['No hard conflict'], warnings: [] }),
  respond_to_invitation: (payload) => ok('respond_to_invitation', { invitation_id: payload.invitation_id, response_state: { response: payload.response, state: 'planned' }, calendar_update_state: { state: 'planned' }, requires_host_execution: true }, [{ effect: 'CalendarInviteRespond' }]),
  propose_new_time: (payload) => ok('propose_new_time', { proposal_state: { state: 'planned' }, proposed_time: payload.proposed_time, requires_host_execution: true }, [{ effect: 'CalendarInviteRespond' }]),
  list_categories: () => ok('list_categories', { categories: SAMPLE_CATEGORIES }),
  save_category: (payload) => ok('save_category', { category: payload.category || SAMPLE_CATEGORIES[0], created: false, updated: true }, [{ effect: 'CalendarStoreWrite' }]),
  delete_category: (payload) => ok('delete_category', { category_id: payload.category_id, event_update_plan: { state: 'remove_from_events' }, requires_host_execution: true }, [{ effect: 'CalendarStoreWrite' }]),
  bulk_update_events: () => ok('bulk_update_events', { affected_count: 2, operation_state: { state: 'planned' }, requires_host_execution: true }, [{ effect: 'CalendarStoreWrite' }]),
  get_sharing_state: () => ok('get_sharing_state', { shares: [{ display_name: 'Mara Lee', access_level: 'edit', state: 'active' }], delegates: [{ display_name: 'Assistant Desk', access_level: 'delegate', send_on_behalf: true }], allowed_actions: ['grant', 'revoke', 'delegate'] }),
  update_sharing: () => ok('update_sharing', { sharing_state: { state: 'planned' }, provisioning_state: { state: 'requires_host_execution' }, requires_host_execution: true }, [{ effect: 'CalendarShareManage' }]),
  accept_shared_calendar: () => ok('accept_shared_calendar', { calendar: SAMPLE_CALENDARS[1], provisioning_state: { state: 'planned' } }, [{ effect: 'CalendarShareManage' }]),
  subscribe_calendar: () => ok('subscribe_calendar', { subscription_state: { state: 'planned' }, calendar_preview: { ...SAMPLE_CALENDARS[2], kind: 'subscription' } }, [{ effect: 'SubscriptionSync' }]),
  get_reminder_state: () => ok('get_reminder_state', { reminders: SAMPLE_EVENTS.flatMap((event) => event.reminders), delivery_state: { state: 'scheduled' } }),
  snooze_or_dismiss_reminder: (payload) => ok('snooze_or_dismiss_reminder', { reminder_state: { reminder_id: payload.reminder_id, action: payload.action, state: payload.action === 'dismiss' ? 'dismissed' : 'snoozed' }, requires_host_execution: true }, [{ effect: 'ReminderSchedule' }]),
  get_user_preferences: () => ok('get_user_preferences', { preferences: SAMPLE_PREFERENCES, defaults: {}, needs_host_refresh: false }),
  save_user_preferences: (payload) => ok('save_user_preferences', { preferences: payload.preferences || SAMPLE_PREFERENCES, updated: true }, [{ effect: 'CalendarStoreWrite' }]),
  get_offline_state: () => ok('get_offline_state', { offline_state: { state: 'online_cached', cache_available: true }, last_sync_at: '2026-05-08T08:55:00+02:00', mutation_policy: 'queued' }),
  get_activity_log: () => ok('get_activity_log', { activities: SAMPLE_ACTIVITIES, pagination: {} }),
};

export const SAMPLE_SUGGESTIONS = [
  { start: '2026-05-08T15:00:00+02:00', end: '2026-05-08T15:30:00+02:00', score: 92, reasons: ['Required attendees are free', 'Room is available', 'Within work hours'] },
  { start: '2026-05-08T16:00:00+02:00', end: '2026-05-08T16:30:00+02:00', score: 78, reasons: ['Only optional attendees have conflicts', 'Within work hours'] },
  { start: '2026-05-09T09:00:00+02:00', end: '2026-05-09T09:30:00+02:00', score: 61, reasons: ['Required attendees are free', 'Outside work hours'] },
];

export const SAMPLE_PREFERENCES = {
  default_view: 'work_week',
  density: 'compact',
  default_calendar_id: 'cal-primary',
  time_zone: 'Europe/Berlin',
  secondary_time_zone: 'UTC',
  work_days: ['MO', 'TU', 'WE', 'TH', 'FR'],
  work_hours: { start: '09:00', end: '17:00' },
  default_reminders: [{ method: 'host_notification', offset_minutes: 15, state: 'planned' }],
  default_online_meeting: true,
  overlay_mode: 'overlay',
  category_sort: 'name',
  agent_activity_link: 'activity',
};

export function getContext() {
  if (!hasHost) return STUB_CONTEXT;
  return window.__synapp.getContext();
}

export async function invoke(operation, payload = {}) {
  if (hasHost) {
    const response = await window.__synapp.invoke(operation, { context: getContext(), ...payload });
    if (response.err) {
      const error = new Error(response.err.message || 'Calendar operation failed');
      error.code = response.err.code;
      error.details = response.err.details;
      throw error;
    }
    return response.ok.data;
  }

  await new Promise((resolve) => setTimeout(resolve, 90));
  const responseFactory = STUB_RESPONSES[operation];
  if (!responseFactory) {
    const error = new Error(`No development response for ${operation}`);
    error.code = 'NotImplemented';
    throw error;
  }
  const response = responseFactory({ context: getContext(), ...payload });
  if (response.err) {
    const error = new Error(response.err.message || 'Calendar operation failed');
    error.code = response.err.code;
    throw error;
  }
  return response.ok.data;
}

export function getToday() {
  return now;
}
