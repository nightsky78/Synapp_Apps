import React from 'react';
import Sidebar from './Sidebar.jsx';
import Toolbar from './Toolbar.jsx';
import EmailList from './EmailList.jsx';
import ReadingPane from './ReadingPane.jsx';
import SearchView from './SearchView.jsx';
import ComposePane from './ComposePane.jsx';
import FolderManager from './FolderManager.jsx';
import RulesManager from './RulesManager.jsx';
import { useApp } from '../context/AppContext.jsx';

export default function Layout() {
  const { state } = useApp();

  return (
    <div className="app-shell">
      <Toolbar />
      <div className="main-area">
        <Sidebar />
        <div className="content-area" style={{ flexDirection: 'column', flex: 1, overflow: 'hidden' }}>
          {state.searchQuery
            ? <SearchView />
            : (
              <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
                <EmailList />
                <ReadingPane />
              </div>
            )
          }
        </div>
      </div>
      {state.composeOpen && <ComposePane />}
      {state.showFolderManager && <FolderManager />}
      {state.showRulesManager && <RulesManager />}
    </div>
  );
}
