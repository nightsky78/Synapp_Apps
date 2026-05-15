import React, { useState, useRef, useEffect } from 'react';
import { useApp } from '../context/AppContext.jsx';
import PermissionBadge from './PermissionBadge.jsx';

const ICON_MAP = {
  inbox:   '📥',
  drafts:  '📝',
  sent:    '📤',
  archive: '🗄️',
  trash:   '🗑️',
  junk:    '⚠️',
  custom:  '📁',
};

export default function Toolbar() {
  const { state, openCompose, search, dispatch, loadEmails, loadMailboxes } = useApp();
  const [localQuery, setLocalQuery] = useState(state.searchQuery);
  const inputRef = useRef(null);

  useEffect(() => {
    if (!state.searchQuery) setLocalQuery('');
  }, [state.searchQuery]);

  function handleSearch(e) {
    const val = e.target.value;
    setLocalQuery(val);
    search(val);
  }

  function clearSearch() {
    setLocalQuery('');
    search('');
  }

  const mailbox = state.mailboxes.find(m => m.mailbox_id === state.activeMailboxId);
  const account = state.accounts.find(a => a.account_id === state.activeAccountId);
  const canCompose = state.accounts.length > 0 && account?.connection_state !== 'receive_only';

  function refreshCurrentView() {
    if (state.activeAccountId) loadMailboxes(state.activeAccountId);
    if (state.activeMailboxId) loadEmails(state.activeMailboxId, state.emailPage);
  }

  return (
    <div className="top-bar">
      <div className="top-bar__logo">
        <span>Synapp Mail</span>
      </div>

      <button
        className="top-bar__btn top-bar__btn--compose"
        data-testid="compose-btn"
        onClick={() => openCompose()}
        title="Compose new email"
        aria-label="Compose new email"
        disabled={!state.hostCtx || !canCompose}
      >
        Compose
      </button>

      {state.accounts.length > 0 && (
        <select
          className="top-bar__account-select"
          data-testid="account-select"
          value={state.activeAccountId || ''}
          onChange={event => dispatch({ type: 'SET_ACTIVE_ACCOUNT', payload: event.target.value })}
          aria-label="Account scope"
        >
          {state.accounts.map(item => <option key={item.account_id} value={item.account_id}>{item.account_label || item.email_address}</option>)}
        </select>
      )}

      <div className="top-bar__search">
        <input
          ref={inputRef}
          type="search"
          placeholder="Search mail, from, subject, attachment"
          value={localQuery}
          onChange={handleSearch}
          aria-label="Search emails"
          autoComplete="off"
        />
        {localQuery && (
          <button className="top-bar__search-clear" onClick={clearSearch} aria-label="Clear search">×</button>
        )}
      </div>

      <div className="top-bar__filter-group" role="group" aria-label="Message filters">
        {['all', 'unread', 'flagged', 'attachments'].map(filter => (
          <button
            key={filter}
            className={state.activeFilter === filter ? 'top-bar__chip top-bar__chip--active' : 'top-bar__chip'}
            onClick={() => dispatch({ type: 'SET_FILTER', payload: filter })}
          >
            {filter === 'all' ? 'All' : filter[0].toUpperCase() + filter.slice(1)}
          </button>
        ))}
      </div>

      {mailbox && !state.searchQuery && (
        <span className="top-bar__folder-label">
          {ICON_MAP[mailbox.kind] || ''} {mailbox.name}
          {mailbox.unread_count > 0 && (
            <span className="top-bar__count">
              {mailbox.unread_count}
            </span>
          )}
        </span>
      )}

      <span className={`health-dot health-dot--${account?.connection_state || 'unknown'}`} title={`Account health: ${account?.connection_state || 'unknown'}`} aria-label={`Account health: ${account?.connection_state || 'unknown'}`} />

      <button className="top-bar__btn" onClick={refreshCurrentView} aria-label="Refresh mailbox" title="Refresh mailbox">Refresh</button>

      <button
        className="top-bar__btn"
        title="Open mail settings"
        onClick={() => dispatch({ type: 'SET_ROUTE', payload: { route: state.activeRoute === 'settings' ? 'inbox' : 'settings' } })}
        aria-label={state.activeRoute === 'settings' ? 'Return to inbox' : 'Open mail settings'}
      >
        {state.activeRoute === 'settings' ? 'Inbox' : 'Settings'}
      </button>

      <PermissionBadge permission={state.hostCtx.permission} />
    </div>
  );
}
