import React, { useState, useEffect, useRef, useCallback } from 'react';
import { useApp } from '../context/AppContext.jsx';
import { invoke } from '../bridge.js';
import { Spinner } from './Spinner.jsx';

const AUTO_SAVE_INTERVAL = 10_000; // 10 seconds

function parseReplyContext(composeDraftId) {
  if (!composeDraftId) return null;
  if (composeDraftId.startsWith('reply:')) return { mode: 'reply', emailId: composeDraftId.slice(6) };
  if (composeDraftId.startsWith('reply-all:')) return { mode: 'reply-all', emailId: composeDraftId.slice(10) };
  if (composeDraftId.startsWith('forward:')) return { mode: 'forward', emailId: composeDraftId.slice(8) };
  return null;
}

function prefixSubject(mode, subject) {
  if (!subject) return '';
  if (mode === 'reply' || mode === 'reply-all') return subject.startsWith('Re: ') ? subject : `Re: ${subject}`;
  if (mode === 'forward') return subject.startsWith('Fwd: ') ? subject : `Fwd: ${subject}`;
  return subject;
}

export default function ComposePane() {
  const { state, closeCompose, toast, can } = useApp();

  const replyCtx = parseReplyContext(state.composeDraftId);
  const parentEmail = replyCtx ? state.emails.find(e => e.email_id === replyCtx.emailId) : null;

  const [to, setTo]           = useState(parentEmail ? [parentEmail.from.email] : []);
  const [cc, setCc]           = useState(replyCtx?.mode === 'reply-all' && parentEmail ? parentEmail.cc.map(a => a.email) : []);
  const [bcc, setBcc]         = useState([]);
  const [showCcBcc, setShowCcBcc] = useState(false);
  const [subject, setSubject] = useState(parentEmail ? prefixSubject(replyCtx.mode, parentEmail.subject) : '');
  const [body, setBody]       = useState(parentEmail && replyCtx.mode !== 'forward'
    ? `\n\n---\nFrom: ${parentEmail.from.email}\n${parentEmail.body_text || ''}`
    : parentEmail
    ? `\n\n---\n---------- Forwarded message ----------\nFrom: ${parentEmail.from.email}\nSubject: ${parentEmail.subject}\n\n${parentEmail.body_text || ''}`
    : '');

  const [toInput, setToInput]   = useState('');
  const [ccInput, setCcInput]   = useState('');
  const [bccInput, setBccInput] = useState('');

  const [sending, setSending]   = useState(false);
  const [draftId, setDraftId]   = useState(null);
  const [saveStatus, setSaveStatus] = useState('');
  const [scheduleOpen, setScheduleOpen] = useState(false);
  const [fromAccountId, setFromAccountId] = useState(state.activeAccountId || '');
  const [includeSignature, setIncludeSignature] = useState(true);

  const bodyRef = useRef(null);
  const autoSaveTimer = useRef(null);

  // Focus body on mount
  useEffect(() => {
    bodyRef.current?.focus();
  }, []);

  const buildDraftPayload = useCallback(() => ({
    draft_id: draftId || undefined,
    account_id: fromAccountId || state.activeAccountId,
    to: to.map(e => ({ email: e })),
    cc: cc.map(e => ({ email: e })),
    bcc: bcc.map(e => ({ email: e })),
    subject,
    body_text: includeSignature ? `${body}\n\nBest regards` : body,
    body_html: `<p>${(includeSignature ? `${body}\n\nBest regards` : body).replace(/\n/g, '<br/>')}</p>`,
    in_reply_to: replyCtx ? replyCtx.emailId : undefined,
  }), [draftId, fromAccountId, state.activeAccountId, to, cc, bcc, subject, body, includeSignature, replyCtx]);

  const saveDraft = useCallback(async (silent = false) => {
    if (!can('draft')) return;
    setSaveStatus('Saving…');
    try {
      const payload = buildDraftPayload();
      let res;
      if (draftId) {
        res = await invoke('update_draft', { draft: payload });
      } else {
        res = await invoke('draft_email', { draft: payload });
        setDraftId(res.data.suggested_draft_id);
      }
      setSaveStatus('Draft saved');
      if (!silent) toast('Draft saved', 'success');
    } catch (err) {
      setSaveStatus('Save failed');
      if (!silent) toast(`Draft save failed: ${err.message}`, 'error');
    }
  }, [can, buildDraftPayload, draftId, toast]);

  // Auto-save
  useEffect(() => {
    if (!can('draft')) return;
    autoSaveTimer.current = setInterval(() => saveDraft(true), AUTO_SAVE_INTERVAL);
    return () => clearInterval(autoSaveTimer.current);
  }, [saveDraft, can]);

  function addEmailChip(list, setList, input, setInput) {
    const email = input.trim();
    if (!email) return;
    if (list.includes(email)) { setInput(''); return; }
    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
      toast('Invalid email address', 'error');
      return;
    }
    setList([...list, email]);
    setInput('');
  }

  function removeChip(list, setList, email) {
    setList(list.filter(e => e !== email));
  }

  function handleChipKeyDown(e, list, setList, input, setInput) {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      addEmailChip(list, setList, input, setInput);
    }
    if (e.key === 'Backspace' && !input && list.length > 0) {
      setList(list.slice(0, -1));
    }
  }

  function EmailChipInput({ label, chips, setChips, input, setInput }) {
    return (
      <div className="compose-field">
        <span className="compose-field__label">{label}</span>
        <div style={{ flex: 1, display: 'flex', flexWrap: 'wrap', gap: 4, padding: '4px 4px' }}>
          {chips.map(e => (
            <span
              key={e}
              style={{ display: 'inline-flex', alignItems: 'center', background: '#e6f2fc', border: '1px solid #b3d6f5', borderRadius: 12, padding: '0 8px', fontSize: 12, gap: 4 }}
            >
              {e}
              <button style={{ background: 'none', border: 'none', cursor: 'pointer', fontSize: 14, color: '#666', lineHeight: 1 }} onClick={() => removeChip(chips, setChips, e)} aria-label={`Remove ${e}`}>×</button>
            </span>
          ))}
          <input
            type="email"
            className="compose-field__input"
            style={{ flex: 1, minWidth: 120 }}
            value={input}
            onChange={e => setInput(e.target.value)}
            onKeyDown={e => handleChipKeyDown(e, chips, setChips, input, setInput)}
            onBlur={() => addEmailChip(chips, setChips, input, setInput)}
            placeholder={chips.length === 0 ? 'Add recipients…' : ''}
          />
        </div>
        {!chips.length && !input && <span className="compose-field__error" />}
      </div>
    );
  }

  async function handleSend() {
    if (to.length === 0) { toast('Add at least one recipient', 'error'); return; }
    if (!subject.trim()) { toast('Subject is required', 'error'); return; }
    if (!can('send')) { toast('You do not have Send permission', 'error'); return; }
    const account = state.accounts.find(item => item.account_id === (fromAccountId || state.activeAccountId));
    if (account?.connection_state === 'receive_only') { toast('This account is receive-only until SMTP passes', 'error'); return; }

    setSending(true);
    try {
      // Save draft first to get a draft_id
      let dId = draftId;
      if (!dId) {
        const dres = await invoke('draft_email', { draft: buildDraftPayload() });
        dId = dres.data.suggested_draft_id;
        setDraftId(dId);
      } else {
        await invoke('update_draft', { draft: buildDraftPayload() });
      }
      await invoke('send_email', { draft_id: dId, account_id: fromAccountId || state.activeAccountId });
      toast('Message sent!', 'success');
      closeCompose();
    } catch (err) {
      toast(`Send failed: ${err.message}`, 'error');
    } finally {
      setSending(false);
    }
  }

  async function handleScheduleSend() {
    if (to.length === 0) { toast('Add at least one recipient', 'error'); return; }
    const account = state.accounts.find(item => item.account_id === (fromAccountId || state.activeAccountId));
    if (account?.connection_state === 'receive_only') { toast('This account is receive-only until SMTP passes', 'error'); return; }
    const tomorrowNoon = Math.floor(Date.now() / 1000) + 24 * 3600;
    setSending(true);
    try {
      let dId = draftId;
      if (!dId) {
        const dres = await invoke('draft_email', { draft: buildDraftPayload() });
        dId = dres.data.suggested_draft_id;
        setDraftId(dId);
      }
      await invoke('schedule_send', { draft_id: dId, account_id: fromAccountId || state.activeAccountId, scheduled_for: tomorrowNoon });
      toast('Scheduled for tomorrow noon', 'success');
      closeCompose();
    } catch (err) {
      toast(`Schedule failed: ${err.message}`, 'error');
    } finally {
      setSending(false);
      setScheduleOpen(false);
    }
  }

  async function handleDiscard() {
    if (draftId) {
      try { await invoke('discard_draft', { draft_id: draftId }); } catch (_) { /* ignore */ }
    }
    closeCompose();
  }

  return (
    <div className="compose-overlay" role="dialog" aria-modal="true" aria-label="Compose email">
      <div className="compose-window">
        <div className="compose-window__header">
          <span className="compose-window__title">
            {replyCtx?.mode === 'reply' ? 'Reply' : replyCtx?.mode === 'reply-all' ? 'Reply All' : replyCtx?.mode === 'forward' ? 'Forward' : 'New Message'}
          </span>
          <button className="compose-window__header-btn" onClick={() => saveDraft()} title="Save draft" aria-label="Save draft">−</button>
          <button className="compose-window__header-btn" onClick={handleDiscard} title="Discard draft" aria-label="Discard and close">×</button>
        </div>

        <div className="compose-window__fields">
          <div className="compose-field">
            <span className="compose-field__label">From</span>
            <select className="compose-field__input" value={fromAccountId} onChange={event => setFromAccountId(event.target.value)} aria-label="From account">
              {state.accounts.map(account => (
                <option key={account.account_id} value={account.account_id}>
                  {(account.account_label || account.display_name || account.email_address)} {account.connection_state === 'receive_only' ? '(receive-only)' : ''}
                </option>
              ))}
            </select>
          </div>
          <EmailChipInput label="To" chips={to} setChips={setTo} input={toInput} setInput={setToInput} />

          {!showCcBcc && (
            <div className="compose-field">
              <span className="compose-field__label" />
              <button style={{ fontSize: 12, color: '#0078d4', padding: '4px 0', background: 'none', border: 'none', cursor: 'pointer' }} onClick={() => setShowCcBcc(true)}>
                + CC / BCC
              </button>
            </div>
          )}

          {showCcBcc && (
            <>
              <EmailChipInput label="CC" chips={cc} setChips={setCc} input={ccInput} setInput={setCcInput} />
              <EmailChipInput label="BCC" chips={bcc} setChips={setBcc} input={bccInput} setInput={setBccInput} />
            </>
          )}

          <div className="compose-field">
            <span className="compose-field__label">Subj</span>
            <input
              className="compose-field__input"
              type="text"
              value={subject}
              onChange={e => setSubject(e.target.value)}
              placeholder="Subject"
              aria-label="Subject"
            />
          </div>
          <div className="compose-field compose-field--compact">
            <span className="compose-field__label" />
            <label className="checkbox-line"><input type="checkbox" checked={includeSignature} onChange={event => setIncludeSignature(event.target.checked)} /> Include default signature</label>
          </div>
        </div>

        <div className="compose-window__body">
          <textarea
            ref={bodyRef}
            value={body}
            onChange={e => setBody(e.target.value)}
            placeholder="Write your message here…"
            aria-label="Message body"
          />
        </div>

        <div className="compose-window__footer">
          <button
            className="btn btn--primary"
            onClick={handleSend}
            disabled={sending || !can('send')}
            title={!can('send') ? 'Send permission required' : 'Send email'}
            aria-label="Send"
          >
            {sending ? <><Spinner size="sm" /> Sending...</> : 'Send'}
          </button>

          <div style={{ position: 'relative' }}>
            <button
              className="btn btn--secondary"
              onClick={() => setScheduleOpen(p => !p)}
              disabled={sending || !can('send')}
              aria-label="Schedule send options"
              title={!can('send') ? 'Send permission required' : 'Schedule send'}
            >
              Schedule
            </button>
            {scheduleOpen && (
              <div style={{ position: 'absolute', bottom: '110%', left: 0, background: '#fff', border: '1px solid #e0e0e0', borderRadius: 4, boxShadow: '0 4px 12px rgba(0,0,0,.12)', minWidth: 180, zIndex: 10 }}>
                <button style={{ display: 'block', width: '100%', padding: '8px 14px', textAlign: 'left', fontSize: 13, cursor: 'pointer', background: 'none', border: 'none' }} onClick={handleScheduleSend}>
                  Tomorrow noon
                </button>
              </div>
            )}
          </div>

          <button
            className="btn btn--secondary"
            onClick={() => saveDraft()}
            disabled={!can('draft')}
            title={!can('draft') ? 'Draft permission required' : 'Save draft'}
            aria-label="Save draft"
          >
            Save draft
          </button>

          <button
            className="btn btn--danger"
            onClick={handleDiscard}
            style={{ marginLeft: 'auto' }}
            aria-label="Discard draft"
            title="Discard draft"
          >
            Discard
          </button>

          <span className="compose-window__save-status">{saveStatus}</span>
        </div>
      </div>
    </div>
  );
}
