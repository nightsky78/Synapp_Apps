import React, { createContext, useContext, useReducer, useCallback, useRef } from 'react';
import { getContext, invoke, hasPermission } from '../bridge.js';

const AppContext = createContext(null);

const initialState = {
  hostCtx: getContext(),
  accounts: [],
  mailboxes: [],
  activeAccountId: null,
  activeMailboxId: 'inbox',
  activeEmailId: null,
  activeEmail: null,
  emails: [],
  emailsTotal: 0,
  emailsUnread: 0,
  emailPage: 0,
  searchQuery: '',
  searchResults: null,
  searching: false,
  composeOpen: false,
  composeDraftId: null,
  loadingMailboxes: false,
  loadingEmails: false,
  loadingEmail: false,
  toasts: [],
  showFolderManager: false,
  showRulesManager: false,
};

let nextToastId = 1;

function reducer(state, action) {
  switch (action.type) {
    case 'SET_ACCOUNTS': return { ...state, accounts: action.payload, activeAccountId: action.payload[0]?.account_id ?? null };
    case 'SET_MAILBOXES': return { ...state, mailboxes: action.payload };
    case 'SET_LOADING_MAILBOXES': return { ...state, loadingMailboxes: action.payload };
    case 'SET_ACTIVE_MAILBOX': return { ...state, activeMailboxId: action.payload, activeEmailId: null, activeEmail: null, emails: [], searchQuery: '', searchResults: null };
    case 'SET_EMAILS': return { ...state, emails: action.payload.messages, emailsTotal: action.payload.total_count, emailsUnread: action.payload.unread_count, loadingEmails: false };
    case 'SET_LOADING_EMAILS': return { ...state, loadingEmails: action.payload };
    case 'SET_ACTIVE_EMAIL': return { ...state, activeEmailId: action.payload?.email_id ?? null, activeEmail: action.payload };
    case 'SET_LOADING_EMAIL': return { ...state, loadingEmail: action.payload };
    case 'SET_EMAIL_PAGE': return { ...state, emailPage: action.payload };
    case 'SET_SEARCH_QUERY': return { ...state, searchQuery: action.payload };
    case 'SET_SEARCH_RESULTS': return { ...state, searchResults: action.payload, searching: false };
    case 'SET_SEARCHING': return { ...state, searching: action.payload };
    case 'SET_COMPOSE': return { ...state, composeOpen: action.payload.open, composeDraftId: action.payload.draftId ?? null };
    case 'SET_SHOW_FOLDER_MANAGER': return { ...state, showFolderManager: action.payload };
    case 'SET_SHOW_RULES_MANAGER': return { ...state, showRulesManager: action.payload };
    case 'ADD_TOAST': return { ...state, toasts: [...state.toasts, action.payload] };
    case 'REMOVE_TOAST': return { ...state, toasts: state.toasts.filter(t => t.id !== action.payload) };
    case 'MARK_READ_LOCAL': {
      const emails = state.emails.map(e => e.email_id === action.payload ? { ...e, is_read: true } : e);
      const activeEmail = state.activeEmail?.email_id === action.payload ? { ...state.activeEmail, is_read: true } : state.activeEmail;
      return { ...state, emails, activeEmail };
    }
    case 'TOGGLE_FLAG_LOCAL': {
      const emails = state.emails.map(e => e.email_id === action.payload ? { ...e, is_flagged: !e.is_flagged } : e);
      const activeEmail = state.activeEmail?.email_id === action.payload ? { ...state.activeEmail, is_flagged: !state.activeEmail.is_flagged } : state.activeEmail;
      return { ...state, emails, activeEmail };
    }
    case 'REMOVE_EMAIL_LOCAL': {
      const emails = state.emails.filter(e => e.email_id !== action.payload);
      const activeEmail = state.activeEmail?.email_id === action.payload ? null : state.activeEmail;
      const activeEmailId = activeEmail?.email_id ?? null;
      return { ...state, emails, activeEmail, activeEmailId };
    }
    default: return state;
  }
}

export function AppProvider({ children }) {
  const [state, dispatch] = useReducer(reducer, initialState);
  const searchTimer = useRef(null);

  const toast = useCallback((msg, type = 'info', opts = {}) => {
    const id = nextToastId++;
    dispatch({ type: 'ADD_TOAST', payload: { id, msg, type, ...opts } });
    if (!opts.persistent) {
      setTimeout(() => dispatch({ type: 'REMOVE_TOAST', payload: id }), opts.duration ?? 5000);
    }
    return id;
  }, []);

  const removeToast = useCallback((id) => dispatch({ type: 'REMOVE_TOAST', payload: id }), []);

  const can = useCallback((perm) => hasPermission(state.hostCtx.permission, perm), [state.hostCtx.permission]);

  const loadAccounts = useCallback(async () => {
    try {
      const res = await invoke('list_accounts');
      dispatch({ type: 'SET_ACCOUNTS', payload: res.data.snapshot });
    } catch (err) {
      toast(`Failed to load accounts: ${err.message}`, 'error');
    }
  }, [toast]);

  const loadMailboxes = useCallback(async (accountId) => {
    if (!accountId) return;
    dispatch({ type: 'SET_LOADING_MAILBOXES', payload: true });
    try {
      const res = await invoke('list_mailboxes', { account_id: accountId });
      dispatch({ type: 'SET_MAILBOXES', payload: res.data.snapshot });
    } catch (err) {
      toast(`Failed to load folders: ${err.message}`, 'error');
    } finally {
      dispatch({ type: 'SET_LOADING_MAILBOXES', payload: false });
    }
  }, [toast]);

  const loadEmails = useCallback(async (mailboxId, page = 0) => {
    dispatch({ type: 'SET_LOADING_EMAILS', payload: true });
    try {
      const res = await invoke('read_emails', { mailbox_id: mailboxId, page_size: 50, page_offset: page * 50 });
      dispatch({ type: 'SET_EMAILS', payload: res.data.snapshot });
      dispatch({ type: 'SET_EMAIL_PAGE', payload: page });
    } catch (err) {
      dispatch({ type: 'SET_LOADING_EMAILS', payload: false });
      toast(`Failed to load emails: ${err.message}`, 'error');
    }
  }, [toast]);

  const selectEmail = useCallback(async (email) => {
    dispatch({ type: 'SET_ACTIVE_EMAIL', payload: email });
    if (!email.is_read) {
      dispatch({ type: 'MARK_READ_LOCAL', payload: email.email_id });
      try {
        await invoke('mark_read', { email_ids: [email.email_id], read: true });
      } catch (_) { /* silent — local state already updated */ }
    }
  }, []);

  const flagEmail = useCallback(async (emailId) => {
    dispatch({ type: 'TOGGLE_FLAG_LOCAL', payload: emailId });
    try {
      await invoke('flag_email', { email_id: emailId, flagged: true });
    } catch (err) {
      dispatch({ type: 'TOGGLE_FLAG_LOCAL', payload: emailId }); // revert
      toast(`Flag failed: ${err.message}`, 'error');
    }
  }, [toast]);

  const archiveEmail = useCallback(async (emailId) => {
    const toastId = toast('Archiving…', 'undo', {
      persistent: true,
      undoAction: () => { removeToast(toastId); },
    });
    dispatch({ type: 'REMOVE_EMAIL_LOCAL', payload: emailId });
    try {
      await invoke('archive_email', { email_ids: [emailId] });
      removeToast(toastId);
      toast('Archived', 'success');
    } catch (err) {
      removeToast(toastId);
      toast(`Archive failed: ${err.message}`, 'error');
      // Reload to recover
      loadEmails(state.activeMailboxId, state.emailPage);
    }
  }, [toast, removeToast, loadEmails, state.activeMailboxId, state.emailPage]);

  const deleteEmail = useCallback(async (emailId) => {
    dispatch({ type: 'REMOVE_EMAIL_LOCAL', payload: emailId });
    try {
      await invoke('delete_email', { email_ids: [emailId], permanent: false });
      toast('Moved to Trash', 'success');
    } catch (err) {
      toast(`Delete failed: ${err.message}`, 'error');
      loadEmails(state.activeMailboxId, state.emailPage);
    }
  }, [toast, loadEmails, state.activeMailboxId, state.emailPage]);

  const search = useCallback((query) => {
    dispatch({ type: 'SET_SEARCH_QUERY', payload: query });
    if (searchTimer.current) clearTimeout(searchTimer.current);
    if (!query.trim()) {
      dispatch({ type: 'SET_SEARCH_RESULTS', payload: null });
      return;
    }
    dispatch({ type: 'SET_SEARCHING', payload: true });
    searchTimer.current = setTimeout(async () => {
      try {
        const res = await invoke('search_emails', { query, max_results: 50 });
        dispatch({ type: 'SET_SEARCH_RESULTS', payload: res.data.snapshot });
      } catch (err) {
        dispatch({ type: 'SET_SEARCHING', payload: false });
        toast(`Search failed: ${err.message}`, 'error');
      }
    }, 500);
  }, [toast]);

  const openCompose = useCallback((draftId = null) => {
    dispatch({ type: 'SET_COMPOSE', payload: { open: true, draftId } });
  }, []);

  const closeCompose = useCallback(() => {
    dispatch({ type: 'SET_COMPOSE', payload: { open: false } });
  }, []);

  const value = {
    state, dispatch,
    can, toast, removeToast,
    loadAccounts, loadMailboxes, loadEmails,
    selectEmail, flagEmail, archiveEmail, deleteEmail,
    search, openCompose, closeCompose,
  };

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}

export function useApp() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error('useApp must be used within AppProvider');
  return ctx;
}
