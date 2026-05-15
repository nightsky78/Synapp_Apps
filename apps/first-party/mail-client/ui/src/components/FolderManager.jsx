import React, { useState } from 'react';
import { useApp } from '../context/AppContext.jsx';
import { invoke } from '../bridge.js';

export default function FolderManager() {
  const { state, dispatch, loadMailboxes, toast } = useApp();
  const [name, setName] = useState('');
  const [creating, setCreating] = useState(false);

  async function handleCreate() {
    if (!name.trim()) { toast('Folder name is required', 'error'); return; }
    setCreating(true);
    try {
      await invoke('create_folder', { account_id: state.activeAccountId, name: name.trim() });
      toast(`Folder "${name}" created`, 'success');
      setName('');
      loadMailboxes(state.activeAccountId);
    } catch (err) {
      toast(`Failed to create folder: ${err.message}`, 'error');
    } finally {
      setCreating(false);
    }
  }

  function close() {
    dispatch({ type: 'SET_SHOW_FOLDER_MANAGER', payload: false });
  }

  const customFolders = state.mailboxes.filter(m => m.kind === 'custom');

  return (
    <div className="modal-overlay" data-testid="folder-manager-dialog" role="dialog" aria-modal="true" aria-label="Manage folders" onClick={e => e.target === e.currentTarget && close()}>
      <div className="modal">
        <div className="modal__header">
          <span className="modal__title">Manage Folders</span>
          <button className="modal__close" data-testid="folder-manager-close" onClick={close} aria-label="Close">×</button>
        </div>
        <div className="modal__body">
          <div className="form-group">
            <label className="form-label" htmlFor="new-folder-name">New Folder Name</label>
            <input
              id="new-folder-name"
              data-testid="new-folder-name"
              className="form-input"
              type="text"
              value={name}
              onChange={e => setName(e.target.value)}
              placeholder="e.g. Work, Personal…"
              onKeyDown={e => e.key === 'Enter' && handleCreate()}
              maxLength={80}
            />
          </div>

          {customFolders.length > 0 && (
            <div>
              <div className="form-label" style={{ marginBottom: 6 }}>Your Folders</div>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 4, maxHeight: 200, overflowY: 'auto' }}>
                {customFolders.map(m => (
                  <div key={m.mailbox_id} data-testid={`folder-manager-folder-${m.mailbox_id}`} style={{ display: 'flex', alignItems: 'center', padding: '6px 8px', border: '1px solid #e0e0e0', borderRadius: 4, fontSize: 13 }}>
                    <span style={{ flex: 1 }}>📁 {m.name}</span>
                    <span style={{ fontSize: 11, color: '#a19f9d', marginRight: 8 }}>{m.total_count} messages</span>
                    <button
                      style={{ fontSize: 12, color: '#c50f1f', background: 'none', border: 'none', cursor: 'pointer', padding: '2px 6px' }}
                      onClick={async () => {
                        if (!window.confirm(`Delete "${m.name}"?`)) return;
                        try {
                          await invoke('delete_folder', { account_id: state.activeAccountId, mailbox_id: m.mailbox_id });
                          loadMailboxes(state.activeAccountId);
                          toast('Folder deleted', 'success');
                        } catch (err) {
                          toast(`Delete failed: ${err.message}`, 'error');
                        }
                      }}
                      aria-label={`Delete folder ${m.name}`}
                    >
                      🗑️
                    </button>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
        <div className="modal__footer">
          <button className="btn btn--secondary" onClick={close}>Cancel</button>
          <button className="btn btn--primary" data-testid="create-folder-button" onClick={handleCreate} disabled={creating || !name.trim()}>
            {creating ? 'Creating…' : 'Create Folder'}
          </button>
        </div>
      </div>
    </div>
  );
}
