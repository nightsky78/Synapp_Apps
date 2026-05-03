# Mail Client App — UX Copy & Microcopy Reference

> **Purpose:** Exact copy for UI buttons, labels, error messages, confirmations, and empty states. All copy is concise, action-oriented, and designed for clarity—whether the user is a human or an AI agent.

---

## 1. Style Guide Principles

- **Action-oriented:** Buttons describe what happens, not the component (e.g., "Send Email" not "Submit")
- **Short & scannable:** Max 6 words for buttons; max 1 sentence for errors
- **No jargon:** Terms are explicit (Draft, Inbox, Send, Archive, Trash)
- **Permission-aware:** Errors explain *why* an action is unavailable
- **Consistent terminology:**
  - Inbox = INBOX folder
  - Drafts = DRAFTS folder (unsent composed emails)
  - Sent = SENT folder (successfully sent emails)
  - Archive = ARCHIVE folder (old emails)
  - Trash = TRASH folder (deleted emails)
  - Draft = A single composed but unsent email
  - Send = Dispatch a draft to recipients

---

## 2. Global UI Copy

### Permission Status Badge
```
[READ]
[DRAFT]
[SEND]
[NONE]
```

### Header / Top-Bar Copy
```
"Permissions: [PERMISSION_LEVEL]"
```

### Global Buttons
| Label | Context | Action |
|-------|---------|--------|
| `Compose` | Header, when permission >= draft | Opens compose screen |
| `Search` | Header, always available | Focus search input |
| `Refresh` | Header, always available | Reload current view |
| `Menu` | Header, on mobile | Open navigation menu |

### Global Error Toast
```
"✗ [Error message]"
```
Example:
```
"✗ Failed to load emails. Please try again."
```

### Global Success Toast
```
"✓ [Success message]"
```
Examples:
```
"✓ Email sent"
"✓ Draft saved"
"✓ Email archived"
```

---

## 3. Email List View Copy

### Folder Sidebar

#### System Folder Labels
```
Inbox
Drafts
Sent
Trash
Junk
Archive
```

#### Folder Unread Count
```
"Inbox (12)"          // 12 unread emails
"Drafts (3)"          // 3 unsent drafts
"Sent"                // No count for sent (all are read)
```

#### Custom Folder
```
"Project Alpha (12)"  // User-created folder with 12 emails
```

#### Add Folder (Button)
```
"New Folder"
```

### List Header Copy

#### List State Labels
```
"[Total] emails"
Example: "42 emails in Inbox"
```

#### Pagination
```
"Page [X] of [Y]"
Example: "Page 1 of 4"
```

#### Search Indicator
```
"Search results for: '[QUERY]'"
Example: "Search results for: 'from:alice urgent'"
```

### List Items (Columns)
| Column | Format | Example |
|--------|--------|---------|
| From | Sender email or name | "alice@company.com" or "Alice Smith" |
| Subject | Truncated, full on hover | "Re: Q2 Planning..." |
| Date | Relative or absolute | "2 days ago" or "May 3, 2:34 PM" |
| Unread Indicator | ● (filled dot) or space | ● = unread; space = read |

### List View Empty State
```
"📬 No emails in this folder"

"Check another folder or refresh."

[Compose New]
```

### List View Loading State
```
"Loading emails..."
```

### List View Error State
```
"✗ Failed to load emails"

[Retry]
```

### Search Field
```
Placeholder: "Search emails... (e.g., from:alice urgent)"
```

### Search Button / Action
```
"🔍 Search"
```

### Clear Search Button
```
"✕ Clear Search"
```

---

## 4. Email Detail View Copy

### Header Buttons
| Button | Copy | Action | Disabled Label |
|--------|------|--------|---|
| Back | `← Back` | Return to list | Never |
| Archive | `Archive` | Move to ARCHIVE folder | "Not available with read-only permission" |
| Delete | `Delete` | Move to TRASH folder | "Not available with read-only permission" |
| More | `⋯` (menu) | Show more actions | Never |

### Metadata Labels
```
"From:"
"To:"
"CC:"
"Date:"
```

### Subject Display
```
[Subject line in bold, large text]
```

### Importance Badge
```
🔴 High       (if importance == "high")
(no badge)    (if importance == "normal")
⚪ Low        (if importance == "low")
```

### Attachment Section Header
```
"Attachments ([COUNT])"
Example: "Attachments (2)"
```

### Attachment Item
```
"📄 [FILENAME] ([SIZE]) [Download]"
Example: "📄 project-plan.pdf (2.3 MB) [Download]"
```

### No Attachments
```
(Section hidden)
```

### Action Buttons (Primary)
```
[Reply]      → Reply to sender
[Reply All]  → Reply to all (sender + CC)
[Forward]    → Forward to new recipients
```

### Action Buttons (Secondary / More Menu)
```
"Mark as read"
"Mark as unread"
"Mark as spam"
"Move to Folder"
"Delete"
```

### Not Found Error
```
"✗ Email not found"

"It may have been deleted or you don't have access."

[← Back to List]
```

### Permission Error
```
"✗ You don't have permission to read this email"

[← Back to List]
```

---

## 5. Compose / Draft Screen Copy

### Form Field Labels
```
"To:"
"CC:"
"BCC:"
"Subject:"
"Body:"
```

### Form Field Placeholders
```
To:      "Enter recipient email addresses (e.g., alice@company.com; bob@company.com)"
CC:      "Enter email addresses"
BCC:     "Enter email addresses"
Subject: "What is this email about?"
Body:    "Write your message here..."
```

### Form Field Validation Errors
```
"✗ Invalid email: '[EMAIL]'"
"✗ Recipient not found: '[EMAIL]'"
"✗ Too many recipients (max 100)"
"✗ Subject is required"
"✗ Body is required"
```

### Primary Action Buttons
```
[💾 Save Draft]   → Save to DRAFTS folder without sending
[📤 Send]         → Send to recipients (requires confirmation)
[← Cancel]        → Return to list (warn if unsaved)
```

### Auto-Save Indicator
```
"Saving draft..."   (while saving)
"✓ Draft saved"     (after save)
```

### Button States

#### Save Draft Button
```
Enabled:  "[💾 Save Draft]"
Disabled: "[💾 Save Draft] (No changes to save)"
Loading:  "[⏳ Saving...]"
```

#### Send Button (with Permission Checks)
```
Permission >= send, form valid:
"[📤 Send]"

Permission >= send, form invalid (missing recipient, subject, etc.):
"[📤 Send] (Complete form to send)"

Permission < send:
"[📤 Send] (Upgrade your permissions to send)"
```

#### Cancel Button
```
"[← Cancel]"
```

### Unsaved Changes Warning (Modal)
```
"You have unsaved changes."

"Save draft before leaving?"

[← Keep Editing]  [Discard]  [Save]
```

### Permission Denied (No Draft Permission)
```
"✗ Draft & Send Not Available"

"Your current permission level is: [READ]"

"To compose or send emails, you need to upgrade your permissions."

"Contact admin@company.com to request higher permission."

[← Back to Inbox]
```

---

## 6. Send Confirmation Modal Copy

### Modal Title
```
"Ready to send?"
```

### Recipient Summary
```
"To:"    [recipient1, recipient2, ...]
"CC:"    [recipients or "(none)"]
"BCC:"   [recipients or "(none)"]
```

### Subject Display
```
"Subject: [subject line]"
```

### Confirmation Buttons
```
[← Cancel]          → Return to compose
[✓ Send]            → Proceed with sending
```

### Button States

#### Send Button (Sending)
```
"[⏳ Sending...]"   (Disabled while sending)
```

#### Send Success
```
"✓ Email sent"
"Returning to Inbox..."
```

#### Send Failure
```
"✗ Failed to send email"
"[Reason: network error / invalid recipient / server error]"

[← Cancel]  [✓ Try Again]
```

### Permission Denied (No Send Permission)
```
"✗ Send Not Available"

"You can draft emails but cannot send them."
"Current permission: [DRAFT]"

[Save as Draft]  [← Cancel]
```

---

## 7. Reply / Forward Copy

### Reply (In Compose Screen)
```
To:      [original sender email] (auto-filled)
CC:      []
BCC:     []
Subject: "Re: [original subject]"
Body:    "On [DATE], [SENDER] wrote:
          > [Quoted original email]
          
          [Cursor here for reply text]"
```

### Reply All (In Compose Screen)
```
To:      [original sender email] (auto-filled)
CC:      [original CC recipients] (auto-filled)
BCC:     []
Subject: "Re: [original subject]"
Body:    "[Same as Reply]"
```

### Forward (In Compose Screen)
```
To:      [] (user fills in)
CC:      []
BCC:     []
Subject: "Fwd: [original subject]"
Body:    "---------- Forwarded message ---------
          From: [original sender]
          To: [original recipients]
          Subject: [original subject]
          
          [Original email body]
          
          [Cursor here for comment]"
```

### Forwarded Email Header
```
"---------- Forwarded message ---------"
"From: [email address]"
"To: [email addresses]"
"Subject: [subject]"
```

---

## 8. Folder Management Screen Copy

### Page Title
```
"Folders"
```

### System Folder Section
```
"System Folders"

📥 Inbox (12)
📝 Drafts (3)
📤 Sent (45)
🗑️  Trash (2)
📂 Archive (120)
⚠️  Junk (8)
```

### Custom Folder Section
```
"Custom Folders"

📂 Project Alpha (12)        [Rename] [Delete]
📂 Q2 Planning (8)           [Rename] [Delete]
📂 Client Feedback (20)      [Rename] [Delete]
```

### Add New Folder Button
```
[+ New Folder]
```

### New Folder Dialog
```
"Create New Folder"

Folder name: [_____________________]

"Folder names must be unique and less than 50 characters."

[← Cancel]  [Create]
```

### Rename Folder Dialog
```
"Rename Folder"

Current name: [Project Alpha]
New name:     [_____________________]

[← Cancel]  [Rename]
```

### Delete Folder Dialog
```
"Delete Folder?"

"This will move [COUNT] emails to your Inbox.
This action cannot be undone."

[← Cancel]  [Delete Folder]
```

### Rename Inline (Alternative)
```
"Project Alpha" → [Click to rename]
Becomes: [Project Alpha ________] [✓] [✕]
```

### Folder Actions Tooltips
```
"Rename folder"
"Delete folder and move emails to Inbox"
```

---

## 9. Search Interface Copy

### Search Input Placeholder
```
"Search emails... (e.g., from:alice urgent)"
```

### Search Syntax Help
```
"Search tips:
 • from:alice — emails from alice
 • to:bob — emails sent to bob
 • subject:budget — emails with 'budget' in subject
 • urgent — search in subject and body
 • 2024 — emails from 2024"
```

### Search Results Header
```
"Search results for: '[QUERY]'"
"[COUNT] emails found"
Example: "Search results for: 'from:alice urgent'"
         "12 emails found"
```

### No Search Results
```
"No results for: '[QUERY]'"

"Try a different search term or check your search syntax."

[← Return to Inbox]  [New Search]
```

### Search Error
```
"✗ Search failed: [REASON]"

Examples:
"✗ Search failed: Invalid query syntax"
"✗ Search failed: Network error, please try again"

[Retry]
```

### Clear Search
```
"✕ Clear Search"  (Button in header)
```

---

## 10. Permission-Related Copy

### Permission Status Indicator
```
Badge: [READ] | [DRAFT] | [SEND] | [NONE]
```

### Permission Tooltip (On Click/Hover)
```
"Your permission level: [PERMISSION]

What you can do:
✓ Read emails
✓ Search emails
✓ Create drafts         (if draft+)
✓ Send emails           (if send+)

[Learn more] [Contact admin]"
```

### Permission Denied Inline (On Disabled Button)
```
Button greyed out. Hover shows:
"Upgrade your permissions to [action]"
Example: "Upgrade your permissions to send emails"
```

### Permission Denied (Modal)
```
"✗ [ACTION] Not Available"

"Your current permission level is: [PERMISSION]"

"To [action], you need to upgrade your permissions."

"Contact admin@company.com to request access."

[← Back to Inbox]
```

### Permission Expired
```
"✗ Your permission has expired"

"Please log in again to refresh your access."

[Log In]
```

### Insufficient Recipients
```
"✗ You cannot send to more than 100 recipients"

"Current recipients: [COUNT]"

[← Back to compose]
```

---

## 11. Error Messages (Comprehensive)

| Error | Copy |
|-------|------|
| Network Error | `"✗ Network error. Please check your connection and try again. [Retry]"` |
| Server Error | `"✗ Something went wrong. (Error: 500) Please try again in a moment. [Retry]"` |
| Email Not Found | `"✗ Email not found. It may have been deleted. [← Back to List]"` |
| Draft Not Found | `"✗ Draft not found. It may have been deleted. [← Back to Inbox]"` |
| Folder Not Found | `"✗ Folder not found. [← Back to Inbox]"` |
| Invalid Email | `"✗ Invalid email: '[EMAIL]'. Please check and try again."` |
| Recipient Not Found | `"✗ Recipient '[EMAIL]' not found. Please check and try again."` |
| Too Many Recipients | `"✗ Too many recipients. Maximum 100 allowed. [Edit recipients]"` |
| Subject Required | `"✗ Subject is required to send an email."` |
| Body Required | `"✗ Body is required to send an email."` |
| Recipient Required | `"✗ At least one recipient (To, CC, or BCC) is required."` |
| Draft Already Sent | `"✗ This draft has already been sent."` |
| Permission Denied (Read) | `"✗ You don't have permission to read this email."` |
| Permission Denied (Draft) | `"✗ You don't have permission to compose emails. Upgrade your permissions. [Contact Admin]"` |
| Permission Denied (Send) | `"✗ You don't have permission to send emails. [Save as draft instead]"` |
| Invalid Query | `"✗ Invalid search query. Check syntax and try again. [Search Help]"` |

### Error Message Guidelines
- Always start with ✗ symbol
- Explain *what* went wrong
- If actionable, suggest remedy: [Retry], [Edit], [Contact Admin], etc.
- Keep to 1–2 sentences max
- Use exact field/email addresses when relevant (e.g., "Invalid email: 'not-an-email'"

---

## 12. Empty States Copy

### No Emails in Inbox
```
"📬 No emails in this folder"

"Compose a new email or check another folder."

[Compose New]  [View Other Folders]
```

### No Drafts
```
"📝 No drafts"

"Start composing a new email to create a draft."

[Compose New]
```

### No Sent Emails
```
"📤 No sent emails yet"

"Your sent emails will appear here."
```

### No Search Results
```
"🔍 No results"

"Try a different search term or check your syntax."

[← New Search]
```

### Inbox Empty (All Archived)
```
"✓ Your inbox is empty"

"Great job! All emails are organized."

[View Archive]
```

### No Folders (No Custom Folders Created)
```
"You have no custom folders yet."

"Create a folder to organize your emails."

[New Folder]
```

---

## 13. Success Messages Copy

| Action | Copy |
|--------|------|
| Email Sent | `"✓ Email sent"` |
| Draft Saved | `"✓ Draft saved"` |
| Draft Deleted | `"✓ Draft deleted"` |
| Email Archived | `"✓ Email archived"` |
| Email Deleted | `"✓ Email moved to trash"` |
| Email Restored | `"✓ Email restored to Inbox"` |
| Folder Created | `"✓ Folder '[NAME]' created"` |
| Folder Renamed | `"✓ Folder renamed to '[NAME]'"` |
| Folder Deleted | `"✓ Folder deleted. [COUNT] emails moved to Inbox."` |
| Marked as Read | `"✓ Marked as read"` |
| Marked as Unread | `"✓ Marked as unread"` |
| Marked as Spam | `"✓ Marked as spam and moved to Junk folder"` |

### Success Message Guidelines
- Start with ✓ symbol
- Confirm the action taken
- If relevant, include result count or affected item name
- Auto-dismiss after 3–5 seconds (or manual close)
- Use light green background, dark text

---

## 14. Confirmation Dialog Copy

### Send Email Confirmation
```
"Ready to send?"

[Show recipient list, CC, BCC, subject]

[← Cancel]  [✓ Send]
```

### Delete Email Confirmation
```
"Delete this email?"

"It will be moved to Trash and can be restored later."

[← Cancel]  [Delete]
```

### Delete Folder Confirmation
```
"Delete folder '[NAME]'?"

"[COUNT] emails will be moved to Inbox.
This action cannot be undone."

[← Cancel]  [Delete Folder]
```

### Discard Draft Confirmation
```
"You have unsaved changes."

"Save draft before leaving?"

[← Keep Editing]  [Discard]  [Save Draft]
```

### Mark as Spam Confirmation
```
"Mark as spam?"

"This email and future emails from this sender
will be moved to Junk."

[← Cancel]  [Yes, Mark as Spam]
```

---

## 15. Keyboard Shortcuts & Accessibility Copy

### Keyboard Shortcut Reference
```
Shortcut | Action
─────────┼──────────────────────────────────
Tab      | Move focus to next element
Shift+Tab| Move focus to previous element
Enter    | Activate focused button
Space    | Toggle checkbox or button
Escape   | Close modal, cancel action
↑ ↓      | Navigate list items
Ctrl+N   | New email
Ctrl+S   | Save draft
Ctrl+⏎   | Send email (in compose)
Ctrl+F   | Focus search
/ (slash)| Focus search (global)
```

### Accessibility Help Text
```
"This application is accessible via keyboard and screen reader."

"Press '?' for keyboard shortcuts."
"[Help]  [Accessibility Statement]  [Report Issue]"
```

---

## 16. Onboarding / First-Run Copy

### Welcome Message
```
"Welcome to Synapp Mail"

"You have [PERMISSION] permission."

What you can do:
✓ Read and search emails
✓ Create draft emails (if draft+)
✓ Send emails (if send+)

[Start Browsing]  [Learn More]
```

### Permission Level Explanation (First Visit)
```
"📖 Your Permission Level: [PERMISSION]"

[READ]:   You can read and search emails, but cannot compose or send.
[DRAFT]:  You can compose and save drafts, but cannot send.
[SEND]:   You have full access to read, compose, and send emails.

Questions? Contact admin@company.com
```

---

## 17. Copy Consistency Examples

### Consistent Terminology Across App
```
✓ "Save Draft"  (not "Save email" or "Save as draft")
✓ "Send Email"  (not "Send draft" or "Transmit")
✓ "Archive"     (not "File away" or "Remove")
✓ "Compose"     (not "New email" or "Create")
✓ "Drafts"      (not "Unsent" or "Work in progress")
✓ "Trash"       (not "Delete" or "Bin")
✓ "Junk"        (not "Spam folder")
✓ "Reply"       (not "Respond")
✓ "Forward"     (not "Pass along")
```

### Consistent Button Order
```
Primary action on right: [← Cancel] [✓ Send]
Confirmation: [← Cancel] [Action]
Destructive: [← Cancel] [Delete]
```

### Consistent Error Pattern
```
[✗ Symbol] [What went wrong] [Optional remedy]
"✗ Failed to send email. [Retry]"
"✗ Invalid recipient. [Edit]"
"✗ Permission denied. [Contact Admin]"
```

---

## 18. Tone & Voice Notes

### For All Copy
- **Professional but friendly:** "Hi there" ✓, "Yo" ✗
- **Active voice:** "Email sent" ✓, "Email has been sent" ✗
- **Clear verbs:** "Save", "Send", "Archive" (not "Process", "Execute", "Handle")
- **Avoid jargon:** "Drafts" ✓, "Ephemeral payloads" ✗
- **No false urgency:** "Your email sent successfully" ✓, "Email dispatched ASAP!" ✗
- **Agent-friendly:** "Permission denied: read required" ✓, "Oops, you can't do that" ✗

### For Error Messages
- Explain the problem
- Don't blame the user or agent
- Suggest recovery if possible
- Keep emotions neutral
- Use consistent terminology

Example:
```
✓ "✗ Invalid recipient: 'not-an-email'. Please check and try again."
✗ "✗ You messed up the email address! Fix it."
✗ "✗ Error 400: Malformed email entity"
```

---

## 19. Copy Localization Notes

While this spec is in English, keep these principles in mind for localization:
- Keep labels short (< 6 words) to fit UI constraints
- Avoid culture-specific idioms
- Use formal "you" in languages with formal/informal distinction
- Date formats: "May 3, 2:34 PM" may vary by region
- Icons (✗, ✓, ⋯) are universal; test for clarity
- Test with screen readers in target language

---

## 20. Testing Checklist for Copy

- [ ] All buttons have clear, action-oriented labels
- [ ] Error messages are specific, not generic
- [ ] Success messages confirm the action taken
- [ ] Permission-denied messages suggest remedy
- [ ] Empty states are helpful, not confusing
- [ ] Confirmation dialogs clearly state consequences
- [ ] Terminology is consistent throughout
- [ ] No jargon; all terms defined or obvious
- [ ] Copy is concise (max 1–2 sentences for most messages)
- [ ] Tone is professional and agent-friendly
- [ ] ARIA labels match visible text where applicable
- [ ] Error codes are provided for support reference
