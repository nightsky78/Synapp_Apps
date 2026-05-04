import React from 'react';
import { useApp } from '../context/AppContext.jsx';
import { Spinner } from './Spinner.jsx';

function formatDate(unix) {
  if (!unix) return '';
  return new Date(unix * 1000).toLocaleString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
}

export default function SearchView() {
  const { state, selectEmail } = useApp();
  const { searchQuery, searchResults, searching, activeEmailId } = state;

  return (
    <div style={{ display: 'flex', flex: 1, flexDirection: 'column', overflow: 'hidden', background: '#fff' }}>
      <div className="search-header">
        {searching
          ? <span>Searching for <strong>"{searchQuery}"</strong>…</span>
          : searchResults
          ? <span>Found <strong>{searchResults.total_count}</strong> result{searchResults.total_count !== 1 ? 's' : ''} for <strong>"{searchQuery}"</strong></span>
          : null
        }
      </div>

      <div style={{ flex: 1, overflowY: 'auto' }} role="list" aria-label="Search results">
        {searching && (
          <div className="centered-spinner"><Spinner size="lg" /><span>Searching…</span></div>
        )}

        {!searching && searchResults && searchResults.results.length === 0 && (
          <div className="empty-state">
            <div className="empty-state__icon">🔍</div>
            <div className="empty-state__title">No results</div>
            <div className="empty-state__body">No emails matched "{searchQuery}". Try a different search term.</div>
          </div>
        )}

        {!searching && searchResults && searchResults.results.map(email => (
          <article
            key={email.email_id}
            className={`email-item${!email.is_read ? ' email-item--unread' : ''}${activeEmailId === email.email_id ? ' email-item--active' : ''}`}
            onClick={() => selectEmail(email)}
            role="button"
            tabIndex={0}
            onKeyDown={e => e.key === 'Enter' && selectEmail(email)}
            aria-label={`Search result: ${email.subject} from ${email.from.email}`}
          >
            <div className="email-item__row1">
              <span className={`email-item__unread-dot${email.is_read ? ' email-item__unread-dot--hidden' : ''}`} aria-hidden="true" />
              <span className="email-item__from">{email.from.display_name || email.from.email}</span>
              <span className="email-item__date">{formatDate(email.received_at)}</span>
            </div>
            <div className="email-item__row2">
              <span className="email-item__subject">{email.subject}</span>
              {email.mailbox_id && (
                <span style={{ fontSize: 11, color: '#0078d4', padding: '1px 6px', background: '#d0e8f8', borderRadius: 10 }}>
                  {email.mailbox_id}
                </span>
              )}
            </div>
            <div className="email-item__row3">
              <span className="email-item__preview">{email.preview}</span>
            </div>
          </article>
        ))}
      </div>
    </div>
  );
}
