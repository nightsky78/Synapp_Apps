# Mail Client App — UI Specification & Behavior

> **Design Principle:** Contract-first UI. Every screen state, user journey, and component behavior is designed from the explicit function contracts and permission model. The UI must make permissions visible and guide users (and agents) toward valid actions.

---

## 1. Design Philosophy

### Accessibility & Clarity for Agents
- **Explicit state display:** UI always shows current operation state (idle, loading, success, error)
- **Permission visibility:** Disabled actions show *why* they're disabled (not just greyed out)
- **Minimal cognitive load:** Clear action sequences, no hidden workflows
- **Keyboard-first:** All major actions accessible via keyboard; focus management on state transitions

### Permission-Aware UI
- At function entry, every Wasm function validates agent permission
- The UI must reflect this: If permission is insufficient, the action is visually unavailable with explanation
- No "attempt then fail" pattern; agents should know their constraints upfront

### Accessibility Baseline
- **WCAG 2.1 AA minimum:** Contrast ratio 4.5:1 for text
- **Semantic HTML:** Use `<button>`, `<label>`, `<form>`, `<main>`, `<section>` for screen readers
- **Keyboard navigation:** Tab order follows visual flow; Enter/Space triggers buttons; Escape closes modals
- **ARIA labels:** All interactive elements have `aria-label` or associated `<label>`
- **Focus indicators:** Clear, visible focus ring on all interactive elements

---

## 2. Global UI Components & States

### Permission Status Indicator (Top-Right)
**Purpose:** Always show the current agent's permission level.

**Component:**
```
┌─────────────────────┐
│ Permissions: [READ] │  (or [DRAFT], [SEND], [NONE])
└─────────────────────┘
```

**Behavior:**
- Displayed in header at all times
- Click to open permission details modal (read-only info, expiration if applicable)
- Color-coded: 
  - `[READ]` → Blue
  - `[DRAFT]` → Orange
  - `[SEND]` → Green
  - `[NONE]` → Gray
- **Accessibility:** `aria-label="Your current permission level: read"`

**States:**
- **Idle:** Steady display of permission badge
- **Loading:** (N/A — never changes during session unless host revokes)
- **Error:** (N/A — displayed on app init via `get_agent_permissions()`)

### Global Error Toast
**Purpose:** Transient error notifications for failed actions.

**Component:**
```
┌─────────────────────────────────────────┐
│ ✗ [Error message]              [Dismiss] │
└─────────────────────────────────────────┘
```

**Behavior:**
- Appears at bottom-left, fixed position
- Auto-dismisses after 5 seconds or on user click
- Color: Red (#d32f2f), white text
- **Accessibility:** `role="alert"`, `aria-live="assertive"`

**States:**
- **Display:** Error toast appears
- **Auto-dismiss:** Fades out after 5s
- **Manual dismiss:** User clicks [Dismiss]

---

## 3. Email List View

**Route:** `/inbox` or `/` (default)

**Purpose:** Display paginated list of emails from current folder, support searching and filtering.

### Screen Layout
```
┌─────────────────────────────────────────────────────────┐
│ Permissions: [READ]                       [Search] [≡]   │  ← Header
├─────────────────────────────────────────────────────────┤
│  Folders:                                                 │
│  ├─ Inbox (12)         ← Current                         │
│  ├─ Drafts (3)                                           │
│  ├─ Sent                                                 │
│  ├─ Trash                                                │
│  └─ Archives                                             │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ☐  From          Subject          Date      Unread    │  ← List Header
│  ───────────────────────────────────────────────────────  │
│  ☑  alice@...     Re: Project Q    May 3     ●         │  ← Email Item (read)
│  ☐  bob@...       Team Standup     May 2            │  ← Email Item (unread)
│  ☑  carol@...     Q2 Budget        May 1     ●         │  ← Email Item (read)
│                                                          │
│  ← [1] [2] [3] [4] →  Page 1 of 4                       │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### Component Behaviors

#### Folder Sidebar
**Purpose:** Navigate between mailboxes.

**States:**
- **Idle:** Display folder list with unread counts
- **Loading:** (Spinner next to folder name if content is loading)
- **Active folder:** Highlighted with background color
- **Error:** If folder fetch fails, show error toast and default to INBOX

**Interactions:**
- Click folder → Load list view for that folder
- Folder counts update after send/move operations
- **Keyboard:** Arrow keys to navigate; Enter to select

**Permission Rules:**
- All folders visible to all permission levels (read, draft, send)
- DRAFTS folder only visible if permission >= draft
- SENT folder only visible if permission >= send

**Accessibility:**
- `role="navigation"`, `aria-label="Mailbox folders"`
- Current folder has `aria-current="page"`

#### Email List
**Purpose:** Scannable, paginated list of emails in current folder.

**Columns:**
- **Checkbox:** Select for batch actions (if implemented)
- **From:** Sender address or friendly name
- **Subject:** Truncated subject with ellipsis
- **Date:** Relative (e.g., "2 days ago") or absolute (e.g., "May 3")
- **Unread indicator:** Dot (●) if unread; space if read

**Rows:**
- Each row is a clickable card
- Hover state: Subtle background highlight
- Selected row: Darker background
- **Keyboard:** Arrow keys to navigate; Enter to open detail view

**List States:**

1. **Idle (Loaded)**
   ```
   ☑  alice@...     Re: Project Q    May 3     ●
   ☐  bob@...       Team Standup     May 2
   ```
   - All items displayed, ready for interaction

2. **Loading**
   ```
   [Spinner] Loading emails...
   ```
   - Replace list with centered spinner
   - Message: "Loading emails..."

3. **Empty**
   ```
   ╔═══════════════════════════════════╗
   ║                                   ║
   ║  📬 No emails in this folder      ║
   ║                                   ║
   ║  Check another folder or refresh  ║
   ║                                   ║
   ║  [Compose New] (if draft+)        ║
   ║                                   ║
   ╚═══════════════════════════════════╝
   ```
   - Centered, with icon and helpful text
   - Show [Compose New] button if permission allows

4. **Error**
   ```
   ╔═══════════════════════════════════╗
   ║  ✗ Failed to load emails          ║
   ║  [Retry]                          ║
   ╚═══════════════════════════════════╝
   ```
   - Centered error with retry button
   - Toast also shown at bottom

**Pagination:**
- Display current page: "Page 1 of 4"
- Previous/Next buttons disabled at boundaries
- Jump-to-page input (if needed)

**Accessibility:**
- List has `role="grid"` or `role="table"` with ARIA attributes
- Each row: `role="row"`, `aria-selected="true|false"`
- Checkbox: `type="checkbox"`, associated `<label>`

#### Search/Filter Header
**Purpose:** Quick search within current folder.

**Component:**
```
[🔍 Search emails... (e.g., "from:alice urgent")]
```

**Behavior:**
- Debounced search (500ms) as user types
- Submits `search_emails(query, limit=100)` to backend
- Results replace current list view
- "Clear search" button appears if search is active
- Show search term in list header: "Search results for: 'urgent'"

**States:**
- **Idle:** Search box empty, placeholder text visible
- **Typing:** Debounce timer active
- **Searching:** Spinner next to search box
- **Results:** Show filtered list or "No results"
- **Error:** Toast with "Search failed: [reason]"

**Keyboard:**
- Enter to submit search
- Escape to clear search

**Accessibility:**
- Search input has `aria-label="Search emails"`
- Search icon has `aria-hidden="true"`

---

## 4. Email Detail View

**Route:** `/email/:emailId`

**Purpose:** Display full email content with read/forward/reply options (permission-gated).

### Screen Layout
```
┌─────────────────────────────────────────────────────────┐
│ Permissions: [READ]        [← Back] [Archive] [Delete]   │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  From: alice@company.com                                │
│  To:   me@company.com                                   │
│  CC:   bob@company.com                                  │
│  Date: May 3, 2026 at 2:34 PM                           │
│                                                          │
│  Subject: Re: Q2 Project Planning                       │
│  ─────────────────────────────────────────────────────  │
│                                                          │
│  [Email body content here, may be HTML]                 │
│                                                          │
│  ─────────────────────────────────────────────────────  │
│  Attachments: project-plan.pdf, budget.xlsx             │
│                                                          │
│  ─────────────────────────────────────────────────────  │
│  [Reply] [Reply All] [Forward]                          │  ← Action Buttons
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### Header Actions
- **[← Back]:** Return to list view
- **[Archive]:** Move to ARCHIVE folder (if read+)
- **[Delete]:** Move to TRASH (if read+)
- **More menu (⋯):** Mark as read, mark as spam, move to folder

### Email Metadata
- **From, To, CC:** Displayed as plaintext; click to copy email address
- **Date:** Human-readable format (e.g., "May 3, 2026 at 2:34 PM")
- **Subject:** Bold, large font
- **Importance indicator:** If importance is "high", show 🔴 badge; if "low", show ⚪

### Email Body
- Render as HTML if provided; escape/sanitize for security
- Preserve formatting (bold, links, etc.)
- Long emails: Use scrollable container, not page scroll

### Attachments Section
**Purpose:** List attachments (reference only; downloads handled by platform).

**Component:**
```
Attachments (2)
├─ 📄 project-plan.pdf (2.3 MB) [Download]
└─ 📊 budget.xlsx (1.1 MB) [Download]
```

**Behavior:**
- Click [Download] → Platform handles file fetch
- Show file size
- If no attachments, section is hidden

### Action Buttons (Permission-Gated)

| Button | Permission Required | State | Behavior |
|--------|---|---|---|
| **Reply** | read | Enabled | Open compose screen in reply mode |
| **Reply All** | read | Enabled | Compose with all recipients pre-filled |
| **Forward** | read | Enabled | Compose with subject prefixed "Fwd: " |
| **Archive** | read | Enabled | Move email to ARCHIVE folder |
| **Delete** | read | Enabled | Move email to TRASH folder |

**Disabled State Display:**
```
[Reply] is not available. You have read-only permission.
```
- Button is visibly disabled (greyed out, not clickable)
- Hover shows tooltip with permission explanation

### Email Detail States

1. **Idle (Loaded)**
   - Full email content displayed, all actions available based on permission

2. **Loading**
   ```
   [Spinner] Loading email...
   ```

3. **Not Found**
   ```
   ╔═══════════════════════════════════╗
   ║  ✗ Email not found                ║
   ║  It may have been deleted.        ║
   ║  [← Back to List]                 ║
   ╚═══════════════════════════════════╝
   ```

4. **Permission Error**
   ```
   ╔═══════════════════════════════════╗
   ║  ✗ You do not have permission to  ║
   ║    read this email.               ║
   ║  [← Back to List]                 ║
   ╚═══════════════════════════════════╝
   ```

### Accessibility
- Header is `<header role="region" aria-label="Email metadata">`
- Subject is `<h1>`
- Body is `<main>`
- Action buttons have clear `aria-label` ("Reply to this email", etc.)

---

## 5. Compose / Draft Screen

**Route:** `/compose` (new) or `/compose?draft=:draftId` (edit draft)

**Permission Required:** `draft` or higher

**Purpose:** Create and edit email drafts. Save as draft or send (if permission allows).

### Screen Layout
```
┌─────────────────────────────────────────────────────────┐
│ Permissions: [DRAFT]      [← Cancel] [Save] [Send (disabled)]│
├─────────────────────────────────────────────────────────┤
│                                                          │
│  To:     [alice@company.com; bob@company.com]           │
│  CC:     [                                    ]           │
│  BCC:    [                                    ]           │
│                                                          │
│  Subject:[Q2 Budget Discussion                ]           │
│                                                          │
│  ┌───────────────────────────────────────────────────┐  │
│  │ Hi team,                                          │  │
│  │                                                   │  │
│  │ Here's the Q2 budget breakdown...                │  │
│  │                                                   │  │
│  │ (Text area, supports rich text if desired)       │  │
│  │                                                   │  │
│  └───────────────────────────────────────────────────┘  │
│                                                          │
│  [💾 Save Draft] [📤 Send] (disabled if no permission)   │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### Form Fields

#### Recipients (To, CC, BCC)
- **Type:** Text input with autocomplete (platform-provided contacts)
- **Format:** Comma or semicolon-separated email addresses
- **Validation:** On blur, validate email format; show inline error if invalid
- **Accessibility:** `aria-label="To recipients"`

**Error States:**
```
To: [alice@company.com; invalid-email]
    ✗ Invalid email: "invalid-email"
```

#### Subject
- **Type:** Single-line text input
- **Max length:** 998 characters (RFC 5322)
- **Required:** Yes (can't save draft without subject)

#### Body
- **Type:** Textarea (or rich text editor if desired)
- **Format:** Plain text or HTML (depending on implementation)
- **Auto-save:** Every 10 seconds (show "Saving..." indicator)
- **Max length:** Platform-dependent (typically 10 MB)

### Action Buttons

| Button | Permission | Behavior |
|--------|---|---|
| **[💾 Save Draft]** | draft+ | Save to DRAFTS folder; show success toast; return to list |
| **[📤 Send]** | send | Validate form, send email, show confirmation, move to SENT |
| **[← Cancel]** | any | Return to list (warn if unsaved changes) |

### Form States

1. **Idle (New)**
   ```
   To:      []
   CC:      []
   BCC:     []
   Subject: []
   Body:    []
   
   [💾 Save Draft] [📤 Send (disabled)]
   ```
   - Save button: Enabled
   - Send button: Disabled (no permission or form invalid)

2. **Idle (Editing Existing Draft)**
   ```
   To:      [alice@company.com]
   CC:      []
   BCC:     []
   Subject: [Q2 Budget]
   Body:    [Hi team...]
   
   [💾 Save Draft] [📤 Send]
   ```
   - Both buttons enabled (if permission >= send for Send button)

3. **Validation Error**
   ```
   To: [alice@company.com; invalid]
       ✗ Invalid email address
   
   [💾 Save Draft (disabled)] [📤 Send (disabled)]
   ```
   - Buttons disabled until form is valid

4. **Saving**
   ```
   [Spinner] Saving draft...
   ```

5. **Saved Success**
   ```
   ✓ Draft saved
   (Toast disappears after 3s)
   ```

6. **Permission Denied (Attempt to Send without Permission)**
   ```
   ✗ You don't have permission to send emails.
     Contact admin to upgrade your permissions.
   ```
   - Button remains disabled
   - Error shown in toast

### Unsaved Changes
- **Leaving without saving:** Show confirmation modal
  ```
  ╔════════════════════════════════════════╗
  ║  You have unsaved changes.             ║
  ║  Save draft before leaving?            ║
  ║                                        ║
  ║  [← Keep Editing] [Discard] [Save]    ║
  ╚════════════════════════════════════════╝
  ```
- **Auto-save:** Every 10s if content changed; show brief "Saving..." indicator

### Accessibility
- Form is `<form>` with `aria-label="Compose email"`
- Each field has associated `<label>` or `aria-label`
- Error messages have `aria-live="polite"` for screen reader announcement
- Tab order: To → CC → BCC → Subject → Body → [Save] → [Send] → [Cancel]

---

## 6. Send Confirmation Modal

**Trigger:** User clicks [Send] button in compose screen

**Permission Required:** `send`

**Purpose:** Prevent accidental sends; show recipients for final review.

### Modal Content
```
╔═════════════════════════════════════════╗
│  Ready to send?                         │
├─────────────────────────────────────────┤
│  To:   alice@company.com                │
│        bob@company.com                  │
│  CC:   (none)                           │
│                                         │
│  Subject: Q2 Budget Discussion          │
│                                         │
│  [← Cancel] [✓ Send]                   │
╚═════════════════════════════════════════╝
```

### Behavior
- **[← Cancel]:** Return to compose screen without sending
- **[✓ Send]:** Invoke `send_email()` function
  - Disable button during send
  - Show spinner: "Sending..."
  - On success: Show toast "Email sent", return to inbox
  - On error: Show toast with error reason, stay on confirmation modal

### Send States

1. **Idle (Ready)**
   - Recipient list and subject visible
   - [Send] button enabled

2. **Sending**
   ```
   [Spinner] Sending email...
   ```
   - Buttons disabled

3. **Success**
   ```
   ✓ Email sent
   [Return to inbox in 2 seconds...]
   ```
   - Toast shown, redirect to inbox

4. **Failure**
   ```
   ✗ Failed to send email
     Invalid recipients or network error
   [← Cancel] [✓ Try Again]
   ```
   - Error message shown
   - Can retry or cancel

### Accessibility
- Modal has `role="dialog"`, `aria-modal="true"`, `aria-label="Confirm send"`
- Focus is trapped within modal
- Escape key closes modal (returns to compose)

---

## 7. Folder Management Screen

**Route:** `/folders`

**Permission Required:** `read` (view), `draft+` (move emails)

**Purpose:** View folder hierarchy, create custom folders, manage folder settings.

### Screen Layout
```
┌─────────────────────────────────────────────────────────┐
│ Permissions: [READ]        [← Back] [New Folder]        │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  System Folders                                          │
│  ├─ 📥 Inbox (12)                                       │
│  ├─ 📝 Drafts (3)                                       │
│  ├─ 📤 Sent (0)                                         │
│  ├─ 🗑️  Trash (2)                                        │
│  ├─ 📂 Archive (45)                                     │
│  └─ ⚠️  Junk (8)                                         │
│                                                          │
│  Custom Folders                                          │
│  ├─ 📂 Project Alpha (12)  [Rename] [Delete]           │
│  ├─ 📂 Q2 Planning (8)     [Rename] [Delete]           │
│  └─ 📂 Client Feedback (20) [Rename] [Delete]          │
│                                                          │
│  [New Folder]                                            │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### Folder List
- **System folders:** Read-only, cannot be deleted
- **Custom folders:** User-created, can be renamed/deleted
- **Email count:** Shown next to folder name
- **Unread indicator:** Small dot (●) if folder has unread emails

### Actions
- **[Rename]:** Edit folder name in-place
- **[Delete]:** Confirm deletion (move emails to INBOX or ARCHIVE)
- **[New Folder]:** Open dialog to create new folder

### Accessibility
- Folder list: `role="list"`, each folder is `role="listitem"`
- Rename/Delete buttons: Inline, clear `aria-label`

---

## 8. Reply / Forward Flows

**Permission Required:** `draft` (to compose reply/forward)

**Purpose:** Simplified compose screen for replies and forwards.

### Reply Flow

**Trigger:** User clicks [Reply] or [Reply All] in detail view

**Compose Screen (Pre-filled):**
```
To:      [alice@company.com]        (from original sender)
CC:      [bob@company.com]          (for Reply All only)
Subject: [Re: Q2 Budget Discussion]
Body:    [On May 3, alice wrote:
         > [Original email quoted]]
```

**Behavior:**
- To field pre-filled with sender
- For "Reply All," CC field pre-filled with original CC recipients
- Subject prefixed with "Re: " if not already
- Original email quoted below body (quoted style: indented, different color)
- User can edit all fields
- [Save Draft] and [Send] buttons available (if permission allows)

### Forward Flow

**Trigger:** User clicks [Forward] in detail view

**Compose Screen (Pre-filled):**
```
To:      []
CC:      []
BCC:     []
Subject: [Fwd: Q2 Budget Discussion]
Body:    [---------- Forwarded message ---------
         From: alice@company.com
         To: me@company.com
         Subject: Q2 Budget Discussion
         
         [Original email content]]
```

**Behavior:**
- To field empty (user specifies recipient)
- Subject prefixed with "Fwd: "
- Full original email included in body with header info
- User can edit all fields
- [Save Draft] and [Send] buttons available

### Accessibility
- Original email in body: Wrapped in `<blockquote>`, quoted style applied
- Clear visual distinction between user's reply and original email
- Focus starts in "To" field (for forwards) or first editable field

---

## 9. Search Results View

**Route:** `/search?q=:query`

**Permission Required:** `read`

**Purpose:** Display search results with clear query context.

### Screen Layout
```
┌─────────────────────────────────────────────────────────┐
│ Permissions: [READ]        [Search] [✕ Clear Search]    │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  Search results for: "from:alice urgent"                │
│  Results: 12 emails                                     │
│                                                          │
│  ☐  From          Subject          Date      Unread    │
│  ───────────────────────────────────────────────────────  │
│  ☑  alice@...     Re: Urgent Task  May 3     ●         │
│  ☐  alice@...     Q4 Planning      May 2     ●         │
│  ☑  bob@...       Urgent Review    May 1            │
│                                                          │
│  [1] [2] →  Page 1 of 2                                 │
│                                                          │
│  [← Return to Inbox]                                    │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### Behavior
- Same list format as email list view
- Show query and result count prominently
- [Clear Search] button to return to inbox
- Pagination available
- Clicking on email opens detail view

### Search States

1. **Results**
   - Show email list with query context

2. **No Results**
   ```
   ╔═══════════════════════════════════╗
   ║  No results for "from:alice"      ║
   ║  Try a different search term.     ║
   ║  [← Return to Inbox]              ║
   ╚═══════════════════════════════════╝
   ```

3. **Search Error**
   ```
   ✗ Search failed: Invalid query syntax
   ```

### Accessibility
- Search query shown prominently for screen readers
- Result count announced: `aria-live="polite"`

---

## 10. Permission Denied Screens

**Purpose:** Clear, helpful messages when agent lacks required permission.

### Read-Only Agent Attempts Draft

**When:** Agent with read-only permission clicks [Compose]

**Screen:**
```
╔═════════════════════════════════════════╗
║  ✗ Draft & Send Not Available           ║
│                                         │
│  Your current permission level is:      │
│  [READ]                                 │
│                                         │
│  To compose or send emails, you need   │
│  upgrade your permissions.             │
│                                         │
│  Contact admin@company.com to request. │
│                                         │
│  [← Back to Inbox]                    │
╚═════════════════════════════════════════╝
```

### Draft Agent Attempts Send

**When:** Agent with draft permission clicks [Send] in confirmation modal

**Modal:**
```
╔═════════════════════════════════════════╗
║  ✗ Send Not Available                   │
│                                         │
│  You can draft emails but cannot send  │
│  them. Current permission: [DRAFT]     │
│                                         │
│  [Save as Draft]  [← Cancel]           │
╚═════════════════════════════════════════╝
```

---

## 11. Error States & Recovery

### Network Error
```
✗ Network error. Please check your connection.
[Retry]
```
- Toast with retry button
- Page content remains visible but greyed out
- Automatic retry after 5 seconds (show countdown)

### Invalid Input
```
✗ Invalid recipient: "not-an-email"
Please check the email address and try again.
```
- Inline error near the problematic field
- Form submission blocked

### Permission Expired
```
✗ Your permission has expired.
Please log in again to continue.
[Log In]
```
- Clear, actionable message
- Redirect to login or refresh session

### Server Error (500, etc.)
```
✗ Something went wrong.
(Error code: 500)
Please try again in a moment or contact support.
[Retry]
```
- Generic but professional
- Error code for support reference

---

## 12. Accessibility Checklist

- [ ] All interactive elements keyboard-accessible (Tab, Enter, Space, Escape)
- [ ] Focus visible and logical (follows visual flow, not DOM order)
- [ ] Color not sole means of conveying information (use text labels, icons)
- [ ] Text contrast ratio ≥ 4.5:1 (WCAG AA)
- [ ] Images/icons have alt text or `aria-hidden="true"`
- [ ] Form fields have associated `<label>` or `aria-label`
- [ ] Errors announced to screen readers (`aria-live="polite"`)
- [ ] Modal focus trapped, escape closes
- [ ] Loading states clearly indicated (spinners + text)
- [ ] Success/failure states persist > 2 seconds
- [ ] No time limits on actions (or extend if needed)
- [ ] Semantic HTML: `<button>`, `<form>`, `<main>`, `<nav>`, etc.

---

## 13. State Machine Summary

### Global States
```
[AppInitialized] → [PermissionLoaded] → [Idle]
                                        ├─→ [Loading]
                                        ├─→ [Composing]
                                        ├─→ [Sending]
                                        ├─→ [PermissionDenied]
                                        └─→ [Error]
```

### List View States
```
[Idle] → [Loading] → [Success (List Displayed)]
              ├─→ [Empty]
              └─→ [Error]
```

### Compose States
```
[Idle] → [Saving] → [Saved]
    │
    └─→ [Sending] → [Sent]
         ├─→ [ValidationError]
         └─→ [PermissionDenied]
```

### Send Confirmation States
```
[ConfirmationShown] → [Sending] → [Success]
                  │
                  ├─→ [Error → Retry]
                  └─→ [Cancelled]
```

---

## 14. Responsive Design Notes

### Desktop (1024px+)
- Three-column layout: Folders | Email List | Detail
- Full compose form with rich text editor
- Folder manager in sidebar

### Tablet (768-1023px)
- Two-column: Folders | Email List+Detail
- Folders collapsible to hamburger menu
- Compose form simplified (plain text only)

### Mobile (< 768px)
- Single column: List view
- Folders in drawer/modal
- Detail view full-screen overlay
- Compose form minimal: To, Subject, Body only

---

## 15. Testing Scenarios

### Permission Scenarios
- [ ] Read-only agent: Compose button hidden or disabled
- [ ] Draft agent: Send button disabled with clear message
- [ ] Send agent: All actions available
- [ ] No permission agent: All actions hidden/disabled with explanation

### State Scenarios
- [ ] Load inbox: Initially loading, then display emails
- [ ] Search: Show results or "no results"
- [ ] Send email: Show confirmation, then success
- [ ] Network error: Show error toast with retry
- [ ] Permission denied: Show permission denied screen

### Accessibility Scenarios
- [ ] Keyboard-only navigation: All actions reachable via Tab/Enter
- [ ] Screen reader: All content readable, errors announced
- [ ] High contrast mode: Text readable, focus visible
- [ ] Slow network: Loading states clear, no blank screens
