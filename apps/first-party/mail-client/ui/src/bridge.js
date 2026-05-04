/**
 * Synapp Host Bridge
 *
 * The Synapp host injects window.__synapp at runtime, providing:
 *   - window.__synapp.invoke(operation, payload)  → Promise<{ok}|{err}>
 *   - window.__synapp.getContext()                → { user_id, permission, available_effects, agent_id }
 *
 * This module wraps those calls and provides a typed, error-normalised API
 * for the React frontend. All Wasm operations are dispatched through invoke().
 *
 * During local development (no host present), a stub implementation is used
 * so the UI can render with sample data.
 */

const isDev = !window.__synapp;

// ─── Development stub ────────────────────────────────────────────────────────

const STUB_CONTEXT = {
  user_id: 'dev-user-001',
  permission: 'send',
  available_effects: [
    'AccountCredentialRead',
    'ImapFetch',
    'ImapSync',
    'SmtpSend',
    'ContactLookup',
    'NotifyUser',
    'ScheduleJob',
    'CancelJob',
    'MailStoreRead',
    'MailStoreWrite',
    'MailStoreDelete',
  ],
  agent_id: null,
  request_id: null,
};

const STUB_ACCOUNTS = [
  {
    account_id: 'acc-1',
    display_name: 'Johannes',
    email_address: 'johannes@example.com',
    provider: 'imap',
    enabled: true,
    is_default: true,
    sync_interval_minutes: 5,
    color: '#0078d4',
    connection_state: 'connected',
  },
];

const STUB_MAILBOXES = [
  { mailbox_id: 'inbox', account_id: 'acc-1', name: 'Inbox', kind: 'inbox', unread_count: 3, total_count: 42, favorite: true },
  { mailbox_id: 'drafts', account_id: 'acc-1', name: 'Drafts', kind: 'drafts', unread_count: 0, total_count: 2, favorite: false },
  { mailbox_id: 'sent', account_id: 'acc-1', name: 'Sent', kind: 'sent', unread_count: 0, total_count: 88, favorite: false },
  { mailbox_id: 'archive', account_id: 'acc-1', name: 'Archive', kind: 'archive', unread_count: 0, total_count: 312, favorite: false },
  { mailbox_id: 'trash', account_id: 'acc-1', name: 'Trash', kind: 'trash', unread_count: 0, total_count: 5, favorite: false },
  { mailbox_id: 'junk', account_id: 'acc-1', name: 'Junk Email', kind: 'junk', unread_count: 1, total_count: 7, favorite: false },
  { mailbox_id: 'projects', account_id: 'acc-1', name: 'Projects', kind: 'custom', unread_count: 0, total_count: 14, favorite: true },
];

function makeEmail(id, from, subject, preview, received, read = false, mailboxId = 'inbox', importance = 'normal') {
  return {
    email_id: `email-${id}`,
    account_id: 'acc-1',
    mailbox_id: mailboxId,
    thread_id: `thread-${id}`,
    from: { email: from, display_name: from.split('@')[0].replace('.', ' ') },
    to: [{ email: 'johannes@example.com', display_name: 'Johannes' }],
    cc: [],
    bcc: [],
    subject,
    preview,
    body_html: `<p>${preview}</p><p>Best regards,<br/>${from.split('@')[0]}</p>`,
    body_text: preview,
    received_at: received,
    sent_at: 0,
    is_read: read,
    is_flagged: false,
    has_attachments: id % 3 === 0,
    importance,
    categories: [],
    attachments: id % 3 === 0 ? [{ attachment_id: `att-${id}`, file_name: 'document.pdf', content_type: 'application/pdf', size_bytes: 245760 }] : [],
  };
}

const now = Math.floor(Date.now() / 1000);
const STUB_EMAILS = [
  makeEmail(1, 'alice.chen@company.com', 'Q2 Budget Review — action required', 'Hi team, please review the attached Q2 budget numbers before our Friday meeting. Key points: marketing spend is up 18%...', now - 3600, false, 'inbox', 'high'),
  makeEmail(2, 'bob.smith@company.com', 'Re: Sprint planning notes', 'Thanks for sharing the notes. I think we should push the analytics feature to next sprint given the current...', now - 7200, true, 'inbox'),
  makeEmail(3, 'carol@example.org', 'Invitation: Team offsite — July 14–16', 'You are invited to the annual team offsite. Please RSVP by June 1st. Agenda includes workshops, strategy sessions...', now - 86400, false, 'inbox'),
  makeEmail(4, 'noreply@github.com', '[GitHub] Pull request review requested', 'bob.smith requested your review on PR #142: Refactor authentication middleware. Changes affect 12 files...', now - 86400 * 2, true, 'inbox'),
  makeEmail(5, 'alice.chen@company.com', 'Re: Q2 Budget Review', 'Thanks for the quick turnaround. I have updated the spreadsheet with the revised numbers. Please check...', now - 86400 * 2, true, 'inbox'),
  makeEmail(6, 'david@partner.io', 'Partnership proposal — follow-up', 'Following up on our discussion last week. I have put together a detailed proposal document which I am attaching...', now - 86400 * 3, false, 'inbox', 'high'),
  makeEmail(7, 'newsletter@medium.com', 'Your weekly digest: 12 new stories', 'This week in technology: AI advancements, open source highlights, and startup funding roundup...', now - 86400 * 4, true, 'inbox', 'low'),
  makeEmail(8, 'carol@example.org', 'Re: Invitation: Team offsite', 'Confirmed — I will be there! Looking forward to it. Do you need me to organise the transport for the group?...', now - 86400 * 5, true, 'inbox'),
];

const STUB_RESPONSES = {
  list_accounts: () => ({ ok: { operation: 'list_accounts', status: 'accepted', data: { snapshot: STUB_ACCOUNTS, total_count: STUB_ACCOUNTS.length, refresh_requested: false }, host_effects: [], warnings: [] } }),
  list_mailboxes: () => ({ ok: { operation: 'list_mailboxes', status: 'accepted', data: { snapshot: STUB_MAILBOXES, total_count: STUB_MAILBOXES.length, refresh_requested: false }, host_effects: [], warnings: [] } }),
  read_emails: (payload) => {
    const filtered = STUB_EMAILS.filter(e => e.mailbox_id === (payload.mailbox_id || 'inbox'));
    return { ok: { operation: 'read_emails', status: 'accepted', data: { snapshot: { messages: filtered, total_count: filtered.length, unread_count: filtered.filter(e => !e.is_read).length }, needs_host_refresh: false }, host_effects: [], warnings: [] } };
  },
  get_email: (payload) => {
    const email = STUB_EMAILS.find(e => e.email_id === payload.email_id);
    return email
      ? { ok: { operation: 'get_email', status: 'accepted', data: email, host_effects: [], warnings: [] } }
      : { err: { code: 'EmailNotFound', message: 'Email not found' } };
  },
  get_thread: (payload) => {
    const msgs = STUB_EMAILS.filter(e => e.thread_id === payload.thread_id);
    return { ok: { operation: 'get_thread', status: 'accepted', data: { thread_id: payload.thread_id, messages: msgs, needs_host_refresh: false }, host_effects: [], warnings: [] } };
  },
  get_unread_count: () => ({ ok: { operation: 'get_unread_count', status: 'accepted', data: { counts: STUB_MAILBOXES.map(m => ({ mailbox_id: m.mailbox_id, unread_count: m.unread_count })), needs_host_refresh: false }, host_effects: [], warnings: [] } }),
  search_emails: (payload) => {
    const q = payload.query.toLowerCase();
    const results = STUB_EMAILS.filter(e => e.subject.toLowerCase().includes(q) || e.preview.toLowerCase().includes(q) || e.from.email.toLowerCase().includes(q));
    return { ok: { operation: 'search_emails', status: 'accepted', data: { query: payload.query, snapshot: { results, total_count: results.length }, needs_host_refresh: false }, host_effects: [], warnings: [] } };
  },
  mark_read: () => ({ ok: { operation: 'mark_read', status: 'accepted', data: { mutation: 'mark_read', resource_ids: [], applied: true }, host_effects: [], warnings: [] } }),
  flag_email: () => ({ ok: { operation: 'flag_email', status: 'accepted', data: { mutation: 'flag_email', resource_ids: [], applied: true }, host_effects: [], warnings: [] } }),
  draft_email: (payload) => ({ ok: { operation: 'draft_email', status: 'accepted', data: { draft: payload.draft, suggested_draft_id: `draft-${Date.now()}` }, host_effects: [], warnings: [] } }),
  update_draft: (payload) => ({ ok: { operation: 'update_draft', status: 'accepted', data: { draft: payload.draft, suggested_draft_id: payload.draft.draft_id }, host_effects: [], warnings: [] } }),
  discard_draft: () => ({ ok: { operation: 'discard_draft', status: 'accepted', data: { mutation: 'discard_draft', resource_ids: [], applied: true }, host_effects: [], warnings: [] } }),
  send_email: () => ({ ok: { operation: 'send_email', status: 'accepted', data: { draft_id: '', account_id: '', undo_window_seconds: 10, notify: true }, host_effects: [], warnings: [] } }),
  reply_email: () => ({ ok: { operation: 'reply_email', status: 'accepted', data: { email_id: '', account_id: '', send_now: false, undo_window_seconds: 10 }, host_effects: [], warnings: [] } }),
  forward_email: () => ({ ok: { operation: 'forward_email', status: 'accepted', data: { email_id: '', account_id: '', recipient_count: 1, send_now: false, undo_window_seconds: 10 }, host_effects: [], warnings: [] } }),
  move_email: () => ({ ok: { operation: 'move_email', status: 'accepted', data: { mutation: 'move_email', resource_ids: [], applied: true }, host_effects: [], warnings: [] } }),
  delete_email: () => ({ ok: { operation: 'delete_email', status: 'accepted', data: { mutation: 'delete_email', resource_ids: [], applied: true }, host_effects: [], warnings: [] } }),
  archive_email: () => ({ ok: { operation: 'archive_email', status: 'accepted', data: { mutation: 'archive_email', resource_ids: [], applied: true }, host_effects: [], warnings: [] } }),
  create_folder: (payload) => ({ ok: { operation: 'create_folder', status: 'accepted', data: { mailbox_id: `custom-${Date.now()}`, account_id: payload.account_id, name: payload.name }, host_effects: [], warnings: [] } }),
  rename_folder: () => ({ ok: { operation: 'rename_folder', status: 'accepted', data: { mutation: 'rename_folder', resource_ids: [], applied: true }, host_effects: [], warnings: [] } }),
  delete_folder: () => ({ ok: { operation: 'delete_folder', status: 'accepted', data: { mutation: 'delete_folder', resource_ids: [], applied: true }, host_effects: [], warnings: [] } }),
  list_rules: () => ({ ok: { operation: 'list_rules', status: 'accepted', data: { snapshot: [], total_count: 0, refresh_requested: false }, host_effects: [], warnings: [] } }),
  create_rule: () => ({ ok: { operation: 'create_rule', status: 'accepted', data: { mutation: 'create_rule', resource_ids: [], applied: true }, host_effects: [], warnings: [] } }),
  delete_rule: () => ({ ok: { operation: 'delete_rule', status: 'accepted', data: { mutation: 'delete_rule', resource_ids: [], applied: true }, host_effects: [], warnings: [] } }),
  schedule_send: () => ({ ok: { operation: 'schedule_send', status: 'accepted', data: { job_id: `job-${Date.now()}`, draft_id: '', account_id: '', scheduled_for: 0, undo_window_seconds: 10 }, host_effects: [], warnings: [] } }),
  cancel_scheduled_send: () => ({ ok: { operation: 'cancel_scheduled_send', status: 'accepted', data: { mutation: 'cancel_scheduled_send', resource_ids: [], applied: true }, host_effects: [], warnings: [] } }),
  get_account_status: () => ({ ok: { operation: 'get_account_status', status: 'accepted', data: { account_id: 'acc-1', connection_state: 'connected', last_sync_at: now - 300, unread_count: 4 }, host_effects: [], warnings: [] } }),
};

// ─── Public API ───────────────────────────────────────────────────────────────

export function getContext() {
  if (isDev) return STUB_CONTEXT;
  return window.__synapp.getContext();
}

/**
 * Invoke a Wasm operation through the Synapp host bridge.
 * Returns the unwrapped data on success, throws an Error with code + message on failure.
 */
export async function invoke(operation, payload = {}) {
  const context = getContext();
  const fullPayload = { context, ...payload };

  let raw;
  if (isDev) {
    const handler = STUB_RESPONSES[operation];
    if (!handler) throw new Error(`Unknown operation: ${operation}`);
    raw = handler(fullPayload);
  } else {
    raw = await window.__synapp.invoke(operation, fullPayload);
  }

  if (raw && raw.err) {
    const e = new Error(raw.err.message || raw.err.code);
    e.code = raw.err.code;
    e.details = raw.err.details;
    throw e;
  }
  if (raw && raw.ok) return raw.ok;
  throw new Error('Unexpected response format from host bridge');
}

export const PERMISSION_LEVELS = { none: 0, read: 1, draft: 2, send: 3, organize: 4 };

export function hasPermission(userPermission, required) {
  return (PERMISSION_LEVELS[userPermission] ?? 0) >= (PERMISSION_LEVELS[required] ?? 0);
}
