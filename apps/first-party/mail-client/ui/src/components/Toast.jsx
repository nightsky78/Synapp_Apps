import React from 'react';
import { useApp } from '../context/AppContext.jsx';

export default function Toast() {
  const { state, removeToast } = useApp();

  if (!state.toasts.length) return null;

  return (
    <div className="toast-container" role="region" aria-label="Notifications" aria-live="assertive">
      {state.toasts.map(t => (
        <div key={t.id} className={`toast toast--${t.type}`}>
          <span className="toast__msg">{t.msg}</span>
          {t.type === 'undo' && t.undoAction && (
            <button className="toast__undo-btn" onClick={() => { t.undoAction(); removeToast(t.id); }}>
              Undo
            </button>
          )}
          <button className="toast__close" onClick={() => removeToast(t.id)} aria-label="Dismiss notification">×</button>
        </div>
      ))}
    </div>
  );
}
