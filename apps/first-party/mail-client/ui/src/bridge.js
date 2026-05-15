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
 * If the host bridge is not injected, the packaged UI talks to the host-owned
 * mail REST API. It must never silently render sample mail as if it were the
 * user's inbox.
 */

const hasHostBridge = Boolean(window.__synapp);

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
    'StorePlatformSecret',
    'ProviderCapabilityRead',
    'OAuthConnect',
    'OAuthDisconnect',
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
    account_label: 'Work Mail',
    enabled: true,
    is_default: true,
    sync_interval_minutes: 5,
    color: '#0078d4',
    connection_state: 'connected',
    status: 'active',
    credential_state: 'stored',
    last_sync_at: Math.floor(Date.now() / 1000) - 300,
    unread_count: 4,
    capabilities: ['read', 'draft', 'send', 'organize'],
  },
];

const STUB_IDENTITIES = [
  {
    identity_id: 'id-1',
    account_id: 'acc-1',
    kind: 'primary',
    display_name: 'Johannes',
    email_address: 'johannes@example.com',
    reply_to: '',
    signature_id: 'sig-1',
    enabled: true,
    is_default_for_account: true,
    is_global_default: true,
    verification_state: 'allowed',
  },
];

const STUB_SIGNATURES = [
  {
    signature_id: 'sig-1',
    account_id: 'acc-1',
    identity_id: 'id-1',
    label: 'Default',
    body_text: 'Best regards,\nJohannes',
    enabled: true,
    use_for_new: true,
    use_for_replies: true,
    use_for_forwards: false,
  },
];

const STUB_PREFERENCES = {
  reading: { preview_pane: 'right', thread_view: true, focused_inbox: true, page_size: 50, list_density: 'comfortable', remote_content: 'ask' },
  compose: { undo_send_delay_seconds: 10, default_format: 'html', default_sender_strategy: 'last_used', reply_quote_mode: 'collapsed', forward_attachments_default: false },
  notifications: { enabled: true, per_account: { 'acc-1': true } },
  sync_defaults: { interval_minutes: 5, initial_range: '30_days', download_attachments: 'metadata_only', offline_cache: false },
};

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
  get_provider_capabilities: () => ({ ok: { operation: 'get_provider_capabilities', status: 'accepted', data: { providers: [{ provider: 'google_oauth', label: 'Google Workspace', status: 'placeholder' }, { provider: 'microsoft_oauth', label: 'Microsoft 365', status: 'placeholder' }], manual_imap_smtp: { enabled: true, security_modes: ['ssl_tls', 'starttls'], default_imap_port: 993, default_smtp_port: 587 }, policy: { allow_receive_only: true, min_sync_interval_minutes: 5, max_recipients: 100 }, oauth_placeholders: ['google_oauth', 'microsoft_oauth'] }, host_effects: [], warnings: [] } }),
  validate_account_setup: (payload) => ({ ok: { operation: 'validate_account_setup', status: 'accepted', data: { valid: Boolean(payload.account?.email_address), normalized_account: payload.account ? { ...payload.account, incoming: payload.account.incoming ? { ...payload.account.incoming, secret_input_ref: undefined, secret_ref: undefined } : undefined, outgoing: payload.account.outgoing ? { ...payload.account.outgoing, secret_input_ref: undefined, secret_ref: undefined } : undefined, oauth: payload.account.oauth ? { ...payload.account.oauth, connection_ref: undefined, redirect_state_ref: undefined } : undefined } : undefined, field_errors: [], warnings: [], next_required_action: 'test_connection' }, host_effects: [], warnings: [] } }),
  plan_connection_test: (payload) => ({ ok: { operation: 'plan_connection_test', status: 'accepted', data: { account_id: payload.account?.account_id || 'new-account', test_id: `test-${Date.now()}`, requires_host_execution: true, steps: ['imap_auth', 'imap_mailbox_discovery', 'smtp_auth', 'smtp_send_capability', 'folder_mapping'] }, host_effects: [], warnings: [] } }),
  complete_account_setup: (payload) => ({ ok: { operation: 'complete_account_setup', status: 'accepted', data: { account: { account_id: payload.account?.account_id || `acc-${Date.now()}`, account_label: payload.account?.account_label, display_name: payload.account?.display_name, email_address: payload.account?.email_address, provider: payload.account?.provider || 'manual_imap_smtp', enabled: payload.save_mode !== 'disabled', is_default: Boolean(payload.make_default), status: payload.save_mode || 'active', connection_state: payload.save_mode === 'receive_only' ? 'receive_only' : 'connected', credential_state: 'stored' }, status: payload.save_mode || 'active', requires_reconnect: false, requires_sync: true }, host_effects: [], warnings: [] } }),
  begin_oauth_account_setup: (payload) => ({ ok: { operation: 'begin_oauth_account_setup', status: 'accepted', data: { provider: payload.provider, authorization_ref_present: true, status: 'host_action_required', next_route: '/settings/accounts' }, host_effects: [], warnings: [] } }),
  complete_oauth_account_setup: (payload) => ({ ok: { operation: 'complete_oauth_account_setup', status: 'accepted', data: { account: { account_id: `acc-${Date.now()}`, provider: payload.provider, status: 'active', connection_state: 'connected' }, identity: STUB_IDENTITIES[0], requires_sync: true }, host_effects: [], warnings: [] } }),
  disconnect_oauth_account: (payload) => ({ ok: { operation: 'disconnect_oauth_account', status: 'accepted', data: { account_id: payload.account_id, credential_state: 'revoked', disabled: Boolean(payload.disable_account) }, host_effects: [], warnings: [] } }),
  remove_account: (payload) => ({ ok: { operation: 'remove_account', status: 'accepted', data: { account_id: payload.account_id, status: 'removed_pending_cleanup', cleanup_planned: ['local_cache', 'scheduled_sends'] }, host_effects: [], warnings: [] } }),
  list_identities: () => ({ ok: { operation: 'list_identities', status: 'accepted', data: { identities: STUB_IDENTITIES, signatures: STUB_SIGNATURES, needs_host_refresh: false }, host_effects: [], warnings: [] } }),
  save_identity: (payload) => ({ ok: { operation: 'save_identity', status: 'accepted', data: { identity: payload.identity, saved: true }, host_effects: [], warnings: [] } }),
  delete_identity: (payload) => ({ ok: { operation: 'delete_identity', status: 'accepted', data: { identity_id: payload.identity_id, deleted: true }, host_effects: [], warnings: [] } }),
  save_signature: (payload) => ({ ok: { operation: 'save_signature', status: 'accepted', data: { signature: payload.signature, saved: true }, host_effects: [], warnings: [] } }),
  delete_signature: (payload) => ({ ok: { operation: 'delete_signature', status: 'accepted', data: { signature_id: payload.signature_id, deleted: true }, host_effects: [], warnings: [] } }),
  get_user_preferences: () => ({ ok: { operation: 'get_user_preferences', status: 'accepted', data: { preferences: STUB_PREFERENCES, defaults_applied: true }, host_effects: [], warnings: [] } }),
  save_user_preferences: (payload) => ({ ok: { operation: 'save_user_preferences', status: 'accepted', data: { preferences: payload.preferences, saved: true }, host_effects: [], warnings: [] } }),
  inspect_legacy_settings: () => ({ ok: { operation: 'inspect_legacy_settings', status: 'accepted', data: { found_legacy_main_schema: true, accounts_requiring_reconnect: 0, migration_warnings: ['Legacy main view settings are now reading preferences.'] }, host_effects: [], warnings: [] } }),
  migrate_legacy_settings: () => ({ ok: { operation: 'migrate_legacy_settings', status: 'accepted', data: { migrated: true, accounts_requiring_reconnect: [], removed_keys: ['ui_schemas.main'] }, host_effects: [], warnings: [] } }),
};

// ─── Host REST bridge ────────────────────────────────────────────────────────

const REST_CONTEXT = {
  user_id: 'current-user',
  permission: 'admin',
  available_effects: [
    'AccountCredentialRead',
    'AccountCredentialWrite',
    'ProviderCapabilityRead',
    'ImapFetch',
    'ImapSync',
    'SmtpSend',
    'MailStoreRead',
    'MailStoreWrite',
    'MailStoreDelete',
  ],
  agent_id: null,
  request_id: null,
};

let lastMailboxSnapshot = { messages: [], total_count: 0, unread_count: 0 };
const draftStore = new Map();

function getAccessToken() {
  if (typeof window.__synappAccessToken === 'string') return window.__synappAccessToken;
  if (typeof window.__SYNAPP_ACCESS_TOKEN__ === 'string') return window.__SYNAPP_ACCESS_TOKEN__;
  const meta = document.querySelector('meta[name="synapp-access-token"]');
  return meta?.getAttribute('content') || '';
}

async function apiJson(path, init = {}) {
  const headers = new Headers(init.headers);
  const token = getAccessToken();
  if (token) headers.set('Authorization', `Bearer ${token}`);
  if (init.body && !headers.has('Content-Type')) headers.set('Content-Type', 'application/json');
  const response = await fetch(path, { ...init, headers, credentials: 'same-origin' });
  if (!response.ok) {
    const body = await response.text();
    throw new Error(apiErrorMessage(response.status, body));
  }
  return response.json();
}

function apiErrorMessage(status, body) {
  try {
    const parsed = JSON.parse(body);
    if (typeof parsed.error === 'string') return parsed.error;
    if (typeof parsed.message === 'string') return parsed.message;
  } catch (_) {
    // Body is plain text or empty.
  }
  if (status === 401) return 'Sign in through Synapp before opening the mail app.';
  if (status === 403) return 'Your session is missing the required mail permission.';
  return body || `Mail API request failed with HTTP ${status}`;
}

function ok(operation, data) {
  return { operation, status: 'accepted', data, host_effects: [], warnings: [] };
}

function normalizeAccounts(accounts) {
  return accounts.map((account) => ({
    ...account,
    provider: account.provider || 'manual_imap_smtp',
    connection_state: account.connection_state?.account_state || account.status || 'unknown',
    connection_result: account.connection_state,
    enabled: account.status !== 'blocked',
    sync_interval_minutes: account.sync_interval_minutes || 15,
    color: account.color || '#0078d4',
    last_sync_at: account.last_sync_at || (account.last_test_at ? Math.floor(Date.parse(account.last_test_at) / 1000) : 0),
    unread_count: account.unread_count || 0,
  }));
}

function normalizeMessage(message) {
  const timestamp = message.date_iso ? Math.floor(Date.parse(message.date_iso) / 1000) : 0;
  return {
    email_id: message.id || message.uid || message.message_id,
    account_id: message.account_id,
    mailbox_id: String(message.mailbox || 'INBOX').toLowerCase(),
    thread_id: message.message_id || message.id || message.uid,
    from: { email: message.from?.email || '', display_name: message.from?.name || message.from?.email || '' },
    to: (message.to || []).map((address) => ({ email: address.email || '', display_name: address.name || address.email || '' })),
    cc: [],
    bcc: [],
    subject: message.subject || '(no subject)',
    preview: message.snippet || message.body_text || '',
    body_html: message.body_html || '',
    body_text: message.body_text || message.snippet || '',
    received_at: timestamp,
    sent_at: 0,
    is_read: true,
    is_flagged: false,
    has_attachments: Boolean(message.has_attachments),
    importance: 'normal',
    categories: [],
    attachments: [],
  };
}

function accountPayload(input) {
  const account = input.account || input;
  return {
    account_id: account.account_id,
    account_label: account.account_label || account.email_address,
    display_name: account.display_name || account.email_address,
    email_address: account.email_address,
    reply_to: account.reply_to,
    incoming: account.incoming,
    outgoing: account.outgoing,
    password: account.password || '',
    make_default: Boolean(input.make_default),
    send_test_message: Boolean(account.send_test_message || input.send_test_message),
  };
}

function validateAccountPayload(payload) {
  const errors = [];
  if (!payload.email_address) errors.push('Email address is required.');
  if (!payload.incoming?.host) errors.push('IMAP host is required.');
  if (!payload.outgoing?.host) errors.push('SMTP host is required.');
  if (!payload.password) errors.push('Password or app password is required.');
  if (errors.length) throw new Error(errors.join(' '));
}

const REST_OPERATIONS = {
  async list_accounts() {
    const data = await apiJson('/api/v1/mail/accounts');
    const accounts = normalizeAccounts(data.accounts || []);
    return ok('list_accounts', { snapshot: accounts, total_count: accounts.length, refresh_requested: true });
  },
  async get_provider_capabilities() {
    return ok('get_provider_capabilities', {
      providers: [],
      manual_imap_smtp: { enabled: true, security_modes: ['ssl_tls', 'starttls'], default_imap_port: 993, default_smtp_port: 587 },
      policy: { allow_receive_only: true, min_sync_interval_minutes: 5, max_recipients: 100 },
      oauth_placeholders: [],
    });
  },
  async validate_account_setup(payload) {
    const account = accountPayload(payload);
    validateAccountPayload(account);
    return ok('validate_account_setup', { valid: true, normalized_account: account, field_errors: [], warnings: [], next_required_action: 'test_connection' });
  },
  async plan_connection_test(payload) {
    const account = accountPayload(payload);
    validateAccountPayload(account);
    const data = await apiJson('/api/v1/mail/accounts/test', { method: 'POST', body: JSON.stringify(account) });
    return ok('plan_connection_test', {
      account_id: account.account_id || account.email_address,
      test_id: data.result?.tested_at || new Date().toISOString(),
      requires_host_execution: false,
      steps: ['imap_auth', 'imap_mailbox_discovery', 'smtp_auth'],
      incoming: data.result?.incoming?.state || 'not_tested',
      outgoing: data.result?.outgoing?.state || 'not_tested',
      folders: data.result?.incoming?.state === 'passed' ? 'passed' : 'not_tested',
      message: data.result?.incoming?.message || data.result?.outgoing?.message || 'Connection test completed by Synapp host.',
      result: data.result,
    });
  },
  async draft_email(payload) {
    const draft = payload.draft || {};
    const draftId = draft.draft_id || `draft-${crypto.randomUUID?.() || Date.now()}`;
    draftStore.set(draftId, { ...draft, draft_id: draftId });
    return ok('draft_email', { draft: draftStore.get(draftId), suggested_draft_id: draftId });
  },
  async update_draft(payload) {
    const draft = payload.draft || {};
    const draftId = draft.draft_id || `draft-${crypto.randomUUID?.() || Date.now()}`;
    draftStore.set(draftId, { ...draft, draft_id: draftId });
    return ok('update_draft', { draft: draftStore.get(draftId), suggested_draft_id: draftId });
  },
  async discard_draft(payload) {
    draftStore.delete(payload.draft_id);
    return ok('discard_draft', { mutation: 'discard_draft', resource_ids: [payload.draft_id], applied: true });
  },
  async complete_account_setup(payload) {
    const account = accountPayload(payload);
    validateAccountPayload(account);
    const data = await apiJson('/api/v1/mail/accounts', { method: 'POST', body: JSON.stringify(account) });
    return ok('complete_account_setup', {
      account: normalizeAccounts([data.account])[0],
      status: data.account?.status || 'active',
      requires_reconnect: false,
      requires_sync: true,
      result: data.result,
    });
  },
  async list_mailboxes(payload) {
    const accountId = payload.account_id;
    return ok('list_mailboxes', {
      snapshot: [
        { mailbox_id: 'inbox', account_id: accountId, name: 'INBOX', kind: 'inbox', unread_count: 0, total_count: lastMailboxSnapshot.total_count, favorite: true },
      ],
      total_count: 1,
      refresh_requested: true,
    });
  },
  async read_emails(payload) {
    const mailbox = (payload.mailbox_id || 'INBOX').toUpperCase();
    const params = new URLSearchParams({ mailbox, max_count: String(payload.pagination?.limit || 50) });
    if (payload.account_id) params.set('account_id', payload.account_id);
    const data = await apiJson(`/api/v1/mail/messages?${params.toString()}`);
    const messages = (data.messages || []).map(normalizeMessage);
    lastMailboxSnapshot = { messages, total_count: messages.length, unread_count: 0 };
    return ok('read_emails', { snapshot: lastMailboxSnapshot, needs_host_refresh: false });
  },
  async get_email(payload) {
    const email = lastMailboxSnapshot.messages.find((message) => message.email_id === payload.email_id);
    if (!email) throw new Error('Message is not loaded. Refresh the mailbox and try again.');
    return ok('get_email', email);
  },
  async get_thread(payload) {
    const messages = lastMailboxSnapshot.messages.filter((message) => message.thread_id === payload.thread_id);
    return ok('get_thread', { thread_id: payload.thread_id, messages, needs_host_refresh: false });
  },
  async search_emails(payload) {
    const query = String(payload.query || '').toLowerCase();
    const results = lastMailboxSnapshot.messages.filter((message) => `${message.subject} ${message.preview} ${message.body_text} ${message.from.email}`.toLowerCase().includes(query));
    return ok('search_emails', { query: payload.query, snapshot: { results, total_count: results.length }, needs_host_refresh: false });
  },
  async send_email(payload) {
    const draft = draftStore.get(payload.draft_id) || payload.draft || payload;
    const data = await apiJson('/api/v1/mail/send', {
      method: 'POST',
      body: JSON.stringify({
        account_id: payload.account_id || draft.account_id,
        to: (draft.to || []).map((recipient) => recipient.email || recipient).join(','),
        cc: (draft.cc || []).map((recipient) => recipient.email || recipient).join(','),
        bcc: (draft.bcc || []).map((recipient) => recipient.email || recipient).join(','),
        subject: draft.subject || '',
        body_text: draft.body_text || '',
        body_html: draft.body_html || '',
      }),
    });
    if (payload.draft_id) draftStore.delete(payload.draft_id);
    return ok('send_email', { draft_id: payload.draft_id || '', account_id: payload.account_id, undo_window_seconds: 0, notify: true, message_id: data.message_id });
  },
};

// ─── Public API ───────────────────────────────────────────────────────────────

export function getContext() {
  if (hasHostBridge) return window.__synapp.getContext();
  return REST_CONTEXT;
}

/**
 * Invoke a Wasm operation through the Synapp host bridge.
 * Returns the unwrapped data on success, throws an Error with code + message on failure.
 */
export async function invoke(operation, payload = {}) {
  const context = getContext();
  const fullPayload = { context, ...payload };

  let raw;
  if (hasHostBridge) {
    raw = await window.__synapp.invoke(operation, fullPayload);
  } else if (REST_OPERATIONS[operation]) {
    raw = { ok: await REST_OPERATIONS[operation](fullPayload) };
  } else {
    throw new Error(`Operation ${operation} requires Synapp host bridge support.`);
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

export const PERMISSION_MATRIX = {
  none: ['none'],
  read: ['none', 'read'],
  draft: ['none', 'read', 'draft'],
  send: ['none', 'read', 'draft', 'send'],
  organize: ['none', 'read', 'organize'],
  accounts: ['none', 'read', 'accounts'],
  admin: ['none', 'read', 'draft', 'send', 'organize', 'accounts', 'admin'],
};

export function hasPermission(userPermission, required) {
  return Boolean(PERMISSION_MATRIX[userPermission]?.includes(required));
}
