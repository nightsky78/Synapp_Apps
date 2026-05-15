import React, { useCallback } from 'react';
import { useApp } from '../context/AppContext.jsx';
import { Spinner } from './Spinner.jsx';

function formatFullDate(unix) {
  if (!unix) return '';
  return new Date(unix * 1000).toLocaleString([], {
    weekday: 'long', year: 'numeric', month: 'long',
    day: 'numeric', hour: '2-digit', minute: '2-digit',
  });
}

function formatBytes(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function AddressList({ addresses }) {
  if (!addresses || addresses.length === 0) return null;
  return (
    <span>
      {addresses.map((a, i) => (
        <React.Fragment key={i}>
          {i > 0 && ', '}
          <span title={a.email}>{a.display_name || a.email}</span>
        </React.Fragment>
      ))}
    </span>
  );
}

function htmlToText(html) {
  return html
    .replace(/<br\s*\/?\s*>/gi, '\n')
    .replace(/<\/p\s*>/gi, '\n\n')
    .replace(/<[^>]*>/g, '')
    .replace(/&nbsp;/gi, ' ')
    .replace(/&amp;/gi, '&')
    .replace(/&lt;/gi, '<')
    .replace(/&gt;/gi, '>')
    .replace(/&quot;/gi, '"')
    .replace(/&#39;/gi, "'")
    .trim();
}

export default function ReadingPane() {
  const { state, archiveEmail, deleteEmail, openCompose, flagEmail, can } = useApp();
  const email = state.activeEmail;

  const handleReply = useCallback(() => {
    if (!email) return;
    openCompose(`reply:${email.email_id}`);
  }, [email, openCompose]);

  const handleReplyAll = useCallback(() => {
    if (!email) return;
    openCompose(`reply-all:${email.email_id}`);
  }, [email, openCompose]);

  const handleForward = useCallback(() => {
    if (!email) return;
    openCompose(`forward:${email.email_id}`);
  }, [email, openCompose]);

  const handleArchive = useCallback(() => {
    if (!email) return;
    archiveEmail(email.email_id);
  }, [email, archiveEmail]);

  const handleDelete = useCallback(() => {
    if (!email) return;
    deleteEmail(email.email_id);
  }, [email, deleteEmail]);

  const handleToggleFlag = useCallback(async () => {
    if (!email) return;
    flagEmail(email.email_id);
  }, [email, flagEmail]);

  if (state.loadingEmail) {
    return (
      <div className="reading-pane" aria-label="Email content">
        <div className="reading-pane__empty">
          <Spinner size="lg" />
        </div>
      </div>
    );
  }

  if (!email) {
    return (
      <div className="reading-pane" aria-label="Email content">
        <div className="reading-pane__empty">
          <div className="reading-pane__empty-icon">✉️</div>
          <div className="reading-pane__empty-text">Select a message to read</div>
        </div>
      </div>
    );
  }

  return (
    <div className="reading-pane" aria-label="Email content">
      <div className="reading-pane__scroll" role="main">
        <div className="reading-pane__header">
          {email.importance === 'high' && (
            <div className="reading-pane__imp-badge reading-pane__imp-badge--high">! High importance</div>
          )}
          {email.importance === 'low' && (
            <div className="reading-pane__imp-badge reading-pane__imp-badge--low">Low importance</div>
          )}
          <h1 className="reading-pane__subject">{email.subject}</h1>

          <div className="reading-pane__meta-row">
            <span className="reading-pane__meta-label">From</span>
            <span className="reading-pane__meta-value">
              {email.from.display_name
                ? <><strong>{email.from.display_name}</strong> &lt;{email.from.email}&gt;</>
                : email.from.email}
            </span>
            <span className="reading-pane__meta-date">{formatFullDate(email.received_at)}</span>
          </div>

          <div className="reading-pane__meta-row">
            <span className="reading-pane__meta-label">To</span>
            <span className="reading-pane__meta-value"><AddressList addresses={email.to} /></span>
          </div>

          {email.cc?.length > 0 && (
            <div className="reading-pane__meta-row">
              <span className="reading-pane__meta-label">CC</span>
              <span className="reading-pane__meta-value"><AddressList addresses={email.cc} /></span>
            </div>
          )}

          {email.remote_sync_state === 'pending_remote_move' && (
            <div className="reading-pane__sync" data-testid={`email-sync-state-${email.email_id}`}>
              Organized locally; remote mailbox sync pending.
            </div>
          )}
        </div>

        <div className="reading-pane__divider" />

        <div className="reading-pane__body" style={{ whiteSpace: 'pre-wrap' }}>
          {email.body_text || htmlToText(email.body_html || '')}
        </div>

        {email.attachments?.length > 0 && (
          <div className="reading-pane__attachments">
            <div className="reading-pane__att-title">Attachments ({email.attachments.length})</div>
            <div className="reading-pane__att-list">
              {email.attachments.map(att => (
                <div key={att.attachment_id} className="reading-pane__att-item" title={att.file_name}>
                  <span>📎</span>
                  <span>{att.file_name}</span>
                  <span className="reading-pane__att-size">{formatBytes(att.size_bytes)}</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Action toolbar */}
      <div className="reading-pane__actions" role="toolbar" aria-label="Email actions">
        <button
          className="reading-pane__action-btn reading-pane__action-btn--primary"
          onClick={handleReply}
          disabled={!can('send')}
          title={!can('send') ? 'Send permission required' : 'Reply'}
          aria-label="Reply"
        >
          ↩ Reply
        </button>

        <button
          className="reading-pane__action-btn"
          onClick={handleReplyAll}
          disabled={!can('send')}
          title={!can('send') ? 'Send permission required' : 'Reply all'}
          aria-label="Reply all"
        >
          ↩↩ Reply All
        </button>

        <button
          className="reading-pane__action-btn"
          onClick={handleForward}
          disabled={!can('send')}
          title={!can('send') ? 'Send permission required' : 'Forward'}
          aria-label="Forward"
        >
          ↪ Forward
        </button>

        <button
          className="reading-pane__action-btn"
          onClick={handleToggleFlag}
          disabled={!can('organize')}
          title={!can('organize') ? 'Organize permission required' : (email.is_flagged ? 'Remove flag' : 'Flag')}
          aria-label={email.is_flagged ? 'Remove flag' : 'Flag email'}
          style={{ color: email.is_flagged ? '#c50f1f' : undefined }}
        >
          🚩 {email.is_flagged ? 'Unflag' : 'Flag'}
        </button>

        <div style={{ flex: 1 }} />

        <button
          className="reading-pane__action-btn"
          onClick={handleArchive}
          disabled={!can('organize')}
          title={!can('organize') ? 'Organize permission required' : 'Archive'}
          aria-label="Archive email"
        >
          🗄️ Archive
        </button>

        <button
          className="reading-pane__action-btn"
          onClick={handleDelete}
          disabled={!can('organize')}
          title={!can('organize') ? 'Organize permission required' : 'Delete'}
          aria-label="Delete email"
          style={{ color: !can('organize') ? undefined : '#c50f1f' }}
        >
          🗑️ Delete
        </button>
      </div>
    </div>
  );
}
