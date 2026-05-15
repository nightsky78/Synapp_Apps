# Mail Client Test Matrix

| ID | Scenario | Expected Outcome |
|---|---|---|
| TC-SA-MAIL-UI-001 | Read mailbox, search, flag, archive | Folder title, message list, and action toasts reflect the visible mailbox state |
| TC-SA-MAIL-UI-002 | Compose, save draft, send, and schedule | Send closes compose, and opening Sent renders the sent message in the mailbox list |
| TC-SA-MAIL-UI-003 | Account setup, preferences, and rules | Settings actions succeed and update visible settings state |
| TC-SA-MAIL-UI-004 | Read-only permission | Send and organize controls are disabled and restricted actions are not invoked |
| TC-SA-MAIL-UI-005 | Host action failures | Visible error toast appears for mailbox/search failures |
| TC-SA-MAIL-UI-006 | No host bridge | Packaged UI does not render sample inbox data |
| TC-SA-MAIL-UI-007 | Sent folder after send | Sent mailbox becomes visible and contains the new message subject |
| TC-SA-MAIL-UI-008 | Folder-read failure | Folder selection failure surfaces a visible error toast and does not crash the UI |
