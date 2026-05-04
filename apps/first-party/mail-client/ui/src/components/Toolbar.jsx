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
  const { state, openCompose, search, dispatch } = useApp();
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

  return (
    <div className="top-bar">
      {/* Logo */}
      <div className="top-bar__logo">
        <span style={{ fontSize: 20 }}>✉️</span>
        <span>Synapp Mail</span>
      </div>

      {/* Compose */}
      <button
        className="top-bar__btn top-bar__btn--compose"
        onClick={() => openCompose()}
        title="Compose new email"
        aria-label="Compose new email"
        disabled={!state.hostCtx || state.accounts.length === 0}
      >
        ✏️ New message
      </button>

      {/* Separator */}
      <div style={{ width: 1, height: 20, background: 'rgba(255,255,255,0.3)', margin: '0 4px' }} />

      {/* Search */}
      <div className="top-bar__search">
        <span className="top-bar__search-icon">🔍</span>
        <input
          ref={inputRef}
          type="search"
          placeholder="Search mail"
          value={localQuery}
          onChange={handleSearch}
          aria-label="Search emails"
          autoComplete="off"
        />
        {localQuery && (
          <button className="top-bar__search-clear" onClick={clearSearch} aria-label="Clear search">×</button>
        )}
      </div>

      <div className="top-bar__spacer" />

      {/* Folder label */}
      {mailbox && !state.searchQuery && (
        <span style={{ fontSize: 13, color: 'rgba(255,255,255,0.85)', display: 'flex', alignItems: 'center', gap: 4 }}>
          {ICON_MAP[mailbox.kind] || '📁'} {mailbox.name}
          {mailbox.unread_count > 0 && (
            <span style={{ background: 'rgba(255,255,255,0.2)', borderRadius: 10, padding: '0 6px', fontSize: 11, fontWeight: 700 }}>
              {mailbox.unread_count}
            </span>
          )}
        </span>
      )}

      {/* Rules */}
      <button
        className="top-bar__btn"
        title="Manage rules"
        onClick={() => dispatch({ type: 'SET_SHOW_RULES_MANAGER', payload: true })}
        aria-label="Manage rules"
      >
        ⚙️ Rules
      </button>

      {/* Permission badge */}
      <PermissionBadge permission={state.hostCtx.permission} />
    </div>
  );
}
