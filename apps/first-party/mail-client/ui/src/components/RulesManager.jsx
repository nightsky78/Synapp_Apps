import React, { useState, useEffect } from 'react';
import { useApp } from '../context/AppContext.jsx';
import { invoke } from '../bridge.js';
import { Spinner } from './Spinner.jsx';

export default function RulesManager({ embedded = false }) {
  const { state, dispatch, toast } = useApp();
  const [rules, setRules] = useState([]);
  const [loading, setLoading] = useState(true);
  const [creating, setCreating] = useState(false);
  const [form, setForm] = useState({ name: '', condition_field: 'from', condition_op: 'contains', condition_value: '', action: 'move', action_mailbox_id: '' });

  useEffect(() => {
    invoke('list_rules', { account_id: state.activeAccountId })
      .then(res => setRules(res.data.snapshot))
      .catch(err => toast(`Failed to load rules: ${err.message}`, 'error'))
      .finally(() => setLoading(false));
  }, [state.activeAccountId, toast]);

  function close() {
    dispatch({ type: 'SET_SHOW_RULES_MANAGER', payload: false });
  }

  async function handleCreate() {
    if (!form.name.trim() || !form.condition_value.trim()) {
      toast('Rule name and condition value are required', 'error');
      return;
    }
    setCreating(true);
    try {
      await invoke('create_rule', {
        account_id: state.activeAccountId,
        rule: {
          name: form.name.trim(),
          enabled: true,
          conditions: [{ field: form.condition_field, op: form.condition_op, value: form.condition_value.trim() }],
          actions: [{ type: form.action, mailbox_id: form.action_mailbox_id || undefined }],
          stop_processing: false,
        },
      });
      toast('Rule created', 'success');
      // Reload rules
      const res = await invoke('list_rules', { account_id: state.activeAccountId });
      setRules(res.data.snapshot);
      setForm({ name: '', condition_field: 'from', condition_op: 'contains', condition_value: '', action: 'move', action_mailbox_id: '' });
    } catch (err) {
      toast(`Failed to create rule: ${err.message}`, 'error');
    } finally {
      setCreating(false);
    }
  }

  async function handleDelete(ruleId) {
    try {
      await invoke('delete_rule', { rule_id: ruleId });
      setRules(rules.filter(r => r.rule_id !== ruleId));
      toast('Rule deleted', 'success');
    } catch (err) {
      toast(`Delete failed: ${err.message}`, 'error');
    }
  }

  const moveFolders = state.mailboxes.filter(m => m.kind !== 'trash');

  const body = (
    <>
          {/* Existing rules */}
          {loading ? (
            <div style={{ textAlign: 'center', padding: 20 }}><Spinner /></div>
          ) : rules.length === 0 ? (
            <div style={{ color: '#a19f9d', fontSize: 13, marginBottom: 16, textAlign: 'center' }}>No rules yet. Create one below.</div>
          ) : (
            <div className="rules-list">
              {rules.map(r => (
                <div key={r.rule_id} className="rule-item">
                  <div>
                    <div className="rule-item__name">{r.name}</div>
                    <div style={{ fontSize: 11, color: '#605e5c', marginTop: 2 }}>
                      If {r.conditions?.[0]?.field} {r.conditions?.[0]?.op} "{r.conditions?.[0]?.value}" → {r.actions?.[0]?.type}
                    </div>
                  </div>
                  <span className={`rule-item__enabled`} style={{ color: r.enabled ? '#107c10' : '#a19f9d' }}>{r.enabled ? 'Active' : 'Inactive'}</span>
                  <button
                    style={{ fontSize: 12, color: '#c50f1f', background: 'none', border: 'none', cursor: 'pointer', padding: '2px 6px' }}
                    onClick={() => handleDelete(r.rule_id)}
                    aria-label={`Delete rule ${r.name}`}
                  >🗑️</button>
                </div>
              ))}
            </div>
          )}

          {/* Create new rule */}
          <div style={{ borderTop: '1px solid #e0e0e0', paddingTop: 14 }}>
            <div className="form-label" style={{ marginBottom: 8 }}>New Rule</div>
            <div className="form-group">
              <label className="form-label" htmlFor="rule-name">Rule Name</label>
              <input id="rule-name" className="form-input" value={form.name} onChange={e => setForm(f => ({ ...f, name: e.target.value }))} placeholder="e.g. Move newsletters" maxLength={80} />
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: 8, marginBottom: 12 }}>
              <div>
                <label className="form-label" htmlFor="cond-field">Field</label>
                <select id="cond-field" className="form-select" value={form.condition_field} onChange={e => setForm(f => ({ ...f, condition_field: e.target.value }))}>
                  <option value="from">From</option>
                  <option value="to">To</option>
                  <option value="subject">Subject</option>
                </select>
              </div>
              <div>
                <label className="form-label" htmlFor="cond-op">Condition</label>
                <select id="cond-op" className="form-select" value={form.condition_op} onChange={e => setForm(f => ({ ...f, condition_op: e.target.value }))}>
                  <option value="contains">Contains</option>
                  <option value="equals">Equals</option>
                  <option value="starts_with">Starts with</option>
                </select>
              </div>
              <div>
                <label className="form-label" htmlFor="cond-value">Value</label>
                <input id="cond-value" className="form-input" value={form.condition_value} onChange={e => setForm(f => ({ ...f, condition_value: e.target.value }))} placeholder="e.g. newsletter@" />
              </div>
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
              <div>
                <label className="form-label" htmlFor="rule-action">Action</label>
                <select id="rule-action" className="form-select" value={form.action} onChange={e => setForm(f => ({ ...f, action: e.target.value }))}>
                  <option value="move">Move to folder</option>
                  <option value="mark_read">Mark as read</option>
                  <option value="flag">Flag</option>
                  <option value="delete">Delete</option>
                </select>
              </div>
              {form.action === 'move' && (
                <div>
                  <label className="form-label" htmlFor="rule-folder">Target Folder</label>
                  <select id="rule-folder" className="form-select" value={form.action_mailbox_id} onChange={e => setForm(f => ({ ...f, action_mailbox_id: e.target.value }))}>
                    <option value="">Select folder…</option>
                    {moveFolders.map(m => <option key={m.mailbox_id} value={m.mailbox_id}>{m.name}</option>)}
                  </select>
                </div>
              )}
            </div>
          </div>
    </>
  );

  if (embedded) {
    return (
      <section className="settings-panel" aria-labelledby="rules-title">
        <div className="settings-panel__header"><div><h1 id="rules-title">Rules</h1><p>User-scoped automation for moving, flagging, marking, archiving, and deleting mail.</p></div></div>
        <div className="settings-section settings-section--wide">{body}</div>
        <div className="settings-actions"><button className="btn btn--primary" onClick={handleCreate} disabled={creating}>{creating ? 'Creating...' : 'Create rule'}</button></div>
      </section>
    );
  }

  return (
    <div className="modal-overlay" role="dialog" aria-modal="true" aria-label="Manage rules" onClick={e => e.target === e.currentTarget && close()}>
      <div className="modal" style={{ width: 520 }}>
        <div className="modal__header">
          <span className="modal__title">Inbox Rules</span>
          <button className="modal__close" onClick={close} aria-label="Close">×</button>
        </div>
        <div className="modal__body">{body}</div>
        <div className="modal__footer">
          <button className="btn btn--secondary" onClick={close}>Close</button>
          <button className="btn btn--primary" onClick={handleCreate} disabled={creating}>{creating ? 'Creating...' : 'Create Rule'}</button>
        </div>
      </div>
    </div>
  );
}
