import React, { useEffect } from 'react';
import { AppProvider, useApp } from './context/AppContext.jsx';
import Layout from './components/Layout.jsx';
import Toast from './components/Toast.jsx';

function AppInner() {
  const { loadAccounts, loadMailboxes, loadEmails, state } = useApp();

  useEffect(() => {
    loadAccounts();
  }, [loadAccounts]);

  useEffect(() => {
    if (state.activeAccountId) {
      loadMailboxes(state.activeAccountId);
    }
  }, [state.activeAccountId, loadMailboxes]);

  useEffect(() => {
    if (state.activeMailboxId) {
      loadEmails(state.activeMailboxId, 0);
    }
  }, [state.activeMailboxId, loadEmails]);

  return (
    <>
      <Layout />
      <Toast />
    </>
  );
}

export default function App() {
  return (
    <AppProvider>
      <AppInner />
    </AppProvider>
  );
}
