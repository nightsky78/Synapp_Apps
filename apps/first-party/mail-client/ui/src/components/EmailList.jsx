import React, { useCallback } from 'react';
import { useApp } from '../context/AppContext.jsx';
import { CenteredSpinner } from './Spinner.jsx';

const PAGE_SIZE = 50;

function formatDate(unix) {
  if (!unix) return '';
  const d = new Date(unix * 1000);
  const now = new Date();
  const isToday = d.toDateString() === now.toDateString();
  if (isToday) return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  const daysAgo = (now - d) / 86400000;
  if (daysAgo < 7) return d.toLocaleDateString([], { weekday: 'short' });
  return d.toLocaleDateString([], { month: 'short', day: 'numeric' });
}

function EmailItem({ email, active, onSelect, onFlag }) {
  return (
    <article
      className={`email-item${!email.is_read ? ' email-item--unread' : ''}${active ? ' email-item--active' : ''}`}
      onClick={() => onSelect(email)}
      role="button"
      tabIndex={0}
      onKeyDown={e => e.key === 'Enter' && onSelect(email)}
      aria-label={`Email from ${email.from.display_name || email.from.email}: ${email.subject}`}
      aria-pressed={active}
    >
      <div className="email-item__row1">
        <span className={`email-item__unread-dot${email.is_read ? ' email-item__unread-dot--hidden' : ''}`} aria-hidden="true" />
        <span className="email-item__from">
          {email.from.display_name || email.from.email}
        </span>
        <span className="email-item__date">{formatDate(email.received_at)}</span>
      </div>
      <div className="email-item__row2">
        <span className="email-item__subject">{email.subject}</span>
        <span className="email-item__icons">
          {email.importance === 'high' && <span className="imp-high" title="High importance">!</span>}
          {email.has_attachments && <span className="email-item__icon" title="Has attachments">📎</span>}
          {email.is_flagged && <span className="email-item__icon" style={{ color: '#c50f1f' }} title="Flagged">🚩</span>}
        </span>
      </div>
      <div className="email-item__row3">
        <span className="email-item__preview">{email.preview}</span>
      </div>
      <button
        className={`email-item__flag-btn${email.is_flagged ? ' email-item__flag-btn--active' : ''}`}
        title={email.is_flagged ? 'Remove flag' : 'Flag email'}
        aria-label={email.is_flagged ? 'Remove flag' : 'Flag email'}
        onClick={e => { e.stopPropagation(); onFlag(email.email_id); }}
      >
        🚩
      </button>
    </article>
  );
}

export default function EmailList() {
  const { state, loadEmails, selectEmail, flagEmail } = useApp();

  const mailbox = state.mailboxes.find(m => m.mailbox_id === state.activeMailboxId);
  const totalPages = Math.ceil(state.emailsTotal / PAGE_SIZE);

  const handleSelect = useCallback((email) => selectEmail(email), [selectEmail]);
  const handleFlag = useCallback((id) => flagEmail(id), [flagEmail]);

  return (
    <section className="list-panel" aria-label="Email list">
      <div className="list-panel__header">
        <div className="list-panel__title">{mailbox?.name ?? 'Inbox'}</div>
        <div className="list-panel__meta">
          {state.emailsTotal} messages
          {state.emailsUnread > 0 && ` · ${state.emailsUnread} unread`}
        </div>
      </div>

      <div className="list-panel__list" role="list">
        {state.loadingEmails && state.emails.length === 0 ? (
          <CenteredSpinner label="Loading emails…" />
        ) : state.emails.length === 0 ? (
          <div className="empty-state">
            <div className="empty-state__icon">📭</div>
            <div className="empty-state__title">No messages</div>
            <div className="empty-state__body">This folder is empty.</div>
          </div>
        ) : (
          state.emails.map(email => (
            <EmailItem
              key={email.email_id}
              email={email}
              active={state.activeEmailId === email.email_id}
              onSelect={handleSelect}
              onFlag={handleFlag}
            />
          ))
        )}
      </div>

      {totalPages > 1 && (
        <div className="pagination">
          <button
            className="pagination__btn"
            disabled={state.emailPage === 0 || state.loadingEmails}
            onClick={() => loadEmails(state.activeMailboxId, state.emailPage - 1)}
            aria-label="Previous page"
          >
            ← Prev
          </button>
          <span className="pagination__info">
            Page {state.emailPage + 1} of {totalPages}
          </span>
          <button
            className="pagination__btn"
            disabled={state.emailPage >= totalPages - 1 || state.loadingEmails}
            onClick={() => loadEmails(state.activeMailboxId, state.emailPage + 1)}
            aria-label="Next page"
          >
            Next →
          </button>
        </div>
      )}
    </section>
  );
}
