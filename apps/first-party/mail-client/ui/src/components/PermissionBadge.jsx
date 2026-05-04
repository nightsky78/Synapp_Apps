import React from 'react';

const LABELS = { none: 'No Access', read: 'Read', draft: 'Draft', send: 'Send', organize: 'Organize' };

export default function PermissionBadge({ permission }) {
  return (
    <span className={`perm-badge perm-badge--${permission ?? 'none'}`} title={`Permission level: ${LABELS[permission] ?? 'None'}`}>
      🔑 {LABELS[permission] ?? 'None'}
    </span>
  );
}
