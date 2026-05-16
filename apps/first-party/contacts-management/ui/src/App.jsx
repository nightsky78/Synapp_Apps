import React, { useEffect, useMemo, useState } from 'react';

const GROUPS = [
  { id: 'all', label: 'All contacts', count: 48, tone: 'accent' },
  { id: 'favorites', label: 'Favorites', count: 12, tone: 'warm' },
  { id: 'coworkers', label: 'Coworkers', count: 21, tone: 'cool' },
  { id: 'vendors', label: 'Vendors', count: 8, tone: 'soft' },
  { id: 'family', label: 'Family', count: 7, tone: 'gold' },
  { id: 'directory', label: 'Company directory', count: 126, tone: 'muted' },
];

const CONTACTS = [
  {
    id: 'alba-ortiz',
    name: 'Alba Ortiz',
    title: 'Design Systems Lead',
    company: 'Northwind',
    email: 'alba.ortiz@northwind.example',
    phone: '+49 30 555 0101',
    location: 'Berlin',
    group: 'coworkers',
    status: 'Available',
    tags: ['Design', 'Critical path'],
    notes: 'Owns the component audit. Prefers concise follow-ups and calendar-linked invites.',
    recent: 'Updated design tokens 2 days ago',
  },
  {
    id: 'marco-iverson',
    name: 'Marco Iverson',
    title: 'Procurement Manager',
    company: 'Blue Ledger',
    email: 'marco.iverson@blueledger.example',
    phone: '+1 415 555 0144',
    location: 'San Francisco',
    group: 'vendors',
    status: 'In a meeting',
    tags: ['Vendor', 'Billing'],
    notes: 'Best contact for invoice follow-up and renewal scheduling.',
    recent: 'Sent pricing update this morning',
  },
  {
    id: 'yasmin-khan',
    name: 'Yasmin Khan',
    title: 'People Ops Partner',
    company: 'Synapp',
    email: 'yasmin.khan@synapp.example',
    phone: '+1 212 555 0188',
    location: 'New York',
    group: 'coworkers',
    status: 'Available',
    tags: ['HR', 'Sensitive'],
    notes: 'Use the dedicated staff channel for policy and onboarding questions.',
    recent: 'Shared onboarding checklist yesterday',
  },
  {
    id: 'sophie-meyer',
    name: 'Sophie Meyer',
    title: 'Finance Lead',
    company: 'Northwind',
    email: 'sophie.meyer@northwind.example',
    phone: '+49 30 555 0192',
    location: 'Hamburg',
    group: 'favorites',
    status: 'Traveling',
    tags: ['Finance', 'Pinned'],
    notes: 'Often coordinates approvals and monthly close milestones.',
    recent: 'Approved the May forecast',
  },
  {
    id: 'jordan-lee',
    name: 'Jordan Lee',
    title: 'Partner Engineer',
    company: 'Fuse Labs',
    email: 'jordan.lee@fuselabs.example',
    phone: '+61 2 5550 1300',
    location: 'Sydney',
    group: 'vendors',
    status: 'Offline',
    tags: ['Integration', 'API'],
    notes: 'Primary technical contact for sync and connector work.',
    recent: 'Left a code review note 3 days ago',
  },
];

const PANEL_ORDER = ['groups', 'people', 'details'];

function useCompactLayout() {
  const [compact, setCompact] = useState(false);

  useEffect(() => {
    if (typeof window === 'undefined' || !window.matchMedia) return undefined;

    const media = window.matchMedia('(max-width: 960px)');
    const update = () => setCompact(media.matches);

    update();
    if (typeof media.addEventListener === 'function') {
      media.addEventListener('change', update);
      return () => media.removeEventListener('change', update);
    }

    media.addListener(update);
    return () => media.removeListener(update);
  }, []);

  return compact;
}

function App() {
  const isCompact = useCompactLayout();
  const [selectedGroupId, setSelectedGroupId] = useState('all');
  const [selectedContactId, setSelectedContactId] = useState(CONTACTS[0].id);
  const [search, setSearch] = useState('');
  const [activePane, setActivePane] = useState('people');

  const visibleContacts = useMemo(() => {
    const query = search.trim().toLowerCase();

    return CONTACTS.filter((contact) => {
      const groupMatch = selectedGroupId === 'all' || contact.group === selectedGroupId;
      const textMatch = !query || [contact.name, contact.title, contact.company, contact.email, contact.location, ...contact.tags]
        .join(' ')
        .toLowerCase()
        .includes(query);

      return groupMatch && textMatch;
    });
  }, [search, selectedGroupId]);

  const selectedContact = visibleContacts.find((contact) => contact.id === selectedContactId) ?? visibleContacts[0] ?? null;

  useEffect(() => {
    if (!visibleContacts.some((contact) => contact.id === selectedContactId) && visibleContacts[0]) {
      setSelectedContactId(visibleContacts[0].id);
    }
  }, [selectedContactId, visibleContacts]);

  useEffect(() => {
    if (!isCompact) return;
    if (activePane === 'details' && !selectedContact) {
      setActivePane('people');
    }
  }, [activePane, isCompact, selectedContact]);

  return (
    <div className={`contacts-shell ${isCompact ? 'contacts-shell--compact' : ''}`} data-testid="contacts-shell">
      <header className="contacts-commandbar" data-testid="contacts-commandbar">
        <div className="brand-block">
          <span className="brand-mark" aria-hidden="true">●</span>
          <div>
            <p className="eyebrow">Synapp Apps</p>
            <h1>Contacts</h1>
          </div>
        </div>

        <label className="search-field" data-testid="contacts-search-field">
          <span className="sr-only">Search contacts</span>
          <input
            data-testid="contacts-search"
            type="search"
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="Search people, emails, notes"
            aria-label="Search contacts"
          />
        </label>

        <div className="command-actions" data-testid="contacts-command-actions">
          <button className="primary-action" data-testid="contacts-new-contact" type="button">New contact</button>
          <button type="button" data-testid="contacts-import">Import</button>
          <button type="button" data-testid="contacts-share-book">Share book</button>
          <span className="workspace-status" data-testid="contacts-status">Schema-driven preview</span>
        </div>
      </header>

      {isCompact && (
        <nav className="mobile-switcher" data-testid="contacts-mobile-switcher" aria-label="Contacts workspace panels">
          {PANEL_ORDER.map((pane) => (
            <button
              key={pane}
              type="button"
              className={activePane === pane ? 'is-active' : ''}
              aria-pressed={activePane === pane}
              onClick={() => setActivePane(pane)}
            >
              {pane}
            </button>
          ))}
        </nav>
      )}

      <div className="contacts-layout">
        {(!isCompact || activePane === 'groups') && (
          <aside className="pane pane--groups" data-testid="contacts-groups-pane" aria-label="Contact groups">
            <div className="pane-header">
              <div>
                <p className="eyebrow">Workspace</p>
                <h2>Groups</h2>
              </div>
              <button type="button">Manage</button>
            </div>

            <div className="group-list" data-testid="contacts-groups-list" role="listbox" aria-label="Contact groups list">
              {GROUPS.map((group) => (
                <button
                  key={group.id}
                  type="button"
                  className={`group-row ${selectedGroupId === group.id ? 'is-selected' : ''} group-row--${group.tone}`}
                  data-testid={group.id === 'all' ? 'contacts-group-all' : `contacts-group-item-${group.id}`}
                  aria-selected={selectedGroupId === group.id}
                  onClick={() => setSelectedGroupId(group.id)}
                >
                  <span>{group.label}</span>
                  <strong>{group.count}</strong>
                </button>
              ))}
            </div>

            <section className="pane-card">
              <p className="eyebrow">Signals</p>
              <h3>Shared view state</h3>
              <p>
                Keep sort order, pinned contacts, and visibility preferences scoped to the current principal.
              </p>
            </section>
          </aside>
        )}

        {(!isCompact || activePane === 'people') && (
          <main className="pane pane--people" data-testid="contacts-list-pane" aria-label="Contact list">
            <div className="pane-header">
              <div>
                <p className="eyebrow">People</p>
                <h2>{visibleContacts.length} contacts</h2>
              </div>
              <div className="pane-controls" data-testid="contacts-list-toolbar">
                <button type="button" data-testid="contacts-sort">Sort</button>
                <button type="button" data-testid="contacts-density">Density</button>
              </div>
            </div>

            <div className="contact-list" data-testid="contacts-contact-list" role="listbox" aria-label="Visible contacts">
              {visibleContacts.map((contact) => (
                <button
                  key={contact.id}
                  type="button"
                  className={`contact-row ${selectedContact?.id === contact.id ? 'is-selected' : ''}`}
                  data-testid={`contacts-contact-item-${contact.id}`}
                  aria-selected={selectedContact?.id === contact.id}
                  onClick={() => {
                    setSelectedContactId(contact.id);
                    if (isCompact) setActivePane('details');
                  }}
                >
                  <span className="avatar" data-testid={`contacts-contact-avatar-${contact.id}`} aria-hidden="true">
                    {contact.name.split(' ').map((part) => part[0]).join('')}
                  </span>
                  <span className="contact-copy">
                    <strong data-testid={`contacts-contact-name-${contact.id}`}>{contact.name}</strong>
                    <span data-testid={`contacts-contact-meta-${contact.id}`}>
                      {contact.title} · {contact.company}
                    </span>
                    <small>{contact.email}</small>
                  </span>
                  <span className={`status-pill status-pill--${contact.status === 'Available' ? 'success' : 'muted'}`}>
                    {contact.status}
                  </span>
                </button>
              ))}

              {visibleContacts.length === 0 && (
                <div className="empty-state" data-testid="contacts-empty-state">
                  <strong>No contacts match the current filters.</strong>
                  <p>Clear the search or switch groups to reveal more records.</p>
                </div>
              )}
            </div>
          </main>
        )}

        {(!isCompact || activePane === 'details') && (
          <aside className="pane pane--details" data-testid="contacts-detail-pane" aria-label="Contact details">
            <div className="pane-header pane-header--details" data-testid="contacts-detail-header">
              <div>
                <p className="eyebrow">Detail</p>
                <h2>{selectedContact?.name ?? 'No contact selected'}</h2>
              </div>
              <div className="pane-controls" data-testid="contacts-detail-actions">
                <button type="button">Edit</button>
                <button type="button">Message</button>
              </div>
            </div>

            {selectedContact ? (
              <div className="detail-stack">
                <section className="detail-card">
                  <p className="eyebrow">Identity</p>
                  <h3>{selectedContact.title}</h3>
                  <p>{selectedContact.company}</p>
                  <dl className="detail-grid">
                    <dt>Email</dt>
                    <dd data-testid="contacts-detail-email">{selectedContact.email}</dd>
                    <dt>Phone</dt>
                    <dd data-testid="contacts-detail-phone">{selectedContact.phone}</dd>
                    <dt>Location</dt>
                    <dd>{selectedContact.location}</dd>
                  </dl>
                </section>

                <section className="detail-card">
                  <p className="eyebrow">Tags</p>
                  <div className="tag-row" data-testid="contacts-detail-tags">
                    {selectedContact.tags.map((tag) => (
                      <span key={tag} className="tag-pill">{tag}</span>
                    ))}
                  </div>
                </section>

                <section className="detail-card">
                  <p className="eyebrow">Notes</p>
                  <p data-testid="contacts-detail-notes">{selectedContact.notes}</p>
                </section>

                <section className="detail-card detail-card--soft">
                  <p className="eyebrow">Recent activity</p>
                  <p data-testid="contacts-detail-activity">{selectedContact.recent}</p>
                </section>
              </div>
            ) : (
              <div className="empty-state">
                <strong>No contact selected.</strong>
                <p>Choose a person from the center list to inspect the contact details.</p>
              </div>
            )}
          </aside>
        )}
      </div>
    </div>
  );
}

export default App;
