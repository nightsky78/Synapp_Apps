# Mail Client UX Spec

## Folder and Send Behavior

The mail client must make mailbox navigation and send outcomes visible to the user. The selected folder label, message list, and reading pane must stay in sync after each folder action.

## Acceptance Criteria

- The sidebar exposes stable controls for all visible mailboxes, including the Sent folder and custom folders.
- Creating, renaming, or deleting a folder refreshes the folder tree and keeps the active account unchanged.
- Selecting a folder updates the mailbox title and message list to match that folder.
- After a successful send, the UI shows a clear success toast and makes the sent message visible when the user opens the Sent folder.
- If folder loading fails, the user sees a visible error toast instead of a blank or crashed view.
- New or changed interactive controls expose stable `data-testid` values for E2E coverage.

## Observable Outcomes

- Sent items are visible immediately after sending when the host returns them in the Sent mailbox.
- Folder-read failures are user-visible and testable.
- Folder management actions remain deterministic across reloads.
