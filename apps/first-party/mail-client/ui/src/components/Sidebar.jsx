import React from 'react';
import { useApp } from '../context/AppContext.jsx';
import { invoke } from '../bridge.js';
import { CenteredSpinner } from './Spinner.jsx';

const KIND_ICONS = {
  inbox: '📥', drafts: '📝', sent: '📤', archive: '🗄️',
  trash: '🗑️', junk: '⚠️', custom: '📁',
};

export default function Sidebar() {
  const { state, dispatch, loadMailboxes, toast, can } = useApp();

  const grouped = {
    favorites: state.mailboxes.filter(m => m.favorite),
    system: state.mailboxes.filter(m => !m.favorite && m.kind !== 'custom'),
    custom: state.mailboxes.filter(m => m.kind === 'custom'),
  };

  function FolderButton({ mailbox }) {
    const active = state.activeMailboxId === mailbox.mailbox_id;
    return (
      <button
        className={`sidebar__folder${active ? ' sidebar__folder--active' : ''}`}
        onClick={() => dispatch({ type: 'SET_ACTIVE_MAILBOX', payload: mailbox.mailbox_id })}
        title={mailbox.name}
        aria-current={active ? 'page' : undefined}
      >
        <span className="sidebar__folder-icon">{KIND_ICONS[mailbox.kind] || '📁'}</span>
        <span className="sidebar__folder-name">{mailbox.name}</span>
        {mailbox.unread_count > 0 && (
          <span className="sidebar__folder-count">{mailbox.unread_count}</span>
        )}
        {can('organize') && mailbox.kind === 'custom' && (
          <span className="folder-ctx">
            <button
              className="folder-ctx__btn"
              title="Rename"
              onClick={async (e) => {
                e.stopPropagation();
                const newName = window.prompt('Rename folder:', mailbox.name);
                if (!newName || newName === mailbox.name) return;
                try {
                  await invoke('rename_folder', { account_id: state.activeAccountId, mailbox_id: mailbox.mailbox_id, new_name: newName });
                  loadMailboxes(state.activeAccountId);
                  toast('Folder renamed', 'success');
                } catch (err) {
                  toast(`Rename failed: ${err.message}`, 'error');
                }
              }}
            >✏️</button>
            <button
              className="folder-ctx__btn"
              title="Delete folder"
              onClick={async (e) => {
                e.stopPropagation();
                if (!window.confirm(`Delete folder "${mailbox.name}"? All emails will be moved to Trash.`)) return;
                try {
                  await invoke('delete_folder', { account_id: state.activeAccountId, mailbox_id: mailbox.mailbox_id });
                  loadMailboxes(state.activeAccountId);
                  toast('Folder deleted', 'success');
                } catch (err) {
                  toast(`Delete failed: ${err.message}`, 'error');
                }
              }}
            >🗑️</button>
          </span>
        )}
      </button>
    );
  }

  if (state.loadingMailboxes && state.mailboxes.length === 0) {
    return (
      <aside className="sidebar" aria-label="Folder navigation">
        <CenteredSpinner label="Loading folders…" />
      </aside>
    );
  }

  const account = state.accounts.find(a => a.account_id === state.activeAccountId);

  return (
    <aside className="sidebar" aria-label="Folder navigation">
      {account && (
        <>
          <div className="sidebar__account">
            <span className="sidebar__account-dot" style={{ background: account.color }} />
            <div>
              <div>{account.account_label || account.display_name}</div>
              <div className="sidebar__account-email">{account.email_address}</div>
            </div>
            <span className={`sidebar__health sidebar__health--${account.connection_state || 'unknown'}`} title={account.connection_state || 'unknown'} />
          </div>
          {account.connection_state === 'receive_only' && <div className="sidebar__notice">Receive-only. Sending is disabled until SMTP passes.</div>}
          <div className="sidebar__divider" />
        </>
      )}

      {grouped.favorites.length > 0 && (
        <>
          <div className="sidebar__section-title">Favorites</div>
          {grouped.favorites.map(m => <FolderButton key={m.mailbox_id} mailbox={m} />)}
          <div className="sidebar__divider" />
        </>
      )}

      {grouped.system.map(m => <FolderButton key={m.mailbox_id} mailbox={m} />)}

      {grouped.custom.length > 0 && (
        <>
          <div className="sidebar__divider" />
          <div className="sidebar__section-title">Folders</div>
          {grouped.custom.map(m => <FolderButton key={m.mailbox_id} mailbox={m} />)}
        </>
      )}

      {can('organize') && (
        <>
          <div className="sidebar__divider" />
          <button className="sidebar__action" onClick={() => dispatch({ type: 'SET_SHOW_FOLDER_MANAGER', payload: true })}>
            New folder
          </button>
        </>
      )}

      <div className="sidebar__divider" />
      <div className="sidebar__section-title">User settings</div>
      <button className="sidebar__action" onClick={() => dispatch({ type: 'SET_SETTINGS_SECTION', payload: 'accounts' })}>Accounts</button>
      <button className="sidebar__action" onClick={() => dispatch({ type: 'SET_SETTINGS_SECTION', payload: 'identities' })}>Identities and signatures</button>
      <button className="sidebar__action" onClick={() => dispatch({ type: 'SET_SETTINGS_SECTION', payload: 'preferences' })}>Preferences</button>
      <button className="sidebar__action" onClick={() => dispatch({ type: 'SET_SETTINGS_SECTION', payload: 'rules' })}>Rules</button>
      <button className="sidebar__action" onClick={() => dispatch({ type: 'SET_SETTINGS_SECTION', payload: 'agents' })}>Agent permissions</button>
    </aside>
  );
}
