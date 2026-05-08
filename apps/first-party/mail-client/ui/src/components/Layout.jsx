import React from 'react';
import Sidebar from './Sidebar.jsx';
import Toolbar from './Toolbar.jsx';
import EmailList from './EmailList.jsx';
import ReadingPane from './ReadingPane.jsx';
import SearchView from './SearchView.jsx';
import ComposePane from './ComposePane.jsx';
import FolderManager from './FolderManager.jsx';
import RulesManager from './RulesManager.jsx';
import SettingsCenter, { AccountSetup } from './SettingsCenter.jsx';
import { useApp } from '../context/AppContext.jsx';

export default function Layout() {
  const { state } = useApp();

  if (!state.loadingAccounts && state.accounts.length === 0) {
    return (
      <div className="app-shell">
        <Toolbar />
        <AccountSetup firstRun />
        {state.composeOpen && <ComposePane />}
      </div>
    );
  }

  return (
    <div className="app-shell">
      <Toolbar />
      <div className="main-area">
        {state.activeRoute === 'settings' ? (
          <SettingsCenter />
        ) : (
          <>
            <Sidebar />
            <div className="content-area content-area--mail">
              {state.searchQuery
                ? <SearchView />
                : (
                  <div className="mail-workspace">
                    <EmailList />
                    <ReadingPane />
                  </div>
                )
              }
            </div>
          </>
        )}
      </div>
      {state.composeOpen && <ComposePane />}
      {state.showFolderManager && <FolderManager />}
      {state.showRulesManager && <RulesManager />}
    </div>
  );
}
