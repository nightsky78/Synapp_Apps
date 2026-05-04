import React from 'react';

export function Spinner({ size = '' }) {
  return <span className={`spinner${size ? ` spinner--${size}` : ''}`} role="status" aria-label="Loading" />;
}

export function CenteredSpinner({ label = 'Loading…' }) {
  return (
    <div className="centered-spinner">
      <Spinner size="lg" />
      <span>{label}</span>
    </div>
  );
}
