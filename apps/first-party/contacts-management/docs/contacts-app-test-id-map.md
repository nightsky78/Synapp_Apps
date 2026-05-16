# Contacts App Test-ID Map

This map defines stable selectors for the contacts workspace. The IDs are intentionally semantic and should not depend on rendered copy.

## Shell And Global Controls

| Test ID | Purpose |
|---|---|
| `contacts-shell` | Root application shell |
| `contacts-commandbar` | Top command bar |
| `contacts-search` | Search input |
| `contacts-new-contact` | Primary create contact action |
| `contacts-import` | Import action |
| `contacts-share-book` | Share address book action |
| `contacts-mobile-switcher` | Compact pane switcher shown on narrow screens |
| `contacts-status` | Inline status chip or message |

## Panes

| Test ID | Purpose |
|---|---|
| `contacts-groups-pane` | Left groups rail |
| `contacts-list-pane` | Center contacts list |
| `contacts-detail-pane` | Right contact detail pane |
| `contacts-empty-state` | Shared empty/filtered state |

## Groups Rail

| Test ID | Purpose |
|---|---|
| `contacts-groups-list` | Group list container |
| `contacts-group-item-${group_id}` | Stable group row derived from the host group id |
| `contacts-group-all` | All contacts filter |
| `contacts-group-favorites` | Favorites or pinned filter |
| `contacts-group-directory` | Global directory filter |

## Contact List

| Test ID | Purpose |
|---|---|
| `contacts-list-toolbar` | List-level toolbar |
| `contacts-sort` | Sort control |
| `contacts-density` | Density control |
| `contacts-contact-list` | Contact list container |
| `contacts-contact-item-${contact_id}` | Stable contact row derived from the host contact id |
| `contacts-contact-avatar-${contact_id}` | Avatar cell for a contact row |
| `contacts-contact-name-${contact_id}` | Visible contact name label |
| `contacts-contact-meta-${contact_id}` | Title, company, or email summary |

## Contact Detail

| Test ID | Purpose |
|---|---|
| `contacts-detail-header` | Detail title and primary actions |
| `contacts-detail-actions` | Detail action button group |
| `contacts-detail-email` | Primary email block |
| `contacts-detail-phone` | Primary phone block |
| `contacts-detail-notes` | Notes field or rendered notes content |
| `contacts-detail-tags` | Tag or label list |
| `contacts-detail-activity` | Recent activity summary |

## Selector Rules

- Use stable host ids whenever they are available.
- Do not build selectors from display names, initials, or localized labels.
- Keep pane IDs fixed even when the active mobile pane changes.
- When a pane is hidden on mobile, keep its test ID present only in the active panel container.
