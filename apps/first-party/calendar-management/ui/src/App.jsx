import React, { useEffect, useMemo, useState } from 'react';
import {
  SAMPLE_ACTIVITIES,
  SAMPLE_CALENDARS,
  SAMPLE_CATEGORIES,
  SAMPLE_EVENTS,
  SAMPLE_INVITATIONS,
  SAMPLE_PREFERENCES,
  SAMPLE_SUGGESTIONS,
  getContext,
  getToday,
  invoke,
} from './bridge.js';

const VIEWS = ['day', 'work_week', 'week', 'month', 'agenda'];
const VIEW_LABELS = { day: 'Day', work_week: 'Work week', week: 'Week', month: 'Month', agenda: 'Agenda' };
const HOURS = Array.from({ length: 11 }, (_, index) => index + 7);
const WEEK_DAYS = ['Mon 4', 'Tue 5', 'Wed 6', 'Thu 7', 'Fri 8', 'Sat 9', 'Sun 10'];
const WORK_DAYS = WEEK_DAYS.slice(0, 5);

function formatTime(value) {
  if (!value) return '';
  if (!value.includes('T')) return value;
  return new Intl.DateTimeFormat('en-US', { hour: '2-digit', minute: '2-digit' }).format(new Date(value));
}

function getHour(value) {
  if (!value || !value.includes('T')) return 8;
  return new Date(value).getHours() + new Date(value).getMinutes() / 60;
}

function makeDraft(selectedCalendarId) {
  return {
    calendar_id: selectedCalendarId,
    kind: 'appointment',
    title: '',
    start: '2026-05-08T15:00:00+02:00',
    end: '2026-05-08T15:30:00+02:00',
    all_day: false,
    time_zone: 'Europe/Berlin',
    location: '',
    availability: 'busy',
    privacy: 'normal',
    importance: 'normal',
    categories: [],
    reminders: [{ method: 'host_notification', offset_minutes: 15, state: 'planned' }],
    attendees: [],
    online_meeting: { requested: false, state: 'not_requested' },
    body_text: '',
  };
}

function App() {
  const [context] = useState(() => getContext());
  const [view, setView] = useState('work_week');
  const [activePanel, setActivePanel] = useState('detail');
  const [dateAnchor, setDateAnchor] = useState('2026-05-08');
  const [calendars, setCalendars] = useState(SAMPLE_CALENDARS);
  const [events, setEvents] = useState(SAMPLE_EVENTS);
  const [categories, setCategories] = useState(SAMPLE_CATEGORIES);
  const [invitations, setInvitations] = useState(SAMPLE_INVITATIONS);
  const [activities, setActivities] = useState(SAMPLE_ACTIVITIES);
  const [preferences, setPreferences] = useState(SAMPLE_PREFERENCES);
  const [selectedEventId, setSelectedEventId] = useState('evt-standup');
  const [draft, setDraft] = useState(() => makeDraft('cal-primary'));
  const [searchQuery, setSearchQuery] = useState('');
  const [status, setStatus] = useState({ loading: false, message: 'Calendar ready.', tone: 'neutral' });
  const [offlineState, setOfflineState] = useState(null);
  const [suggestions, setSuggestions] = useState(SAMPLE_SUGGESTIONS);
  const [selectedIds, setSelectedIds] = useState([]);

  const visibleCalendarIds = useMemo(() => calendars.filter((calendar) => calendar.visible).map((calendar) => calendar.calendar_id), [calendars]);
  const selectedEvent = events.find((event) => event.event_id === selectedEventId) || null;
  const displayedEvents = useMemo(() => {
    const visibleEvents = events.filter((event) => visibleCalendarIds.includes(event.calendar_id));
    if (!searchQuery) return visibleEvents;
    return visibleEvents.filter((event) => event.title.toLowerCase().includes(searchQuery.toLowerCase()));
  }, [events, searchQuery, visibleCalendarIds]);

  async function runOperation(operation, payload = {}, successMessage = 'Operation planned.') {
    setStatus({ loading: true, message: successMessage, tone: 'neutral' });
    try {
      const data = await invoke(operation, payload);
      if (operation === 'list_calendars') setCalendars(data.calendars || calendars);
      if (operation === 'list_events') setEvents(data.events || events);
      if (operation === 'search_events') setEvents(data.results?.length ? data.results : SAMPLE_EVENTS);
      if (operation === 'list_categories') setCategories(data.categories || categories);
      if (operation === 'list_invitations') setInvitations(data.invitations || invitations);
      if (operation === 'get_user_preferences') setPreferences(data.preferences || preferences);
      if (operation === 'get_offline_state') setOfflineState(data);
      if (operation === 'get_activity_log') setActivities(data.activities || activities);
      if (operation === 'suggest_meeting_times') setSuggestions(data.suggestions || suggestions);
      setStatus({ loading: false, message: successMessage, tone: 'success' });
      return data;
    } catch (error) {
      setStatus({ loading: false, message: `${error.code || 'Error'}: ${error.message}`, tone: 'error' });
      return null;
    }
  }

  useEffect(() => {
    runOperation('get_calendar_capabilities', {}, 'Capabilities loaded.');
    runOperation('list_calendars', {}, 'Calendars loaded.');
    runOperation('list_events', { range: { start: '2026-05-08T00:00:00+02:00', end: '2026-05-11T00:00:00+02:00', time_zone: 'Europe/Berlin' }, calendar_ids: visibleCalendarIds }, 'Events loaded.');
    runOperation('list_categories', {}, 'Categories loaded.');
  }, []);

  function changeView(nextView) {
    setView(nextView);
    runOperation('set_calendar_view_state', {
      view_state: { selected_calendar_ids: visibleCalendarIds, hidden_calendar_ids: calendars.filter((calendar) => !calendar.visible).map((calendar) => calendar.calendar_id), display_order: calendars.map((calendar) => calendar.calendar_id), mode: preferences.overlay_mode, view: nextView, date_anchor: dateAnchor },
    }, `${VIEW_LABELS[nextView]} view selected.`);
  }

  function toggleCalendar(calendarId) {
    const updated = calendars.map((calendar) => calendar.calendar_id === calendarId ? { ...calendar, visible: !calendar.visible } : calendar);
    setCalendars(updated);
    runOperation('set_calendar_view_state', {
      view_state: { selected_calendar_ids: updated.filter((calendar) => calendar.visible).map((calendar) => calendar.calendar_id), hidden_calendar_ids: updated.filter((calendar) => !calendar.visible).map((calendar) => calendar.calendar_id), display_order: updated.map((calendar) => calendar.calendar_id), mode: preferences.overlay_mode, view, date_anchor: dateAnchor },
    }, 'Calendar visibility updated.');
  }

  async function openEvent(eventId) {
    setSelectedEventId(eventId);
    setActivePanel('detail');
    await runOperation('get_event', { event_id: eventId, projection: 'full' }, 'Event loaded.');
  }

  function startNewEvent() {
    setDraft(makeDraft(visibleCalendarIds[0] || 'cal-primary'));
    setSelectedEventId(null);
    setActivePanel('edit');
  }

  async function saveDraft(sendInvites = false) {
    const operation = selectedEvent ? 'update_event' : 'create_event';
    const payload = selectedEvent
      ? { event_id: selectedEvent.event_id, patch: draft, scope: 'single', notification_scope: sendInvites ? 'all_attendees' : 'none', revision: selectedEvent.revision }
      : { draft, send_invites: sendInvites, client_operation_id: `ui-${Date.now()}` };
    const data = await runOperation(operation, payload, sendInvites ? 'Meeting invite planned.' : 'Event save planned.');
    if (data?.event_preview) {
      const preview = data.event_preview;
      setEvents((current) => selectedEvent ? current.map((event) => event.event_id === selectedEvent.event_id ? { ...event, ...preview } : event) : [...current, { ...preview, event_id: preview.event_id || `evt-${Date.now()}`, status: 'draft', revision: 'ui-draft', created_at: new Date().toISOString(), updated_at: new Date().toISOString() }]);
    }
  }

  return (
    <div className="calendar-app">
      <CommandBar
        context={context}
        dateAnchor={dateAnchor}
        setDateAnchor={setDateAnchor}
        view={view}
        changeView={changeView}
        searchQuery={searchQuery}
        setSearchQuery={setSearchQuery}
        startNewEvent={startNewEvent}
        status={status}
        openPanel={setActivePanel}
        refresh={() => runOperation('list_events', { range: { start: `${dateAnchor}T00:00:00+02:00`, end: '2026-05-11T00:00:00+02:00', time_zone: 'Europe/Berlin' }, calendar_ids: visibleCalendarIds, refresh: true }, 'Calendar refreshed.')}
        search={() => runOperation('search_events', { query: searchQuery, projection: 'full' }, 'Search complete.')}
      />

      <div className="workspace">
        <CalendarSidebar
          calendars={calendars}
          categories={categories}
          dateAnchor={dateAnchor}
          toggleCalendar={toggleCalendar}
          openPanel={setActivePanel}
          runOperation={runOperation}
        />

        <main className="calendar-main" aria-label="Calendar workspace">
          <StatusBanner status={status} offlineState={offlineState} openPanel={setActivePanel} />
          {selectedIds.length > 0 && <BulkBar selectedIds={selectedIds} clear={() => setSelectedIds([])} runOperation={runOperation} />}
          <CalendarCanvas
            view={view}
            events={displayedEvents}
            calendars={calendars}
            categories={categories}
            openEvent={openEvent}
            startNewEvent={startNewEvent}
            selectedIds={selectedIds}
            setSelectedIds={setSelectedIds}
          />
        </main>

        <aside className="right-panel" aria-label="Calendar details panel">
          <PanelHeader activePanel={activePanel} openPanel={setActivePanel} />
          {activePanel === 'detail' && <EventDetail event={selectedEvent} calendars={calendars} categories={categories} edit={() => { setDraft(selectedEvent || makeDraft('cal-primary')); setActivePanel('edit'); }} runOperation={runOperation} />}
          {activePanel === 'edit' && <EventEditor draft={draft} setDraft={setDraft} calendars={calendars} categories={categories} saveDraft={saveDraft} runOperation={runOperation} openScheduling={() => setActivePanel('scheduling')} />}
          {activePanel === 'scheduling' && <SchedulingAssistant draft={draft} setDraft={setDraft} suggestions={suggestions} runOperation={runOperation} />}
          {activePanel === 'invitations' && <InvitationsPanel invitations={invitations} runOperation={runOperation} />}
          {activePanel === 'calendars' && <CalendarsPanel calendars={calendars} setCalendars={setCalendars} runOperation={runOperation} />}
          {activePanel === 'settings' && <SettingsPanel preferences={preferences} setPreferences={setPreferences} categories={categories} setCategories={setCategories} runOperation={runOperation} />}
          {activePanel === 'activity' && <ActivityPanel activities={activities} offlineState={offlineState} runOperation={runOperation} />}
        </aside>
      </div>
    </div>
  );
}

function CommandBar({ context, dateAnchor, setDateAnchor, view, changeView, searchQuery, setSearchQuery, startNewEvent, status, openPanel, refresh, search }) {
  return (
    <header className="command-bar">
      <div className="brand-block">
        <strong>Calendar Management</strong>
        <span className="permission-pill">{context.permission}</span>
      </div>
      <button className="primary-button" onClick={startNewEvent} aria-label="Create a new calendar event">+ New event</button>
      <button onClick={() => setDateAnchor('2026-05-08')} aria-label="Go to today">Today</button>
      <button onClick={() => setDateAnchor('2026-05-01')} aria-label="Show previous calendar range">Previous</button>
      <button onClick={() => setDateAnchor('2026-05-15')} aria-label="Show next calendar range">Next</button>
      <label className="date-field">
        <span>Go to date</span>
        <input type="date" value={dateAnchor} onChange={(event) => setDateAnchor(event.target.value)} />
      </label>
      <div className="segmented" aria-label="Change calendar view">
        {VIEWS.map((item) => <button key={item} aria-pressed={view === item} className={view === item ? 'is-active' : ''} onClick={() => changeView(item)}>{VIEW_LABELS[item]}</button>)}
      </div>
      <div className="search-box">
        <input value={searchQuery} onChange={(event) => setSearchQuery(event.target.value)} onKeyDown={(event) => { if (event.key === 'Enter') search(); }} placeholder="Search calendar" aria-label="Search calendar events" />
        <button onClick={search}>Search</button>
      </div>
      <button onClick={refresh} aria-label="Refresh calendars and events">Refresh</button>
      <button onClick={() => openPanel('invitations')}>Invitations</button>
      <button onClick={() => openPanel('calendars')}>Calendars</button>
      <button onClick={() => openPanel('settings')}>Settings</button>
      <button onClick={() => openPanel('activity')}>Activity</button>
      <span className={`sync-dot sync-dot--${status.tone}`} title={status.message} />
    </header>
  );
}

function CalendarSidebar({ calendars, categories, dateAnchor, toggleCalendar, openPanel, runOperation }) {
  return (
    <nav className="calendar-sidebar" aria-label="Calendar navigation">
      <section className="mini-month" aria-label="Mini month picker">
        <div className="mini-month__header">May 2026</div>
        <div className="mini-month__grid">
          {Array.from({ length: 35 }, (_, index) => {
            const day = index - 3;
            const label = day > 0 && day <= 31 ? day : '';
            return <button key={index} className={dateAnchor.endsWith(String(label).padStart(2, '0')) ? 'is-selected' : ''}>{label}</button>;
          })}
        </div>
      </section>
      <SidebarSection title="My calendars">
        {calendars.map((calendar) => (
          <label className="calendar-toggle" key={calendar.calendar_id}>
            <input type="checkbox" checked={calendar.visible} onChange={() => toggleCalendar(calendar.calendar_id)} aria-label={`Toggle ${calendar.display_name} visibility`} />
            <span className="swatch" style={{ backgroundColor: calendar.color }} />
            <span>{calendar.display_name}</span>
            <small>{calendar.role}</small>
          </label>
        ))}
      </SidebarSection>
      <SidebarSection title="Categories">
        {categories.map((category) => <button className="category-filter" key={category.category_id} onClick={() => runOperation('list_events', { filters: { categories: [category.category_id] } }, `${category.name} filter applied.`)}><span className="swatch" style={{ backgroundColor: category.color }} />{category.name}</button>)}
      </SidebarSection>
      <SidebarSection title="Shortcuts">
        <button onClick={() => openPanel('scheduling')}>Scheduling Assistant</button>
        <button onClick={() => openPanel('invitations')}>Pending invitations</button>
        <button onClick={() => openPanel('activity')}>Offline and activity</button>
        <button onClick={() => runOperation('get_offline_state', {}, 'Offline state loaded.')}>Check offline state</button>
      </SidebarSection>
    </nav>
  );
}

function SidebarSection({ title, children }) {
  return <section className="sidebar-section"><h2>{title}</h2>{children}</section>;
}

function StatusBanner({ status, offlineState, openPanel }) {
  return (
    <div className={`status-banner status-banner--${status.tone}`} role="status" aria-live="polite">
      <span>{status.loading ? 'Working...' : status.message}</span>
      {offlineState && <button onClick={() => openPanel('activity')}>Offline: {offlineState.mutation_policy}</button>}
    </div>
  );
}

function CalendarCanvas({ view, events, calendars, categories, openEvent, startNewEvent, selectedIds, setSelectedIds }) {
  if (view === 'agenda') {
    return <AgendaView events={events} calendars={calendars} categories={categories} openEvent={openEvent} selectedIds={selectedIds} setSelectedIds={setSelectedIds} />;
  }
  if (view === 'month') {
    return <MonthView events={events} calendars={calendars} openEvent={openEvent} startNewEvent={startNewEvent} />;
  }
  const days = view === 'day' ? ['Fri 8'] : view === 'work_week' ? WORK_DAYS : WEEK_DAYS;
  return (
    <section className="time-grid" style={{ '--day-count': days.length }} aria-label={`${VIEW_LABELS[view]} calendar grid`}>
      <div className="time-grid__header"><span />{days.map((day) => <strong key={day}>{day}</strong>)}</div>
      <div className="all-day-row"><span>All day</span>{days.map((day) => <div key={day}>{events.filter((event) => event.all_day).map((event) => <EventChip key={event.event_id} event={event} calendars={calendars} categories={categories} openEvent={openEvent} compact />)}</div>)}</div>
      <div className="time-grid__body">
        <div className="time-labels">{HOURS.map((hour) => <span key={hour}>{String(hour).padStart(2, '0')}:00</span>)}</div>
        {days.map((day, dayIndex) => <DayColumn key={day} dayIndex={dayIndex} events={events} calendars={calendars} categories={categories} openEvent={openEvent} startNewEvent={startNewEvent} />)}
      </div>
    </section>
  );
}

function DayColumn({ dayIndex, events, calendars, categories, openEvent, startNewEvent }) {
  const dayEvents = events.filter((event) => !event.all_day && (dayIndex === 4 || dayIndex === 0));
  return (
    <div className="day-column">
      {HOURS.map((hour) => <button key={hour} className="slot" onClick={startNewEvent} aria-label={`Create event at ${hour}:00`} />)}
      {dayEvents.map((event) => <EventBlock key={event.event_id} event={event} calendars={calendars} categories={categories} openEvent={openEvent} />)}
      {dayIndex === 4 && <div className="current-time" style={{ top: `${((getToday().getHours() - 7) / HOURS.length) * 100}%` }} />}
    </div>
  );
}

function EventBlock({ event, calendars, categories, openEvent }) {
  const calendar = calendars.find((item) => item.calendar_id === event.calendar_id);
  const top = Math.max(0, (getHour(event.start) - 7) * 58);
  const height = Math.max(34, (getHour(event.end) - getHour(event.start)) * 58);
  return (
    <button className={`event-block event-block--${event.privacy}`} style={{ top, height, borderLeftColor: calendar?.color }} onClick={() => openEvent(event.event_id)} aria-label={`Open event details for ${event.title}`}>
      <strong>{event.privacy === 'private' ? 'Private event' : event.title}</strong>
      <span>{formatTime(event.start)} - {formatTime(event.end)}</span>
      <EventMeta event={event} categories={categories} />
    </button>
  );
}

function EventChip({ event, calendars, categories, openEvent, compact = false }) {
  const calendar = calendars.find((item) => item.calendar_id === event.calendar_id);
  return (
    <button className="event-chip" style={{ borderLeftColor: calendar?.color }} onClick={() => openEvent(event.event_id)}>
      <span>{compact ? event.title : `${formatTime(event.start)} ${event.title}`}</span>
      <EventMeta event={event} categories={categories} />
    </button>
  );
}

function EventMeta({ event, categories }) {
  const category = categories.find((item) => event.categories?.includes(item.category_id));
  return <span className="event-meta">{event.kind === 'meeting' ? 'Meeting' : event.kind === 'out_of_office' ? 'OOF' : 'Appointment'}{event.recurrence ? ' | Repeats' : ''}{event.reminders?.length ? ' | Reminder' : ''}{event.created_by_agent_id ? ' | Agent' : ''}{category ? ` | ${category.name}` : ''}</span>;
}

function AgendaView({ events, calendars, categories, openEvent, selectedIds, setSelectedIds }) {
  if (events.length === 0) return <EmptyState title="No events in this range." action="Create event" />;
  return (
    <section className="agenda-view" aria-label="Agenda events">
      <h1>Agenda</h1>
      {events.map((event) => (
        <div className="agenda-row" key={event.event_id}>
          <input type="checkbox" checked={selectedIds.includes(event.event_id)} onChange={(change) => setSelectedIds(change.target.checked ? [...selectedIds, event.event_id] : selectedIds.filter((id) => id !== event.event_id))} aria-label={`Select ${event.title}`} />
          <EventChip event={event} calendars={calendars} categories={categories} openEvent={openEvent} />
          <span>{event.location || 'No location'}</span>
          <span>{event.availability}</span>
        </div>
      ))}
    </section>
  );
}

function MonthView({ events, calendars, openEvent, startNewEvent }) {
  return (
    <section className="month-view" aria-label="Month calendar">
      {Array.from({ length: 35 }, (_, index) => {
        const day = index - 3;
        const dayEvents = day === 8 ? events.slice(0, 3) : day === 9 ? events.filter((event) => event.all_day) : [];
        return (
          <button className="month-cell" key={index} onDoubleClick={startNewEvent}>
            <strong>{day > 0 && day <= 31 ? day : ''}</strong>
            {dayEvents.map((event) => <span key={event.event_id} style={{ borderLeftColor: calendars.find((calendar) => calendar.calendar_id === event.calendar_id)?.color }} onClick={(click) => { click.stopPropagation(); openEvent(event.event_id); }}>{event.title}</span>)}
          </button>
        );
      })}
    </section>
  );
}

function EmptyState({ title, action }) {
  return <div className="empty-state"><strong>{title}</strong>{action && <button className="primary-button">{action}</button>}</div>;
}

function BulkBar({ selectedIds, clear, runOperation }) {
  return (
    <div className="bulk-bar">
      <span>{selectedIds.length} selected</span>
      <button onClick={() => runOperation('bulk_update_events', { selection: { event_ids: selectedIds }, operation: { type: 'category_add', category_id: 'cat-focus' }, confirmation: { confirmed: true } }, 'Bulk update planned.')}>Categorize</button>
      <button onClick={() => runOperation('bulk_update_events', { selection: { event_ids: selectedIds }, operation: { type: 'delete' }, confirmation: { confirmed: true, acknowledged_risks: ['delete_events'] } }, 'Bulk delete planned.')}>Delete</button>
      <button onClick={clear}>Clear</button>
    </div>
  );
}

function PanelHeader({ activePanel, openPanel }) {
  const tabs = [['detail', 'Details'], ['edit', 'Editor'], ['scheduling', 'Scheduling'], ['invitations', 'Invites'], ['calendars', 'Calendars'], ['settings', 'Settings'], ['activity', 'Activity']];
  return <div className="panel-tabs">{tabs.map(([id, label]) => <button key={id} className={activePanel === id ? 'is-active' : ''} onClick={() => openPanel(id)}>{label}</button>)}</div>;
}

function EventDetail({ event, calendars, categories, edit, runOperation }) {
  if (!event) return <EmptyState title="No event selected." action="Create or select an event" />;
  return (
    <section className="panel-body">
      <h1>{event.privacy === 'private' ? 'Private event' : event.title}</h1>
      <p className="muted">{formatTime(event.start)} - {formatTime(event.end)} | {event.time_zone}</p>
      <dl className="detail-list">
        <dt>Calendar</dt><dd>{calendars.find((calendar) => calendar.calendar_id === event.calendar_id)?.display_name}</dd>
        <dt>Location</dt><dd>{event.location || 'No location'}</dd>
        <dt>Availability</dt><dd>{event.availability}</dd>
        <dt>Privacy</dt><dd>{event.privacy}</dd>
        <dt>Organizer</dt><dd>{event.organizer?.display_name || 'You'}</dd>
        <dt>Categories</dt><dd>{categories.filter((category) => event.categories?.includes(category.category_id)).map((category) => category.name).join(', ') || 'None'}</dd>
      </dl>
      <div className="attendee-list">{event.attendees?.map((attendee) => <span key={attendee.email}>{attendee.display_name} - {attendee.response_status}</span>)}</div>
      {event.created_by_agent_id && <p className="notice">Created by agent: {event.created_by_agent_id}</p>}
      <div className="button-row">
        <button className="primary-button" onClick={edit}>Edit</button>
        <button onClick={() => runOperation('copy_event', { event_id: event.event_id, destination_calendar_id: 'cal-team', copy_options: { include_reminders: true, as_draft: true } }, 'Event copy planned.')}>Copy</button>
        <button onClick={() => runOperation('move_event', { event_id: event.event_id, destination_calendar_id: 'cal-team', revision: event.revision, notification_scope: 'host_default' }, 'Event move planned.')}>Move</button>
        <button onClick={() => runOperation('get_reminder_state', { event_id: event.event_id }, 'Reminder state loaded.')}>Reminders</button>
        <button className="danger-button" onClick={() => runOperation('delete_event', { target: { event_id: event.event_id }, scope: event.recurrence ? 'series' : 'single', mode: event.kind === 'meeting' ? 'cancel' : 'delete', revision: event.revision, confirmation: { confirmed: true } }, 'Delete or cancel planned.')}>{event.kind === 'meeting' ? 'Cancel meeting' : 'Delete'}</button>
      </div>
    </section>
  );
}

function EventEditor({ draft, setDraft, calendars, categories, saveDraft, runOperation, openScheduling }) {
  function update(field, value) {
    setDraft((current) => ({ ...current, [field]: value }));
  }
  return (
    <section className="panel-body editor-form">
      <h1>Event editor</h1>
      <label>Title<input value={draft.title} onChange={(event) => update('title', event.target.value)} placeholder="Untitled event" /></label>
      <label>Calendar<select value={draft.calendar_id} onChange={(event) => update('calendar_id', event.target.value)}>{calendars.map((calendar) => <option key={calendar.calendar_id} value={calendar.calendar_id}>{calendar.display_name}</option>)}</select></label>
      <div className="field-pair"><label>Starts<input type="datetime-local" value={draft.start.slice(0, 16)} onChange={(event) => update('start', `${event.target.value}:00+02:00`)} /></label><label>Ends<input type="datetime-local" value={draft.end.slice(0, 16)} onChange={(event) => update('end', `${event.target.value}:00+02:00`)} /></label></div>
      <label className="checkbox-line"><input type="checkbox" checked={draft.all_day} onChange={(event) => update('all_day', event.target.checked)} />All day</label>
      <label>Location<input value={draft.location} onChange={(event) => update('location', event.target.value)} /></label>
      <div className="field-pair"><label>Availability<select value={draft.availability} onChange={(event) => update('availability', event.target.value)}><option>busy</option><option>free</option><option>tentative</option><option>out_of_office</option></select></label><label>Privacy<select value={draft.privacy} onChange={(event) => update('privacy', event.target.value)}><option>normal</option><option>private</option><option>confidential</option><option>public</option></select></label></div>
      <label>Category<select value={draft.categories?.[0] || ''} onChange={(event) => update('categories', event.target.value ? [event.target.value] : [])}><option value="">None</option>{categories.map((category) => <option key={category.category_id} value={category.category_id}>{category.name}</option>)}</select></label>
      <label>Attendees<textarea value={draft.attendees?.map((attendee) => attendee.email).join(', ') || ''} onChange={(event) => update('attendees', event.target.value.split(',').filter(Boolean).map((email) => ({ email: email.trim(), kind: 'required', response_status: 'none' })))} placeholder="name@example.com, team@example.com" /></label>
      <label className="checkbox-line"><input type="checkbox" checked={draft.online_meeting?.requested || false} onChange={(event) => update('online_meeting', { requested: event.target.checked, state: event.target.checked ? 'requested' : 'not_requested' })} />Online meeting</label>
      <label>Notes<textarea value={draft.body_text} onChange={(event) => update('body_text', event.target.value)} /></label>
      <div className="button-row">
        <button onClick={() => runOperation('validate_event_draft', { draft }, 'Draft validated.')}>Validate</button>
        <button onClick={() => runOperation('validate_recurrence', { event_time: { start: draft.start, end: draft.end, time_zone: draft.time_zone }, recurrence: { frequency: 'weekly', interval: 1, days_of_week: ['FR'], end: { mode: 'after_count', count: 8 }, time_zone: draft.time_zone } }, 'Recurrence validated.')}>Validate recurrence</button>
        <button onClick={() => runOperation('preview_recurrence', { event_time: { start: draft.start, end: draft.end, time_zone: draft.time_zone }, recurrence: { frequency: 'weekly', interval: 1, days_of_week: ['FR'], end: { mode: 'after_count', count: 8 }, time_zone: draft.time_zone }, limit: 5 }, 'Recurrence preview loaded.')}>Preview recurrence</button>
        <button onClick={openScheduling}>Scheduling</button>
        <button className="primary-button" onClick={() => saveDraft(false)}>Save</button>
        <button className="primary-button" onClick={() => saveDraft(true)}>Send invite</button>
      </div>
    </section>
  );
}

function SchedulingAssistant({ draft, setDraft, suggestions, runOperation }) {
  return (
    <section className="panel-body">
      <h1>Scheduling Assistant</h1>
      <p className="muted">Compare attendees, rooms, and suggested times.</p>
      <div className="button-row">
        <button onClick={() => runOperation('resolve_attendees', { attendees: draft.attendees, allow_manual_email: true }, 'Attendees resolved.')}>Resolve attendees</button>
        <button onClick={() => runOperation('plan_free_busy_lookup', { attendees: draft.attendees, range: { start: draft.start, end: draft.end, time_zone: draft.time_zone }, granularity_minutes: 30 }, 'Free/busy lookup planned.')}>Check availability</button>
        <button onClick={() => runOperation('suggest_meeting_times', { meeting_draft: draft, free_busy_snapshot: [] }, 'Suggested times loaded.')}>Find times</button>
        <button onClick={() => runOperation('search_rooms', { query: 'Berlin', range: { start: draft.start, end: draft.end, time_zone: draft.time_zone }, capacity: 8, features: ['display'] }, 'Room search planned.')}>Search rooms</button>
      </div>
      <div className="freebusy-grid" aria-label="Attendee availability">
        {['Johannes', 'Mara Lee', 'Berlin 4A'].map((name, rowIndex) => <div className="freebusy-row" key={name}><strong>{name}</strong>{Array.from({ length: 8 }, (_, index) => <span key={index} className={(index + rowIndex) % 4 === 0 ? 'busy' : (index + rowIndex) % 5 === 0 ? 'unknown' : 'free'} />)}</div>)}
      </div>
      <h2>Suggested times</h2>
      <div className="suggestion-list">{suggestions.map((slot) => <button key={slot.start} onClick={() => setDraft((current) => ({ ...current, start: slot.start, end: slot.end }))}><strong>{formatTime(slot.start)} - {formatTime(slot.end)}</strong><span>Score {slot.score}</span><small>{slot.reasons.join(' | ')}</small></button>)}</div>
    </section>
  );
}

function InvitationsPanel({ invitations, runOperation }) {
  if (!invitations.length) return <EmptyState title="No pending invitations." />;
  return (
    <section className="panel-body">
      <h1>Invitations</h1>
      {invitations.map((invitation) => <div className="list-row" key={invitation.invitation_id}><strong>{invitation.title}</strong><span>{invitation.organizer} | {formatTime(invitation.start)}</span><small>{invitation.conflicts.join(' ')}</small><div className="button-row"><button onClick={() => runOperation('get_invitation', { invitation_id: invitation.invitation_id, include_conflicts: true }, 'Invitation loaded.')}>Load invitation</button><button onClick={() => runOperation('respond_to_invitation', { invitation_id: invitation.invitation_id, response: 'accept', send_response: true }, 'Invitation accepted.')}>Accept</button><button onClick={() => runOperation('respond_to_invitation', { invitation_id: invitation.invitation_id, response: 'tentative', send_response: true }, 'Invitation tentatively accepted.')}>Tentative</button><button onClick={() => runOperation('respond_to_invitation', { invitation_id: invitation.invitation_id, response: 'decline', send_response: true }, 'Invitation declined.')}>Decline</button><button onClick={() => runOperation('propose_new_time', { invitation_id: invitation.invitation_id, proposed_time: { start: '2026-05-08T17:00:00+02:00', end: '2026-05-08T17:30:00+02:00', time_zone: 'Europe/Berlin' } }, 'New time proposed.')}>Propose time</button></div></div>)}
      <button onClick={() => runOperation('list_invitations', { refresh: true }, 'Invitations refreshed.')}>Refresh invitations</button>
    </section>
  );
}

function CalendarsPanel({ calendars, setCalendars, runOperation }) {
  return (
    <section className="panel-body">
      <h1>Calendars</h1>
      {calendars.map((calendar) => <div className="list-row" key={calendar.calendar_id}><strong><span className="swatch" style={{ backgroundColor: calendar.color }} />{calendar.display_name}</strong><span>{calendar.kind} | {calendar.role} | {calendar.sync_state}</span><div className="button-row"><button onClick={() => runOperation('save_calendar', { calendar: { ...calendar, display_name: `${calendar.display_name} updated` }, revision: calendar.revision }, 'Calendar saved.')}>Save calendar</button><button onClick={() => runOperation('delete_calendar', { calendar_id: calendar.calendar_id, mode: calendar.kind === 'subscription' ? 'unsubscribe' : 'hide', confirmation: { confirmed: true } }, 'Calendar change planned.')}>Hide</button><button onClick={() => runOperation('get_sharing_state', { calendar_id: calendar.calendar_id }, 'Sharing state loaded.')}>Sharing state</button><button onClick={() => runOperation('update_sharing', { calendar_id: calendar.calendar_id, changes: [{ principal_id: 'usr-mara', access_level: 'edit' }], confirmation: { confirmed: true } }, 'Sharing update planned.')}>Share</button></div></div>)}
      <div className="button-row"><button className="primary-button" onClick={() => runOperation('save_calendar', { create: true, calendar: { calendar_id: `cal-${Date.now()}`, display_name: 'New calendar', kind: 'secondary', role: 'owner', color: '#107c10', visible: true, capabilities: {}, time_zone: 'Europe/Berlin' } }, 'Calendar created.')}>Create calendar</button><button onClick={() => runOperation('accept_shared_calendar', { share_invitation_id: 'share-123', display_options: { color: '#498205' } }, 'Shared calendar accepted.')}>Accept shared calendar</button><button onClick={() => runOperation('subscribe_calendar', { subscription: { subscription_ref: 'host-subscription-ref', display_name: 'Industry events', color: '#ca5010' }, confirmation: { confirmed: true } }, 'Subscription planned.')}>Subscribe</button></div>
    </section>
  );
}

function SettingsPanel({ preferences, setPreferences, categories, setCategories, runOperation }) {
  return (
    <section className="panel-body editor-form">
      <h1>Settings</h1>
      <label>Default view<select value={preferences.default_view} onChange={(event) => setPreferences({ ...preferences, default_view: event.target.value })}>{VIEWS.map((item) => <option key={item} value={item}>{VIEW_LABELS[item]}</option>)}</select></label>
      <label>Density<select value={preferences.density} onChange={(event) => setPreferences({ ...preferences, density: event.target.value })}><option>compact</option><option>comfortable</option><option>spacious</option></select></label>
      <label>Primary time zone<input value={preferences.time_zone} onChange={(event) => setPreferences({ ...preferences, time_zone: event.target.value })} /></label>
      <label className="checkbox-line"><input type="checkbox" checked={preferences.default_online_meeting} onChange={(event) => setPreferences({ ...preferences, default_online_meeting: event.target.checked })} />Default online meeting</label>
      <div className="button-row"><button onClick={() => runOperation('get_user_preferences', { include_defaults: true }, 'Preferences loaded.')}>Load preferences</button><button className="primary-button" onClick={() => runOperation('save_user_preferences', { preferences }, 'Preferences saved.')}>Save preferences</button><button onClick={() => runOperation('get_offline_state', {}, 'Offline state loaded.')}>Check offline state</button></div>
      <h2>Categories</h2>
      {categories.map((category) => <div className="list-row" key={category.category_id}><strong><span className="swatch" style={{ backgroundColor: category.color }} />{category.name}</strong><div className="button-row"><button onClick={() => runOperation('save_category', { category: { ...category, name: `${category.name}` } }, 'Category saved.')}>Save category</button><button onClick={() => runOperation('delete_category', { category_id: category.category_id, confirmation: { confirmed: true } }, 'Category delete planned.')}>Delete category</button></div></div>)}
      <button onClick={() => runOperation('list_categories', {}, 'Categories loaded.')}>Load categories</button>
      <h2>Reminders</h2>
      <div className="button-row"><button onClick={() => runOperation('get_reminder_state', { calendar_id: preferences.default_calendar_id }, 'Reminder state loaded.')}>Load reminders</button><button onClick={() => runOperation('snooze_or_dismiss_reminder', { reminder_id: 'rem-customer', action: 'snooze', snooze_until: '2026-05-08T13:55:00+02:00' }, 'Reminder snoozed.')}>Snooze</button><button onClick={() => runOperation('snooze_or_dismiss_reminder', { reminder_id: 'rem-customer', action: 'dismiss' }, 'Reminder dismissed.')}>Dismiss</button></div>
    </section>
  );
}

function ActivityPanel({ activities, offlineState, runOperation }) {
  return (
    <section className="panel-body">
      <h1>Activity</h1>
      <div className="button-row"><button onClick={() => runOperation('get_activity_log', {}, 'Activity loaded.')}>Load activity</button><button onClick={() => runOperation('get_offline_state', {}, 'Offline state loaded.')}>Check offline state</button></div>
      {offlineState && <p className="notice">Offline mutation policy: {offlineState.mutation_policy}. Last sync: {offlineState.last_sync_at || 'unknown'}.</p>}
      {activities.length === 0 ? <EmptyState title="No recent activity is available." /> : activities.map((activity) => <div className="list-row" key={activity.activity_id}><strong>{activity.label}</strong><span>{activity.actor} | {activity.target}</span><small>{activity.at} | Risk: {activity.risk}</small></div>)}
    </section>
  );
}

export default App;
