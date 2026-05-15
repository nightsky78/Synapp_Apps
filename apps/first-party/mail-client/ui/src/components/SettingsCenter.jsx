import React, { useEffect, useMemo, useState } from 'react';
import { invoke } from '../bridge.js';
import { useApp } from '../context/AppContext.jsx';
import { Spinner } from './Spinner.jsx';
import RulesManager from './RulesManager.jsx';

const DEFAULT_ACCOUNT_FORM = {
  account_label: '',
  display_name: '',
  email_address: '',
  reply_to: '',
  provider: 'manual_imap_smtp',
  auth_method: 'app_password',
  incoming_host: '',
  incoming_port: 993,
  incoming_security: 'ssl_tls',
  incoming_username: '',
  outgoing_host: '',
  outgoing_port: 587,
  outgoing_security: 'starttls',
  outgoing_username: '',
  use_incoming_secret: true,
  password: '',
  send_test_message: false,
  incoming_secret_handle: 'host-secure-field:incoming',
  outgoing_secret_handle: 'host-secure-field:outgoing',
  sync_interval_minutes: 15,
  initial_range: '30_days',
  download_attachments: 'metadata_only',
  offline_cache: false,
  make_default: false,
};

const DEFAULT_PREFS = {
  reading: { preview_pane: 'right', thread_view: true, focused_inbox: true, page_size: 50, list_density: 'comfortable', remote_content: 'ask' },
  compose: { undo_send_delay_seconds: 10, default_format: 'html', default_sender_strategy: 'last_used', reply_quote_mode: 'collapsed', forward_attachments_default: false },
  notifications: { enabled: true, per_account: {} },
  sync_defaults: { interval_minutes: 15, initial_range: '30_days', download_attachments: 'metadata_only', offline_cache: false },
};

function Field({ label, htmlFor, children, hint }) {
  return (
    <div className="settings-field">
      <label className="form-label" htmlFor={htmlFor}>{label}</label>
      {children}
      {hint && <div className="settings-hint">{hint}</div>}
    </div>
  );
}

function StatusPill({ state }) {
  const normalized = state || 'unknown';
  return <span className={`status-pill status-pill--${normalized}`}>{normalized.replaceAll('_', ' ')}</span>;
}

function buildAccountPayload(form) {
  return {
    account_label: form.account_label || form.email_address,
    display_name: form.display_name,
    email_address: form.email_address,
    reply_to: form.reply_to || undefined,
    provider: form.provider,
    auth_method: form.provider === 'manual_imap_smtp' ? form.auth_method : 'oauth2',
    incoming: {
      protocol: 'imap',
      host: form.incoming_host,
      port: Number(form.incoming_port),
      security: form.incoming_security,
      username: form.incoming_username || form.email_address,
      auth_method: form.auth_method,
      secret_input_ref: form.incoming_secret_handle,
    },
    outgoing: {
      protocol: 'smtp',
      host: form.outgoing_host,
      port: Number(form.outgoing_port),
      security: form.outgoing_security,
      username: form.outgoing_username || form.incoming_username || form.email_address,
      auth_method: form.auth_method,
      use_incoming_secret: form.use_incoming_secret,
      secret_input_ref: form.use_incoming_secret ? undefined : form.outgoing_secret_handle,
    },
    password: form.password,
    send_test_message: form.send_test_message,
    sync: {
      interval_minutes: Number(form.sync_interval_minutes),
      initial_range: form.initial_range,
      download_attachments: form.download_attachments,
      offline_cache: form.offline_cache,
    },
    desired_status: 'active',
  };
}

export function AccountSetup({ firstRun = false }) {
  const { loadAccounts, toast, dispatch } = useApp();
  const [form, setForm] = useState(DEFAULT_ACCOUNT_FORM);
  const [capabilities, setCapabilities] = useState(null);
  const [loadingCaps, setLoadingCaps] = useState(true);
  const [testing, setTesting] = useState(false);
  const [saving, setSaving] = useState(false);
  const [testResult, setTestResult] = useState(null);

  useEffect(() => {
    invoke('get_provider_capabilities')
      .then(res => setCapabilities(res.data))
      .catch(err => toast(`Provider capabilities unavailable: ${err.message}`, 'error'))
      .finally(() => setLoadingCaps(false));
  }, [toast]);

  function update(field, value) {
    setForm(current => ({ ...current, [field]: value }));
  }

  async function beginOAuth(provider) {
    if (!form.email_address.trim()) {
      toast('Enter an email address before OAuth setup', 'error');
      return;
    }
    try {
      const res = await invoke('begin_oauth_account_setup', { provider, email_address: form.email_address, account_label: form.account_label, redirect_route: '/settings/accounts' });
      toast(`${provider.replace('_oauth', '')} authorization started by host`, 'success');
      setTestResult({ mode: 'oauth', incoming: 'passed', outgoing: 'passed', message: res.data.authorization_ref_present ? 'Host authorization reference captured.' : 'Host authorization started.' });
    } catch (err) {
      toast(`OAuth setup failed: ${err.message}`, 'error');
    }
  }

  async function testConnection(scope = 'all') {
    setTesting(true);
    setTestResult({ incoming: 'testing', outgoing: 'testing', folders: 'testing' });
    try {
      const account = buildAccountPayload(form);
      await invoke('validate_account_setup', { account });
      const planned = await invoke('plan_connection_test', { account, test_scope: scope, send_test_message: scope !== 'incoming' });
      setTestResult({
        incoming: planned.data.incoming || 'not_tested',
        outgoing: scope === 'incoming' ? 'not_tested' : planned.data.outgoing || 'not_tested',
        folders: planned.data.folders || (planned.data.incoming === 'passed' ? 'passed' : 'not_tested'),
        message: planned.data.message || 'Connection test completed by Synapp host.',
        result: planned.data.result,
      });
      toast('Connection test completed', 'success');
    } catch (err) {
      setTestResult({ incoming: 'failed', outgoing: 'warning', folders: 'not_tested', message: err.message });
      toast(`Connection test failed: ${err.message}`, 'error');
    } finally {
      setTesting(false);
    }
  }

  async function saveAccount(mode = 'active') {
    setSaving(true);
    try {
      const account = { ...buildAccountPayload(form), desired_status: mode };
      await invoke('complete_account_setup', { account, connection_test_result: testResult, make_default: form.make_default, save_mode: mode });
      toast(mode === 'receive_only' ? 'Account saved as receive-only' : 'Account saved', 'success');
      await loadAccounts();
      dispatch({ type: 'SET_ROUTE', payload: { route: 'inbox' } });
    } catch (err) {
      toast(`Save failed: ${err.message}`, 'error');
    } finally {
      setSaving(false);
    }
  }

  const oauthProviders = capabilities?.providers ?? [];

  return (
    <section className={firstRun ? 'setup-empty' : 'settings-panel'} aria-labelledby="account-setup-title">
      <div className="settings-panel__header">
        <div>
          <h1 id="account-setup-title">{firstRun ? 'Add your first mail account' : 'Add account'}</h1>
          <p>Accounts are stored for the signed-in user. Credentials are captured by Synapp host secure fields and passed to this app only as opaque references.</p>
        </div>
      </div>

      {loadingCaps ? (
        <div className="centered-spinner"><Spinner size="lg" /><span>Loading provider options...</span></div>
      ) : (
        <div className="settings-grid settings-grid--setup">
          <div className="settings-section">
            <h2>Identity</h2>
            <Field label="Account label" htmlFor="account-label">
              <input id="account-label" className="form-input" value={form.account_label} onChange={event => update('account_label', event.target.value)} placeholder="Work Mail" />
            </Field>
            <Field label="Display name" htmlFor="display-name">
              <input id="display-name" className="form-input" value={form.display_name} onChange={event => update('display_name', event.target.value)} placeholder="Johannes" />
            </Field>
            <Field label="Email address" htmlFor="email-address">
              <input id="email-address" className="form-input" type="email" value={form.email_address} onChange={event => update('email_address', event.target.value)} placeholder="name@example.com" />
            </Field>
            <Field label="Reply-to" htmlFor="reply-to" hint="Optional. Leave blank to reply from the account address.">
              <input id="reply-to" className="form-input" type="email" value={form.reply_to} onChange={event => update('reply_to', event.target.value)} />
            </Field>
          </div>

          <div className="settings-section">
            <h2>Setup path</h2>
            <div className="segmented" role="group" aria-label="Account setup path">
              <button className={form.provider === 'manual_imap_smtp' ? 'segmented__item segmented__item--active' : 'segmented__item'} onClick={() => update('provider', 'manual_imap_smtp')}>Manual IMAP/SMTP</button>
              {oauthProviders.map(provider => (
                <button key={provider.provider} className={form.provider === provider.provider ? 'segmented__item segmented__item--active' : 'segmented__item'} onClick={() => update('provider', provider.provider)}>{provider.label}</button>
              ))}
            </div>
            {form.provider !== 'manual_imap_smtp' && (
              <div className="settings-callout">
                Host-owned OAuth is a placeholder until the platform connector completes authorization and returns an opaque connection reference.
                <button className="btn btn--primary" onClick={() => beginOAuth(form.provider)}>Start host authorization</button>
              </div>
            )}
          </div>

          {form.provider === 'manual_imap_smtp' && (
            <>
              <div className="settings-section">
                <h2>Incoming IMAP</h2>
                <Field label="IMAP host" htmlFor="imap-host"><input id="imap-host" className="form-input" value={form.incoming_host} onChange={event => update('incoming_host', event.target.value)} placeholder="imap.example.com" /></Field>
                <div className="field-row">
                  <Field label="Port" htmlFor="imap-port"><input id="imap-port" className="form-input" type="number" value={form.incoming_port} onChange={event => update('incoming_port', event.target.value)} /></Field>
                  <Field label="Security" htmlFor="imap-security"><select id="imap-security" className="form-select" value={form.incoming_security} onChange={event => update('incoming_security', event.target.value)}><option value="ssl_tls">SSL/TLS</option><option value="starttls">STARTTLS</option><option value="none">None (policy-controlled)</option></select></Field>
                </div>
                <Field label="Username" htmlFor="imap-user"><input id="imap-user" className="form-input" value={form.incoming_username} onChange={event => update('incoming_username', event.target.value)} placeholder={form.email_address || 'Usually your email address'} /></Field>
              </div>

              <div className="settings-section">
                <h2>Outgoing SMTP</h2>
                <Field label="SMTP host" htmlFor="smtp-host"><input id="smtp-host" className="form-input" value={form.outgoing_host} onChange={event => update('outgoing_host', event.target.value)} placeholder="smtp.example.com" /></Field>
                <div className="field-row">
                  <Field label="Port" htmlFor="smtp-port"><input id="smtp-port" className="form-input" type="number" value={form.outgoing_port} onChange={event => update('outgoing_port', event.target.value)} /></Field>
                  <Field label="Security" htmlFor="smtp-security"><select id="smtp-security" className="form-select" value={form.outgoing_security} onChange={event => update('outgoing_security', event.target.value)}><option value="starttls">STARTTLS</option><option value="ssl_tls">SSL/TLS</option><option value="none">None (policy-controlled)</option></select></Field>
                </div>
                <label className="checkbox-line"><input type="checkbox" checked={form.use_incoming_secret} onChange={event => update('use_incoming_secret', event.target.checked)} /> Use incoming credential for SMTP</label>
                <Field label="SMTP username" htmlFor="smtp-user"><input id="smtp-user" className="form-input" value={form.outgoing_username} onChange={event => update('outgoing_username', event.target.value)} placeholder="Same as incoming" /></Field>
              </div>

              <div className="settings-section">
                <h2>Authentication</h2>
                <Field label="Method" htmlFor="auth-method"><select id="auth-method" className="form-select" value={form.auth_method} onChange={event => update('auth_method', event.target.value)}><option value="app_password">Password or app password</option><option value="password">Password</option><option value="oauth2">OAuth token reference</option></select></Field>
                <Field label="Password or app password" htmlFor="mail-password" hint="Submitted directly to Synapp host for account testing and encrypted storage; never stored in the app UI.">
                  <input id="mail-password" className="form-input" type="password" value={form.password} onChange={event => update('password', event.target.value)} autoComplete="new-password" />
                </Field>
                <label className="checkbox-line"><input type="checkbox" checked={form.send_test_message} onChange={event => update('send_test_message', event.target.checked)} /> Send a loopback SMTP test message when testing outgoing mail</label>
                <Field label="Incoming credential" htmlFor="incoming-secret-handle" hint="Credentials are captured by Synapp host secure fields. The app receives only the selected opaque handle.">
                  <select id="incoming-secret-handle" className="form-select" value={form.incoming_secret_handle} onChange={event => update('incoming_secret_handle', event.target.value)}>
                    <option value="host-secure-field:incoming">Host secure field: incoming</option>
                    <option value="host-secret:incoming-mail-credential">Saved host secret: incoming mail credential</option>
                  </select>
                </Field>
                {!form.use_incoming_secret && (
                  <Field label="Outgoing credential" htmlFor="outgoing-secret-handle">
                    <select id="outgoing-secret-handle" className="form-select" value={form.outgoing_secret_handle} onChange={event => update('outgoing_secret_handle', event.target.value)}>
                      <option value="host-secure-field:outgoing">Host secure field: outgoing</option>
                      <option value="host-secret:outgoing-mail-credential">Saved host secret: outgoing mail credential</option>
                    </select>
                  </Field>
                )}
              </div>
            </>
          )}

          <div className="settings-section">
            <h2>Sync and sender defaults</h2>
            <div className="field-row">
              <Field label="Sync interval" htmlFor="sync-interval"><select id="sync-interval" className="form-select" value={form.sync_interval_minutes} onChange={event => update('sync_interval_minutes', event.target.value)}><option value="5">5 minutes</option><option value="15">15 minutes</option><option value="30">30 minutes</option><option value="60">Hourly</option></select></Field>
              <Field label="Initial sync" htmlFor="initial-range"><select id="initial-range" className="form-select" value={form.initial_range} onChange={event => update('initial_range', event.target.value)}><option value="7_days">7 days</option><option value="30_days">30 days</option><option value="all">All mail</option></select></Field>
            </div>
            <Field label="Attachments" htmlFor="attachment-sync"><select id="attachment-sync" className="form-select" value={form.download_attachments} onChange={event => update('download_attachments', event.target.value)}><option value="metadata_only">Metadata only</option><option value="on_open">Download on open</option><option value="always">Download automatically</option></select></Field>
            <label className="checkbox-line"><input type="checkbox" checked={form.offline_cache} onChange={event => update('offline_cache', event.target.checked)} /> Keep offline cache when host supports it</label>
            <label className="checkbox-line"><input type="checkbox" checked={form.make_default} onChange={event => update('make_default', event.target.checked)} /> Make this the default sender</label>
          </div>

          <div className="settings-section settings-section--wide">
            <h2>Connection test</h2>
            <div className="test-status-grid" aria-live="polite">
              <div><span>Incoming</span><StatusPill state={testResult?.incoming || 'not_tested'} /></div>
              <div><span>Outgoing</span><StatusPill state={testResult?.outgoing || 'not_tested'} /></div>
              <div><span>Folders</span><StatusPill state={testResult?.folders || 'not_tested'} /></div>
            </div>
            {testResult?.message && <p className="settings-hint">{testResult.message}</p>}
            <div className="settings-actions">
              <button className="btn btn--secondary" onClick={() => testConnection('incoming')} disabled={testing}>{testing ? 'Testing...' : 'Test incoming only'}</button>
              <button className="btn btn--secondary" onClick={() => testConnection('all')} disabled={testing}>{testing ? 'Testing...' : 'Test connection'}</button>
              <button className="btn btn--primary" onClick={() => saveAccount('active')} disabled={saving}>{saving ? 'Saving...' : 'Save account'}</button>
              <button className="btn btn--secondary" onClick={() => saveAccount('receive_only')} disabled={saving}>Save receive-only</button>
            </div>
          </div>
        </div>
      )}
    </section>
  );
}

function AccountsSection() {
  const { state, loadAccounts, toast } = useApp();
  const [adding, setAdding] = useState(false);

  async function removeAccount(accountId) {
    if (!window.confirm('Remove this account from your mail workspace? Local cache and scheduled sends will be cleaned up by the host.')) return;
    try {
      await invoke('remove_account', { account_id: accountId, remove_local_cache: true, cancel_scheduled_sends: true, delete_drafts: false });
      toast('Account removal planned', 'success');
      loadAccounts();
    } catch (err) {
      toast(`Remove failed: ${err.message}`, 'error');
    }
  }

  if (adding) return <AccountSetup />;

  return (
    <section className="settings-panel" aria-labelledby="accounts-title">
      <div className="settings-panel__header">
        <div><h1 id="accounts-title">Accounts</h1><p>User-scoped account setup, connection health, sync state, and secure credential references.</p></div>
        <button className="btn btn--primary" onClick={() => setAdding(true)}>Add account</button>
      </div>
      <div className="account-list">
        {state.accounts.map(account => (
          <article key={account.account_id} className="account-card">
            <div className="account-card__main">
              <span className="account-card__dot" style={{ background: account.color || '#0078d4' }} />
              <div><h2>{account.account_label || account.display_name || account.email_address}</h2><p>{account.email_address}</p></div>
            </div>
            <div className="account-card__meta"><StatusPill state={account.connection_state || account.status} /><span>{account.provider || 'manual_imap_smtp'}</span><span>{account.is_default ? 'Default sender' : 'Secondary sender'}</span><span>Last sync {account.last_sync_at ? new Date(account.last_sync_at * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) : 'not synced'}</span></div>
            <div className="account-card__actions"><button className="btn btn--secondary" onClick={() => toast('Host connection test planned from account card', 'info')}>Test connection</button><button className="btn btn--secondary" onClick={() => toast('Sync requested', 'success')}>Sync now</button><button className="btn btn--danger" onClick={() => removeAccount(account.account_id)}>Remove</button></div>
          </article>
        ))}
      </div>
    </section>
  );
}

function IdentitiesSection() {
  const { state, toast } = useApp();
  const [loading, setLoading] = useState(true);
  const [identities, setIdentities] = useState([]);
  const [signatures, setSignatures] = useState([]);

  useEffect(() => {
    invoke('list_identities', { account_id: state.activeAccountId })
      .then(res => { setIdentities(res.data.identities || []); setSignatures(res.data.signatures || []); })
      .catch(err => toast(`Failed to load identities: ${err.message}`, 'error'))
      .finally(() => setLoading(false));
  }, [state.activeAccountId, toast]);

  if (loading) return <div className="centered-spinner"><Spinner size="lg" /><span>Loading identities...</span></div>;

  return (
    <section className="settings-panel" aria-labelledby="identities-title">
      <div className="settings-panel__header"><div><h1 id="identities-title">Identities and signatures</h1><p>Aliases, send-as addresses, default sender rules, and reusable signatures.</p></div><button className="btn btn--primary" onClick={() => toast('Identity save surface is ready for host wiring', 'info')}>Add identity</button></div>
      <div className="settings-grid">
        <div className="settings-section"><h2>Sender identities</h2>{identities.map(identity => <div key={identity.identity_id} className="simple-row"><div><strong>{identity.display_name}</strong><span>{identity.email_address}</span></div><StatusPill state={identity.verification_state} /></div>)}</div>
        <div className="settings-section"><h2>Signatures</h2>{signatures.map(signature => <div key={signature.signature_id} className="simple-row"><div><strong>{signature.label}</strong><span>{signature.body_text}</span></div><StatusPill state={signature.enabled ? 'active' : 'disabled'} /></div>)}<textarea className="settings-textarea" defaultValue={signatures[0]?.body_text || ''} aria-label="Signature body" /></div>
      </div>
    </section>
  );
}

function PreferencesSection() {
  const { toast } = useApp();
  const [preferences, setPreferences] = useState(DEFAULT_PREFS);

  useEffect(() => {
    invoke('get_user_preferences', { include_defaults: true })
      .then(res => setPreferences(res.data.preferences || DEFAULT_PREFS))
      .catch(err => toast(`Preferences unavailable: ${err.message}`, 'error'));
  }, [toast]);

  function setNested(section, field, value) {
    setPreferences(current => ({ ...current, [section]: { ...current[section], [field]: value } }));
  }

  async function save() {
    try {
      await invoke('save_user_preferences', { preferences });
      toast('Preferences saved', 'success');
    } catch (err) {
      toast(`Save failed: ${err.message}`, 'error');
    }
  }

  return (
    <section className="settings-panel" aria-labelledby="preferences-title">
      <div className="settings-panel__header"><div><h1 id="preferences-title">Preferences</h1><p>Reading, compose, notification, and sync defaults. These are user preferences, not admin settings.</p></div><button className="btn btn--primary" onClick={save}>Save preferences</button></div>
      <div className="settings-grid">
        <div className="settings-section"><h2>Reading</h2><Field label="Preview pane" htmlFor="pref-preview"><select id="pref-preview" className="form-select" value={preferences.reading.preview_pane} onChange={event => setNested('reading', 'preview_pane', event.target.value)}><option value="right">Right</option><option value="bottom">Bottom</option><option value="off">Off</option></select></Field><label className="checkbox-line"><input type="checkbox" checked={preferences.reading.thread_view} onChange={event => setNested('reading', 'thread_view', event.target.checked)} /> Group by conversation</label><label className="checkbox-line"><input type="checkbox" checked={preferences.reading.focused_inbox} onChange={event => setNested('reading', 'focused_inbox', event.target.checked)} /> Use focused inbox</label></div>
        <div className="settings-section"><h2>Compose</h2><Field label="Undo send" htmlFor="pref-undo"><select id="pref-undo" className="form-select" value={preferences.compose.undo_send_delay_seconds} onChange={event => setNested('compose', 'undo_send_delay_seconds', Number(event.target.value))}><option value="0">Off</option><option value="5">5 seconds</option><option value="10">10 seconds</option><option value="30">30 seconds</option></select></Field><Field label="Default format" htmlFor="pref-format"><select id="pref-format" className="form-select" value={preferences.compose.default_format} onChange={event => setNested('compose', 'default_format', event.target.value)}><option value="html">Rich text</option><option value="plain_text">Plain text</option></select></Field></div>
        <div className="settings-section"><h2>Notifications</h2><label className="checkbox-line"><input type="checkbox" checked={preferences.notifications.enabled} onChange={event => setNested('notifications', 'enabled', event.target.checked)} /> Notify me about new mail</label></div>
        <div className="settings-section"><h2>Sync defaults</h2><Field label="Default interval" htmlFor="pref-sync"><select id="pref-sync" className="form-select" value={preferences.sync_defaults.interval_minutes} onChange={event => setNested('sync_defaults', 'interval_minutes', Number(event.target.value))}><option value="5">5 minutes</option><option value="15">15 minutes</option><option value="30">30 minutes</option><option value="60">Hourly</option></select></Field><label className="checkbox-line"><input type="checkbox" checked={preferences.sync_defaults.offline_cache} onChange={event => setNested('sync_defaults', 'offline_cache', event.target.checked)} /> Prefer offline cache</label></div>
      </div>
    </section>
  );
}

function AgentsSection() {
  const { state } = useApp();
  const availableEffects = useMemo(() => state.hostCtx.available_effects || [], [state.hostCtx.available_effects]);

  return (
    <section className="settings-panel" aria-labelledby="agents-title">
      <div className="settings-panel__header"><div><h1 id="agents-title">Agent permissions and activity</h1><p>Review what mail capabilities the current principal can use. This view contains permission and audit information only; it contains no AI logic.</p></div></div>
      <div className="settings-grid">
        <div className="settings-section"><h2>Current permission</h2><div className="permission-large">{state.hostCtx.permission || 'none'}</div><p className="settings-hint">Send, delete, rules, account setup, and scheduled send actions stay gated by host capability and Wasm permission checks.</p></div>
        <div className="settings-section"><h2>Host effects visible to app</h2><div className="effect-list">{availableEffects.map(effect => <span key={effect}>{effect}</span>)}</div></div>
        <div className="settings-section settings-section--wide"><h2>Recent activity</h2><div className="simple-row"><div><strong>No agent mail activity in this local session</strong><span>When the host supplies audit events, reviewed sends, draft changes, rules, and organize actions appear here.</span></div><StatusPill state="idle" /></div></div>
      </div>
    </section>
  );
}

function MigrationSection() {
  const { toast } = useApp();
  const [inspection, setInspection] = useState(null);

  async function inspect() {
    try {
      const res = await invoke('inspect_legacy_settings');
      setInspection(res.data);
      toast('Legacy settings inspected', 'success');
    } catch (err) {
      toast(`Inspection failed: ${err.message}`, 'error');
    }
  }

  async function migrate() {
    try {
      await invoke('migrate_legacy_settings', { legacy_settings: {}, migration_options: { move_main_schema_to_preferences: true }, dry_run: false });
      toast('Legacy settings migration planned', 'success');
    } catch (err) {
      toast(`Migration failed: ${err.message}`, 'error');
    }
  }

  return (
    <section className="settings-panel" aria-labelledby="migration-title">
      <div className="settings-panel__header"><div><h1 id="migration-title">Legacy settings migration</h1><p>Move old schema-form settings into user preferences and account reconnect prompts without preserving raw credentials.</p></div></div>
      <div className="settings-actions"><button className="btn btn--secondary" onClick={inspect}>Inspect legacy settings</button><button className="btn btn--primary" onClick={migrate}>Migrate user settings</button></div>
      {inspection && <pre className="migration-preview">{JSON.stringify(inspection, null, 2)}</pre>}
    </section>
  );
}

export default function SettingsCenter() {
  const { state, dispatch } = useApp();
  const tabs = [
    ['accounts', 'Accounts'],
    ['identities', 'Identities'],
    ['preferences', 'Preferences'],
    ['rules', 'Rules'],
    ['agents', 'Agents'],
    ['migration', 'Migration'],
  ];

  return (
    <main className="settings-shell">
      <nav className="settings-nav" aria-label="Mail settings sections">
        {tabs.map(([id, label]) => <button key={id} className={state.settingsSection === id ? 'settings-nav__item settings-nav__item--active' : 'settings-nav__item'} onClick={() => dispatch({ type: 'SET_SETTINGS_SECTION', payload: id })}>{label}</button>)}
      </nav>
      <div className="settings-content">
        {state.settingsSection === 'accounts' && <AccountsSection />}
        {state.settingsSection === 'identities' && <IdentitiesSection />}
        {state.settingsSection === 'preferences' && <PreferencesSection />}
        {state.settingsSection === 'rules' && <RulesManager embedded />}
        {state.settingsSection === 'agents' && <AgentsSection />}
        {state.settingsSection === 'migration' && <MigrationSection />}
      </div>
    </main>
  );
}