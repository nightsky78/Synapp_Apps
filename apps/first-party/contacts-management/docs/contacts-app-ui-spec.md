# Contacts App UI Spec

## Purpose

This document defines the initial contacts workspace shell for Synapp Apps. The shell is a screenshot-inspired three-pane layout that stays schema-driven and host-safe.

## Layout Model

The workspace is built around a fixed three-pane grid:

- Left pane: groups, address books, smart lists, and pin state
- Center pane: searchable contact list with selection, density, and sort controls
- Right pane: contact detail, notes, quick actions, and activity hints

The top command bar stays outside the panes so the primary actions remain visible on both wide and narrow layouts.

### Layout Tokens

Use these design tokens in the packaged UI:

- `--contacts-bg`: page background gradient
- `--contacts-surface`: main card surface
- `--contacts-surface-strong`: stronger elevated surface
- `--contacts-border`: neutral border color
- `--contacts-text`: primary text
- `--contacts-muted`: supporting text
- `--contacts-accent`: primary action color
- `--contacts-accent-soft`: selected-row fill
- `--contacts-success`: active/available state
- `--contacts-warning`: attention state
- `--contacts-danger`: destructive state

Suggested sizing tokens:

- Shell padding: `24px`
- Pane gap: `16px`
- Left rail width: `280px`
- Center pane width: `minmax(360px, 1.1fr)`
- Detail pane width: `420px`
- Card radius: `18px`
- Control radius: `12px`

## Interaction States

The UI must handle these states explicitly:

- Default: no selection, first run data present, commands enabled
- Hover: cards and chips get a subtle surface lift
- Focus: visible 2px focus ring with sufficient contrast
- Selected: active group and active contact use a filled accent surface
- Empty: no contacts, no groups, or filtered-out list show a clear empty state
- Loading: use a non-blocking skeleton or progress tone, not a blank shell
- Unsupported: host renderer missing the contacts workspace kind
- Disabled: app installed but disabled by the host

Primary actions should be button-based and never depend on hover-only affordances.

## Mobile Collapse Behavior

At `<= 960px`, the workspace collapses into a single-column experience:

- The command bar wraps into multiple rows.
- The three panes become a single active pane controlled by a compact segmented switcher.
- The default active pane should be the contacts list.
- The contact detail pane should remain reachable without requiring horizontal scrolling.
- The left group rail should collapse into a tab-like panel selector rather than a permanently visible column.

At `<= 640px`:

- Reduce the shell padding and card density.
- Keep touch targets at least `44px` high.
- Prefer stacked action buttons over inline icon-only buttons.

## Accessibility

The package should meet these baseline accessibility rules:

- Use semantic landmarks: `header`, `nav`, `main`, and `aside`.
- Use a visible `aria-label` for search, list selection, and pane switchers.
- Preserve a logical tab order: command bar, groups rail, contacts list, detail actions.
- Provide text labels for icons and status chips.
- Ensure selection is exposed with `aria-selected` or equivalent listbox semantics.
- Keep the color system readable in high-contrast environments.
- Respect `prefers-reduced-motion` by disabling heavy transitions and transforms.

## Host Contract

The app stays schema-driven only:

- No raw HTML injection
- No inline script injection
- No DOM selector contracts in the host schema
- No host-specific business logic in the UI manifest

The packaged React UI is a presentation scaffold. Host persistence, syncing, and access control must come from platform contracts, not from UI-side assumptions.

## Platform Mismatch Risks And Mitigations

### Risk: Host may not yet support `contacts.workspace.v1`

The host may not have a native renderer for the contacts workspace kind yet.

Mitigation:

- Keep `ui.entrypoint` packaged and buildable so the UI can still be validated locally.
- Treat the schema as the source of truth for the future host renderer.
- Surface a structured unsupported state instead of falling back to unsafe markup.

### Risk: Host contacts API may lag behind the UI skeleton

The UI scaffold can outpace the backend contract.

Mitigation:

- Keep the sample data model aligned with stable host concepts: groups, people, detail, and workspace state.
- Avoid hard-coding persistence behaviors in the UI.
- Gate integration against host capabilities rather than optimistic assumptions.

### Risk: Unsupported data shapes could leak into E2E

Dynamic host records may differ from the sample contacts used in the scaffold.

Mitigation:

- Use stable data-testid contracts based on semantic roles, not display names.
- Derive item test IDs from host resource IDs when available.
- Keep the render path resilient to empty and partial records.
